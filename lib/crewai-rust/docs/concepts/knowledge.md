# Knowledge

The knowledge system enables agents to access domain-specific information through a RAG (Retrieval Augmented Generation) pipeline. Knowledge sources are ingested, chunked, embedded, and stored in a searchable backend.

**Source**: `src/knowledge/` (corresponds to Python `crewai/knowledge/`)

---

## Architecture

```
Knowledge
  |
  +-- sources: Vec<Box<dyn BaseKnowledgeSource>>
  |     +-- StringKnowledgeSource
  |     +-- BaseFileKnowledgeSource (trait)
  |     +-- (PDF, CSV, JSON, etc. -- future)
  |
  +-- storage: KnowledgeStorage
        +-- implements BaseKnowledgeStorage
        +-- search(), save(), reset()
```

## Knowledge Struct

```rust
use crewai::knowledge::{Knowledge, StringKnowledgeSource, KnowledgeStorage};

// Create with a string source
let source = StringKnowledgeSource::new(
    "Rust is a systems programming language focused on safety and performance."
        .to_string(),
);

let knowledge = Knowledge::new(
    vec![Box::new(source)],          // sources
    None,                             // embedder config
    Some("my_collection".to_string()), // collection name
    None,                             // pre-configured storage
);
```

### Fields

| Field | Type | Description |
|-------|------|-------------|
| `sources` | `Vec<Box<dyn BaseKnowledgeSource>>` | Knowledge sources to manage |
| `storage` | `Arc<KnowledgeStorage>` | Storage backend |
| `embedder_config` | `Option<Value>` | Embedder provider configuration |
| `collection_name` | `Option<String>` | Storage collection name (default: "knowledge") |

## Querying Knowledge

```rust
// Synchronous query
let results = knowledge.query(
    "What is Rust?",  // query string
    Some(5),          // max results (default: 3)
    Some(0.5),        // min score threshold (default: 0.35)
)?;

for result in &results {
    println!("Match: {}", result);
}

// Async query
let results = knowledge.aquery("memory safety", None, None).await?;
```

## Ingesting Sources

Add all configured sources to the storage backend:

```rust
// Synchronous
knowledge.add_sources()?;

// Async
knowledge.aadd_sources().await?;
```

## Resetting Knowledge

Clear all stored knowledge:

```rust
knowledge.reset()?;
```

## BaseKnowledgeSource Trait

All knowledge sources implement this trait:

```rust
pub trait BaseKnowledgeSource: Send + Sync {
    /// Add the source's content to the knowledge storage.
    fn add(&self, storage: &KnowledgeStorage) -> Result<(), anyhow::Error>;

    /// Async version of add.
    async fn aadd(&self, storage: &KnowledgeStorage) -> Result<(), anyhow::Error>;
}
```

### StringKnowledgeSource

The simplest source -- raw text:

```rust
use crewai::knowledge::StringKnowledgeSource;

let source = StringKnowledgeSource::new(
    "The quick brown fox jumps over the lazy dog.".to_string(),
);
```

### BaseFileKnowledgeSource Trait

For file-based sources (to be implemented):

```rust
pub trait BaseFileKnowledgeSource: BaseKnowledgeSource {
    /// File paths to read from.
    fn file_paths(&self) -> &[String];
}
```

Python supports 10 source types: PDF, CSV, Excel, JSON, String, Text, CrewDocling, GitHub, YouTube, and custom sources. The Rust port currently provides the trait definitions and `StringKnowledgeSource`.

## BaseKnowledgeStorage Trait

Storage backends implement this trait:

```rust
pub trait BaseKnowledgeStorage: Send + Sync {
    fn search(&self, query: &str, limit: usize, score_threshold: f64)
        -> Result<Vec<Value>, anyhow::Error>;

    async fn asearch(&self, query: &str, limit: usize, score_threshold: f64)
        -> Result<Vec<Value>, anyhow::Error>;

    fn save(&self, documents: &[Value]) -> Result<(), anyhow::Error>;

    fn reset(&self) -> Result<(), anyhow::Error>;
}
```

### KnowledgeStorage

The default storage implementation:

```rust
use crewai::knowledge::KnowledgeStorage;

let storage = KnowledgeStorage::new(
    Some(serde_json::json!({"provider": "openai"})),  // embedder config
    Some("my_collection".to_string()),                  // collection name
);
```

## Knowledge Configuration

```rust
use crewai::knowledge::KnowledgeConfig;

// KnowledgeConfig fields:
// - results_limit: usize       -- max results per query (default: 3)
// - score_threshold: f64       -- min similarity score (default: 0.35)
// - chunk_size: usize          -- text chunk size for ingestion
// - chunk_overlap: usize       -- overlap between chunks
```

## Using Knowledge with Agents

```rust
let mut agent = Agent::new(
    "Knowledge Expert".to_string(),
    "Answer questions using the knowledge base".to_string(),
    "Expert with access to internal documentation".to_string(),
);

// Configure knowledge sources
agent.knowledge_sources = Some(vec![
    serde_json::json!({
        "type": "string",
        "content": "Internal policy document content..."
    }).as_object().unwrap().iter().map(|(k, v)| (k.clone(), v.clone())).collect(),
]);

// Configure embedder
agent.embedder = Some(serde_json::json!({
    "provider": "openai",
    "config": {"model": "text-embedding-3-small"}
}).as_object().unwrap().iter().map(|(k, v)| (k.clone(), v.clone())).collect());
```

## Using Knowledge with Crews

```rust
let mut crew = Crew::new(tasks, agents);

crew.knowledge_sources = Some(vec![
    // Knowledge source configurations
]);

crew.embedder = Some(/* embedder config */);
```

## Implementation Status

The `Knowledge` struct, traits (`BaseKnowledgeSource`, `BaseKnowledgeStorage`), and `StringKnowledgeSource` are complete. File-based sources (PDF, CSV, Excel) and the RAG vector storage backend require additional library integration. See the [Technical Debt Report](../../TECHNICAL_DEBT.md) for details.
