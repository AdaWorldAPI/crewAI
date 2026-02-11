//! Blackboard — shared workspace for multi-agent A2A orchestration.
//!
//! Three flavors selected via `CREWAI_BLACKBOARD_FLAVOR` env var:
//!
//! | Value       | Backend                           | Description                                    |
//! |-------------|-----------------------------------|------------------------------------------------|
//! | `original`  | ContextualMemory (STM/LTM/E/Ext)  | Drop-in crewAI compatible. Default.            |
//! | `hashed`    | Content-addressed blackboard       | Epochs, fingerprints, cache thumbprints.       |
//! | `lance`     | LanceDB + optional S3              | Vector + structured, production endgame.       |
//!
//! All three implement `BlackboardStore`, so the execution layer is agnostic.
//!
//! ## ladybug-rs integration
//!
//! The `BlackboardStore` trait is designed to be re-exported by ladybug-rs
//! without modification. ladybug-rs adds BindSpace addressing, NARS-revise
//! conflict resolution, and GrammarTriangle fingerprinting on top. crewai-rust
//! owns the trait; ladybug-rs owns the advanced operations.

pub mod entry;
pub mod store;
pub mod original;
pub mod hashed;
pub mod lance;
pub mod snapshot;
pub mod cache;

use std::sync::OnceLock;

pub use entry::{BlackboardEntry, EntryType};
pub use store::BlackboardStore;
pub use snapshot::BlackboardSnapshot;
pub use cache::CacheThumbprint;

/// Global flavor selection, resolved once from env.
static FLAVOR: OnceLock<BlackboardFlavor> = OnceLock::new();

/// Which blackboard implementation to use.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BlackboardFlavor {
    /// Drop-in: wraps crewAI's ContextualMemory (STM/LTM/Entity/External).
    /// No content hashing, no epochs. Just concatenated search results.
    Original,

    /// Content-addressed blackboard with epochs, Merkle chains, cache alignment.
    /// Uses SQLite + optional RAG for storage. Pruning configurable.
    Hashed,

    /// LanceDB-backed blackboard. Vector search + structured columns + S3 backup.
    /// Full production tier. Requires `lancedb` feature or runtime dependency.
    Lance,
}

impl BlackboardFlavor {
    /// Resolve from `CREWAI_BLACKBOARD_FLAVOR` env var. Defaults to `original`.
    pub fn from_env() -> Self {
        match std::env::var("CREWAI_BLACKBOARD_FLAVOR")
            .unwrap_or_default()
            .to_lowercase()
            .as_str()
        {
            "hashed" | "blackboard" | "content-addressed" => Self::Hashed,
            "lance" | "lancedb" => Self::Lance,
            _ => Self::Original,
        }
    }

    /// Get the globally resolved flavor (cached after first call).
    pub fn global() -> Self {
        *FLAVOR.get_or_init(Self::from_env)
    }
}

/// Configuration for blackboard behavior, also env-driven.
#[derive(Debug, Clone)]
pub struct BlackboardConfig {
    pub flavor: BlackboardFlavor,

    /// Hashed flavor: whether to prune expired entries or tombstone them.
    /// `CREWAI_BLACKBOARD_PRUNE=true` → delete expired entries (breaks Merkle chain).
    /// `CREWAI_BLACKBOARD_PRUNE=false` → tombstone (preserves chain, costs storage).
    /// Default: false (tombstone).
    pub prune_expired: bool,

    /// Hashed flavor: whether to use a separate SQLite DB or share with LTM.
    /// `CREWAI_BLACKBOARD_SEPARATE_DB=true` → own blackboard.db.
    /// `CREWAI_BLACKBOARD_SEPARATE_DB=false` → tables in crewai LTM database.
    /// Default: true.
    pub separate_db: bool,

    /// Lance flavor: S3 URI for remote backup.
    /// `CREWAI_BLACKBOARD_LANCE_S3=s3://bucket/path`
    /// If unset, Lance uses local directory only.
    pub lance_s3_uri: Option<String>,

    /// Lance flavor: local directory for Lance tables.
    /// `CREWAI_BLACKBOARD_LANCE_PATH=./blackboard_lance`
    /// Default: `./blackboard_lance`
    pub lance_path: String,

    /// Max entries before compaction (hashed + lance flavors).
    /// `CREWAI_BLACKBOARD_MAX_ENTRIES=10000`
    /// Default: 10_000.
    pub max_entries: usize,

    /// Default TTL for STM-tier entries in seconds.
    /// `CREWAI_BLACKBOARD_STM_TTL=3600`
    /// Default: 3600 (1 hour). 0 = no expiry.
    pub stm_ttl_seconds: u64,
}

impl Default for BlackboardConfig {
    fn default() -> Self {
        Self {
            flavor: BlackboardFlavor::global(),
            prune_expired: std::env::var("CREWAI_BLACKBOARD_PRUNE")
                .map(|v| v == "true" || v == "1")
                .unwrap_or(false),
            separate_db: std::env::var("CREWAI_BLACKBOARD_SEPARATE_DB")
                .map(|v| v != "false" && v != "0")
                .unwrap_or(true),
            lance_s3_uri: std::env::var("CREWAI_BLACKBOARD_LANCE_S3").ok(),
            lance_path: std::env::var("CREWAI_BLACKBOARD_LANCE_PATH")
                .unwrap_or_else(|_| "./blackboard_lance".to_string()),
            max_entries: std::env::var("CREWAI_BLACKBOARD_MAX_ENTRIES")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(10_000),
            stm_ttl_seconds: std::env::var("CREWAI_BLACKBOARD_STM_TTL")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(3600),
        }
    }
}

/// Factory: create the right blackboard from config.
pub fn create_blackboard(config: BlackboardConfig) -> Box<dyn BlackboardStore> {
    match config.flavor {
        BlackboardFlavor::Original => Box::new(original::OriginalBlackboard::new(config)),
        BlackboardFlavor::Hashed => Box::new(hashed::HashedBlackboard::new(config)),
        BlackboardFlavor::Lance => Box::new(lance::LanceBlackboard::new(config)),
    }
}

/// Convenience: create blackboard from env vars with defaults.
pub fn create_blackboard_from_env() -> Box<dyn BlackboardStore> {
    create_blackboard(BlackboardConfig::default())
}
