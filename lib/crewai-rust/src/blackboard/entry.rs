//! Blackboard entry — the atomic unit of shared agent state.
//!
//! Content-addressed via SHA-256(author_fingerprint + content + parent_hash).
//! Used by `hashed` and `lance` flavors. The `original` flavor wraps
//! crewAI memory items into this format for trait compatibility.

use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// Re-use crewAI's existing fingerprint and policy types.
// In the actual crate these would be:
//   use crate::security::fingerprint::Fingerprint;
//   use crate::policy::PolicyDecision;
// Placeholder types here for standalone readability:
type Fingerprint = String;
type PolicyDecision = serde_json::Value;

/// What kind of assertion this entry represents.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EntryType {
    /// Verified or sourced information.
    Fact,
    /// Proposed interpretation, not yet confirmed.
    Hypothesis,
    /// Committed decision (ice-caked in ladybug-rs terms).
    Decision,
    /// Explicit rejection of another entry.
    Veto,
    /// Incomplete work product, awaiting continuation.
    Partial,
    /// Request for information from other agents.
    Query,
    /// Observation from tool execution.
    Observation,
    /// Agent's reasoning trace (optional, for auditability).
    Reasoning,
}

/// Tier determines TTL behavior and storage priority.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EntryTier {
    /// Short-lived, expires per config TTL. Observations, partial results.
    Stm,
    /// Session-scoped. Survives within a crew run but not across runs.
    Session,
    /// Persistent. Survives across crew runs. Decisions, verified facts.
    Ltm,
}

impl Default for EntryTier {
    fn default() -> Self {
        Self::Session
    }
}

/// A single blackboard entry.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlackboardEntry {
    // ── Identity ────────────────────────────────────────────────────

    /// Content hash: SHA-256(author + content + parent_hash).
    /// This IS the primary key and the cache reference thumbprint.
    /// Computed on construction, never changes.
    pub content_hash: [u8; 32],

    /// Who wrote this entry. Maps to crewai-rust `Fingerprint`.
    pub author: Fingerprint,

    /// What kind of assertion.
    pub entry_type: EntryType,

    /// Storage tier (determines TTL and persistence behavior).
    #[serde(default)]
    pub tier: EntryTier,

    // ── Content ─────────────────────────────────────────────────────

    /// The actual content. This is what gets injected into LLM prompts.
    pub content: String,

    /// Structured metadata (tool name, task ID, confidence breakdown, etc.).
    #[serde(default)]
    pub metadata: HashMap<String, serde_json::Value>,

    // ── Structural links ────────────────────────────────────────────

    /// Hash of the entry this refines, responds to, or extends.
    /// `None` for root entries.
    pub parent_hash: Option<[u8; 32]>,

    /// Entries this explicitly replaces (supersession chain).
    #[serde(default)]
    pub supersedes: Vec<[u8; 32]>,

    /// Entries that support this conclusion (evidence chain).
    #[serde(default)]
    pub evidence: Vec<[u8; 32]>,

    // ── Cache alignment ─────────────────────────────────────────────

    /// Hash of the LLM prompt prefix this entry was generated within.
    /// Used to track which cache epoch produced this reasoning.
    /// Maps to Anthropic's `cache_creation_input_tokens` context.
    pub prompt_prefix_hash: Option<[u8; 32]>,

    // ── Policy & audit ──────────────────────────────────────────────

    /// The policy decision that authorized this write.
    /// `None` for `original` flavor (no policy checks).
    /// For `hashed`/`lance`: PolicyEngine evaluates BlackboardCommit action.
    pub policy_audit: Option<PolicyDecision>,

    // ── Confidence ──────────────────────────────────────────────────

    /// Agent's self-assessed confidence [0.0, 1.0].
    pub confidence: f64,

    // ── Lifecycle ───────────────────────────────────────────────────

    pub created_at: DateTime<Utc>,

    /// Explicit TTL override. If `None`, uses tier default from config.
    pub ttl: Option<Duration>,

    /// Whether this entry has been tombstoned (logically deleted).
    /// When `prune_expired=false`, expired entries get this flag
    /// instead of physical deletion, preserving the hash chain.
    #[serde(default)]
    pub tombstoned: bool,
}

