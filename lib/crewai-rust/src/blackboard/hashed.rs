//! Flavor 2: Hashed — content-addressed blackboard with epochs and cache alignment.
//!
//! `CREWAI_BLACKBOARD_FLAVOR=hashed`
//!
//! This is the architect's choice. Content-addressed entries with Merkle-style
//! hash chains, epoch-based snapshots for cache alignment, and configurable
//! pruning vs tombstoning. Uses DashMap for concurrent access.
//!
//! Storage: in-memory (DashMap) + optional SQLite for persistence.

use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};

use dashmap::DashMap;
use parking_lot::RwLock;

use super::entry::{BlackboardEntry, EntryType, EntryTier};
use super::store::{BlackboardStore, BlackboardQuery, BlackboardResult, BlackboardError, CompactionStats};
use super::snapshot::{BlackboardSnapshot, CacheThumbprint};
use super::BlackboardConfig;

/// Hashed blackboard — the content-addressed, epoch-aware implementation.
pub struct HashedBlackboard {
    config: BlackboardConfig,

    /// Live entries: hash → entry.
    live: DashMap<[u8; 32], BlackboardEntry>,

    /// Pending entries: posted since last epoch advance.
    /// On `advance_epoch()`, these move to `live` and a new snapshot is generated.
    pending: DashMap<[u8; 32], BlackboardEntry>,

    /// Secondary indices.
    by_type: DashMap<EntryType, Vec<[u8; 32]>>,
    by_author: DashMap<String, Vec<[u8; 32]>>,
    by_parent: DashMap<[u8; 32], Vec<[u8; 32]>>,

    /// Canonical ordering: the deterministic sequence that defines prompt order.
    /// Append-only within an epoch. Rebuilt on compaction.
    canonical_order: RwLock<Vec<[u8; 32]>>,

    /// Current epoch counter.
    epoch: AtomicU64,

    /// Cached snapshot (invalidated on epoch advance).
    cached_snapshot: RwLock<Option<BlackboardSnapshot>>,
}

impl HashedBlackboard {
    pub fn new(config: BlackboardConfig) -> Self {
        Self {
            config,
            live: DashMap::new(),
            pending: DashMap::new(),
            by_type: DashMap::new(),
            by_author: DashMap::new(),
            by_parent: DashMap::new(),
            canonical_order: RwLock::new(Vec::new()),
            epoch: AtomicU64::new(0),
            cached_snapshot: RwLock::new(None),
        }
    }

    /// Index an entry in the secondary indices.
    fn index_entry(&self, entry: &BlackboardEntry) {
        // By type
        self.by_type
            .entry(entry.entry_type)
            .or_default()
            .push(entry.content_hash);

        // By author
        self.by_author
            .entry(entry.author.clone())
            .or_default()
            .push(entry.content_hash);

        // By parent
        if let Some(parent) = entry.parent_hash {
            self.by_parent
                .entry(parent)
                .or_default()
                .push(entry.content_hash);
        }
    }

    /// Remove entry from secondary indices.
    fn deindex_entry(&self, entry: &BlackboardEntry) {
        if let Some(mut v) = self.by_type.get_mut(&entry.entry_type) {
            v.retain(|h| h != &entry.content_hash);
        }
        if let Some(mut v) = self.by_author.get_mut(&entry.author) {
            v.retain(|h| h != &entry.content_hash);
        }
        if let Some(parent) = entry.parent_hash {
            if let Some(mut v) = self.by_parent.get_mut(&parent) {
                v.retain(|h| h != &entry.content_hash);
            }
        }
    }

    /// Build snapshot from current live entries in canonical order.
    fn build_snapshot(&self) -> BlackboardSnapshot {
        let order = self.canonical_order.read();
        let ttl = chrono::Duration::seconds(self.config.stm_ttl_seconds as i64);

        let entries: Vec<BlackboardEntry> = order
            .iter()
            .filter_map(|hash| {
                self.live.get(hash).map(|e| e.clone())
            })
            .filter(|e| !e.tombstoned && !e.is_expired(ttl))
            .collect();

        let epoch = self.epoch.load(Ordering::Relaxed);
        BlackboardSnapshot::new(epoch, entries)
    }
}

