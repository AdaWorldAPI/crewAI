# Events

The events system provides a publish-subscribe architecture for monitoring and extending crewAI operations. Every significant action -- agent execution, task completion, tool usage, LLM calls, flow transitions, memory operations -- emits an event that listeners can observe.

**Source**: `src/events/` (corresponds to Python `crewai/events/`)

---

## Architecture

```
CrewAIEventsBus (singleton)
  |
  +-- emit(event: Box<dyn BaseEvent>)
  |
  +-- listeners: Vec<Box<dyn BaseEventListener>>
  |     |
  |     +-- on_event(event) -- handler with dependency graph
  |
  +-- handler_graph: dependency resolution for ordered execution
```

## BaseEvent Trait

All events implement the `BaseEvent` trait:

```rust
pub trait BaseEvent: Send + Sync + std::fmt::Debug {
    /// Unique identifier for this event instance (UUID v4)
    fn event_id(&self) -> &str;

    /// UTC timestamp when the event was created
    fn timestamp(&self) -> DateTime<Utc>;

    /// Event type discriminator string (e.g., "crew_kickoff_started")
    fn event_type(&self) -> &str;

    /// UUID of the source entity fingerprint
    fn source_fingerprint(&self) -> Option<&str>;

    /// Source entity kind ("agent", "task", "crew", etc.)
    fn source_type(&self) -> Option<&str>;

    /// Arbitrary fingerprint metadata
    fn fingerprint_metadata(&self) -> Option<&HashMap<String, Value>>;

    /// Associated task ID
    fn task_id(&self) -> Option<&str>;

    /// Associated task name
    fn task_name(&self) -> Option<&str>;
}
```

### BaseEventData

A concrete struct implementing `BaseEvent` used as the base for all event types:

```rust
use crewai::events::BaseEventData;

let event = BaseEventData::new("my_custom_event");
println!("ID: {}", event.event_id);        // UUID v4
println!("Time: {}", event.timestamp);      // UTC now
println!("Type: {}", event.event_type);     // "my_custom_event"
println!("Seq: {}", event.emission_sequence); // auto-incrementing
```

## Event Types

### Agent Events

| Event | Description |
|-------|-------------|
| `AgentExecutionStartedEvent` | Agent begins executing a task |
| `AgentExecutionCompletedEvent` | Agent finishes a task successfully |
| `AgentExecutionErrorEvent` | Agent encounters an error |
| `AgentEvaluationStartedEvent` | Agent evaluation begins |
| `AgentEvaluationCompletedEvent` | Agent evaluation completes |
| `AgentEvaluationFailedEvent` | Agent evaluation fails |
| `LiteAgentExecutionStartedEvent` | Lite agent execution begins |
| `LiteAgentExecutionCompletedEvent` | Lite agent execution completes |
| `LiteAgentExecutionErrorEvent` | Lite agent execution fails |

### Crew Events

| Event | Description |
|-------|-------------|
| `CrewKickoffStartedEvent` | Crew begins execution |
| `CrewKickoffCompletedEvent` | Crew completes successfully |
| `CrewKickoffFailedEvent` | Crew execution fails |
| `CrewTestStartedEvent` | Crew testing begins |
| `CrewTestCompletedEvent` | Crew testing completes |
| `CrewTestFailedEvent` | Crew testing fails |
| `CrewTestResultEvent` | Individual test result |
| `CrewTrainStartedEvent` | Crew training begins |
| `CrewTrainCompletedEvent` | Crew training completes |
| `CrewTrainFailedEvent` | Crew training fails |

### Task Events

| Event | Description |
|-------|-------------|
| `TaskStartedEvent` | Task execution begins |
| `TaskCompletedEvent` | Task completes successfully |
| `TaskFailedEvent` | Task execution fails |
| `TaskEvaluationEvent` | Task quality evaluation result |

### Tool Events

| Event | Description |
|-------|-------------|
| `ToolUsageEvent` | Tool invoked |
| `ToolUsageStartedEvent` | Tool execution begins |
| `ToolUsageFinishedEvent` | Tool execution completes |
| `ToolUsageErrorEvent` | Tool execution fails |
| `ToolSelectionErrorEvent` | Tool selection/matching fails |
| `ToolExecutionErrorEvent` | Tool runtime error |
| `ToolValidateInputErrorEvent` | Tool input validation fails |

### LLM Events

| Event | Description |
|-------|-------------|
| `LLMCallStartedEvent` | LLM API call initiated |
| `LLMCallCompletedEvent` | LLM API call completed |
| `LLMCallFailedEvent` | LLM API call failed |
| `LLMStreamChunkEvent` | Streaming chunk received |

### Flow Events

