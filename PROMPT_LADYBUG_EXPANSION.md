# Ladybug-rs Expansion Prompt: crewAI Integration Surface (v2)

## Context

This prompt specifies the expansions needed in **ladybug-rs** (`AdaWorldAPI/ladybug-rs`) to serve as the cognitive backbone for **crewAI-rust** (`/lib/crewai-rust/`), a 1:1 Rust port of the crewAI Python multi-agent orchestration framework.

**What has changed since v1**: A thorough audit of ladybug-rs reveals that most of what v1 proposed as "to be built" already exists. The orchestration layer (`CrewBridge`, `MetaOrchestrator`, `SemanticKernel`, `A2AProtocol`, `PersonaRegistry`, `HandoverPolicy`, `KernelExtensions`) is fully implemented. This v2 prompt focuses on:
1. Gaps that actually remain
2. Architecture patterns harvested from Amazon Bedrock AgentCore that should inform ladybug-rs's agent infrastructure
3. Production-hardening the existing integration surface

**Reference architecture**: `AdaWorldAPI/amazon-bedrock-agentcore-samples` demonstrates enterprise-grade agent infrastructure patterns that ladybug-rs should match or exceed:
- **Runtime**: Serverless agent deployment with session isolation, middleware support, bidirectional streaming
- **Gateway**: Automatic API → MCP tool conversion
- **Memory**: Managed persistent memory with session continuity
- **Identity**: Agent identity and access management across services
- **Observability**: OpenTelemetry-native tracing
- **Multi-agent**: Supervisor patterns, A2A orchestration, distributed hosting

---

## What Already Exists (Audit Confirmed — Do NOT Reimplement)

| Component | File | Status |
|-----------|------|--------|
| CrewBridge | `orchestration/crew_bridge.rs` | Complete — agents, personas, blackboards, A2A, orchestrator, kernel, filters, guardrails, memory, observability, verification all composed |
| MetaOrchestrator | `orchestration/meta_orchestrator.rs` | Complete — affinity graph, resonance routing, flow-state handovers |
| A2AProtocol | `orchestration/a2a.rs` | Complete — 8 message types, XOR superposition channels, awareness metrics |
| PersonaRegistry | `orchestration/persona.rs` | Complete — 5-axis volition, communication style, fingerprint compatibility |
| HandoverPolicy | `orchestration/handover.rs` | Complete — 7 thresholds, DK gap detection, metacognitive review |
| SemanticKernel | `orchestration/semantic_kernel.rs` | Complete — 20+ ops, NARS truth, Pearl's 3-rung, plugin registry |
| ThinkingTemplates | `orchestration/thinking_template.rs` | Complete — 12 base + 244 custom, HDR matching |
| BlackboardAgent | `orchestration/blackboard_agent.rs` | Complete — per-agent state, ice-caking, coherence tracking |
| KernelExtensions | `orchestration/kernel_extensions.rs` | Complete — filters, guardrails, workflows, memory, observability, verification |
| Arrow Flight crew_actions | `flight/crew_actions.rs` | Complete — 50+ DoAction handlers for crewAI operations |
| BindSpace | `storage/bind_space.rs` | Complete — 65,536 O(1) addresses |
| GrammarTriangle | `grammar/triangle.rs` | Complete — NSM + Causality + Qualia → 10K-bit fingerprints |
| NARS Engine | `nars/` | Complete — 4 inference rules, evidence tracking |
| Fabric | `fabric/` | Complete — Firefly frames, UDP transport, zero-copy, mRNA resonance |
| Counterfactuals | `world/counterfactual.rs` | Complete — fork, diff, merge |

---

## Required Expansions

### 1. Agent Runtime Layer (Inspired by Bedrock AgentCore Runtime)

Bedrock AgentCore provides a serverless runtime with session isolation, middleware chains, and bidirectional streaming. Ladybug-rs should offer an equivalent Rust-native runtime that goes further.

#### 1.1 `AgentRuntime` — Managed Agent Lifecycle

Create `src/runtime/mod.rs`:

