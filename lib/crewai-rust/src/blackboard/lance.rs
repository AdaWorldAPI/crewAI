//! Flavor 3: Lance — LanceDB-backed blackboard with optional S3 backup.
//!
//! `CREWAI_BLACKBOARD_FLAVOR=lance`
//!
//! The production endgame. Uses LanceDB for:
//! - Vector similarity search (embedded entry content)
//! - Structured column queries (type, author, epoch, confidence)
//! - S3-backed persistence (optional, via `CREWAI_BLACKBOARD_LANCE_S3`)
//! - Zero-copy reads via memory-mapped Arrow tables
//!
//! ## Why LanceDB
//!
//! LanceDB combines vector search with structured queries in a single table.
//! For the blackboard this means:
//! - `query("market analysis")` does semantic search over entry content
//! - `by_type(Decision)` does columnar filter (no embedding needed)
//! - `snapshot()` reads the full table in insertion order (zero-copy Arrow)
//! - S3 backup is built-in (just point lance_path to `s3://bucket/prefix`)
//!
//! ## Dependency
//!
//! Requires the `lancedb` crate. Since this is optional/feature-gated,
//! the implementation below uses a trait-compatible stub that logs warnings
//! when Lance is not available, falling back to the `hashed` flavor.
//!
//! When `lancedb` is available, the real implementation uses:
//! ```toml
//! [dependencies]
//! lancedb = "0.15"        # or latest
//! arrow = { version = "53", features = ["prettyprint"] }
//! arrow-schema = "53"
//! ```

use std::collections::HashMap;

use super::entry::{BlackboardEntry, EntryType};
use super::store::{BlackboardStore, BlackboardQuery, BlackboardResult, BlackboardError, CompactionStats};
use super::snapshot::{BlackboardSnapshot, CacheThumbprint};
use super::hashed::HashedBlackboard;
use super::BlackboardConfig;

/// Lance-flavored blackboard.
///
/// When lancedb crate is available: uses LanceDB tables with Arrow schema.
/// When not available: falls back to HashedBlackboard with a warning.
pub struct LanceBlackboard {
    config: BlackboardConfig,
    /// Fallback: hashed blackboard for when lancedb is not compiled in.
    /// In a real build with `lancedb` feature, this would be replaced by:
    /// ```rust
    /// db: lancedb::Connection,
    /// table: lancedb::Table,
    /// embedder: Box<dyn EmbedFunction>,
    /// ```
    inner: HashedBlackboard,
    lance_available: bool,
}

impl LanceBlackboard {
    pub fn new(config: BlackboardConfig) -> Self {
        // Try to initialize LanceDB. If the crate isn't available,
        // fall back to hashed mode.
        let lance_available = Self::try_init_lance(&config);

        if !lance_available {
            log::warn!(
                "LanceDB not available — falling back to hashed blackboard. \
                 To enable Lance, add `lancedb` to Cargo.toml dependencies."
            );
        } else {
            log::info!(
                "LanceDB blackboard initialized at: {} (S3: {})",
                config.lance_path,
                config.lance_s3_uri.as_deref().unwrap_or("none")
            );
        }

        Self {
            inner: HashedBlackboard::new(config.clone()),
            config,
            lance_available,
        }
    }

    /// Attempt to initialize LanceDB connection.
    ///
    /// Production implementation:
    /// ```rust
    /// fn try_init_lance(config: &BlackboardConfig) -> bool {
    ///     let uri = config.lance_s3_uri.as_deref()
    ///         .unwrap_or(&config.lance_path);
    ///
    ///     let db = lancedb::connect(uri).execute().await?;
    ///
    ///     // Create or open table with schema:
    ///     // | content_hash BINARY(32) PK | author VARCHAR | entry_type VARCHAR |
    ///     // | content TEXT | embedding VECTOR(1536) | confidence FLOAT |
    ///     // | parent_hash BINARY(32) | epoch UINT64 | created_at TIMESTAMP |
    ///     // | tombstoned BOOLEAN | metadata JSON | tier VARCHAR |
    ///
    ///     let schema = Arc::new(Schema::new(vec![
    ///         Field::new("content_hash", DataType::FixedSizeBinary(32), false),
    ///         Field::new("author", DataType::Utf8, false),
    ///         Field::new("entry_type", DataType::Utf8, false),
    ///         Field::new("content", DataType::Utf8, false),
    ///         Field::new("embedding", DataType::FixedSizeList(
    ///             Box::new(Field::new("item", DataType::Float32, false)), 1536
    ///         ), true),
    ///         Field::new("confidence", DataType::Float64, false),
    ///         Field::new("parent_hash", DataType::FixedSizeBinary(32), true),
    ///         Field::new("epoch", DataType::UInt64, false),
    ///         Field::new("created_at", DataType::Timestamp(TimeUnit::Microsecond, None), false),
    ///         Field::new("tombstoned", DataType::Boolean, false),
    ///         Field::new("metadata", DataType::Utf8, true),
    ///         Field::new("tier", DataType::Utf8, false),
    ///     ]));
    ///
    ///     let table = db.create_table("blackboard", schema)
    ///         .execute().await
    ///         .or_else(|_| db.open_table("blackboard").execute().await)?;
    ///
    ///     true
    /// }
    /// ```
    fn try_init_lance(_config: &BlackboardConfig) -> bool {
        // Stub: return false until lancedb crate is added as dependency.
        // When lancedb is in Cargo.toml, replace this with actual init.
        false
    }