impl BlackboardStore for HashedBlackboard {
    fn post(&self, entry: BlackboardEntry) -> BlackboardResult<[u8; 32]> {
        let hash = entry.content_hash;

        // Dedup: if this exact hash already exists, skip.
        if self.live.contains_key(&hash) || self.pending.contains_key(&hash) {
            return Ok(hash);
        }

        // Handle supersession: if this entry supersedes others, tombstone them.
        for superseded_hash in &entry.supersedes {
            if let Some(mut old) = self.live.get_mut(superseded_hash) {
                old.tombstoned = true;
            }
        }

        // Index the entry.
        self.index_entry(&entry);

        // Add to pending buffer (not yet in canonical snapshot).
        self.pending.insert(hash, entry);

        // Invalidate cached snapshot.
        *self.cached_snapshot.write() = None;

        Ok(hash)
    }

    fn get(&self, hash: &[u8; 32]) -> BlackboardResult<Option<BlackboardEntry>> {
        if let Some(entry) = self.live.get(hash) {
            return Ok(Some(entry.clone()));
        }
        if let Some(entry) = self.pending.get(hash) {
            return Ok(Some(entry.clone()));
        }
        Ok(None)
    }

    fn query(&self, q: &BlackboardQuery) -> BlackboardResult<Vec<BlackboardEntry>> {
        let ttl = chrono::Duration::seconds(self.config.stm_ttl_seconds as i64);

        // If querying by type, use the index.
        let candidate_hashes: Option<Vec<[u8; 32]>> = if let Some(ref types) = q.entry_types {
            let mut hashes = Vec::new();
            for t in types {
                if let Some(v) = self.by_type.get(t) {
                    hashes.extend(v.iter().copied());
                }
            }
            Some(hashes)
        } else if let Some(ref authors) = q.authors {
            let mut hashes = Vec::new();
            for a in authors {
                if let Some(v) = self.by_author.get(a) {
                    hashes.extend(v.iter().copied());
                }
            }
            Some(hashes)
        } else if let Some(ref parent) = q.parent_hash {
            self.by_parent.get(parent).map(|v| v.clone())
        } else {
            None // Full scan
        };

        let entries_iter: Box<dyn Iterator<Item = BlackboardEntry>> = if let Some(hashes) = candidate_hashes {
            Box::new(
                hashes.into_iter().filter_map(|h| {
                    self.live.get(&h).map(|e| e.clone())
                        .or_else(|| self.pending.get(&h).map(|e| e.clone()))
                })
            )
        } else {
            // Full scan over live + pending
            Box::new(
                self.live.iter().map(|e| e.value().clone())
                    .chain(self.pending.iter().map(|e| e.value().clone()))
            )
        };

        let results: Vec<BlackboardEntry> = entries_iter
            .filter(|e| {
                if !q.include_tombstoned && e.tombstoned { return false; }
                if e.is_expired(ttl) && !q.include_tombstoned { return false; }
                if e.confidence < q.min_confidence { return false; }
                if let Some(ref text) = q.text {
                    if !e.content.to_lowercase().contains(&text.to_lowercase()) {
                        return false;
                    }
                }
                true
            })
            .take(q.limit)
            .collect();

        Ok(results)
    }

    fn len(&self) -> usize {
        self.live.len() + self.pending.len()
    }

    fn snapshot(&self) -> BlackboardResult<BlackboardSnapshot> {
        // Return cached snapshot if available.
        if let Some(ref snap) = *self.cached_snapshot.read() {
            return Ok(snap.clone());
        }

        let snap = self.build_snapshot();
        *self.cached_snapshot.write() = Some(snap.clone());
        Ok(snap)
    }

    fn cache_thumbprint(&self) -> CacheThumbprint {
        self.snapshot()
            .map(|s| s.thumbprint)
            .unwrap_or_else(|_| CacheThumbprint::zero())
    }

    fn epoch(&self) -> u64 {
        self.epoch.load(Ordering::Relaxed)
    }

    fn advance_epoch(&self) -> u64 {
        // Move all pending entries into live.
        let mut order = self.canonical_order.write();

        for entry in self.pending.iter() {
            let hash = *entry.key();
            let e = entry.value().clone();
            self.live.insert(hash, e);
            order.push(hash);
        }
        self.pending.clear();

        // Invalidate cached snapshot.
        *self.cached_snapshot.write() = None;

        // Bump epoch.
        let new_epoch = self.epoch.fetch_add(1, Ordering::Relaxed) + 1;
        new_epoch
    }

    fn tombstone(&self, hash: &[u8; 32]) -> BlackboardResult<()> {
        if let Some(mut entry) = self.live.get_mut(hash) {
            entry.tombstoned = true;
            *self.cached_snapshot.write() = None;
            return Ok(());
        }
        if let Some(mut entry) = self.pending.get_mut(hash) {
            entry.tombstoned = true;
            return Ok(());
        }
        Err(BlackboardError::NotFound(format!("Entry not found")))
    }

