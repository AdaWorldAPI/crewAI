//! Flavor 1: Original — drop-in wrapper around crewAI's ContextualMemory.
//!
//! `CREWAI_BLACKBOARD_FLAVOR=original` (default)
//!
//! This flavor makes the blackboard invisible. It wraps the existing
//! STM/LTM/Entity/External memory system and presents it through the
//! `BlackboardStore` trait. No content hashing, no epochs, no cache alignment.
//! Just the same concatenated search results crewAI has always used.
//!
//! Use this when you want zero behavior change from stock crewAI.

use std::collections::HashMap;
use std::sync::RwLock;

use super::entry::{BlackboardEntry, EntryType, EntryTier};
use super::store::{BlackboardStore, BlackboardQuery, BlackboardResult, BlackboardError, CompactionStats};
use super::snapshot::{BlackboardSnapshot, CacheThumbprint};
use super::BlackboardConfig;

/// Original-flavor blackboard: thin wrapper over crewAI memory.
pub struct OriginalBlackboard {
    config: BlackboardConfig,
    /// In-memory store for entries posted via the BlackboardStore trait.
    /// These supplement (not replace) the crewAI memory system.
    entries: RwLock<Vec<BlackboardEntry>>,
}

impl OriginalBlackboard {
    pub fn new(config: BlackboardConfig) -> Self {
        Self {
            config,
            entries: RwLock::new(Vec::new()),
        }
    }
}

impl BlackboardStore for OriginalBlackboard {
    fn post(&self, entry: BlackboardEntry) -> BlackboardResult<[u8; 32]> {
        let hash = entry.content_hash;
        let mut entries = self.entries.write()
            .map_err(|e| BlackboardError::Storage(format!("Lock poisoned: {}", e)))?;

        // Dedup by hash (even in original mode, don't store exact duplicates)
        if !entries.iter().any(|e| e.content_hash == hash) {
            entries.push(entry);
        }
        Ok(hash)
    }

    fn get(&self, hash: &[u8; 32]) -> BlackboardResult<Option<BlackboardEntry>> {
        let entries = self.entries.read()
            .map_err(|e| BlackboardError::Storage(format!("Lock poisoned: {}", e)))?;
        Ok(entries.iter().find(|e| &e.content_hash == hash).cloned())
    }

    fn query(&self, q: &BlackboardQuery) -> BlackboardResult<Vec<BlackboardEntry>> {
        let entries = self.entries.read()
            .map_err(|e| BlackboardError::Storage(format!("Lock poisoned: {}", e)))?;

        let ttl = chrono::Duration::seconds(self.config.stm_ttl_seconds as i64);
        let results: Vec<BlackboardEntry> = entries
            .iter()
            .filter(|e| {
                if !q.include_tombstoned && e.tombstoned { return false; }
                if e.is_expired(ttl) && !q.include_tombstoned { return false; }
                if e.confidence < q.min_confidence { return false; }
                if let Some(ref types) = q.entry_types {
                    if !types.contains(&e.entry_type) { return false; }
                }
                if let Some(ref authors) = q.authors {
                    if !authors.contains(&e.author) { return false; }
                }
                if let Some(ref parent) = q.parent_hash {
                    if e.parent_hash.as_ref() != Some(parent) { return false; }
                }
                if let Some(ref text) = q.text {
                    // Simple substring search for original flavor
                    if !e.content.to_lowercase().contains(&text.to_lowercase()) {
                        return false;
                    }
                }
                true
            })
            .take(q.limit)
            .cloned()
            .collect();

        Ok(results)
    }

    fn len(&self) -> usize {
        self.entries.read().map(|e| e.len()).unwrap_or(0)
    }

    fn snapshot(&self) -> BlackboardResult<BlackboardSnapshot> {
        let entries = self.entries.read()
            .map_err(|e| BlackboardError::Storage(format!("Lock poisoned: {}", e)))?;
        let ttl = chrono::Duration::seconds(self.config.stm_ttl_seconds as i64);
        let active: Vec<BlackboardEntry> = entries
            .iter()
            .filter(|e| !e.tombstoned && !e.is_expired(ttl))
            .cloned()
            .collect();
        Ok(BlackboardSnapshot::new(0, active))
    }

    fn cache_thumbprint(&self) -> CacheThumbprint {
        self.snapshot()
            .map(|s| s.thumbprint)
            .unwrap_or_else(|_| CacheThumbprint::zero())
    }

    fn tombstone(&self, hash: &[u8; 32]) -> BlackboardResult<()> {
        let mut entries = self.entries.write()
            .map_err(|e| BlackboardError::Storage(format!("Lock poisoned: {}", e)))?;
        if let Some(entry) = entries.iter_mut().find(|e| &e.content_hash == hash) {
            entry.tombstoned = true;
            Ok(())
        } else {
            Err(BlackboardError::NotFound(format!("Entry not found")))
        }
    }

    fn compact(&self) -> BlackboardResult<CompactionStats> {
        let mut entries = self.entries.write()
            .map_err(|e| BlackboardError::Storage(format!("Lock poisoned: {}", e)))?;

        let before = entries.len();
        let ttl = chrono::Duration::seconds(self.config.stm_ttl_seconds as i64);

        // In original mode, just remove tombstoned and expired entries
        let tombstoned = entries.iter().filter(|e| e.tombstoned).count();
        let expired = entries.iter().filter(|e| e.is_expired(ttl)).count();
        entries.retain(|e| !e.tombstoned && !e.is_expired(ttl));

        Ok(CompactionStats {
            entries_before: before,
            entries_after: entries.len(),
            tombstoned,
            pruned: expired,
            superseded_removed: 0,
        })
    }

    fn clear(&self) -> BlackboardResult<()> {
        let mut entries = self.entries.write()
            .map_err(|e| BlackboardError::Storage(format!("Lock poisoned: {}", e)))?;
        entries.clear();
        Ok(())
    }

    fn export_entries(&self, _since_epoch: Option<u64>) -> BlackboardResult<Vec<BlackboardEntry>> {
        let entries = self.entries.read()
            .map_err(|e| BlackboardError::Storage(format!("Lock poisoned: {}", e)))?;
        Ok(entries.clone())
    }

    fn import_entries(&self, entries: Vec<BlackboardEntry>) -> BlackboardResult<Vec<[u8; 32]>> {
        let mut imported = Vec::new();
        for entry in entries {
            let hash = self.post(entry)?;
            imported.push(hash);
        }
        Ok(imported)
    }

    fn build_context_for_task(&self, task_description: &str, additional_context: &str) -> String {
        // Original flavor: just query by task description and format as bullet points
        let query = format!("{} {}", task_description, additional_context).trim().to_string();
        if query.is_empty() {
            return String::new();
        }

        let results = self.query(&BlackboardQuery::new(&query).with_limit(10))
            .unwrap_or_default();

        if results.is_empty() {
            return String::new();
        }

        let formatted: Vec<String> = results
            .iter()
            .map(|e| format!("- {}", e.content))
            .collect();

        format!("Blackboard Context:\n{}", formatted.join("\n"))
    }

    fn flavor_name(&self) -> &'static str {
        "original"
    }
}
