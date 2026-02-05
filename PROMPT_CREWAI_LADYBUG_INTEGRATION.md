# crewAI-Rust + Ladybug-rs Integration Prompt (v2)

## Context

**crewAI-rust** (`/lib/crewai-rust/`) is a 1:1 Rust port of the crewAI Python multi-agent framework: 215+ source files, ~38K lines, 25+ modules, 163 passing tests, compiling with 0 errors.

**ladybug-rs** (`AdaWorldAPI/ladybug-rs`) is **not merely a cognitive database**. It is a complete agent orchestration substrate: ~27K lines implementing a `CrewBridge` that composes agent registries, persona modeling, A2A messaging, semantic kernel operations, handover policies, thinking templates, blackboard awareness, filter pipelines, guardrails, workflow DAGs, memory banks, observability tracing, and verification engines — all operating on a unified BindSpace of 10,000-bit fingerprints. It already exposes 50+ Arrow Flight DoAction handlers purpose-built for crewAI integration.

**The integration is not "wrap ladybug as a storage backend." It is "replace crewAI's stub execution layer with ladybug's fully-realized cognitive substrate."**

---

## What Ladybug-rs Actually Provides (Not Speculative — Implemented)

### Orchestration (`src/orchestration/`)

| Component | File | What It Does |
|-----------|------|-------------|
| **CrewBridge** | `crew_bridge.rs` | Single coordination point composing ALL subsystems: agents, personas, blackboards, A2A, orchestrator, kernel, filters, guardrails, memory, observability, verification. Methods: `register_agent()`, `submit_task()`, `dispatch_crew()`, `route_task()`, `evaluate_handover()`, `execute_handover()`, `tick_orchestrator()`, `total_awareness()`, `bind_all()` |
| **MetaOrchestrator** | `meta_orchestrator.rs` | Personality-resonance routing with an affinity graph that learns from collaboration history. Scoring: `compatibility (base) + affinity_boost (history) + task_boost (relevance) + flow_penalty (availability)`. Events: HandoverDecided, FlowTransition, AffinityUpdated, ResonanceRouted, Escalation |
| **A2AProtocol** | `a2a.rs` | 8 message types (Delegate, Result, Status, Knowledge, Sync, Query, Response, PersonaExchange). XOR superposition in BindSpace channels. `field_resonance()` and `superposition_depth()` for awareness metrics. Channel addressing via prefix `0x0F` |
| **PersonaRegistry** | `persona.rs` | Full personality modeling: VolitionDTO (curiosity, autonomy, persistence, risk_tolerance, collaboration), CommunicationStyle (formality, verbosity, directness, technical_depth, emotional_tone), FeatureAd (proficiency + CAM opcode). `to_fingerprint()` encodes into 10K-bit HDR. `compatibility()` computes Hamming similarity. `best_for_task()` ranks by volition alignment |
| **HandoverPolicy** | `handover.rs` | 7-threshold delegation: min_resonance, coherence_floor, max_hold_cycles, flow_momentum_shield, volition_floor, dk_gap_threshold, flow_preserving. FlowState: Flow/Hold/Block/Handover. Dunning-Kruger gap detection. Metacognitive review triggers |
| **SemanticKernel** | `semantic_kernel.rs` | 20+ kernel operations (Bind, Query, XorBind, XorUnbind, Bundle, Permute, Resonate, Collapse, Deduce, Induce, Abduct, Revise, Correlate, Intervene, Imagine, Escalate, Crystallize, Dissolve, Introspect, ZoneDensity). KernelTruth with NARS truth values. Pearl's 3-rung causal hierarchy (See/Do/Imagine) with auto-escalation. Plugin-extensible ExpansionRegistry |
| **ThinkingTemplates** | `thinking_template.rs` | 12 base cognitive styles (Analytical, Creative, Systematic, etc.) + 244 custom variants. Fingerprint-encoded for HDR resonance matching. YAML configuration |
| **BlackboardAgent** | `blackboard_agent.rs` | Per-agent awareness state: reasoning coherence (0.0-1.0), task history (50 records), knowledge fingerprints, flow state, ice-caked commitments. Confidence-vs-coherence gap detection |
| **KernelExtensions** | `kernel_extensions.rs` | FilterPipeline (12 phases: Pre/PostBind, Query, Resonate, Collapse, Inference, Escalation). KernelGuardrail (content categories, denied topics with fingerprint matching, PII detection, grounding verification). WorkflowNode (Sequential/Parallel/Loop/Conditional DAGs). MemoryBank (episodic/semantic/procedural). ObservabilityManager (sessions/traces/spans). VerificationEngine (density, distance, zone, parity, truth consistency, causal ordering) |

