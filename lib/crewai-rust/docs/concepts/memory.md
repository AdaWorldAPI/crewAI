# Memory

The memory system gives agents the ability to store and retrieve information across task executions. crewAI supports multiple memory types, each backed by a storage interface that can use RAG (vector search), SQLite, or external services.

**Source**: `src/memory/` (corresponds to Python `crewai/memory/`)

---

## Memory Types

| Type | Module | Description |
|------|--------|-------------|
| **Short-Term Memory** | `memory::short_term` | Recent context within a single execution |
| **Long-Term Memory** | `memory::long_term` | Persistent knowledge across executions (SQLite) |
| **Entity Memory** | `memory::entity` | Facts about specific entities (people, orgs, concepts) |
| **Contextual Memory** | `memory::contextual` | Combines short-term, long-term, and entity memory |
| **External Memory** | `memory::external` | Integration with external memory services (e.g., Mem0) |

## Base Memory Struct

All memory types build on the `Memory` struct:

```rust
use crewai::memory::Memory;

// Create with a storage backend
let memory = Memory::new(Box::new(storage_backend));

// Create with embedder configuration
let memory = Memory::with_embedder(
    Box::new(storage_backend),
    Some(serde_json::json!({"provider": "openai", "model": "text-embedding-3-small"})),
);
```

### Operations

```rust
use std::collections::HashMap;
use serde_json::Value;

// Save to memory
let mut metadata = HashMap::new();
metadata.insert("agent".to_string(), Value::from("researcher"));
memory.save("Important finding about AI trends", Some(metadata))?;

// Async save
memory.asave("Another finding", None).await?;

// Search memory
let results = memory.search(
    "AI trends",  // query
    5,            // limit
    0.7,          // score threshold
)?;

// Async search
let results = memory.asearch("AI trends", 5, 0.7).await?;
```

### Context References

Memory instances track their parent crew, agent, and task:

```rust
memory.set_crew(Box::new(crew_ref));
memory.set_agent(Some(Box::new(agent_ref)));
memory.set_task(Some(Box::new(task_ref)));
```

## Short-Term Memory

Stores recent context from the current execution:

```rust
use crewai::memory::ShortTermMemory;

// ShortTermMemoryItem fields:
// - data: String       -- the content
// - metadata: HashMap  -- associated metadata
// - agent: String      -- agent that created this
```

## Long-Term Memory

Persists knowledge across crew executions using SQLite:

```rust
use crewai::memory::LongTermMemory;

// LongTermMemoryItem fields:
// - task_description: String  -- what the task was
// - metadata: Value           -- structured metadata
// - agent: String             -- agent that created this
// - quality: Option<f64>      -- quality score
// - datetime: String          -- ISO timestamp
```

## Entity Memory

Tracks facts about specific entities:

```rust
use crewai::memory::EntityMemory;

// EntityMemoryItem fields:
// - name: String        -- entity name
// - entity_type: String -- category (person, org, concept, etc.)
// - description: String -- entity description
// - metadata: HashMap   -- additional metadata
```

## Contextual Memory

Combines all memory types for comprehensive context:

```rust
use crewai::memory::ContextualMemory;

// ContextualMemory aggregates results from:
// - short-term memory (recent context)
// - long-term memory (historical knowledge)
// - entity memory (entity facts)
```

## External Memory

Integration with external memory services:

```rust
use crewai::memory::ExternalMemory;

// ExternalMemoryItem fields:
// - data: String       -- content
// - metadata: HashMap  -- metadata
```

## Storage Backends

The `Storage` trait defines the interface for memory backends:

```rust
// Located in memory::storage::interface
pub trait Storage: Send + Sync {
    fn save(&self, value: &str, metadata: &HashMap<String, Value>)
        -> Result<(), anyhow::Error>;

    async fn asave(&self, value: &str, metadata: &HashMap<String, Value>)
        -> Result<(), anyhow::Error>;

    fn search(&self, query: &str, limit: usize, score_threshold: f64)
        -> Result<Vec<Value>, anyhow::Error>;

    async fn asearch(&self, query: &str, limit: usize, score_threshold: f64)
        -> Result<Vec<Value>, anyhow::Error>;

    fn reset(&self) -> Result<(), anyhow::Error>;
}
```

### Available Backends

| Backend | Module | Status | Description |
|---------|--------|--------|-------------|
| RAG Storage | `memory::storage::rag_storage` | Stub | Vector similarity search |
| SQLite LTM | `memory::storage::ltm_sqlite_storage` | Stub | SQLite for long-term memory |
| Mem0 Storage | `memory::storage::mem0_storage` | Stub | Mem0 external service |
| Kickoff Outputs | `memory::storage::kickoff_task_outputs_storage` | Stub | Task output persistence |

## Configuring Memory in a Crew

Enable memory in the crew configuration:

```rust
let mut crew = Crew::new(tasks, agents);
crew.memory = true;

// Configure specific memory backends
crew.embedder = Some(serde_json::json!({
    "provider": "openai",
    "config": {
        "model": "text-embedding-3-small"
    }
}).as_object().unwrap().iter().map(|(k, v)| (k.clone(), v.clone())).collect());
```

## Implementation Status

Memory type definitions and the `Storage` trait are complete. Storage backend implementations (RAG vector search, SQLite persistence, Mem0 integration) require additional provider integration. See the [Technical Debt Report](../../TECHNICAL_DEBT.md) for details.
