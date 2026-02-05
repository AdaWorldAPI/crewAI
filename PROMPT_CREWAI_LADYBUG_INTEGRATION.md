# crewAI-Rust + Ladybug-rs Integration Prompt

## Context

**crewAI-rust** (`/lib/crewai-rust/`) is a 1:1 Rust port of the crewAI Python multi-agent framework: 209 source files, ~36K lines, 25+ modules, compiling with 0 errors.

**ladybug-rs** (`AdaWorldAPI/ladybug-rs`) is a cognitive database: SQL + Cypher + Vector ANN + Hamming distance over Lance/Arrow, with NARS reasoning, GrammarTriangle embeddings, blackboard state, and Arrow Flight gRPC.

This prompt specifies how to integrate ladybug-rs into crewAI-rust as a modular provider layer, adding capabilities the original Python crewAI never had — while breaking nothing.

---

## Architecture Principle

Every integration is a **new file** behind a `ladybug` feature flag. No existing file is modified except for:
- `Cargo.toml` — add `ladybug` feature and dependency
- `src/lib.rs` — add `#[cfg(feature = "ladybug")] pub mod ladybug;`
- Module `mod.rs` files — add `#[cfg(feature = "ladybug")]` re-exports where needed

Everything else is additive. If the `ladybug` feature is disabled, crewAI-rust compiles and behaves exactly as before.

---

## Module 1: Ladybug RAG Provider

### Files to Create

**`src/rag/ladybug/mod.rs`**

```rust
//! Ladybug-rs backed RAG provider.
//!
//! Provides vector + Hamming hybrid search over Lance columnar storage.
//! Three search modes: vector-only, hamming-only, hybrid (vector pre-filter + hamming re-rank).

#[cfg(feature = "ladybug")]
pub mod client;
#[cfg(feature = "ladybug")]
pub mod config;
```

**`src/rag/ladybug/client.rs`** — Implement `BaseClient` trait:

```rust
use crate::rag::core::{
    BaseClient, BaseRecord, CollectionAddParams, CollectionParams,
    CollectionSearchParams, SearchResult,
};

pub struct LadybugRagClient {
    db: Arc<ladybug_rs::Database>,
    search_mode: SearchMode,
    nars_scoring: bool,
}

pub enum SearchMode {
    /// Standard vector ANN via Lance IVF-PQ indices
    Vector,
    /// AVX-512 SIMD Hamming distance on 10K-bit fingerprints
    Hamming,
    /// Vector pre-filter (top-N) → Hamming re-rank (final top-K)
    /// This is the recommended mode for production use.
    Hybrid { pre_filter_factor: usize },
}
```

Implement all `BaseClient` methods. The key innovation is `search()`:

1. **Vector mode**: Delegate to Lance `nearest_to()` with IVF-PQ
2. **Hamming mode**: Load fingerprints, SIMD compare, return top-K
3. **Hybrid mode**: Vector search for `limit * pre_filter_factor` results, then Hamming re-rank to `limit`

If `nars_scoring` is enabled, annotate each result with NARS truth values `{frequency, confidence}` in the metadata field.

**`src/rag/ladybug/config.rs`** — Configuration:

```rust
pub struct LadybugRagConfig {
    pub db_path: String,
    pub search_mode: SearchMode,
    pub nars_scoring: bool,
    pub embedding_mode: EmbeddingMode,
    pub collection_prefix: Option<String>,
}

pub enum EmbeddingMode {
    /// Use ladybug GrammarTriangle (local, no API calls)
    GrammarTriangle,
    /// Use external Jina API (deprecated path)
    Jina { api_key: String },
    /// Dual: GrammarTriangle fingerprint + dense vector
    Dual,
}
```

### Registration

Add to `src/rag/factory.rs`:

```rust
#[cfg(feature = "ladybug")]
SupportedProvider::Ladybug => {
    let config = /* extract LadybugRagConfig from base config */;
    Ok(Box::new(ladybug::client::LadybugRagClient::new(config)?))
}
```

Add `Ladybug` variant to `SupportedProvider` enum (feature-gated).

---

## Module 2: Ladybug Embedding Provider

### Files to Create

**`src/rag/embeddings/providers/ladybug/mod.rs`**

Implement `BaseEmbeddingsProvider` and `EmbeddingFunctionTrait`:

