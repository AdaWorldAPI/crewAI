//! BlackboardStore — the agnostic trait that all three flavors implement.
//!
//! crewai-rust codes to this trait. ladybug-rs codes to this trait.
//! The env var picks the backend. Everything above is unaware.

use std::collections::HashMap;

use super::entry::{BlackboardEntry, EntryType};
use super::snapshot::BlackboardSnapshot;
use super::cache::CacheThumbprint;

/// Result type for blackboard operations.
pub type BlackboardResult<T> = Result<T, BlackboardError>;

/// Blackboard errors.
#[derive(Debug, thiserror::Error)]
pub enum BlackboardError {
    #[error("Entry not found: {0}")]
    NotFound(String),

    #[error("Policy denied: {0}")]
    PolicyDenied(String),

    #[error("Storage error: {0}")]
    Storage(String),

    #[error("Serialization error: {0}")]
    Serialization(String),

    #[error("Lance error: {0}")]
    Lance(String),

    #[error("Sync error: {0}")]
    Sync(String),
}

/// Query parameters for searching the blackboard.
#[derive(Debug, Clone, Default)]
pub struct BlackboardQuery {
    /// Semantic search query (content similarity).
    pub text: Option<String>,

    /// Filter by entry type.
    pub entry_types: Option<Vec<EntryType>>,

    /// Filter by author fingerprint.
    pub authors: Option<Vec<String>>,

    /// Filter by parent hash (direct children only).
    pub parent_hash: Option<[u8; 32]>,

    /// Include tombstoned entries?
    pub include_tombstoned: bool,

    /// Max results.
    pub limit: usize,

    /// Minimum confidence threshold.
    pub min_confidence: f64,

    /// Only entries from this epoch or later (hashed/lance flavors).
    pub min_epoch: Option<u64>,
}

impl BlackboardQuery {
    pub fn new(text: impl Into<String>) -> Self {
        Self {
            text: Some(text.into()),
            limit: 10,
            ..Default::default()
        }
    }

    pub fn by_type(entry_type: EntryType) -> Self {
        Self {
            entry_types: Some(vec![entry_type]),
            limit: 50,
            ..Default::default()
        }
    }

    pub fn by_author(author: impl Into<String>) -> Self {
        Self {
            authors: Some(vec![author.into()]),
            limit: 50,
            ..Default::default()
        }
    }

    pub fn children_of(parent: [u8; 32]) -> Self {
        Self {
            parent_hash: Some(parent),
            limit: 50,
            ..Default::default()
        }
    }

    pub fn with_limit(mut self, limit: usize) -> Self {
        self.limit = limit;
        self
    }

    pub fn with_min_confidence(mut self, c: f64) -> Self {
        self.min_confidence = c;
        self
    }
}

/// Compaction statistics returned by `compact()`.
#[derive(Debug, Clone, Default)]
pub struct CompactionStats {
    pub entries_before: usize,
    pub entries_after: usize,
    pub tombstoned: usize,
    pub pruned: usize,
    pub superseded_removed: usize,
}

/// Sync direction for A2A blackboard sharing.
#[derive(Debug, Clone, Copy)]
pub enum SyncDirection {
    /// Send local entries to remote.
    Push,
    /// Receive remote entries.
    Pull,
    /// Bidirectional merge.
    Both,
}

// ─── The Trait ───────────────────────────────────────────────────────────────

/// The trait that all three blackboard flavors implement.
///
/// crewai-rust and ladybug-rs program to this interface.
/// The runtime env var picks which implementation backs it.
pub trait BlackboardStore: Send + Sync {
    // ── Write ───────────────────────────────────────────────────────

    /// Post an entry to the blackboard. Returns the content hash.
    ///
    /// For `original` flavor: wraps into a memory save.
    /// For `hashed` flavor: appends to pending buffer (pre-epoch).
    /// For `lance` flavor: inserts into Lance table with embedding.
    fn post(&self, entry: BlackboardEntry) -> BlackboardResult<[u8; 32]>;

    /// Post multiple entries atomically.
    fn post_batch(&self, entries: Vec<BlackboardEntry>) -> BlackboardResult<Vec<[u8; 32]>> {
        entries.into_iter().map(|e| self.post(e)).collect()
    }