| Event | Description |
|-------|-------------|
| `FlowCreatedEvent` | Flow instance created |
| `FlowStartedEvent` | Flow execution begins |
| `FlowFinishedEvent` | Flow execution completes |
| `FlowPausedEvent` | Flow paused for feedback |
| `FlowPlotEvent` | Flow visualization generated |
| `MethodExecutionStartedEvent` | Flow method begins |
| `MethodExecutionFinishedEvent` | Flow method completes |
| `MethodExecutionFailedEvent` | Flow method fails |
| `MethodExecutionPausedEvent` | Flow method paused |
| `HumanFeedbackRequestedEvent` | Human feedback requested |
| `HumanFeedbackReceivedEvent` | Human feedback received |

### Knowledge Events

| Event | Description |
|-------|-------------|
| `KnowledgeQueryStartedEvent` | Knowledge query initiated |
| `KnowledgeQueryCompletedEvent` | Knowledge query completed |
| `KnowledgeQueryFailedEvent` | Knowledge query failed |
| `KnowledgeRetrievalStartedEvent` | Knowledge retrieval begins |
| `KnowledgeRetrievalCompletedEvent` | Knowledge retrieval completes |
| `KnowledgeSearchQueryFailedEvent` | Knowledge search failed |

### Memory Events

| Event | Description |
|-------|-------------|
| `MemorySaveStartedEvent` | Memory save initiated |
| `MemorySaveCompletedEvent` | Memory save completed |
| `MemorySaveFailedEvent` | Memory save failed |
| `MemoryQueryStartedEvent` | Memory query initiated |
| `MemoryQueryCompletedEvent` | Memory query completed |
| `MemoryQueryFailedEvent` | Memory query failed |
| `MemoryRetrievalStartedEvent` | Memory retrieval begins |
| `MemoryRetrievalCompletedEvent` | Memory retrieval completes |
| `MemoryRetrievalFailedEvent` | Memory retrieval fails |

### Additional Event Categories

| Category | Module | Description |
|----------|--------|-------------|
| A2A Events | `types::a2a_events` | Agent-to-Agent protocol events |
| MCP Events | `types::mcp_events` | MCP server/tool events |
| LLM Guardrail Events | `types::llm_guardrail_events` | Output guardrail checks |
| Reasoning Events | `types::reasoning_events` | Agent reasoning steps |
| System Events | `types::system_events` | System-level events |
| Logging Events | `types::logging_events` | Log-related events |
| Tool Usage Events | `types::tool_usage_events` | Detailed tool usage tracking |

## Event Bus

The global event bus is a singleton:

```rust
use crewai::events::{CrewAIEventsBus, CREWAI_EVENT_BUS};

// Access the global event bus
// CREWAI_EVENT_BUS is a lazy_static singleton

// Register a listener
// CREWAI_EVENT_BUS.register(listener);

// Emit an event
// CREWAI_EVENT_BUS.emit(event);
```

## BaseEventListener Trait

Create custom event listeners:

```rust
use crewai::events::BaseEventListener;

pub trait BaseEventListener: Send + Sync {
    /// Handle an incoming event
    fn on_event(&self, event: &dyn BaseEvent);

    /// Listener identifier
    fn listener_id(&self) -> &str;

    /// Event types this listener is interested in (empty = all)
    fn event_types(&self) -> &[&str];
}
```

## Listener Struct

The `Listener` struct provides a convenient way to handle events:

```rust
use crewai::events::Listener;

let listener = Listener::new("my_logger", move |event| {
    println!("[{}] {}: {}",
        event.timestamp(),
        event.event_type(),
        event.event_id(),
    );
});
```

## Event Context

Track parent-child event relationships:

```rust
use crewai::events::event_context::EventContext;

// EventContext tracks:
// - parent_event_id: Option<String>
// - chain_id: Option<String>
// - depth: usize
```

## Handler Dependencies

Declare execution ordering between event handlers:

```rust
use crewai::events::{Depends, HandlerId};

// Declare that handler B depends on handler A
let dep = Depends {
    handler: HandlerId::new("handler_b"),
    depends_on: vec![HandlerId::new("handler_a")],
};
```

The handler graph resolves dependencies using topological sort and detects circular dependencies:

```rust
use crewai::events::CircularDependencyError;

// If A depends on B and B depends on A:
// -> CircularDependencyError
```

## Emission Sequence

Events carry an auto-incrementing emission sequence number per thread:

```rust
use crewai::events::base_event::{get_next_emission_sequence, reset_emission_counter};

let seq1 = get_next_emission_sequence(); // 1
let seq2 = get_next_emission_sequence(); // 2

reset_emission_counter(); // resets to 1
```
