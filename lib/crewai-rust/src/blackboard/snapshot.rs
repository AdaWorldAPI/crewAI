//! Snapshot — immutable view of the blackboard for prompt construction.

use super::entry::{BlackboardEntry, hex_encode};
use serde::{Deserialize, Serialize};

/// An immutable snapshot of the blackboard at a specific epoch.
///
/// Agents that work against the same snapshot share the same LLM prompt prefix,
/// maximizing cache hits from Anthropic's `cache_control` and OpenAI's
/// implicit prefix caching.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlackboardSnapshot {
    /// Epoch at which this snapshot was taken.
    pub epoch: u64,

    /// Entries in canonical order (deterministic, stable between snapshots
    /// that haven't changed — this is what makes cache alignment work).
    pub entries: Vec<BlackboardEntry>,

    /// Cache thumbprint: hash of the canonical entry ordering.
    pub thumbprint: CacheThumbprint,

    /// Pre-rendered prompt string (cached so multiple agents don't re-render).
    rendered: String,
}

impl BlackboardSnapshot {
    /// Create a new snapshot from entries.
    pub fn new(epoch: u64, entries: Vec<BlackboardEntry>) -> Self {
        let thumbprint = CacheThumbprint::from_entries(&entries);
        let rendered = Self::render_for_prompt(&entries);
        Self {
            epoch,
            entries,
            thumbprint,
            rendered,
        }
    }

    /// Empty snapshot.
    pub fn empty() -> Self {
        Self::new(0, Vec::new())
    }

    /// Get the pre-rendered prompt string.
    ///
    /// Format:
    /// ```text
    /// [Blackboard — epoch 3, 12 entries, thumbprint a1b2c3d4]
    ///
    /// ## Decisions
    /// - [a1b2c3d4] (agent-X, conf=0.95): Market entry approved based on...
    ///
    /// ## Facts
    /// - [e5f6a7b8] (agent-Y, conf=1.00): Market size is $4.2B as of 2025
    ///
    /// ## Observations
    /// - [c9d0e1f2] (tool-serper, conf=1.00): Search results indicate...
    /// ```
    pub fn as_prompt(&self) -> &str {
        &self.rendered
    }

    /// Number of entries in this snapshot.
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Render entries into a prompt-friendly string.
    fn render_for_prompt(entries: &[BlackboardEntry]) -> String {
        use super::entry::EntryType;

        if entries.is_empty() {
            return String::new();
        }

        let thumbprint = CacheThumbprint::from_entries(entries);
        let mut sections: std::collections::BTreeMap<&str, Vec<String>> =
            std::collections::BTreeMap::new();

        // Group by entry type, with a stable ordering
        let type_order = [
            (EntryType::Decision, "Decisions"),
            (EntryType::Fact, "Facts"),
            (EntryType::Hypothesis, "Hypotheses"),
            (EntryType::Observation, "Observations"),
            (EntryType::Partial, "In Progress"),
            (EntryType::Query, "Open Questions"),
            (EntryType::Veto, "Vetoed"),
            (EntryType::Reasoning, "Reasoning Traces"),
        ];

        for (et, label) in &type_order {
            let matching: Vec<String> = entries
                .iter()
                .filter(|e| e.entry_type == *et && !e.tombstoned)
                .map(|e| {
                    format!(
                        "- [{}] ({}, conf={:.2}): {}",
                        &e.hash_hex()[..8],
                        e.author,
                        e.confidence,
                        e.content,
                    )
                })
                .collect();
            if !matching.is_empty() {
                sections.insert(label, matching);
            }
        }

        let mut out = format!(
            "[Blackboard — {} entries, thumbprint {}]\n",
            entries.len(),
            &thumbprint.hex()[..8],
        );

        for (label, items) in &sections {
            out.push_str(&format!("\n## {}\n", label));
            for item in items {
                out.push_str(item);
                out.push('\n');
            }
        }

        out
    }
}

// ─── Cache Thumbprint ────────────────────────────────────────────────────────

/// Cache thumbprint — hash of the canonical blackboard state.
///
/// Two agents that see the same thumbprint will have identical prompt prefixes,
/// enabling LLM API cache hits:
/// - Anthropic: aligns with `cache_creation_input_tokens` / `cache_read_input_tokens`
/// - OpenAI: aligns with implicit prefix caching (`cached_prompt_tokens`)
///
/// The thumbprint is the SHA-256 of the concatenated content hashes
/// in canonical order.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct CacheThumbprint {
    bytes: [u8; 32],
}

impl CacheThumbprint {
    /// Compute from a slice of entries (must be in canonical order).
    pub fn from_entries(entries: &[BlackboardEntry]) -> Self {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        // NOTE: Production should use SHA-256 via `sha2` crate.
        let mut hasher = DefaultHasher::new();
        for entry in entries {
            entry.content_hash.hash(&mut hasher);
        }
        let h1 = hasher.finish();
        entries.len().hash(&mut hasher);
        let h2 = hasher.finish();

        let mut bytes = [0u8; 32];
        bytes[..8].copy_from_slice(&h1.to_le_bytes());
        bytes[8..16].copy_from_slice(&h2.to_le_bytes());
        // Fill rest for distribution
        for entry in entries.iter().take(2) {
            entry.content_hash[0..8].hash(&mut hasher);
        }
        let h3 = hasher.finish();
        bytes[16..24].copy_from_slice(&h3.to_le_bytes());
        let h4 = hasher.finish();
        bytes[24..32].copy_from_slice(&h4.to_le_bytes());

        Self { bytes }
    }

    /// Compute from raw bytes.
    pub fn from_bytes(bytes: [u8; 32]) -> Self {
        Self { bytes }
    }

    /// Get the raw bytes.
    pub fn as_bytes(&self) -> &[u8; 32] {
        &self.bytes
    }

    /// Hex-encoded string.
    pub fn hex(&self) -> String {
        hex_encode(&self.bytes)
    }

    /// Zero thumbprint (empty blackboard).
    pub fn zero() -> Self {
        Self { bytes: [0u8; 32] }
    }
}

impl std::fmt::Display for CacheThumbprint {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", &self.hex()[..16])
    }
}