```rust
pub struct LadybugEmbeddingProvider {
    output_mode: EmbedderOutputMode,
}

pub enum EmbedderOutputMode {
    /// Project NSM 65-dim weights to 1024-dim dense vector
    DenseVector,
    /// Return 10K-bit fingerprint as binary f32 vector (each element 0.0 or 1.0)
    BinaryFingerprint,
    /// Both dense + fingerprint (for hybrid search)
    Dual,
}

impl BaseEmbeddingsProvider for LadybugEmbeddingProvider {
    fn provider_name(&self) -> &str { "ladybug-grammar-triangle" }

    fn build_embedding_function(&self) -> Result<Box<dyn EmbeddingFunctionTrait>, anyhow::Error> {
        Ok(Box::new(GrammarTriangleFunction::new(self.output_mode)))
    }

    fn config(&self) -> Value {
        json!({
            "provider": "ladybug-grammar-triangle",
            "output_mode": format!("{:?}", self.output_mode),
            "dimensions": match self.output_mode {
                EmbedderOutputMode::DenseVector => 1024,
                EmbedderOutputMode::BinaryFingerprint => 10000,
                EmbedderOutputMode::Dual => 11024,
            }
        })
    }
}
```

The `GrammarTriangleFunction` wraps ladybug's `GrammarTriangle::analyze()`:
- Accepts text input
- Runs NSM + Causality + Qualia analysis
- Outputs dense vector, fingerprint, or both depending on mode

Register in `src/rag/embeddings/providers/mod.rs` provider registry:
```rust
#[cfg(feature = "ladybug")]
registry.insert("ladybug-grammar-triangle", "rag::embeddings::providers::ladybug");
```

---

## Module 3: MCP Transports (Arrow Flight + Hamming UDP)

### Files to Create

**`src/mcp/transports/arrow_flight.rs`** — Arrow Flight gRPC transport:

```rust
pub struct ArrowFlightTransport {
    endpoint: String,
    client: Option</* arrow-flight FlightClient */>,
    connected: bool,
    timeout: Duration,
}

impl BaseTransport for ArrowFlightTransport {
    fn transport_type(&self) -> TransportType { TransportType::ArrowFlight }
    // ... connect/disconnect/server_identifier
}
```

Add `ArrowFlight` variant to `TransportType` enum (feature-gated).

MCP tool calls map to Arrow Flight `DoAction`:
- `list_tools` → `DoAction("list_tools")` returns `RecordBatch` with tool definitions
- `call_tool` → `DoAction(tool_name)` with arguments serialized in Flight descriptor

Tool results that are tabular (search results, data queries) arrive as zero-copy `RecordBatch` — converted to JSON only at the final agent prompt boundary.

**`src/mcp/transports/hamming_udp.rs`** — Ultra-low-latency UDP transport:

```rust
pub struct HammingUdpTransport {
    endpoint: String,
    socket: Option<UdpSocket>,
    lane: u8,
    connected: bool,
    fallback: Option<Box<dyn BaseTransport>>,
}

impl BaseTransport for HammingUdpTransport {
    fn transport_type(&self) -> TransportType { TransportType::HammingUdp }
    // ...
}
```

Add `HammingUdp` variant to `TransportType` enum.

Tool calls are encoded as Firefly instruction frames (1250-bit packed):
- 8-bit sender + 8-bit receiver + 12-bit operation code + 1222-bit payload
- Total: 156 bytes per message
- Sub-millisecond latency on local network

Automatic fallback to HTTP on delivery failure (no ACK within 5ms configurable timeout).

**`src/mcp/config.rs`** — Add new config variants (feature-gated):

```rust
#[cfg(feature = "ladybug")]
pub struct MCPServerArrowFlight {
    pub endpoint: String,
    pub tls: bool,
    pub auth_token: Option<String>,
    pub tool_filter: Option<ArcToolFilter>,
    pub cache_tools_list: bool,
}

#[cfg(feature = "ladybug")]
pub struct MCPServerHammingUdp {
    pub endpoint: String,
    pub lane: u8,
    pub fallback_http: Option<String>,
    pub ack_timeout_ms: u64,
    pub tool_filter: Option<ArcToolFilter>,
    pub cache_tools_list: bool,
}
```

Add `ArrowFlight(MCPServerArrowFlight)` and `HammingUdp(MCPServerHammingUdp)` variants to `MCPServerConfig` enum.

---

## Module 4: xAI / Grok LLM Provider

### Files to Create

**`src/llms/providers/xai/mod.rs`**