### Arrow Flight Integration (`src/flight/`)

| File | What It Does |
|------|-------------|
| **crew_actions.rs** | 50+ DoAction handlers already mapping crewAI operations to Arrow IPC: register agents/templates, submit/dispatch tasks, A2A send/receive, persona attachment/compatibility, handover evaluation/execution, task routing, affinity scoring, kernel introspection, filter/guardrail management, memory store/recall, observability sessions/traces, verification rules |
| **server.rs** | Arrow Flight gRPC server |
| **capabilities.rs** | Server capability advertisement |

### Cognitive Substrate

| Component | Files | What It Does |
|-----------|-------|-------------|
| **BindSpace** | `storage/bind_space.rs` | 65,536-address O(1) lookup registry (8-bit prefix + 8-bit slot). Reserved prefixes: 0x0C Agents, 0x0D Thinking, 0x0E Blackboards, 0x0F A2A Routing |
| **GrammarTriangle** | `grammar/triangle.rs` | NSM (65 universal primes) + Causality + Qualia → 10K-bit fingerprints. Language-independent semantic encoding |
| **NARS Engine** | `nars/` | 4 inference rules (deduction, induction, abduction, revision). Evidence tracking. KernelTruth {frequency, confidence} |
| **4096 CAM Operations** | `learning/cam_ops.rs` | Content-addressable methods across 16 ranges (Lance, SQL, Cypher, Hamming, NARS, Filesystem, Crystal, NSM, ACT-R, RL, Causal, Qualia, Rung, Meta, Learning, User-defined) |
| **Fabric** | `fabric/` | Firefly frames (1250-bit packed, 156 bytes), UDP transport, zero-copy IPC, mRNA resonance fields |
| **Counterfactuals** | `world/counterfactual.rs` | Copy-on-write database forks, diff_forks(), merge with confidence thresholds |
| **Crystal LM** | `extensions/crystal_lm.rs` | 140M:1 compression for long-term memory compaction |

---

## Architecture Principle (Unchanged)

Every integration is a **new file** behind a `ladybug` feature flag. No existing file is modified except for:
- `Cargo.toml` — add `ladybug` feature and dependency
- `src/lib.rs` — add `#[cfg(feature = "ladybug")] pub mod ladybug;`
- Module `mod.rs` files — add `#[cfg(feature = "ladybug")]` re-exports where needed

Everything else is additive. If the `ladybug` feature is disabled, crewAI-rust compiles and behaves exactly as before.

---

## Integration Layer 1: CrewBridge Adapter (Core — This Replaces crewAI Stubs)

The single most important integration. crewAI-rust's P0 technical debt (LLM unimplemented, Agent executor unimplemented, Crew kickoff unimplemented, Task execute unimplemented) can be resolved by delegating execution to ladybug's `CrewBridge`.

### Files to Create

**`src/ladybug/bridge.rs`** — The main adapter connecting crewAI's `Crew::kickoff()` to ladybug's `CrewBridge`:

```rust
use ladybug_rs::orchestration::crew_bridge::{CrewBridge, CrewTask, CrewDispatch, TaskStatus};
use ladybug_rs::orchestration::meta_orchestrator::MetaOrchestrator;
use ladybug_rs::orchestration::semantic_kernel::SemanticKernel;
use ladybug_rs::storage::bind_space::BindSpace;

pub struct LadybugCrewExecutor {
    bridge: CrewBridge,
    space: BindSpace,
    config: LadybugExecutorConfig,
}

pub struct LadybugExecutorConfig {
    pub handover_policy: Option<HandoverPolicy>,
    pub enable_guardrails: bool,
    pub enable_observability: bool,
    pub enable_verification: bool,
    pub memory_kinds: Vec<MemoryKind>,
    pub thinking_templates_yaml: Option<String>,
}

impl LadybugCrewExecutor {
    pub fn new(config: LadybugExecutorConfig) -> Self;

    /// Convert a crewAI Crew into a CrewDispatch and execute via CrewBridge.
    pub fn execute_crew(&mut self, crew: &Crew) -> Result<CrewOutput, CrewAIError> {
        // 1. Register each crewAI Agent as a ladybug AgentCard
        // 2. Attach Persona from agent's role/goal/backstory
        // 3. Register thinking templates if provided
        // 4. Convert crewAI Tasks → CrewTasks with dependency chain
        // 5. dispatch_crew() through CrewBridge
        // 6. Run tick_orchestrator() loop with handover evaluation
        // 7. Collect results, map back to crewAI CrewOutput
    }

    /// Access the semantic kernel for direct operations.
    pub fn kernel(&self) -> &SemanticKernel;

    /// Access the BindSpace for fingerprint operations.
    pub fn space(&self) -> &BindSpace;

    /// Get crew-wide awareness metrics.
    pub fn awareness(&self) -> f32;
}
```