impl BlackboardEntry {
    /// Create a new entry. Computes content_hash automatically.
    pub fn new(
        author: Fingerprint,
        entry_type: EntryType,
        content: impl Into<String>,
        parent_hash: Option<[u8; 32]>,
    ) -> Self {
        let content = content.into();
        let content_hash = Self::compute_hash(&author, &content, parent_hash.as_ref());
        Self {
            content_hash,
            author,
            entry_type,
            tier: EntryTier::default(),
            content,
            metadata: HashMap::new(),
            parent_hash,
            supersedes: Vec::new(),
            evidence: Vec::new(),
            prompt_prefix_hash: None,
            policy_audit: None,
            confidence: 1.0,
            created_at: Utc::now(),
            ttl: None,
            tombstoned: false,
        }
    }

    /// Compute SHA-256(author + content + parent_hash).
    fn compute_hash(
        author: &str,
        content: &str,
        parent: Option<&[u8; 32]>,
    ) -> [u8; 32] {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        // NOTE: In production, use SHA-256 via `sha2` crate.
        // Using a simulated hash here for zero-dependency compilation.
        // The trait contract is: deterministic, collision-resistant, 32 bytes.
        let mut hasher = DefaultHasher::new();
        author.hash(&mut hasher);
        content.hash(&mut hasher);
        if let Some(p) = parent {
            p.hash(&mut hasher);
        }
        let h = hasher.finish();
        let mut out = [0u8; 32];
        out[..8].copy_from_slice(&h.to_le_bytes());
        // Fill remaining bytes with secondary hash for better distribution
        content.len().hash(&mut hasher);
        let h2 = hasher.finish();
        out[8..16].copy_from_slice(&h2.to_le_bytes());
        author.len().hash(&mut hasher);
        let h3 = hasher.finish();
        out[16..24].copy_from_slice(&h3.to_le_bytes());
        if let Some(p) = parent {
            p[0..8].hash(&mut hasher);
        }
        let h4 = hasher.finish();
        out[24..32].copy_from_slice(&h4.to_le_bytes());
        out
    }

    // ── Builder methods ─────────────────────────────────────────────

    pub fn with_tier(mut self, tier: EntryTier) -> Self {
        self.tier = tier;
        self
    }

    pub fn with_confidence(mut self, c: f64) -> Self {
        self.confidence = c.clamp(0.0, 1.0);
        self
    }

    pub fn with_evidence(mut self, evidence: Vec<[u8; 32]>) -> Self {
        self.evidence = evidence;
        self
    }

    pub fn with_supersedes(mut self, supersedes: Vec<[u8; 32]>) -> Self {
        self.supersedes = supersedes;
        self
    }

    pub fn with_metadata(mut self, key: impl Into<String>, value: serde_json::Value) -> Self {
        self.metadata.insert(key.into(), value);
        self
    }

    pub fn with_ttl(mut self, ttl: Duration) -> Self {
        self.ttl = Some(ttl);
        self
    }

    pub fn with_prompt_prefix_hash(mut self, h: [u8; 32]) -> Self {
        self.prompt_prefix_hash = Some(h);
        self
    }

    pub fn with_policy_audit(mut self, decision: PolicyDecision) -> Self {
        self.policy_audit = Some(decision);
        self
    }

    // ── Queries ─────────────────────────────────────────────────────

    /// Is this entry expired based on its TTL and the given default?
    pub fn is_expired(&self, default_stm_ttl: Duration) -> bool {
        if self.tombstoned {
            return true;
        }
        let ttl = self.ttl.unwrap_or_else(|| match self.tier {
            EntryTier::Stm => default_stm_ttl,
            EntryTier::Session => Duration::max_value(),
            EntryTier::Ltm => Duration::max_value(),
        });
        Utc::now() - self.created_at > ttl
    }

    /// Hex-encoded content hash for display/logging.
    pub fn hash_hex(&self) -> String {
        hex_encode(&self.content_hash)
    }
}

/// Encode bytes as hex string.
pub fn hex_encode(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{:02x}", b)).collect()
}

impl PartialEq for BlackboardEntry {
    fn eq(&self, other: &Self) -> bool {
        self.content_hash == other.content_hash
    }
}

impl Eq for BlackboardEntry {}

impl std::hash::Hash for BlackboardEntry {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.content_hash.hash(state);
    }
}

impl std::fmt::Display for BlackboardEntry {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "[{}] {:?} by {} (conf={:.2}): {}",
            &self.hash_hex()[..8],
            self.entry_type,
            self.author,
            self.confidence,
            if self.content.len() > 80 {
                format!("{}...", &self.content[..80])
            } else {
                self.content.clone()
            }
        )
    }
}