```rust
pub struct AgentRuntime {
    pub bridge: CrewBridge,
    pub space: BindSpace,
    pub sessions: SessionManager,
    pub middleware: MiddlewareChain,
    pub config: RuntimeConfig,
}

pub struct RuntimeConfig {
    pub max_concurrent_sessions: usize,
    pub session_timeout: Duration,
    pub enable_bidirectional_streaming: bool,
    pub middleware_stack: Vec<Box<dyn Middleware>>,
    pub identity_provider: Option<Box<dyn IdentityProvider>>,
}

impl AgentRuntime {
    /// Deploy an agent into the runtime. Returns a handle for invocation.
    pub fn deploy(&mut self, agent_card: AgentCard, persona: Persona) -> Result<AgentHandle>;

    /// Invoke an agent within an isolated session.
    /// Same session_id preserves context across calls (Bedrock pattern).
    pub async fn invoke(
        &self,
        agent_handle: &AgentHandle,
        session_id: &str,
        payload: Value,
    ) -> Result<AgentResponse>;

    /// Invoke with bidirectional streaming (WebSocket-like).
    pub async fn invoke_streaming(
        &self,
        agent_handle: &AgentHandle,
        session_id: &str,
        input_stream: impl Stream<Item = Value>,
    ) -> impl Stream<Item = AgentResponse>;

    /// Hot-reload an agent without dropping sessions.
    pub fn redeploy(&mut self, handle: &AgentHandle, new_card: AgentCard) -> Result<()>;

    /// Get runtime metrics: active sessions, agent utilization, queue depth.
    pub fn metrics(&self) -> RuntimeMetrics;
}
```

#### 1.2 `SessionManager` — Isolated Session State

```rust
pub struct SessionManager {
    sessions: HashMap<String, AgentSession>,
    isolation_mode: IsolationMode,
}

pub struct AgentSession {
    pub id: String,
    pub agent_slot: u8,
    pub created_at: DateTime<Utc>,
    pub last_active: DateTime<Utc>,
    pub state: SessionState,
    pub memory_snapshot: Option<Addr>,
    pub blackboard_snapshot: Option<Addr>,
}

pub enum IsolationMode {
    /// Each session gets its own BindSpace prefix range (strongest isolation).
    PrefixIsolated,
    /// Sessions share BindSpace but blackboard entries are tagged (moderate isolation).
    TagIsolated,
    /// Shared state — all sessions see all data (for collaborative crews).
    Shared,
}

impl SessionManager {
    /// Create or resume a session. Bedrock pattern: same ID = same context.
    pub fn get_or_create(&mut self, session_id: &str, agent_slot: u8) -> &mut AgentSession;

    /// Persist session state to Lance for durability.
    pub async fn persist(&self, session_id: &str) -> Result<()>;

    /// Restore session state from Lance.
    pub async fn restore(&mut self, session_id: &str) -> Result<()>;

    /// Expire sessions that have been idle beyond timeout.
    pub fn expire_idle(&mut self, timeout: Duration) -> Vec<String>;
}
```

#### 1.3 `MiddlewareChain` — Request/Response Interceptors

Inspired by Bedrock's ASGI middleware support. Ladybug's version operates on fingerprints instead of HTTP.

```rust
pub trait Middleware: Send + Sync {
    /// Process before agent execution. Can modify the request or short-circuit.
    fn before(&self, ctx: &mut MiddlewareContext) -> MiddlewareAction;

    /// Process after agent execution. Can modify the response.
    fn after(&self, ctx: &mut MiddlewareContext, response: &mut AgentResponse);
}

pub enum MiddlewareAction {
    Continue,
    ShortCircuit(AgentResponse),
}

pub struct MiddlewareContext {
    pub session_id: String,
    pub agent_slot: u8,
    pub payload: Value,
    pub fingerprint: Option<[u64; FINGERPRINT_WORDS]>,
    pub metadata: HashMap<String, String>,
    pub timing: MiddlewareTiming,
}
```

**Built-in middleware** (shipped with ladybug-rs):

| Middleware | Purpose |
|-----------|---------|
| `GuardrailMiddleware` | Run KernelGuardrail on input/output |
| `ObservabilityMiddleware` | Auto-create spans for every invocation |
| `RateLimitMiddleware` | Token bucket rate limiting per session |
| `AuthMiddleware` | Validate agent identity tokens |
| `FingerprintCacheMiddleware` | Cache fingerprint computations for repeated queries |
| `FilterMiddleware` | Run FilterPipeline on request/response |

---

### 2. Gateway — API-to-Tool Conversion (Inspired by Bedrock AgentCore Gateway)

Bedrock converts Lambda functions and APIs into MCP-compatible tools. Ladybug should convert arbitrary services into CAM-addressable operations.

#### 2.1 `ToolGateway` — Automatic Service Discovery

Create `src/gateway/mod.rs`:

```rust
pub struct ToolGateway {
    pub tools: Vec<GatewayTool>,
    pub space: Arc<RwLock<BindSpace>>,
}

pub struct GatewayTool {
    pub name: String,
    pub description: String,
    pub cam_opcode: u16,
    pub source: ToolSource,
    pub fingerprint: [u64; FINGERPRINT_WORDS],
    pub input_schema: Value,
    pub output_schema: Value,
}

pub enum ToolSource {
    /// HTTP REST endpoint (OpenAPI spec)
    OpenApi { spec: Value, base_url: String },
    /// MCP server (stdio, HTTP, or SSE)
    Mcp { transport: TransportConfig },
    /// Arrow Flight DoAction
    ArrowFlight { endpoint: String, action: String },
    /// Local Rust function
    Native { handler: Arc<dyn Fn(Value) -> Result<Value> + Send + Sync> },
    /// CAM operation (internal)
    CamOp { opcode: u16 },
}

impl ToolGateway {
    /// Import tools from an OpenAPI specification.
    /// Each endpoint becomes a GatewayTool with auto-generated fingerprint.
    pub fn import_openapi(&mut self, spec: &Value, base_url: &str) -> Result<Vec<u16>>;

    /// Import tools from an MCP server's tool list.
    pub fn import_mcp(&mut self, transport: &TransportConfig) -> Result<Vec<u16>>;

    /// Discover tools semantically: "find tools that can do X"
    /// Fingerprints the query and Hamming-matches against all tool fingerprints.
    pub fn discover(&self, description: &str, limit: usize) -> Vec<(GatewayTool, f32)>;

    /// Invoke a tool by name or CAM opcode.
    pub async fn invoke(&self, tool_id: &str, args: Value) -> Result<Value>;

    /// Bind all tools into BindSpace for kernel operations.
    pub fn bind_all(&self, space: &mut BindSpace);
}
```

This goes beyond Bedrock's Gateway: tools are fingerprinted for semantic discovery, addressable by CAM opcode for O(1) dispatch, and stored in BindSpace alongside agents and knowledge.

---

### 3. Agent Identity Layer (Inspired by Bedrock AgentCore Identity)

Bedrock provides agent identity across AWS services. Ladybug should provide identity that works with any service.

#### 3.1 `AgentIdentity` — Cryptographic Agent Identity

Create `src/identity/mod.rs`:

```rust
pub struct AgentIdentity {
    pub agent_slot: u8,
    pub agent_id: String,
    pub keypair: Ed25519Keypair,
    pub capabilities: Vec<Capability>,
    pub issued_at: DateTime<Utc>,
    pub expires_at: Option<DateTime<Utc>>,
}

pub struct Capability {
    pub resource: String,
    pub actions: Vec<String>,
    pub conditions: Option<Value>,
}

pub trait IdentityProvider: Send + Sync {
    /// Issue an identity for a newly registered agent.
    fn issue(&self, agent_card: &AgentCard) -> Result<AgentIdentity>;

    /// Verify an agent's identity token.
    fn verify(&self, token: &str) -> Result<AgentIdentity>;

    /// Check if an agent has a specific capability.
    fn authorize(&self, identity: &AgentIdentity, resource: &str, action: &str) -> bool;
}

impl AgentIdentity {
    /// Sign a message (tool call, A2A message, etc.) with this agent's key.
    pub fn sign(&self, message: &[u8]) -> Signature;

    /// Verify a signature from another agent.
    pub fn verify_signature(&self, message: &[u8], signature: &Signature, peer_id: &AgentIdentity) -> bool;

    /// Create a delegation token: agent A authorizes agent B to act on its behalf.
    pub fn delegate(&self, delegatee: &AgentIdentity, scope: Vec<Capability>, ttl: Duration) -> DelegationToken;
}
```

This enables:
- Agents sign tool calls and A2A messages → non-repudiation
- Delegation chains: orchestrator delegates capabilities to sub-agents
- Capability-based access control per agent

---

### 4. Enhanced crewAI Compatibility Layer

The existing `crewai_compat` module needs updating to match the newly audited crewAI-rust trait signatures.

#### 4.1 Update `LadybugClient` — `BaseClient` implementation

Ensure the `BaseClient` implementation matches the exact crewAI-rust trait:

```rust
#[async_trait]
pub trait BaseClient: Send + Sync {
    fn create_collection(&self, params: CollectionParams) -> Result<()>;
    fn get_or_create_collection(&self, params: CollectionParams) -> Result<()>;
    fn add_documents(&self, params: CollectionAddParams) -> Result<()>;
    fn search(&self, params: CollectionSearchParams) -> Result<Vec<SearchResult>>;
    fn delete_collection(&self, params: CollectionParams) -> Result<()>;
    fn reset(&self) -> Result<()>;
}
```

The implementation must support all three search modes and return NARS-annotated results when enabled.

#### 4.2 Update `LadybugMemoryStorage` — `Storage` trait

Match the exact crewAI-rust `Storage` trait:

```rust
pub trait Storage: Send + Sync {
    fn save(&self, value: &str, metadata: &HashMap<String, Value>) -> Result<()>;
    fn search(&self, query: &str, limit: usize, score_threshold: f64) -> Result<Vec<Value>>;
    fn reset(&self) -> Result<()>;
}
```

#### 4.3 Update `LadybugKnowledgeStorage` — `BaseKnowledgeStorage` trait