### What This Enables

| crewAI P0 Stub | Resolution via CrewBridge |
|-----------------|--------------------------|
| `Crew::kickoff()` unimplemented | `bridge.dispatch_crew()` + `tick_orchestrator()` loop |
| `Agent::execute_task()` unimplemented | `bridge.submit_task()` routes to best agent via persona+affinity scoring |
| `Task::execute()` unimplemented | `CrewTask` lifecycle: Queued → Assigned → InProgress → Completed/Failed/Delegated |
| Static task delegation | `bridge.route_task()` uses Hamming similarity between task fingerprint and agent capability fingerprints |
| No inter-agent communication | `bridge.send_a2a()` / `receive_a2a()` with 8 message types over XOR-composed channels |

---

## Integration Layer 2: Persona-Enriched Agents

crewAI agents have `role`, `goal`, `backstory`. Ladybug personas add 5-axis volition, communication style, personality traits, feature proficiencies, and fingerprint-based compatibility.

**`src/ladybug/persona_adapter.rs`**

```rust
/// Build a ladybug Persona from a crewAI Agent.
///
/// Maps crewAI's string-based agent definition to ladybug's
/// multi-dimensional persona model. The role/goal/backstory are analyzed
/// via GrammarTriangle to infer volition dimensions and communication style.
pub fn agent_to_persona(agent: &Agent) -> Persona {
    // 1. Parse role → infer technical_depth, formality, directness
    // 2. Parse goal → infer curiosity, autonomy, persistence, risk_tolerance
    // 3. Parse backstory → infer collaboration, affinities, aversions
    // 4. Convert agent tools → FeatureAd entries with CAM opcodes
    // 5. Build fingerprint for HDR compatibility matching
}

/// Find the best agent for a task using persona fingerprint matching.
///
/// This replaces crewAI's static delegation with semantic routing:
/// - Fingerprint the task description via GrammarTriangle
/// - Hamming-compare against all registered agent persona fingerprints
/// - Score by: volition alignment + feature match + affinity history
pub fn route_task_to_agent(
    task: &Task,
    personas: &PersonaRegistry,
    orchestrator: &MetaOrchestrator,
) -> Option<(u8, f32)>;
```

---

## Integration Layer 3: Semantic Memory (Replaces Stub Memory Backends)

crewAI-rust has `Storage` trait with stub implementations for RAG, Mem0, and SQLite. Ladybug provides a unified memory system with episodic/semantic/procedural types, blackboard awareness, and ice-caked committed facts.

**`src/memory/storage/ladybug_storage.rs`**

```rust
pub struct LadybugMemoryStorage {
    bridge: Arc<RwLock<CrewBridge>>,
    agent_slot: u8,
}

impl Storage for LadybugMemoryStorage {
    fn save(&self, value: &str, metadata: &HashMap<String, Value>) -> Result<()> {
        // 1. Determine MemoryKind from metadata (episodic/semantic/procedural)
        // 2. Store via bridge.memory (MemoryBank)
        // 3. Record in agent's blackboard for awareness
    }

    fn search(&self, query: &str, limit: usize, score_threshold: f64) -> Result<Vec<Value>> {
        // 1. Check blackboard ice-caked layers (committed facts) first
        // 2. Resonate across MemoryBank via kernel.resonate()
        // 3. NARS-score results for truth values
        // 4. Merge ice-caked facts (boosted) with search results
    }
}
```

---

## Integration Layer 4: Guardrailed Execution

crewAI-rust has a `Guardrail` type but no implementation. Ladybug has a full `KernelGuardrail` with content filtering, denied topic detection (via fingerprint similarity), PII detection, and grounding verification.

**`src/ladybug/guardrail_adapter.rs`**

