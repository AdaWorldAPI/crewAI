# Blackboard Integration Guide

Post-merge status after PR #9 (P0 execution pipeline).

## Current State

The execution chain is LIVE:
```
Crew.kickoff() → execute_tasks() → Task.execute_sync()
  → AgentExecutorFn callback → CrewAgentExecutor.invoke_loop()
    → invoke_loop_react() OR invoke_loop_native_tools()
      → OpenAICompletion.acall() [reqwest HTTP + retry]
      → execute_tool() → tool callback → Observation
      → loop until AgentFinish
  → TaskOutput with messages
```

The blackboard module is committed to `src/blackboard/` (8 files) but not yet wired.

## TODO: Wire the Blackboard

### 1. Register module in `src/lib.rs`

Add after `pub mod agents;`:
```rust
pub mod blackboard;
```

### 2. Add dependencies to `Cargo.toml`

```toml
dashmap = "6"
parking_lot = "0.12"
# Optional for lance flavor:
# lancedb = { version = "0.15", optional = true }
```

### 3. Add blackboard field to `Crew` (`src/crew.rs`)

```rust
use crate::blackboard::{self, BlackboardStore, BlackboardConfig};

pub struct Crew {
    // ... existing fields ...

    /// Shared blackboard for multi-agent state (selected by CREWAI_BLACKBOARD_FLAVOR env var).
    #[serde(skip)]
    pub blackboard: Option<Box<dyn BlackboardStore>>,
}
```

In `Crew::new()` and `Crew::with_agents()`:
```rust
blackboard: if std::env::var("CREWAI_BLACKBOARD_FLAVOR").is_ok() {
    Some(blackboard::create_blackboard_from_env())
} else {
    None
},
```

### 4. Inject blackboard snapshot into LLM prompt (`src/crew.rs`)

In `wire_task_executor_static()`, wrap the agent execution to include blackboard context:

```rust
fn wire_task_executor_static(
    task: &mut Task,
    role: &str,
    agent_objects: &HashMap<String, Arc<std::sync::RwLock<Agent>>>,
    blackboard: Option<Arc<dyn BlackboardStore>>,  // NEW PARAM
) {
    if let Some(agent_lock) = agent_objects.get(role) {
        let agent_clone = agent_lock.clone();
        let bb = blackboard.clone();

        task.set_agent_executor(move |prompt, context, tools| {
            // Inject blackboard context
            let enriched_context = if let Some(ref bb) = bb {
                let bb_ctx = bb.build_context_for_task(prompt, context.unwrap_or(""));
                match context {
                    Some(ctx) => Some(format!("{}\n\n{}", ctx, bb_ctx)),
                    None if !bb_ctx.is_empty() => Some(bb_ctx),
                    None => None,
                }
            } else {
                context.map(|s| s.to_string())
            };

            let mut agent = agent_clone.write().map_err(|e| format!("{}", e))?;
            let result = agent.execute_task(
                prompt,
                enriched_context.as_deref(),
                if tools.is_empty() { None } else { Some(tools) },
            )?;

            // Post result to blackboard
            if let Some(ref bb) = bb {
                use crate::blackboard::entry::{BlackboardEntry, EntryType, EntryTier};
                let _ = bb.post(
                    BlackboardEntry::new(
                        agent.role.clone(),
                        EntryType::Decision,
                        &result,
                        None,
                    ).with_tier(EntryTier::Session)
                );
            }

            // ... rest of message conversion
        });
    }
}
```

### 5. Advance epoch between tasks (`src/crew.rs`)

In `execute_tasks()`, after each task completes:
```rust
if let Some(ref bb) = self.blackboard {
    bb.advance_epoch();
}
```

### 6. Policy check on blackboard commit

In `src/blackboard/hashed.rs` `post()`, before inserting:
```rust
// if policy engine is available:
// let decision = policy_engine.evaluate(&PolicyRequest {
//     action: PolicyAction::BlackboardCommit,
//     resource: PolicyResource::Memory,
//     ...
// });
// entry.policy_audit = Some(decision);
```

`PolicyAction::BlackboardCommit` already exists in the enum.

### 7. LLM cache alignment

Use `blackboard::cache::build_cached_message_array()` when constructing
the LLM prompt. This places the blackboard snapshot as a stable prefix
with Anthropic `cache_control` markers, so multiple agents reading the
same epoch share one cached prefix.

## Environment Variables

```bash
CREWAI_BLACKBOARD_FLAVOR=original|hashed|lance    # Default: original (no-op wrapper)
CREWAI_BLACKBOARD_PRUNE=true|false                 # Prune vs tombstone expired (default: false)
CREWAI_BLACKBOARD_SEPARATE_DB=true|false           # Own DB vs shared LTM (default: true)
CREWAI_BLACKBOARD_LANCE_S3=s3://bucket/prefix      # S3 for lance flavor
CREWAI_BLACKBOARD_LANCE_PATH=./blackboard_lance    # Local lance dir
CREWAI_BLACKBOARD_MAX_ENTRIES=10000                # Compaction threshold
CREWAI_BLACKBOARD_STM_TTL=3600                     # STM entry TTL in seconds
```

## Blackboard Files

```
src/blackboard/
  mod.rs          — Factory, config, env var routing (BlackboardFlavor enum)
  entry.rs        — BlackboardEntry, EntryType, content hash (SHA-256)
  store.rs        — BlackboardStore trait (the agnostic interface)
  snapshot.rs     — BlackboardSnapshot, CacheThumbprint
  cache.rs        — LLM cache alignment, Anthropic cache_control markers
  original.rs     — Flavor 1: drop-in crewAI wrapper (Vec + substring search)
  hashed.rs       — Flavor 2: DashMap + epochs + Merkle chain
  lance.rs        — Flavor 3: LanceDB stub (falls back to hashed until dep added)
```

## ladybug-rs Boundary

crewai-rust owns: `BlackboardStore` trait, all 3 flavors, `CacheThumbprint`.

ladybug-rs adds on top:
- BindSpace addressing (O(1) 65K slots)
- NARS-revise conflict resolution between contradicting entries
- GrammarTriangle fingerprinting (NSM + Causality + Qualia → 10K-bit)
- Crystal-compress for LTM-tier compaction
- Ice-caking ceremony (via `PolicyAction::BlackboardCommit`)