```rust
pub trait BaseKnowledgeStorage: Send + Sync {
    fn search(&self, query: &str, limit: usize, score_threshold: f64) -> Result<Vec<Value>>;
    fn save(&self, documents: Vec<String>) -> Result<()>;
    fn save_chunks(&self, chunks: Vec<String>, metadata: Vec<HashMap<String, Value>>) -> Result<()>;
    fn reset(&self) -> Result<()>;
}
```

---

### 5. Multi-Agent Hosting Patterns (Inspired by Bedrock Multi-Runtime)

Bedrock demonstrates hosting multiple agents in separate runtimes with a supervisor orchestrator. Ladybug should support this natively.

#### 5.1 `DistributedCrewBridge` — Multi-Process Agent Hosting

```rust
pub struct DistributedCrewBridge {
    local_bridge: CrewBridge,
    remote_agents: HashMap<u8, RemoteAgentHandle>,
    discovery: AgentDiscoveryService,
}

pub struct RemoteAgentHandle {
    pub agent_slot: u8,
    pub endpoint: String,
    pub transport: RemoteTransport,
    pub health: AgentHealth,
    pub persona_cache: Option<Persona>,
}

pub enum RemoteTransport {
    ArrowFlight(String),
    HammingUdp { endpoint: String, lane: u8 },
    Http(String),
}

impl DistributedCrewBridge {
    /// Register a remote agent accessible via Arrow Flight.
    pub fn register_remote(&mut self, endpoint: &str) -> Result<u8>;

    /// Discover agents on the local network via UDP broadcast.
    pub fn discover_agents(&mut self, timeout: Duration) -> Vec<RemoteAgentHandle>;

    /// Submit a task — routes to local or remote agent transparently.
    pub fn submit_task(&mut self, task: CrewTask) -> DispatchResult;

    /// Health check all remote agents.
    pub fn health_check(&mut self) -> Vec<(u8, AgentHealth)>;
}
```

---

### 6. Managed Memory with Session Continuity (Inspired by Bedrock AgentCore Memory)

Bedrock provides managed memory enabling "rich, personalized agent experiences." Ladybug's `MemoryBank` needs persistence and session-aware recall.

#### 6.1 Memory Persistence

Extend `MemoryBank` in `kernel_extensions.rs`:

```rust
impl MemoryBank {
    /// Persist all memories to Lance storage.
    pub async fn persist(&self, db: &Database) -> Result<()>;

    /// Load memories from Lance storage.
    pub async fn load(&mut self, db: &Database, agent_slot: Option<u8>) -> Result<usize>;

    /// Session-scoped recall: only return memories from the current session.
    pub fn recall_session(
        &self,
        session_id: &str,
        query: &str,
        limit: usize,
    ) -> Vec<&KernelMemory>;

    /// Cross-session recall: search all memories with session attribution.
    pub fn recall_global(
        &self,
        query: &str,
        limit: usize,
        score_threshold: f32,
    ) -> Vec<(&KernelMemory, f32)>;

    /// Memory consolidation: merge duplicates, NARS-revise conflicts,
    /// crystal-compress old memories.
    pub fn consolidate(&mut self, kernel: &SemanticKernel, space: &BindSpace) -> ConsolidationReport;
}

pub struct ConsolidationReport {
    pub duplicates_merged: usize,
    pub conflicts_revised: usize,
    pub memories_compressed: usize,
    pub total_after: usize,
}
```

---

### 7. OpenTelemetry-Native Observability (Inspired by Bedrock AgentCore Observability)

Bedrock uses OpenTelemetry for unified dashboards. Ladybug's `ObservabilityManager` should export OTEL-compatible data.

#### 7.1 OTEL Export

Extend `ObservabilityManager`:

```rust
impl ObservabilityManager {
    /// Export all traces as OpenTelemetry spans.
    pub fn export_otel(&self) -> Vec<OtelSpan>;

    /// Create a live OTEL exporter that streams spans as they're created.
    pub fn live_exporter(&self) -> OtelExporter;

    /// Dashboard summary: agent utilization, task throughput, error rates,
    /// handover frequency, memory usage, guardrail trigger rates.
    pub fn dashboard(&self) -> DashboardMetrics;
}

pub struct OtelSpan {
    pub trace_id: String,
    pub span_id: String,
    pub parent_span_id: Option<String>,
    pub operation_name: String,
    pub start_time: u64,
    pub end_time: u64,
    pub attributes: HashMap<String, String>,
    pub status: SpanStatus,
}

pub struct DashboardMetrics {
    pub agents_active: usize,
    pub tasks_completed: u64,
    pub tasks_failed: u64,
    pub avg_task_duration_ms: f64,
    pub handovers_executed: u64,
    pub guardrail_blocks: u64,
    pub memory_entries: u64,
    pub a2a_messages_sent: u64,
    pub kernel_ops_executed: u64,
}
```