```rust
//! xAI Grok LLM provider.
//!
//! Supports Grok-2, Grok-3, and future models via the xAI API.
//! API is OpenAI-compatible with additional reasoning capabilities.

pub struct XaiCompletion {
    state: BaseLLMState,
    api_key: String,
    base_url: String,       // default: "https://api.x.ai/v1"
    reasoning_effort: Option<String>, // "low", "medium", "high"
    search_enabled: bool,    // Grok's real-time web search
}

impl BaseLLM for XaiCompletion {
    fn model(&self) -> &str { &self.state.model }
    fn provider(&self) -> &str { "xai" }
    fn supports_function_calling(&self) -> bool { true }
    fn supports_multimodal(&self) -> bool { true }
    fn get_context_window_size(&self) -> usize { 131072 } // Grok-2: 128K

    fn call(&self, messages, tools, available_functions) -> Result<Value, _> {
        // POST to xAI API (OpenAI-compatible format)
        // Handle tool_choice, response_format
        // Parse xAI-specific fields (search_results, reasoning_content)
    }

    async fn acall(&self, messages, tools, available_functions) -> Result<Value, _> {
        // Async version using reqwest
    }
}
```

**Key xAI-specific features to expose**:

1. **Reasoning effort**: `reasoning_effort` parameter controls how much "thinking" Grok does
2. **Live search**: `search_enabled` lets Grok search the web in real-time
3. **Streaming**: Support `stream: true` with SSE parsing
4. **Tool use**: OpenAI-compatible function calling format

**Authentication**: `XAI_API_KEY` environment variable or explicit `api_key` parameter.

Register in `src/llms/providers/mod.rs`:
```rust
#[cfg(feature = "xai")]
pub mod xai;
```

---

## Module 5: Ladybug Memory Backend

### Files to Create

**`src/memory/storage/ladybug_storage.rs`**

Implement `Storage` trait backed by ladybug's Database + Blackboard:

```rust
pub struct LadybugStorage {
    db: Arc<ladybug_rs::Database>,
    blackboard: Arc<RwLock<ladybug_rs::Blackboard>>,
    collection_name: String,
    agent_id: String,
    search_mode: SearchMode,
}

impl Storage for LadybugStorage {
    fn save(&self, value: &str, metadata: &HashMap<String, Value>) -> Result<(), anyhow::Error> {
        // 1. Encode value via GrammarTriangle → fingerprint + embedding
        // 2. Store in Lance nodes table
        // 3. Record in blackboard decision history
    }

    fn search(&self, query: &str, limit: usize, score_threshold: f64) -> Result<Vec<Value>, anyhow::Error> {
        // 1. Check blackboard frozen layers (ice-caked commitments) first
        // 2. Hybrid search on Lance table
        // 3. NARS score results
        // 4. Merge ice-caked facts (boosted) with search results
    }

    fn reset(&self) -> Result<(), anyhow::Error> {
        // Clear collection + reset blackboard for this agent
    }
}
```

**The ice-cake boost**: When an agent or crew explicitly commits a fact (marks it as decided), it becomes an ice-caked layer in the blackboard. On subsequent searches, ice-caked facts matching the query get a 2x score boost and are always included in results regardless of threshold. This gives agents "institutional memory" — committed decisions persist with high priority.

---

## Module 6: Shared Awareness Between Agents

### Files to Create

**`src/memory/storage/shared_blackboard.rs`**

A specialized storage backend that enables cross-agent awareness:

```rust
pub struct SharedBlackboardStorage {
    blackboard: Arc<RwLock<ladybug_rs::Blackboard>>,
    agent_id: String,
}

impl SharedBlackboardStorage {
    /// Post an observation visible to all agents in the crew.
    pub fn post_observation(&self, content: Value) -> Result<(), anyhow::Error>;

    /// Read observations from other agents since the given timestamp.
    pub fn read_observations(&self, since: Option<DateTime<Utc>>) -> Result<Vec<Value>, anyhow::Error>;

    /// Get the awareness state: who's working on what.
    pub fn awareness_state(&self) -> Result<SharedAwarenessState, anyhow::Error>;
}
```

This is NOT a replacement for `Storage` — it's an additional capability. Agents get both their private `LadybugStorage` and a shared `SharedBlackboardStorage`. The crew executor wires them together.

---

## Module 7: A2A Agent Card Extensions

### Files to Create

**`src/a2a/ladybug_card.rs`**

Extend A2A with semantic capability discovery:

```rust
pub struct LadybugAgentCard {
    pub base_config: A2AServerConfig,
    pub cam_capabilities: Vec<CamCapability>,
    pub db: Arc<ladybug_rs::Database>,
}

pub struct CamCapability {
    pub id: u16,
    pub name: String,
    pub description: String,
    pub fingerprint: Vec<u8>, // 10K-bit fingerprint of this capability
    pub input_schema: Option<Value>,
    pub output_schema: Option<Value>,
}

impl LadybugAgentCard {
    /// Build an agent card from a crewAI Agent, automatically discovering
    /// capabilities from the agent's tools and knowledge.
    pub fn from_agent(agent: &Agent, db: Arc<Database>) -> Self;

    /// Semantic capability query: "what can this agent do for task X?"
    pub fn discover_capabilities(&self, task_description: &str) -> Vec<DiscoveredCapability>;

    /// Standard A2A agent card JSON (with extended capabilities section).
    pub fn to_json(&self) -> Value;
}

pub struct DiscoveredCapability {
    pub capability: CamCapability,
    pub relevance_score: f64,        // Hamming similarity to task
    pub nars_confidence: f64,        // How confident we are this capability applies
    pub estimated_complexity: String, // "simple", "moderate", "complex"
}
```

---

## Module 8: German Translations

### Files to Create

**`src/translations/de.json`**

Translate the entire `en.json` structure to German. The JSON structure must match exactly — same keys, same nesting, only values translated.

Key sections:
- `hierarchical_manager_agent` — Agent role descriptions, goals, backstory
- `slices` — Prompt templates (role_playing, tools, task descriptions, observations)
- `errors` — Error messages

Example entries:
```json
{
  "hierarchical_manager_agent": {
    "role": "Crew-Manager",
    "goal": "Verwalte die Crew und delegiere Aufgaben an die richtigen Agenten...",
    "backstory": "Du bist ein erfahrener Manager, der ein Team von Agenten koordiniert..."
  },
  "slices": {
    "observation": "\nBeobachtung",
    "task": "\nAktuelle Aufgabe: {task}",
    "tools": "\nVerfuegbare Werkzeuge: {tools}",
    "role_playing": "Du bist {role}.\n{backstory}\n\nDein persoenliches Ziel: {goal}"
  },
  "errors": {
    "tool_usage_error": "Fehler bei der Werkzeugverwendung: {error}",
    "task_execution_error": "Fehler bei der Aufgabenausfuehrung: {error}"
  }
}
```

**Note**: Use ASCII-safe German (ae/oe/ue instead of umlauts) in code-facing strings to avoid encoding issues in prompts. Display-facing strings can use proper umlauts.

**`src/translations/mod.rs`** — Update to support language selection:

```rust
pub const EN_JSON: &str = include_str!("en.json");
pub const DE_JSON: &str = include_str!("de.json");

impl Translations {
    pub fn load(language: &str) -> Self {
        match language {
            "de" | "german" => Self::from_json(DE_JSON).unwrap_or_else(|_| Self::load_default()),
            _ => Self::load_default(),
        }
    }
}
```

---

## Module 9: Structured Recall for RAG

### Files to Create

**`src/rag/scoring/mod.rs`**

```rust
//! Result scoring and re-ranking for RAG pipelines.
//!
//! Provides NARS-based truth value scoring, cross-reference verification,
//! and temporal decay for search results.

pub struct StructuredRecallScorer {
    /// Enable NARS truth value computation
    pub nars_enabled: bool,
    /// Enable cross-reference verification (check if multiple sources agree)
    pub cross_reference: bool,
    /// Temporal decay half-life in days (older results score lower)
    pub temporal_decay_days: Option<f64>,
    /// Minimum confidence to include in results
    pub min_confidence: f64,
}

impl StructuredRecallScorer {
    pub fn score(&self, results: Vec<SearchResult>, context: &RecallContext) -> Vec<ScoredResult>;
}

pub struct RecallContext {
    pub query: String,
    pub agent_role: Option<String>,
    pub task_type: Option<String>,
    pub prior_queries: Vec<String>,
    pub known_facts: Vec<String>,
}

pub struct ScoredResult {
    pub result: SearchResult,
    pub truth_value: TruthValue,
    pub cross_references: usize,
    pub temporal_freshness: f64,
    pub composite_score: f64,
}

pub struct TruthValue {
    pub frequency: f64,   // 0.0-1.0: how often is this true across sources
    pub confidence: f64,  // 0.0-1.0: how much evidence do we have
    pub expectation: f64, // frequency * confidence + 0.5 * (1 - confidence)
}
```