```rust
pub struct LadybugGuardrail {
    guardrail: KernelGuardrail,
    kernel: Arc<SemanticKernel>,
    space: Arc<RwLock<BindSpace>>,
}

impl LadybugGuardrail {
    /// Check task output against content safety, denied topics, PII,
    /// and factual grounding (verifying claims against stored knowledge).
    pub fn validate(&self, output: &str) -> GuardrailResult;

    /// Add a denied topic by description — automatically fingerprinted
    /// for fast Hamming-distance matching at validation time.
    pub fn deny_topic(&mut self, name: &str, description: &str, threshold: f32);
}
```

---

## Integration Layer 5: Observable Execution

crewAI-rust has telemetry stubs. Ladybug has `ObservabilityManager` with sessions, traces, spans, and grounding metadata.

**`src/ladybug/observability_adapter.rs`**

```rust
pub struct LadybugObservability {
    manager: Arc<RwLock<ObservabilityManager>>,
}

impl LadybugObservability {
    /// Create a session for a crew execution.
    pub fn start_session(&mut self, crew_id: &str) -> String;

    /// Create a trace for a task within a session.
    pub fn start_trace(&mut self, session_id: &str, task_id: &str) -> String;

    /// Record a span (agent action, tool call, handover, etc).
    pub fn record_span(&mut self, trace_id: &str, operation: &str, duration_ns: u64);

    /// Annotate a trace with grounding metadata.
    pub fn add_grounding(&mut self, trace_id: &str, grounding: GroundingMetadata);

    /// Export traces as OpenTelemetry-compatible spans.
    pub fn export_otel(&self) -> Vec<OtelSpan>;
}
```

---

## Integration Layer 6: Workflow DAGs

crewAI has sequential and hierarchical process types. Ladybug has full `WorkflowNode` DAGs with Sequential, Parallel, Loop, and Conditional execution — all operating on BindSpace without requiring LLM invocation for control flow.

**`src/ladybug/workflow_adapter.rs`**

```rust
pub struct LadybugWorkflowExecutor {
    kernel: Arc<SemanticKernel>,
    space: Arc<RwLock<BindSpace>>,
}

impl LadybugWorkflowExecutor {
    /// Convert a crewAI sequential/hierarchical process into a WorkflowNode DAG.
    pub fn from_crew_process(crew: &Crew) -> WorkflowNode;

    /// Execute a workflow, returning per-step results.
    pub fn execute(&mut self, workflow: &WorkflowNode) -> WorkflowResult;

    /// Build a conditional workflow: if agent confidence > threshold, proceed;
    /// else delegate to another agent or escalate.
    pub fn conditional_delegation(
        task: &CrewTask,
        condition_addr: Addr,
        branches: Vec<(f32, WorkflowNode)>,
    ) -> WorkflowNode;
}
```

---

## Integration Layer 7: RAG + Embeddings (Same as v1 but uses existing infrastructure)

**`src/rag/ladybug/client.rs`** — `BaseClient` backed by ladybug's `Database`:
- Vector mode: Lance `nearest_to()` with IVF-PQ
- Hamming mode: AVX-512 SIMD on 10K-bit fingerprints (400M ops/sec)
- Hybrid mode: Vector pre-filter → Hamming re-rank

**`src/rag/embeddings/providers/ladybug/mod.rs`** — `BaseEmbeddingsProvider` backed by `GrammarTriangle`:
- DenseVector (1024-dim projected from 65 NSM weights)
- BinaryFingerprint (10K-bit)
- Dual (both, for hybrid search)

---

## Integration Layer 8: MCP Transports

**`src/mcp/transports/arrow_flight.rs`** — Arrow Flight transport:
- Maps MCP operations to the 50+ DoAction handlers already implemented in ladybug's `crew_actions.rs`
- Zero-copy `RecordBatch` for tabular tool results

**`src/mcp/transports/hamming_udp.rs`** — Firefly frame transport:
- 1250-bit packed frames (156 bytes)
- Sub-millisecond latency for same-network agent communication
- Automatic HTTP fallback on delivery failure

---

## Integration Layer 9: A2A with Semantic Discovery

crewAI-rust has A2A stubs with `AgentCard`. Ladybug has a full `AgentCard` + `A2AProtocol` with XOR-composed channels and persona exchange.

**`src/a2a/ladybug_card.rs`**