---

### 8. Bidirectional Streaming Agent Support

Bedrock demonstrates bidirectional streaming for voice agents. Ladybug should support streaming for any data modality.

#### 8.1 Streaming Protocol

Create `src/runtime/streaming.rs`:

```rust
pub struct StreamingSession {
    pub session_id: String,
    pub agent_slot: u8,
    pub input_tx: mpsc::Sender<StreamChunk>,
    pub output_rx: mpsc::Receiver<StreamChunk>,
}

pub struct StreamChunk {
    pub sequence: u64,
    pub data: StreamData,
    pub fingerprint: Option<[u64; FINGERPRINT_WORDS]>,
}

pub enum StreamData {
    Text(String),
    Json(Value),
    Binary(Vec<u8>),
    ToolCall { name: String, args: Value },
    ToolResult { name: String, result: Value },
    Interrupt,
    Complete,
}

impl AgentRuntime {
    /// Open a bidirectional streaming session.
    pub async fn open_stream(
        &self,
        agent_handle: &AgentHandle,
        session_id: &str,
    ) -> Result<StreamingSession>;
}
```

---

### 9. Counterfactual Fork API Enhancements

The existing `world/counterfactual.rs` needs crewAI-friendly exposure:

```rust
impl Database {
    /// Fork the database for counterfactual exploration.
    /// Returns a new Database instance with copy-on-write semantics.
    pub fn fork_for_agent(&self, agent_id: &str, hypothesis: &str) -> Result<Database, Error>;

    /// Compare two forks: what changed between them?
    pub fn diff_forks(&self, other: &Database) -> ForkDiff;

    /// Merge a fork back if the hypothesis was confirmed.
    pub fn merge_fork(&mut self, fork: Database, merge_strategy: MergeStrategy) -> Result<(), Error>;
}

pub enum MergeStrategy {
    AcceptAll,
    ConfidenceThreshold(f64),
    Manual,
}
```

---

### 10. German Translations Support

Expose I18N capability for multi-lingual crews:

```rust
impl GrammarTriangle {
    /// Analyze text with language hint for improved NSM mapping.
    pub fn analyze_with_lang(&self, text: &str, lang: &str) -> AnalysisResult;
}
```

The 65 NSM universal primitives are inherently language-independent. Adding language hints improves tokenization and morphological analysis before NSM projection.

---

## File Structure (New Files Only)

```
src/
    runtime/
        mod.rs                  // AgentRuntime, RuntimeConfig
        session.rs              // SessionManager, AgentSession, IsolationMode
        middleware.rs           // Middleware trait, MiddlewareChain, built-in middleware
        streaming.rs            // StreamingSession, StreamChunk, StreamData
    gateway/
        mod.rs                  // ToolGateway, GatewayTool, ToolSource
        openapi.rs              // OpenAPI spec → GatewayTool conversion
        mcp_import.rs           // MCP server → GatewayTool import
    identity/
        mod.rs                  // AgentIdentity, IdentityProvider, Capability
        ed25519.rs              // Ed25519 keypair management
        delegation.rs           // DelegationToken, capability chains
    policy/
        mod.rs                  // PolicyEngine, PolicyRule, PolicyEffect
        cedar.rs                // Cedar import/export
        evaluator.rs            // Policy condition evaluation
    evaluation/
        mod.rs                  // EvaluationEngine, EvalContext, EvalScore
        evaluators/
            mod.rs              // Built-in evaluator registry
            tool_selection.rs   // ToolSelectionAccuracy
            helpfulness.rs      // OutputHelpfulness
            completeness.rs     // OutputCompleteness
            coherence.rs        // ReasoningCoherence
            grounding.rs        // FactualGrounding
            handover.rs         // HandoverQuality
            efficiency.rs       // ToolCallEfficiency
            memory.rs           // MemoryUtilization
            safety.rs           // SafetyCompliance
            policy.rs           // PolicyCompliance
            latency.rs          // LatencyBudget
            causal.rs           // CausalConsistency
            persona.rs          // PersonaAlignment
    crewai_compat/
        mod.rs                  // (existing, update re-exports)
        client.rs               // (existing, update trait signatures)
        memory_storage.rs       // (existing, update trait signatures)
        knowledge_storage.rs    // (existing, update trait signatures)
        distributed_bridge.rs   // DistributedCrewBridge
```

## Cargo.toml Additions

```toml
[features]
default = []
crewai = ["dep:crewai"]
runtime = ["tokio/full"]
gateway = []
identity = ["dep:ed25519-dalek"]
full = ["crewai", "runtime", "gateway", "identity"]

[dependencies]
crewai = { path = "../crewai-rust", optional = true }
ed25519-dalek = { version = "2", optional = true }
```

---