    // ── Lance-specific operations (available only when lance_available) ──

    /// Vector similarity search over entry content.
    ///
    /// Production implementation:
    /// ```rust
    /// pub async fn vector_search(&self, query_embedding: &[f32], limit: usize) -> Vec<BlackboardEntry> {
    ///     self.table.search(query_embedding)
    ///         .limit(limit)
    ///         .filter("tombstoned = false")
    ///         .execute().await
    ///         .map(|results| results.into_iter().map(Self::row_to_entry).collect())
    ///         .unwrap_or_default()
    /// }
    /// ```
    pub fn vector_search(&self, _query: &str, _limit: usize) -> Vec<BlackboardEntry> {
        if !self.lance_available {
            log::warn!("vector_search called but Lance not available");
        }
        Vec::new()
    }

    /// Columnar filter — fast structured queries without embeddings.
    ///
    /// Production:
    /// ```rust
    /// pub async fn filter(&self, predicate: &str, limit: usize) -> Vec<BlackboardEntry> {
    ///     self.table.search(None)  // no vector search
    ///         .filter(predicate)   // e.g. "entry_type = 'decision' AND confidence > 0.8"
    ///         .limit(limit)
    ///         .execute().await
    ///         .map(|results| results.into_iter().map(Self::row_to_entry).collect())
    ///         .unwrap_or_default()
    /// }
    /// ```
    pub fn filter(&self, _predicate: &str, _limit: usize) -> Vec<BlackboardEntry> {
        if !self.lance_available {
            log::warn!("filter called but Lance not available");
        }
        Vec::new()
    }

    /// Create a Lance index on the embedding column for faster ANN search.
    ///
    /// Call this after bulk loading entries. Lance's IVF_PQ index gives
    /// sub-millisecond search over millions of entries.
    pub fn create_index(&self) -> BlackboardResult<()> {
        if !self.lance_available {
            return Err(BlackboardError::Lance("Lance not available".into()));
        }
        // Production: self.table.create_index(&["embedding"]).ivf_pq().execute().await?
        Ok(())
    }

    /// Compact Lance table — merges small row groups, reclaims deleted rows.
    /// Lance's compaction is different from logical compaction (tombstone removal).
    pub fn compact_lance(&self) -> BlackboardResult<()> {
        if !self.lance_available {
            return Err(BlackboardError::Lance("Lance not available".into()));
        }
        // Production: self.table.compact_files().execute().await?
        Ok(())
    }
}

// ── Delegate to inner HashedBlackboard (or Lance when available) ────────────

impl BlackboardStore for LanceBlackboard {
    fn post(&self, entry: BlackboardEntry) -> BlackboardResult<[u8; 32]> {
        // When lance is available: insert into Lance table AND in-memory index.
        // For now: delegate to hashed.
        self.inner.post(entry)
    }

    fn get(&self, hash: &[u8; 32]) -> BlackboardResult<Option<BlackboardEntry>> {
        self.inner.get(hash)
    }

    fn query(&self, q: &BlackboardQuery) -> BlackboardResult<Vec<BlackboardEntry>> {
        // When lance is available and q.text is set: use vector_search.
        // Otherwise: delegate to hashed.
        self.inner.query(q)
    }

    fn len(&self) -> usize {
        self.inner.len()
    }

    fn snapshot(&self) -> BlackboardResult<BlackboardSnapshot> {
        self.inner.snapshot()
    }

    fn cache_thumbprint(&self) -> CacheThumbprint {
        self.inner.cache_thumbprint()
    }

    fn epoch(&self) -> u64 {
        self.inner.epoch()
    }

    fn advance_epoch(&self) -> u64 {
        self.inner.advance_epoch()
    }

    fn tombstone(&self, hash: &[u8; 32]) -> BlackboardResult<()> {
        self.inner.tombstone(hash)
    }

    fn compact(&self) -> BlackboardResult<CompactionStats> {
        let stats = self.inner.compact()?;
        // When lance is available: also compact_lance()
        if self.lance_available {
            let _ = self.compact_lance();
        }
        Ok(stats)
    }

    fn clear(&self) -> BlackboardResult<()> {
        self.inner.clear()
    }

    fn export_entries(&self, since_epoch: Option<u64>) -> BlackboardResult<Vec<BlackboardEntry>> {
        self.inner.export_entries(since_epoch)
    }

    fn import_entries(&self, entries: Vec<BlackboardEntry>) -> BlackboardResult<Vec<[u8; 32]>> {
        self.inner.import_entries(entries)
    }

    fn build_context_for_task(&self, task_description: &str, additional_context: &str) -> String {
        self.inner.build_context_for_task(task_description, additional_context)
    }

    fn flavor_name(&self) -> &'static str {
        if self.lance_available { "lance" } else { "lance (fallback: hashed)" }
    }

    fn stats(&self) -> HashMap<String, serde_json::Value> {
        let mut m = self.inner.stats();
        m.insert("flavor".into(), serde_json::json!(self.flavor_name()));
        m.insert("lance_available".into(), serde_json::json!(self.lance_available));
        m.insert("lance_path".into(), serde_json::json!(self.config.lance_path));
        m.insert("lance_s3".into(), serde_json::json!(self.config.lance_s3_uri));
        m
    }
}