This works with ANY RAG backend (ChromaDB, Qdrant, Ladybug) — it's a post-processing layer that re-scores results before they reach the agent.

---

## Module 10: RAG Provider Management

### Files to Create

**`src/rag/manager.rs`**

```rust
//! RAG provider management: hot-swap providers, multi-provider search,
//! provider health monitoring, and automatic failover.

pub struct RagManager {
    providers: HashMap<String, Box<dyn BaseClient>>,
    primary: String,
    fallback_chain: Vec<String>,
    health_status: HashMap<String, ProviderHealth>,
}

pub struct ProviderHealth {
    pub last_check: DateTime<Utc>,
    pub is_healthy: bool,
    pub avg_latency_ms: f64,
    pub error_rate: f64,
    pub total_queries: u64,
}

impl RagManager {
    /// Register a new provider.
    pub fn register(&mut self, name: &str, client: Box<dyn BaseClient>);

    /// Set the primary provider and fallback chain.
    pub fn set_primary(&mut self, name: &str, fallbacks: Vec<String>);

    /// Search across providers with automatic failover.
    pub fn search(&self, params: &CollectionSearchParams) -> Result<Vec<SearchResult>, anyhow::Error>;

    /// Multi-provider search: query all providers and merge results.
    /// De-duplicates by content hash, keeps highest score per unique result.
    pub fn multi_search(&self, params: &CollectionSearchParams) -> Result<Vec<SearchResult>, anyhow::Error>;

    /// Health check all providers.
    pub fn health_check(&mut self) -> HashMap<String, ProviderHealth>;

    /// Hot-swap a provider without downtime.
    pub fn swap(&mut self, name: &str, new_client: Box<dyn BaseClient>);
}
```

This gives crewAI the ability to run multiple RAG backends simultaneously — e.g., ChromaDB for general knowledge + Ladybug for agent-specific memory — and merge results.

---

## Module 11: Ladybug-Specific Events

### Files to Create

**`src/events/types/ladybug_events.rs`**

```rust
pub struct HammingSearchEvent { /* query, result_count, avg_hamming_distance, latency_us */ }
pub struct NarsInferenceEvent { /* query, truth_value, evidence_count, inference_type */ }
pub struct BlackboardCommitEvent { /* agent_id, commitment, layer_name, vote_result */ }
pub struct FingerprintGeneratedEvent { /* text_preview, fingerprint_hash, nsm_activations */ }
pub struct ArrowFlightCallEvent { /* action, endpoint, bytes_transferred, latency_us */ }
pub struct AgentAwarenessEvent { /* agent_id, observation_type, content_preview */ }
pub struct CounterfactualForkEvent { /* agent_id, hypothesis, fork_id */ }
pub struct CounterfactualMergeEvent { /* fork_id, merge_strategy, divergence_score */ }
```

All implement `BaseEvent`. Emitted through the existing event bus singleton.

---

## Cargo.toml Changes

```toml
[features]
default = []
ladybug = ["dep:ladybug-rs"]
xai = []  # No extra deps, just reqwest (already a dep)
full = ["ladybug", "xai"]

[dependencies]
ladybug-rs = { git = "https://github.com/AdaWorldAPI/ladybug-rs", optional = true, features = ["crewai"] }
```

---

## lib.rs Changes

```rust
#[cfg(feature = "ladybug")]
pub mod ladybug;
```

**`src/ladybug/mod.rs`** — Re-export convenience module:

```rust
//! Ladybug-rs integration for crewAI.
//!
//! Enable with `features = ["ladybug"]` in Cargo.toml.

pub use crate::rag::ladybug::client::LadybugRagClient;
pub use crate::rag::ladybug::config::{LadybugRagConfig, SearchMode, EmbeddingMode};
pub use crate::rag::embeddings::providers::ladybug::LadybugEmbeddingProvider;
pub use crate::memory::storage::ladybug_storage::LadybugStorage;
pub use crate::memory::storage::shared_blackboard::SharedBlackboardStorage;
pub use crate::mcp::transports::arrow_flight::ArrowFlightTransport;
pub use crate::mcp::transports::hamming_udp::HammingUdpTransport;
pub use crate::a2a::ladybug_card::LadybugAgentCard;
pub use crate::rag::scoring::StructuredRecallScorer;
pub use crate::rag::manager::RagManager;
```

---

## Nice-to-Have Features (Not Yet Implemented Anywhere)