```rust
pub struct LadybugAgentCard {
    base_config: A2AServerConfig,
    agent_card: ladybug_rs::orchestration::agent_card::AgentCard,
    persona: Persona,
}

impl LadybugAgentCard {
    /// Semantic capability query: "can you help with X?"
    /// Fingerprints the question, Hamming-matches against agent capabilities.
    pub fn discover_for_task(&self, task_description: &str) -> Vec<DiscoveredCapability>;

    /// Exchange persona profiles between agents for adaptive communication.
    pub fn exchange_persona(&self, target_slot: u8) -> PersonaExchange;
}
```

---

## Integration Layer 10: Verification Engine

No equivalent in crewAI. Ladybug provides deterministic verification rules that validate fingerprint operations without LLM invocation.

**`src/ladybug/verification_adapter.rs`**

```rust
pub struct LadybugVerifier {
    engine: VerificationEngine,
}

impl LadybugVerifier {
    /// Verify that an agent's output meets structural constraints.
    pub fn verify_output(&self, output: &str, rules: &[VerificationRule]) -> Vec<VerificationResult>;

    /// Built-in rules:
    /// - MinimumDensity: output fingerprint must have min popcount
    /// - TruthConsistency: NARS truth values don't contradict
    /// - CausalOrdering: temporal dependencies respected
    /// - ZoneConstraint: data stays in allowed BindSpace zones
    pub fn default_rules() -> Vec<VerificationRule>;
}
```

---

## Integration Layer 11: Counterfactual Exploration

**`src/ladybug/counterfactual.rs`**

```rust
pub struct CounterfactualExplorer {
    db: Arc<Database>,
}

impl CounterfactualExplorer {
    /// Fork the world state to explore a hypothesis.
    pub fn fork(&self, agent_id: &str, hypothesis: &str) -> Result<ForkHandle>;

    /// Compare outcomes between two forks.
    pub fn diff(&self, a: &ForkHandle, b: &ForkHandle) -> ForkDiff;

    /// Merge a fork back if the hypothesis held (confidence-gated).
    pub fn merge(&self, fork: ForkHandle, min_confidence: f64) -> Result<()>;
}
```

---

## What Was Previously "Nice-to-Have" but Is Now Implemented

The v1 prompt listed these as speculative features. They already exist in ladybug-rs:

| v1 "Nice-to-Have" | Ladybug Implementation |
|--------------------|----------------------|
| N4. Semantic Agent Routing | `MetaOrchestrator::route_task()` + `PersonaRegistry::best_for_task()` |
| N7. Resonance-Based Knowledge Propagation | `fabric/mrna.rs` mRNA resonance fields |
| N3. Live Reasoning Trace | `ObservabilityManager` with sessions/traces/spans |
| N8. Confidence-Gated Tool Execution | `KernelGuardrail` + `KernelTruth` confidence checks |
| Workflow DAGs | `WorkflowNode` (Sequential/Parallel/Loop/Conditional) |
| Filter Pipeline | `FilterPipeline` with 12 phases |
| Content Guardrails | `KernelGuardrail` with fingerprint-matched denied topics |
| Agent Personality | `PersonaRegistry` with 5-axis volition + communication style |
| Affinity Learning | `AffinityEdge` with 60% static + 40% dynamic blending |
| Dunning-Kruger Detection | `HandoverPolicy::dk_gap_threshold` |

---

## Remaining Nice-to-Have (Genuinely Not Yet Implemented)

### N1. Adaptive Search Mode Selection

The RAG adapter automatically selects vector/hamming/hybrid per query based on query length, type detection, and historical performance tracking.

### N2. Agent Memory Consolidation

Post-crew-execution: identify frequently accessed memories, NARS revision on conflicts, consolidate into shared "crew memory", crystal-compress old per-agent memories.

### N3. Multi-Lingual Agent Crews

GrammarTriangle's 65 NSM universal primitives produce language-independent fingerprints. Agents operating in different languages produce comparable semantic representations. Add German, Spanish, Japanese translations.

### N4. Crew Execution Replay

Lance versioning + blackboard history → step-through replay of any crew execution with fork-and-try-alternatives at any decision point.

### N5. Crystal LM Memory Compaction

Use `extensions/crystal_lm.rs` (140M:1 compression) for long-term agent memory. Decompress on demand during search.

---

## Module 12: xAI / Grok LLM Provider

(Unchanged from v1 — independent of ladybug integration)

**`src/llms/providers/xai/mod.rs`**