### 11. Policy Engine — Deterministic Action Control (Inspired by Bedrock AgentCore Policy)

Bedrock's Policy feature intercepts every tool call at the Gateway layer and enforces deterministic rules defined in natural language (compiled to Cedar policy language). This enforcement happens **outside the LLM reasoning loop** — it cannot be circumvented by prompt manipulation.

Ladybug already has `KernelGuardrail` for content filtering. This extends it with **action-level policy enforcement**.

Create `src/policy/mod.rs`:

```rust
pub struct PolicyEngine {
    pub rules: Vec<PolicyRule>,
    pub enforcement: EnforcementMode,
}

pub struct PolicyRule {
    pub name: String,
    pub description: String,           // Natural language definition
    pub effect: PolicyEffect,
    pub principal: PolicyPrincipal,     // Which agent(s) this applies to
    pub action: PolicyAction,           // Which operations are constrained
    pub resource: PolicyResource,       // Which targets are protected
    pub conditions: Vec<PolicyCondition>,
}

pub enum PolicyEffect {
    Allow,
    Deny,
}

pub enum PolicyPrincipal {
    AllAgents,
    Agent(u8),                          // Specific agent slot
    AgentWithRole(String),              // Agents matching a role
    AgentGroup(Vec<u8>),
}

pub enum PolicyAction {
    ToolCall(String),                   // Specific tool
    AnyToolCall,
    A2AMessage(MessageKind),
    MemoryWrite,
    MemoryRead,
    BlackboardCommit,
    Handover,
    CamOp(u16),                         // Specific CAM opcode
    Custom(String),
}

pub enum PolicyResource {
    Any,
    Tool(String),
    Collection(String),
    Zone(KernelZone),
    Prefix(u8),
    Custom(String),
}

pub struct PolicyCondition {
    pub key: String,
    pub operator: ConditionOperator,
    pub value: Value,
}

pub enum ConditionOperator {
    Equals,
    NotEquals,
    Contains,
    GreaterThan,
    LessThan,
    Matches(String),                    // Regex
}

pub enum EnforcementMode {
    /// Block denied actions (production)
    Strict,
    /// Log but allow denied actions (testing)
    AuditOnly,
    /// Block + escalate to orchestrator for review
    Escalate,
}

impl PolicyEngine {
    /// Evaluate a proposed action against all rules.
    /// Returns Allow/Deny with the rule that decided.
    pub fn evaluate(&self, request: &PolicyRequest) -> PolicyDecision;

    /// Define a rule from natural language description.
    /// The description is fingerprinted for semantic matching
    /// against future requests.
    pub fn add_rule_natural(&mut self, description: &str) -> Result<PolicyRule>;

    /// Export all rules as Cedar-compatible policy text.
    pub fn export_cedar(&self) -> String;

    /// Import rules from Cedar policy text.
    pub fn import_cedar(&mut self, cedar: &str) -> Result<usize>;
}

pub struct PolicyRequest {
    pub agent_slot: u8,
    pub action: PolicyAction,
    pub resource: PolicyResource,
    pub context: HashMap<String, Value>,
}

pub struct PolicyDecision {
    pub effect: PolicyEffect,
    pub rule_name: Option<String>,
    pub reason: String,
}
```

**Example policies**:
```
"Agent 3 cannot delete any collection"
"Only the orchestrator can initiate handovers"
"No agent can write to BindSpace zone 0x80-0xFF without verification"
"Tool calls to external APIs require confidence > 0.7"
```

---

### 12. Evaluation Engine — Continuous Quality Scoring (Inspired by Bedrock AgentCore Evaluations)

Bedrock Evaluations provides 13 built-in evaluators for helpfulness, tool selection accuracy, and output quality. Ladybug should provide fingerprint-native evaluation that's richer than LLM-based scoring.

Create `src/evaluation/mod.rs`:

```rust
pub struct EvaluationEngine {
    pub evaluators: Vec<Box<dyn Evaluator>>,
    pub history: Vec<EvaluationResult>,
}

pub trait Evaluator: Send + Sync {
    fn name(&self) -> &str;
    fn evaluate(&self, context: &EvalContext) -> EvalScore;
}

pub struct EvalContext {
    pub agent_slot: u8,
    pub task_description: String,
    pub task_output: String,
    pub tool_calls: Vec<ToolCallRecord>,
    pub handovers: Vec<HandoverRecord>,
    pub memory_accesses: Vec<MemoryAccessRecord>,
    pub duration_ms: u64,
    pub truth_values: Vec<KernelTruth>,
}

pub struct EvalScore {
    pub evaluator: String,
    pub score: f32,        // 0.0-1.0
    pub details: String,
    pub pass: bool,
}

pub struct EvaluationResult {
    pub session_id: String,
    pub agent_slot: u8,
    pub scores: Vec<EvalScore>,
    pub composite_score: f32,
    pub timestamp: u64,
}
```