These don't exist in either codebase yet. They represent what would make this integration extraordinary:

### N1. Adaptive Search Mode Selection

The `RagManager` automatically selects the best search mode (vector/hamming/hybrid) per query based on:
- Query length (short queries → hamming; long queries → vector)
- Query type detection (factual → hamming; semantic → vector; exploratory → hybrid)
- Historical performance (track which mode produces results the agent actually uses)

### N2. Agent Memory Consolidation

When a crew finishes, the `LadybugStorage` automatically:
1. Identifies frequently accessed memories across all agents
2. Runs NARS revision on conflicting memories
3. Consolidates into a shared "crew memory" collection
4. Crystal-compresses old per-agent memories

This mimics how human teams build institutional knowledge after a project.

### N3. Live Reasoning Trace

Every NARS inference step is emitted as a `NarsInferenceEvent` with full trace:
- Input premises
- Inference rule applied (deduction/induction/abduction)
- Resulting truth value
- Confidence change

Agents can inspect their own reasoning: "why do I believe this fact?"

### N4. Semantic Agent Routing

Instead of static task delegation, use Hamming similarity between task fingerprints and agent capability fingerprints to automatically select the best agent for a task. The crew executor fingerprints the task description, compares against all registered `LadybugAgentCard` capabilities, and routes to the best match.

### N5. Zero-Copy Agent Pipelines

When two agents in the same process pass data via Arrow Flight transport, use shared memory instead of network I/O. The `RecordBatch` is passed by `Arc` reference — true zero-copy, zero-serialization agent communication.

### N6. Temporal Knowledge Graphs

Use ladybug's edge types with timestamps to build knowledge graphs that evolve over time. Agents can query "what was true at time T?" using Lance's version-based time travel. Combined with counterfactual forks, this enables: "what would have happened if we knew X at time T?"

### N7. Resonance-Based Knowledge Propagation

When agent A discovers a fact that resonates (Hamming similarity > 0.85) with agent B's current query context, automatically surface it to B without explicit communication. This is passive inter-agent learning — agents benefit from each other's discoveries in real-time.

### N8. Confidence-Gated Tool Execution

Before executing a tool call, check NARS confidence on the tool's expected behavior. If confidence is below threshold (the tool hasn't been reliable), emit a warning event and optionally skip or use a fallback tool. This gives agents "learned tool preferences" based on experience.

### N9. Multi-Lingual Agent Crews

With German (and future language) translations, crews can have agents that operate in different languages. The GrammarTriangle's NSM layer (65 universal semantic primitives from Wierzbicka's linguistic research) provides language-independent fingerprints — an agent thinking in German and an agent thinking in English produce comparable fingerprints for the same concept.

### N10. Crew Execution Replay

Use Lance versioning + blackboard decision history to replay any crew execution:
1. Load the initial state snapshot
2. Step through each agent's decisions
3. At any point, fork and try alternatives
4. Compare outcomes via `diff_forks()`

This is the ultimate debugging tool for multi-agent systems.

---

## Testing Strategy

Each module gets its own test file under `tests/`:

```
tests/
    ladybug_rag_test.rs         // LadybugRagClient with all 3 search modes
    ladybug_embedding_test.rs   // GrammarTriangle embedding generation
    arrow_flight_test.rs        // Arrow Flight transport connect/call
    hamming_udp_test.rs         // UDP transport with fallback
    xai_provider_test.rs        // xAI API call (mock server)
    ladybug_storage_test.rs     // Memory storage save/search/reset
    shared_blackboard_test.rs   // Multi-agent awareness
    ladybug_card_test.rs        // A2A capability discovery
    structured_recall_test.rs   // NARS scoring on search results
    rag_manager_test.rs         // Multi-provider search, failover
    german_translations_test.rs // de.json structure matches en.json
```

All tests must pass with `cargo test --features full`.

---

## Constraints

1. **Zero breakage**: `cargo check` without features must produce same result as before
2. **Feature isolation**: Every ladybug import is behind `#[cfg(feature = "ladybug")]`
3. **Trait compatibility**: All implementations satisfy existing crewAI trait signatures exactly
4. **Async parity**: Every sync method has an async counterpart
5. **No unwrap in library code**: All errors propagated via `Result` / `anyhow`
6. **Documentation**: Every public type and method has doc comments with examples
7. **Backward compatibility**: Existing ChromaDB/Qdrant/OpenAI/Anthropic paths unchanged