```rust
pub struct XaiCompletion {
    state: BaseLLMState,
    api_key: String,
    base_url: String,
    reasoning_effort: Option<String>,
    search_enabled: bool,
}

impl BaseLLM for XaiCompletion {
    fn provider(&self) -> &str { "xai" }
    fn supports_function_calling(&self) -> bool { true }
    fn supports_multimodal(&self) -> bool { true }
    fn get_context_window_size(&self) -> usize { 131072 }
}
```

---

## Module 13: German Translations

(Unchanged from v1 — independent of ladybug integration)

---

## Cargo.toml Changes

```toml
[features]
default = []
ladybug = ["dep:ladybug-rs"]
xai = []
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

**`src/ladybug/mod.rs`**:

```rust
//! Ladybug-rs integration for crewAI.
//!
//! Enable with `features = ["ladybug"]` in Cargo.toml.
//!
//! This replaces crewAI's stub execution layer with ladybug's cognitive
//! substrate: CrewBridge orchestration, persona-based routing, semantic
//! kernel operations, guardrails, observability, and verification.

pub mod bridge;
pub mod persona_adapter;
pub mod guardrail_adapter;
pub mod observability_adapter;
pub mod workflow_adapter;
pub mod verification_adapter;
pub mod counterfactual;

pub use bridge::{LadybugCrewExecutor, LadybugExecutorConfig};
pub use persona_adapter::{agent_to_persona, route_task_to_agent};
pub use guardrail_adapter::LadybugGuardrail;
pub use observability_adapter::LadybugObservability;
pub use workflow_adapter::LadybugWorkflowExecutor;
pub use verification_adapter::LadybugVerifier;
pub use counterfactual::CounterfactualExplorer;

// Re-export storage/transport adapters
pub use crate::rag::ladybug::client::LadybugRagClient;
pub use crate::rag::ladybug::config::{LadybugRagConfig, SearchMode, EmbeddingMode};
pub use crate::rag::embeddings::providers::ladybug::LadybugEmbeddingProvider;
pub use crate::memory::storage::ladybug_storage::LadybugMemoryStorage;
pub use crate::mcp::transports::arrow_flight::ArrowFlightTransport;
pub use crate::mcp::transports::hamming_udp::HammingUdpTransport;
pub use crate::a2a::ladybug_card::LadybugAgentCard;
```

---

## Testing Strategy

```
tests/
    ladybug_bridge_test.rs          // CrewBridge adapter: register agents, dispatch, tick
    ladybug_persona_test.rs         // Agent→Persona conversion, compatibility, routing
    ladybug_guardrail_test.rs       // Content filtering, denied topics, grounding
    ladybug_observability_test.rs   // Sessions, traces, spans, OTEL export
    ladybug_workflow_test.rs        // DAG execution: sequential, parallel, loop, conditional
    ladybug_verification_test.rs    // Rule-based output verification
    ladybug_counterfactual_test.rs  // Fork, diff, merge
    ladybug_rag_test.rs             // Vector/Hamming/Hybrid search
    ladybug_embedding_test.rs       // GrammarTriangle embedding generation
    arrow_flight_test.rs            // Arrow Flight transport via crew_actions
    hamming_udp_test.rs             // Firefly frame transport with fallback
    ladybug_memory_test.rs          // Episodic/semantic/procedural memory
    ladybug_a2a_test.rs             // A2A discovery, persona exchange
    xai_provider_test.rs            // xAI API call (mock server)
    german_translations_test.rs     // de.json structure matches en.json
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
8. **Delegate, don't duplicate**: Where ladybug already implements a capability (orchestration, guardrails, workflows, etc.), wrap it — do not reimplement it in crewAI

---

## Key Insight: What Makes This Integration Different

The v1 prompt treated ladybug-rs as a storage backend to plug into crewAI's architecture. That undersells both projects.

**The correct framing**: crewAI-rust provides the **agent definition API** (Agent, Task, Crew, Tool, Memory trait signatures, I18N, process types) — the interface developers use. Ladybug-rs provides the **execution substrate** (orchestration, routing, awareness, verification, guardrails, observability) — the engine that makes agents actually work.

With this integration:
- `Crew::kickoff()` delegates to `CrewBridge::dispatch_crew()`
- Agent delegation uses persona fingerprint matching instead of static assignment
- Task handovers are flow-state-aware with Dunning-Kruger gap detection
- Memory includes episodic/semantic/procedural types with ice-caked committed facts
- Every execution is observable, guardrailed, and verifiable without LLM invocation
- Agents develop affinity over repeated collaborations
- Counterfactual forks enable hypothesis testing without corrupting shared state