    fn compact(&self) -> BlackboardResult<CompactionStats> {
        let ttl = chrono::Duration::seconds(self.config.stm_ttl_seconds as i64);
        let before = self.live.len();
        let mut tombstoned = 0;
        let mut pruned = 0;
        let mut superseded_removed = 0;

        // Collect hashes to remove.
        let to_remove: Vec<[u8; 32]> = self.live.iter()
            .filter(|e| {
                let entry = e.value();
                if entry.tombstoned {
                    tombstoned += 1;
                    return self.config.prune_expired; // Only physically remove if pruning
                }
                if entry.is_expired(ttl) {
                    pruned += 1;
                    return self.config.prune_expired;
                }
                false
            })
            .map(|e| *e.key())
            .collect();

        // Remove superseded entries that are also tombstoned.
        let superseded_to_remove: Vec<[u8; 32]> = self.live.iter()
            .filter(|e| {
                let entry = e.value();
                // Check if any other entry supersedes this one
                self.live.iter().any(|other| {
                    other.value().supersedes.contains(&entry.content_hash) && !other.value().tombstoned
                })
            })
            .filter(|e| e.value().tombstoned)
            .map(|e| *e.key())
            .collect();

        for hash in &to_remove {
            if let Some((_, entry)) = self.live.remove(hash) {
                self.deindex_entry(&entry);
            }
        }

        for hash in &superseded_to_remove {
            if let Some((_, entry)) = self.live.remove(hash) {
                self.deindex_entry(&entry);
                superseded_removed += 1;
            }
        }

        // Rebuild canonical order.
        {
            let mut order = self.canonical_order.write();
            order.retain(|h| self.live.contains_key(h));
        }

        // Enforce max_entries.
        if self.live.len() > self.config.max_entries {
            let excess = self.live.len() - self.config.max_entries;
            let order = self.canonical_order.read();
            let to_evict: Vec<[u8; 32]> = order.iter().take(excess).copied().collect();
            drop(order);
            for hash in &to_evict {
                if let Some((_, entry)) = self.live.remove(hash) {
                    self.deindex_entry(&entry);
                    pruned += 1;
                }
            }
            self.canonical_order.write().drain(..to_evict.len());
        }

        *self.cached_snapshot.write() = None;

        Ok(CompactionStats {
            entries_before: before,
            entries_after: self.live.len(),
            tombstoned,
            pruned,
            superseded_removed,
        })
    }

    fn clear(&self) -> BlackboardResult<()> {
        self.live.clear();
        self.pending.clear();
        self.by_type.clear();
        self.by_author.clear();
        self.by_parent.clear();
        self.canonical_order.write().clear();
        *self.cached_snapshot.write() = None;
        Ok(())
    }

    fn export_entries(&self, since_epoch: Option<u64>) -> BlackboardResult<Vec<BlackboardEntry>> {
        // For simplicity, export all live entries. A production impl would
        // tag entries with the epoch they were committed in.
        Ok(self.live.iter().map(|e| e.value().clone()).collect())
    }

    fn import_entries(&self, entries: Vec<BlackboardEntry>) -> BlackboardResult<Vec<[u8; 32]>> {
        let mut imported = Vec::new();
        for entry in entries {
            let hash = entry.content_hash;
            if !self.live.contains_key(&hash) && !self.pending.contains_key(&hash) {
                self.index_entry(&entry);
                self.pending.insert(hash, entry);
                imported.push(hash);
            }
        }
        if !imported.is_empty() {
            *self.cached_snapshot.write() = None;
        }
        Ok(imported)
    }

    fn build_context_for_task(&self, task_description: &str, additional_context: &str) -> String {
        // Use snapshot for consistent view
        match self.snapshot() {
            Ok(snap) => snap.as_prompt().to_string(),
            Err(_) => String::new(),
        }
    }

    fn flavor_name(&self) -> &'static str {
        "hashed"
    }

    fn stats(&self) -> HashMap<String, serde_json::Value> {
        let mut m = HashMap::new();
        m.insert("flavor".into(), serde_json::json!("hashed"));
        m.insert("live_entries".into(), serde_json::json!(self.live.len()));
        m.insert("pending_entries".into(), serde_json::json!(self.pending.len()));
        m.insert("epoch".into(), serde_json::json!(self.epoch()));
        m.insert("canonical_order_len".into(), serde_json::json!(self.canonical_order.read().len()));
        m.insert("thumbprint".into(), serde_json::json!(self.cache_thumbprint().hex()));
        m
    }
}
