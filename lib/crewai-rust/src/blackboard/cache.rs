//! Cache alignment — utilities for maximizing LLM API cache hits.
//!
//! The key insight: if the blackboard snapshot is placed as a stable prefix
//! in the LLM prompt, every agent working against the same snapshot epoch
//! shares one cached prefix. This module provides utilities to:
//!
//! 1. Position the blackboard snapshot in the message array for cache alignment
//! 2. Add Anthropic `cache_control` markers at the right positions
//! 3. Track cache efficiency metrics

pub use super::snapshot::CacheThumbprint;

use serde_json::Value;

/// Anthropic cache control marker.
///
/// Insert this into the message array after the blackboard snapshot
/// to tell Anthropic's API to cache everything before this point.
///
/// ```json
/// { "type": "text", "text": "<blackboard snapshot>", "cache_control": { "type": "ephemeral" } }
/// ```
pub fn anthropic_cache_marker(content: &str) -> Value {
    serde_json::json!({
        "type": "text",
        "text": content,
        "cache_control": { "type": "ephemeral" }
    })
}

/// Build the message array with blackboard snapshot positioned for cache alignment.
///
/// Layout:
/// ```text
/// [0] system: agent role + instructions                      ← stable, cacheable
/// [1] system: blackboard snapshot + cache_control marker     ← CACHED PREFIX BOUNDARY
/// [2] user: task-specific context + current task             ← varies per agent/task
/// [3..] assistant/user: conversation history                 ← varies per turn
/// ```
///
/// This ensures that for N agents reading the same blackboard epoch:
/// - Agent 1 pays full prompt token cost (cache write)
/// - Agents 2..N get a cache read (typically 90% cheaper on Anthropic)
pub fn build_cached_message_array(
    system_prompt: &str,
    blackboard_snapshot: &str,
    task_context: &str,
    history: &[Value],
) -> Vec<Value> {
    let mut messages = Vec::new();

    // System message with blackboard snapshot as cacheable prefix
    // For Anthropic: use content array with cache_control on the snapshot block
    let system_content = if blackboard_snapshot.is_empty() {
        serde_json::json!([{
            "type": "text",
            "text": system_prompt
        }])
    } else {
        serde_json::json!([
            {
                "type": "text",
                "text": system_prompt
            },
            // The blackboard snapshot — this is the cache boundary
            {
                "type": "text",
                "text": blackboard_snapshot,
                "cache_control": { "type": "ephemeral" }
            }
        ])
    };

    messages.push(serde_json::json!({
        "role": "system",
        "content": system_content
    }));

    // User message: task-specific content
    messages.push(serde_json::json!({
        "role": "user",
        "content": task_context
    }));

    // Conversation history
    messages.extend_from_slice(history);

    messages
}

/// Cache efficiency tracker.
///
/// Tracks how many tokens were cached vs freshly computed
/// across a crew execution run. Feeds into UsageMetrics.
#[derive(Debug, Clone, Default)]
pub struct CacheEfficiency {
    /// Total prompt tokens sent.
    pub total_prompt_tokens: u64,
    /// Tokens served from cache.
    pub cached_tokens: u64,
    /// Tokens that missed cache (freshly computed).
    pub fresh_tokens: u64,
    /// Number of LLM calls that got a cache hit.
    pub cache_hits: u64,
    /// Number of LLM calls with no cache hit.
    pub cache_misses: u64,
    /// Thumbprint that was active during this tracking period.
    pub active_thumbprint: Option<CacheThumbprint>,
}

impl CacheEfficiency {
    /// Record a call's cache performance from LLM usage response.
    ///
    /// For Anthropic: check `cache_creation_input_tokens` and `cache_read_input_tokens`.
    /// For OpenAI: check `cached_prompt_tokens` in `usage.prompt_tokens_details`.
    pub fn record_call(
        &mut self,
        total_prompt: u64,
        cached: u64,
    ) {
        self.total_prompt_tokens += total_prompt;
        self.cached_tokens += cached;
        self.fresh_tokens += total_prompt.saturating_sub(cached);
        if cached > 0 {
            self.cache_hits += 1;
        } else {
            self.cache_misses += 1;
        }
    }

    /// Cache hit ratio [0.0, 1.0].
    pub fn hit_ratio(&self) -> f64 {
        let total = self.cache_hits + self.cache_misses;
        if total == 0 {
            0.0
        } else {
            self.cache_hits as f64 / total as f64
        }
    }

    /// Estimated cost savings ratio (Anthropic caches at 90% discount).
    pub fn estimated_savings_ratio(&self) -> f64 {
        if self.total_prompt_tokens == 0 {
            0.0
        } else {
            (self.cached_tokens as f64 * 0.9) / self.total_prompt_tokens as f64
        }
    }
}