**Built-in evaluators** (13, matching Bedrock):

| Evaluator | What It Measures |
|-----------|-----------------|
| `ToolSelectionAccuracy` | Did the agent pick the right tool? (Compare tool fingerprint vs task fingerprint similarity) |
| `OutputHelpfulness` | Does the output address the task? (Fingerprint similarity between task and output) |
| `OutputCompleteness` | Are all required outputs present? (Schema validation) |
| `ReasoningCoherence` | Does the reasoning chain hold? (NARS truth value consistency) |
| `FactualGrounding` | Are claims supported by evidence? (KernelGuardrail grounding check) |
| `HandoverQuality` | Were handovers justified? (FlowState + affinity delta) |
| `ToolCallEfficiency` | Were tool calls necessary? (Redundancy detection via fingerprint dedup) |
| `MemoryUtilization` | Did the agent use available memory? (Recall rate vs available knowledge) |
| `SafetyCompliance` | Did the agent stay within guardrails? (GuardrailResult review) |
| `PolicyCompliance` | Did the agent respect all policies? (PolicyDecision log review) |
| `LatencyBudget` | Did execution fit within time constraints? |
| `CausalConsistency` | Were causal claims valid? (Pearl's rung verification) |
| `PersonaAlignment` | Did the agent behave consistently with its persona? (Fingerprint drift) |

---

## Architecture Comparison: Bedrock AgentCore vs Ladybug-rs

| Bedrock AgentCore | Ladybug-rs Equivalent | Gap |
|---|---|---|
| Runtime (serverless, session isolation) | **NEW**: `AgentRuntime` + `SessionManager` | To build |
| Gateway (Lambda/API → MCP) | **NEW**: `ToolGateway` (OpenAPI/MCP → CAM) | To build |
| Memory (managed, episodic + long-term) | `MemoryBank` exists, needs persistence + session-aware recall + episodic learning | Extend |
| Identity (IAM, Cognito, Okta, Auth0) | **NEW**: `AgentIdentity` with Ed25519 + delegation | To build |
| Observability (OpenTelemetry) | `ObservabilityManager` exists, needs OTEL export | Extend |
| Policy (Cedar rules, deterministic enforcement) | **NEW**: `PolicyEngine` with fingerprint-native rules + Cedar export | To build |
| Evaluations (13 built-in evaluators) | **NEW**: `EvaluationEngine` with 13 fingerprint-native evaluators | To build |
| Tools (Code Interpreter, Browser) | CAM operations + Gateway tools | Partial — add built-in tools |
| Multi-agent (supervisor patterns) | `CrewBridge` + `MetaOrchestrator` | Complete |
| A2A (agent-to-agent) | `A2AProtocol` with XOR channels | Complete |
| Middleware (ASGI) | **NEW**: `MiddlewareChain` | To build |
| Bidirectional streaming | **NEW**: `StreamingSession` | To build |

**Ladybug-rs advantages over Bedrock AgentCore** (no equivalent in Bedrock):
- Semantic kernel with NARS reasoning, causal hierarchy, crystallization
- Fingerprint-based semantic routing (no string matching)
- Persona compatibility with 5-axis volition model
- Dunning-Kruger gap detection in handover policy
- Counterfactual exploration with copy-on-write forks
- 4096 CAM-addressable operations
- Crystal LM 140M:1 memory compression
- mRNA resonance fields for ambient knowledge propagation
- Ice-caked committed facts in blackboard
- Affinity learning from collaboration history
- Policy enforcement via fingerprint similarity (not just string matching)
- Causal consistency evaluation via Pearl's 3-rung hierarchy
- Agent persona drift detection

**Key architectural distinction**: Bedrock AgentCore enforces policies at the **Gateway layer** (intercepting HTTP/MCP calls). Ladybug can enforce policies at the **BindSpace layer** (intercepting fingerprint operations). This is more fundamental — it controls not just tool calls but memory writes, knowledge access, A2A messages, and kernel operations. Every addressable action in the 65,536-address space can be policy-gated.

---

## 13. Interface Gateway — External System Control (NEW in crewAI-rust)

crewAI-rust now includes a complete interface gateway system that enables agents to control arbitrary external systems through YAML-referenced capability imports. This is the manifestation of the Bedrock AgentCore Gateway pattern at the crewAI level.

### What Was Built (crewAI-rust `src/capabilities/`, `src/interfaces/`, `src/policy/`)

| Component | File | Purpose |
|-----------|------|---------|
| `Capability` | `capabilities/capability.rs` | YAML-loadable capability definition (tools, interface, RBAC, policy) |
| `CapabilityRegistry` | `capabilities/registry.rs` | Resolve `minecraft:server_control` → Capability struct from YAML |
| `InterfaceGateway` | `interfaces/gateway.rs` | Route tool calls to protocol adapters, manage lifecycle |
| `InterfaceAdapter` | `interfaces/adapter.rs` | Trait for protocol adapters (connect, execute, disconnect) |
| `RestApiAdapter` | `interfaces/adapters/rest_api.rs` | Generic REST/OpenAPI adapter |
| `RconAdapter` | `interfaces/adapters/rcon.rs` | RCON protocol (Minecraft, Source engine game servers) |
| `GraphApiAdapter` | `interfaces/adapters/graph_api.rs` | Microsoft Graph API (O365 mail, calendar, Teams, OneDrive) |
| `McpBridgeAdapter` | `interfaces/adapters/mcp_bridge.rs` | Bridge MCP servers as InterfaceAdapters |
| `PolicyEngine` | `policy/mod.rs` | Deterministic action-level policy enforcement, Cedar export |
| `RbacManager` | `policy/rbac.rs` | Agent → Role → Capability RBAC |

### YAML Capability Definitions (crewAI-rust `capabilities/`)

| File | Capability ID | Protocol | Description |
|------|--------------|----------|-------------|
| `minecraft/server_control.yaml` | `minecraft:server_control` | RCON | Minecraft server management (commands, players, whitelist, ops) |
| `o365/mail.yaml` | `o365:mail` | MS Graph | Microsoft 365 email (list, read, send) |
| `o365/calendar.yaml` | `o365:calendar` | MS Graph | Microsoft 365 calendar (list events, create meetings) |
| `rest_api/generic.yaml` | `rest_api:generic` | REST | Generic REST API adapter for any HTTP endpoint |
| `mcp/bridge.yaml` | `mcp:bridge` | MCP | Bridge to any MCP server (auto-discovery) |

### How It Works

```yaml
# In agent card:
capabilities:
  imports:
    - "minecraft:server_control"
    - "o365:mail"
  connections:
    "minecraft:server_control":
      host: "${MINECRAFT_HOST}"
      password: "${RCON_PASSWORD}"
    "o365:mail":
      tenant_id: "${AZURE_TENANT_ID}"
      client_id: "${AZURE_CLIENT_ID}"
      client_secret: "${AZURE_CLIENT_SECRET}"
roles:
  assigned: ["server_admin", "mail_user"]
```

### What Ladybug-rs Should Integrate

The crewAI-rust capability system provides the **API surface**. Ladybug-rs should provide the **semantic substrate**:

1. **Fingerprint capabilities**: Each `Capability` should get a 10K-bit fingerprint for semantic discovery. `kernel.resonate(task_fp, capability_registry)` finds the best capability for a task.

2. **Policy enforcement at BindSpace level**: The crewAI `PolicyEngine` enforces at the tool-call level. Ladybug's `KernelGuardrail` should intercept at the fingerprint level — blocking CAM operations before they reach the adapter.

3. **Capability-aware handover**: When the MetaOrchestrator detects an agent needs a capability it doesn't have, it should handover to an agent that does (via capability fingerprint matching, not just persona affinity).

4. **Interface adapter as CAM operations**: Each adapter operation should map to a CAM opcode range. `capability.cam_opcode_range: (0x1000, 0x10FF)` reserves 256 opcodes for Minecraft operations. The kernel can then `operate(0x1002, args)` to call a specific adapter operation.

5. **Cross-system orchestration**: An agent with both `minecraft:server_control` and `o365:mail` capabilities can: detect server issues via RCON → compose a report → email it to admins. The blackboard awareness system enables this: the RCON check writes findings to 0x0E:agent, the mail composition task resonates against it.

### Extending to New Systems

To control a new external system (e.g., AWS infrastructure, Kubernetes clusters, IoT devices):

1. Create a capability YAML in `capabilities/aws/ec2_control.yaml`
2. Implement `InterfaceAdapter` for the AWS SDK protocol
3. Register the factory with `gateway.register_factory(Box::new(AwsSdkAdapterFactory))`
4. Agents declare `capabilities: [aws:ec2_control]` in their YAML card

The architecture is intentionally extensible: **any system with an API becomes an agent-controllable interface, gated by RBAC and deterministic policy enforcement**.

---

## Constraints

1. **No breaking changes** to existing ladybug-rs public APIs
2. All crewAI integration behind `#[cfg(feature = "crewai")]`
3. Runtime/Gateway/Identity behind their own feature flags
4. Existing tests must continue to pass
5. New code must compile with `cargo check --all-features`
6. Blackboard extensions must be backward-compatible (new fields are `Option<T>`)
7. Arrow Flight transport must work without ladybug-rs server running (graceful fallback)
8. Every public type has doc comments with examples
9. No `unwrap()` in library code — all errors via `Result`/`anyhow`
