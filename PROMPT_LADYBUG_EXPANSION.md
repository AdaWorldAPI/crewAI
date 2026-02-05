# Ladybug-rs Expansion Prompt: crewAI Integration Surface

## Context

This prompt specifies the expansions needed in **ladybug-rs** (`AdaWorldAPI/ladybug-rs`) to serve as the cognitive backbone for **crewAI-rust** (`/lib/crewai-rust/`), a 1:1 Rust port of the crewAI Python multi-agent orchestration framework.

crewAI-rust has these trait-based extension points that ladybug-rs must satisfy:

- `BaseClient` — vector store interface (create/search/add/delete collections)
- `BaseEmbeddingsProvider` + `EmbeddingFunctionTrait` — embedding generation
- `BaseTransport` — MCP transport layer (currently: Stdio, HTTP, SSE)
- `Storage` — agent memory persistence (save/search/reset)
- `BaseKnowledgeStorage` — knowledge chunk storage and retrieval
- `BaseLLM` — LLM provider interface (for xAI/Grok integration)
- `BaseEvent` — event emission for observability

The goal: ladybug-rs provides **one unified crate** that crewAI-rust depends on behind a `ladybug` feature flag. All expansions are additive — no breaking changes to existing ladybug-rs APIs.

---

## Required Expansions

### 1. `crewai_compat` module — Trait adapter layer

Create `src/crewai_compat/mod.rs` exposing adapter structs that wrap existing ladybug internals to satisfy crewAI trait signatures. This is the primary integration surface.

#### 1.1 `LadybugClient` — implements crewAI `BaseClient`

Wrap `Database` to implement the vector store interface:

```rust
pub struct LadybugClient {
    db: Arc<Database>,
    default_limit: usize,
    default_score_threshold: f64,
}
```

**Method mapping:**

| crewAI `BaseClient` method | ladybug-rs implementation |
|---|---|
| `create_collection(params)` | `db.create_lance_table(collection_name, schema)` |
| `get_or_create_collection(params)` | Check existence, create if missing |
| `add_documents(params)` | Convert `BaseRecord` → Arrow `RecordBatch`, append to Lance table |
| `search(params)` | **Dual-path**: if query has fingerprint → Hamming search; else → vector ANN via Lance `nearest_to()` |
| `delete_collection(params)` | Drop Lance table |
| `reset()` | Drop and recreate all tables |

**Critical**: The `search` method must support **three search modes** selectable via metadata filter:
- `"mode": "vector"` — IVF-PQ ANN on 1024-dim embeddings (current Lance behavior)
- `"mode": "hamming"` — AVX-512 SIMD on 10K-bit fingerprints
- `"mode": "hybrid"` — Vector pre-filter → Hamming re-rank (new, highest quality)

The hybrid mode is what makes this integration special. Vector search casts a wide net (top-100), then Hamming distance on fingerprints re-ranks for interpretable precision.

#### 1.2 `GrammarTriangleEmbedder` — implements crewAI `BaseEmbeddingsProvider` + `EmbeddingFunctionTrait`

Wrap `GrammarTriangle` to produce embeddings:

```rust
pub struct GrammarTriangleEmbedder {
    triangle: GrammarTriangle,
    output_mode: EmbedderOutputMode,
}

pub enum EmbedderOutputMode {
    /// 1024-dim dense vector (from NSM weights, padded/projected)
    DenseVector,
    /// 10K-bit fingerprint as f32 binary vector (1.0/0.0 per bit)
    BinaryFingerprint,
    /// Both: dense for ANN index, fingerprint for Hamming re-rank
    Dual,
}
```

| crewAI trait method | Implementation |
|---|---|
| `provider_name()` | `"ladybug-grammar-triangle"` |
| `embed_text(text)` | `triangle.analyze(text)` → extract NSM weights → project to 1024-dim |
| `embed_documents(docs)` | Batch `analyze()` with SIMD-parallel fingerprint generation |
| `embed_query(input)` | Same as `embed_text`, optimized for single query |
| `call(inputs)` | Batch embedding, returns `Vec<Vec<f32>>` |

**The NSM-to-dense projection**: The 65 NSM primitive weights need to be projected into a 1024-dim space compatible with Lance IVF-PQ indices. Use a fixed learned projection matrix (65 × 1024) stored as a const array. This allows the GrammarTriangle output to be stored alongside Jina embeddings in the same Lance table, enabling gradual migration.

#### 1.3 `LadybugMemoryStorage` — implements crewAI `Storage`

Wrap `Database` + `Blackboard` for agent memory:

```rust
pub struct LadybugMemoryStorage {
    db: Arc<Database>,
    blackboard: Arc<RwLock<Blackboard>>,
    collection_name: String,
}
```

| crewAI `Storage` method | Implementation |
|---|---|
| `save(value, metadata)` | Encode via GrammarTriangle → store in Lance nodes table with metadata |
| `search(query, limit, threshold)` | Hybrid search (vector + Hamming re-rank) |
| `reset()` | Clear collection, reset blackboard |

**Blackboard integration**: Every `save()` also records the operation in the blackboard's decision history. Every `search()` checks the blackboard's frozen layers first — if a relevant fact is ice-caked (committed), it's returned with boosted score. This gives agents "committed knowledge" that survives across crew executions.

#### 1.4 `LadybugKnowledgeStorage` — implements crewAI `BaseKnowledgeStorage`

Similar to memory storage but specialized for knowledge ingestion:

```rust
pub struct LadybugKnowledgeStorage {
    db: Arc<Database>,
    collection_prefix: String,
    chunk_size: usize,
    chunk_overlap: usize,
}
```

| crewAI `BaseKnowledgeStorage` method | Implementation |
|---|---|
| `search(query, limit, threshold)` | Hybrid search on knowledge collection |
| `save(documents)` | Chunk → encode → store with source metadata |
| `save_chunks(chunks, metadata)` | Encode each chunk → batch insert into Lance |
| `reset()` | Drop knowledge collection |

**Structured recall enhancement**: Before returning search results, run NARS inference on the result set. For each result, compute a truth value `{frequency, confidence}` based on:
- How many independent sources mention this fact (frequency)
- How recent and how many retrievals confirmed it (confidence)

Return results annotated with NARS truth values in metadata:
```json
{"content": "...", "score": 0.87, "nars_truth": {"frequency": 0.92, "confidence": 0.78}}
```

This is the **structured recall improvement** — agents can distinguish between "frequently confirmed facts" and "one-off mentions."

---

### 2. Arrow Flight MCP Transport

Create `src/crewai_compat/flight_transport.rs`:

```rust
pub struct ArrowFlightTransport {
    endpoint: String,
    client: Option<FlightClient>,
    connected: bool,
}
```

This implements crewAI's `BaseTransport` trait, mapping MCP operations to Arrow Flight:

| MCP operation | Arrow Flight mapping |
|---|---|
| `connect()` | `FlightClient::connect(endpoint)` |
| `disconnect()` | Drop client |
| `list_tools()` | `DoAction("list_tools")` → deserialize tool list from RecordBatch |
| `call_tool(name, args)` | `DoAction(name, args)` → return result as string |
| `list_prompts()` | `DoAction("list_prompts")` |

**Transport type**: Register as `TransportType::ArrowFlight` (new variant).

**Zero-copy advantage**: Tool results that contain tabular data (search results, aggregations) arrive as Arrow RecordBatch — no JSON serialization overhead. For agents processing large datasets, this is orders of magnitude faster than JSON-RPC.

---

### 3. Bitpacked Hamming UDP Transport

Create `src/crewai_compat/udp_transport.rs`:

Leverage ladybug's existing `fabric/udp_transport.rs` and `fabric/firefly.rs`:

```rust
pub struct HammingUdpTransport {
    endpoint: String,
    lane: u8,
    connected: bool,
}
```

This is the **ultra-low-latency transport** for same-network agent communication:

| MCP operation | UDP mapping |
|---|---|
| `connect()` | Bind UDP socket, join multicast group |
| `call_tool(name, args)` | Encode as Firefly instruction frame (1250-bit packed), send via UDP lane |
| Result | Receive response frame, decode |

**When to use**: Inter-agent tool calls within the same crew execution where latency matters more than reliability. The Firefly frame format packs a tool call into 1250 bits — at UDP speeds, this means sub-millisecond agent-to-agent tool invocation.

**Fallback**: If UDP delivery fails (no ACK within 5ms), automatically retry via HTTP transport. This makes it safe to use as default for local crews.

---

### 4. A2A Agent Card with CAM Capabilities

Create `src/crewai_compat/agent_card.rs`:

Extend ladybug's 4096 CAM operations to serve as a **queryable agent capability registry**:

```rust
pub struct LadybugAgentCard {
    pub name: String,
    pub description: String,
    pub version: String,
    pub url: Option<String>,
    pub capabilities: Arc<CamDictionary>,
    pub db: Arc<Database>,
}

impl LadybugAgentCard {
    /// Query capabilities by semantic description.
    /// Returns matching CAM operations ranked by Hamming similarity.
    pub fn query_capabilities(&self, description: &str) -> Vec<CamCapability>;

    /// Standard A2A agent card JSON representation.
    pub fn to_agent_card_json(&self) -> Value;

    /// Discover what this agent can do for a given task description.
    /// Uses GrammarTriangle to encode the task, then Hamming-matches
    /// against all 4096 CAM operation fingerprints.
    pub fn discover_for_task(&self, task_description: &str) -> Vec<DiscoveredCapability>;
}

pub struct DiscoveredCapability {
    pub cam_id: u16,
    pub name: String,
    pub description: String,
    pub hamming_similarity: f64,
    pub nars_confidence: f64,
}
```

This is what makes A2A discovery **semantic** rather than static. Instead of listing capabilities in a JSON file, agents ask "can you help with X?" and get ranked, confidence-scored answers.

---

### 5. Shared Awareness via Blackboard Extensions

Extend `src/learning/blackboard.rs` with multi-agent awareness:

```rust
impl Blackboard {
    /// Register an agent's presence in the shared blackboard.
    pub fn register_agent(&mut self, agent_id: &str, role: &str, capabilities: Vec<String>);

    /// Post an observation visible to all agents.
    pub fn post_observation(&mut self, agent_id: &str, observation: Value);

    /// Read observations from other agents since last read.
    pub fn read_observations(&self, agent_id: &str, since: Option<DateTime<Utc>>) -> Vec<Observation>;

    /// Propose a commitment (FLOW/HOLD/BLOCK) that other agents can see.
    pub fn propose_commitment(&mut self, agent_id: &str, commitment: IceCakedLayer);

    /// Vote on another agent's proposed commitment.
    pub fn vote_commitment(&mut self, voter_id: &str, commitment_id: &str, vote: CommitmentVote);

    /// Get the current shared awareness state for an agent.
    pub fn awareness_state(&self, agent_id: &str) -> SharedAwarenessState;
}

pub struct Observation {
    pub agent_id: String,
    pub timestamp: DateTime<Utc>,
    pub content: Value,
    pub fingerprint: Fingerprint,
}

pub struct SharedAwarenessState {
    pub registered_agents: Vec<AgentPresence>,
    pub pending_observations: Vec<Observation>,
    pub active_commitments: Vec<IceCakedLayer>,
    pub consensus_score: f64,
}

pub enum CommitmentVote {
    Agree,
    Disagree(String), // reason
    Abstain,
}
```

This transforms the single-agent blackboard into a **multi-agent shared awareness system**. Agents can:
- See what other agents are working on
- Share intermediate findings
- Reach consensus on committed facts (ice-caked layers require majority vote)

---

### 6. NARS-Enhanced Result Scoring

Create `src/crewai_compat/nars_scoring.rs`:

```rust
pub struct NarsScorer {
    nars: NarsEngine,
}

impl NarsScorer {
    /// Score a set of search results using NARS inference.
    ///
    /// For each result, computes truth value {frequency, confidence} by:
    /// 1. Checking how many independent evidence paths support the claim
    /// 2. Applying deduction/induction/abduction rules
    /// 3. Combining with retrieval score as prior
    pub fn score_results(
        &self,
        results: &[SearchResult],
        query_context: &QueryContext,
    ) -> Vec<ScoredResult>;

    /// Perform abductive reasoning: given a result, what's the best explanation?
    pub fn explain_result(&self, result: &SearchResult) -> NarsExplanation;
}

pub struct ScoredResult {
    pub result: SearchResult,
    pub nars_truth: TruthValue,
    pub explanation: Option<String>,
    pub evidence_count: usize,
}

pub struct QueryContext {
    pub query: String,
    pub agent_role: Option<String>,
    pub task_description: Option<String>,
    pub prior_results: Vec<Value>,
}
```

---

### 7. Counterfactual Fork API

Expose `Database::fork()` with a crewAI-friendly interface:

```rust
impl Database {
    /// Fork the database for counterfactual exploration.
    /// Returns a new Database instance with copy-on-write semantics.
    /// The fork shares all data with the parent until writes diverge.
    pub fn fork_for_agent(&self, agent_id: &str, hypothesis: &str) -> Result<Database, Error>;

    /// Compare two forks: what changed between them?
    pub fn diff_forks(&self, other: &Database) -> ForkDiff;

    /// Merge a fork back if the hypothesis was confirmed.
    pub fn merge_fork(&mut self, fork: Database, merge_strategy: MergeStrategy) -> Result<(), Error>;
}

pub struct ForkDiff {
    pub added_nodes: Vec<String>,
    pub modified_nodes: Vec<(String, Value, Value)>, // id, old, new
    pub added_edges: Vec<String>,
    pub divergence_score: f64,
}

pub enum MergeStrategy {
    /// Accept all changes from fork
    AcceptAll,
    /// Only accept changes with NARS confidence above threshold
    ConfidenceThreshold(f64),
    /// Manual review (returns pending changes)
    Manual,
}
```