    // ── Read ────────────────────────────────────────────────────────

    /// Get a single entry by content hash.
    fn get(&self, hash: &[u8; 32]) -> BlackboardResult<Option<BlackboardEntry>>;

    /// Query entries matching filter criteria.
    fn query(&self, q: &BlackboardQuery) -> BlackboardResult<Vec<BlackboardEntry>>;

    /// Get all entries of a specific type.
    fn by_type(&self, t: EntryType) -> BlackboardResult<Vec<BlackboardEntry>> {
        self.query(&BlackboardQuery::by_type(t))
    }

    /// Get all entries by a specific author.
    fn by_author(&self, author: &str) -> BlackboardResult<Vec<BlackboardEntry>> {
        self.query(&BlackboardQuery::by_author(author))
    }

    /// Count total active (non-tombstoned) entries.
    fn len(&self) -> usize;

    fn is_empty(&self) -> bool {
        self.len() == 0
    }

    // ── Snapshot (for prompt construction) ───────────────────────────

    /// Create an immutable snapshot for LLM prompt injection.
    ///
    /// The snapshot has a deterministic ordering that maximizes LLM cache hits.
    /// Agents working against the same snapshot share the same prompt prefix.
    ///
    /// For `original` flavor: builds context string from memory search results.
    /// For `hashed` flavor: returns entries in canonical hash-chain order.
    /// For `lance` flavor: returns entries in insertion order with embeddings.
    fn snapshot(&self) -> BlackboardResult<BlackboardSnapshot>;

    /// Get the cache thumbprint — hash of the current canonical entry ordering.
    ///
    /// If two agents see the same thumbprint, they share a prompt cache.
    /// For `original` flavor: hash of the concatenated context string.
    fn cache_thumbprint(&self) -> CacheThumbprint;

    // ── Epoch management (hashed + lance only) ──────────────────────

    /// Current epoch number. Returns 0 for `original` flavor.
    fn epoch(&self) -> u64 {
        0
    }

    /// Advance the epoch: promote pending entries to the snapshot.
    /// Returns new epoch number. No-op for `original` flavor.
    fn advance_epoch(&self) -> u64 {
        0
    }

    // ── Lifecycle ───────────────────────────────────────────────────

    /// Tombstone an entry (logical delete, preserves hash chain).
    fn tombstone(&self, hash: &[u8; 32]) -> BlackboardResult<()>;

    /// Compact the blackboard: remove tombstoned entries if pruning is enabled,
    /// remove superseded entries, enforce max_entries.
    fn compact(&self) -> BlackboardResult<CompactionStats>;

    /// Clear all entries. Use with caution.
    fn clear(&self) -> BlackboardResult<()>;

    // ── A2A Sync ────────────────────────────────────────────────────

    /// Export entries as serialized bytes for A2A transfer.
    /// Entries are identified by content hash — receiving end deduplicates.
    fn export_entries(
        &self,
        since_epoch: Option<u64>,
    ) -> BlackboardResult<Vec<BlackboardEntry>>;

    /// Import entries from a remote blackboard (A2A sync).
    /// Deduplicates by content hash. Returns hashes of newly imported entries.
    fn import_entries(
        &self,
        entries: Vec<BlackboardEntry>,
    ) -> BlackboardResult<Vec<[u8; 32]>>;

    // ── Context string (crewAI compatibility) ───────────────────────

    /// Build a context string for task prompt injection.
    ///
    /// This is the bridge to crewAI's `ContextualMemory.build_context_for_task()`.
    /// All three flavors produce the same output format:
    /// "Recent Insights:\n- ...\nDecisions:\n- ...\nEntities:\n- ..."
    fn build_context_for_task(
        &self,
        task_description: &str,
        additional_context: &str,
    ) -> String;

    // ── Diagnostics ─────────────────────────────────────────────────

    /// Flavor name for logging/debugging.
    fn flavor_name(&self) -> &'static str;

    /// Storage statistics.
    fn stats(&self) -> HashMap<String, serde_json::Value> {
        let mut m = HashMap::new();
        m.insert("flavor".into(), serde_json::Value::String(self.flavor_name().into()));
        m.insert("entries".into(), serde_json::json!(self.len()));
        m.insert("epoch".into(), serde_json::json!(self.epoch()));
        m
    }
}