This enables agents to **speculatively explore** without corrupting shared state. An agent can fork, try a hypothesis, and merge back only if results are good.

---

## Nice-to-Have Expansions (Future)

### 8. Crystal LM Compression for Memory (experimental)

Use the existing `extensions/codebook/` crystal compression for long-term memory compaction. When agent memory exceeds a threshold, older memories get crystal-compressed (claimed 140M:1 ratio). Decompression happens on-demand during search.

### 9. Consciousness Stack for Agent Self-Monitoring

Expose the 7-layer consciousness stack (`cognitive/consciousness.rs`) as agent introspection:

```rust
pub struct AgentConsciousness {
    pub coherence_score: f64,        // 0.0-1.0, how consistent is agent behavior
    pub thinking_style: [f64; 7],    // weights across 7 cognitive dimensions
    pub emergence_metric: f64,       // novelty of agent's recent outputs
    pub dominant_layer: usize,       // which cognitive layer is most active
}
```

Agents could self-monitor: "my coherence dropped below 0.5, I should re-read the task description."

### 10. mRNA Resonance Fields for Cross-Agent Knowledge Pollination

Use `fabric/mrna_resonance.rs` to automatically propagate relevant knowledge between agents without explicit communication. When agent A stores a fact that resonates (Hamming similarity > threshold) with agent B's active query context, the fact is automatically surfaced to B. This is **ambient awareness** — agents learn from each other's discoveries passively.

### 11. Quantum-Inspired Operators for Embedding Similarity

Use `core/quantum.rs` linear mappings and measurement collapse for a novel similarity metric:
- Instead of cosine similarity, use the quantum measurement probability
- Fingerprints exist in superposition until "measured" by a query
- Collapse produces a definite result with a probability score

This is mathematically equivalent to a specific kernel function but provides a more nuanced similarity metric for edge cases where cosine similarity plateaus.

### 12. Firefly Instruction Frames as Agent Communication Protocol

Define a compact binary protocol for inter-agent messages using the existing 1250-bit Firefly frame format. Each frame encodes: sender(8bit) + receiver(8bit) + operation(12bit) + payload(1222bit). At ~156 bytes per message, this is 10-100x more compact than JSON-RPC for structured agent communication.

### 13. BindSpace as Universal Agent Address Book

Use the 8+8 bit addressing (`prefix:slot`, 65,536 addresses) as a global agent/tool/knowledge registry. Each agent, tool, and knowledge source gets a BindSpace address. Lookups are O(1). This replaces string-based name resolution throughout crewAI.

---

## File Structure

```
src/crewai_compat/
    mod.rs                      // Feature-gated: #[cfg(feature = "crewai")]
    client.rs                   // LadybugClient (BaseClient impl)
    embedder.rs                 // GrammarTriangleEmbedder (BaseEmbeddingsProvider impl)
    memory_storage.rs           // LadybugMemoryStorage (Storage impl)
    knowledge_storage.rs        // LadybugKnowledgeStorage (BaseKnowledgeStorage impl)
    flight_transport.rs         // ArrowFlightTransport (BaseTransport impl)
    udp_transport.rs            // HammingUdpTransport (BaseTransport impl)
    agent_card.rs               // LadybugAgentCard (A2A capability discovery)
    nars_scoring.rs             // NarsScorer (structured recall)
    events.rs                   // Ladybug-specific event types
    xai_provider.rs             // xAI/Grok LLM provider adapter
```

## Cargo.toml Addition

```toml
[features]
default = []
crewai = ["dep:crewai"]

[dependencies]
crewai = { path = "../crewai-rust", optional = true }
```

## Constraints

1. **No breaking changes** to existing ladybug-rs public APIs
2. All crewAI integration behind `#[cfg(feature = "crewai")]`
3. Existing tests must continue to pass
4. New code must compile with `cargo check --all-features`
5. Blackboard extensions must be backward-compatible (new fields are `Option<T>`)
6. Arrow Flight transport must work without ladybug-rs server running (graceful fallback)
