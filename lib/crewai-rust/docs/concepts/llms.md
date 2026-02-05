# LLMs

The LLM system provides a unified interface for configuring and calling language models across multiple providers. It includes model identification, provider routing, context window management, and a builder pattern for configuration.

**Source**: `src/llm/mod.rs` and `src/llms/base_llm.rs` (corresponds to Python `crewai/llm.py` and `crewai/llms/base_llm.py`)

---

## Architecture

```
LLM (src/llm/mod.rs)
  |-- configuration struct with builder pattern
  |-- provider inference
  |-- context window lookup
  |-- implements BaseLLMTrait
  |
BaseLLM (src/llms/base_llm.rs)
  |-- abstract trait for provider implementations
  |-- message formatting
  |-- token usage tracking
  |-- stop word handling
  |-- event emission
  |
BaseLLMState (src/llms/base_llm.rs)
  |-- shared state for provider implementations
```

## LLM Struct

### Creating an LLM

```rust
use crewai::llm::LLM;

// Simple creation with model name
let llm = LLM::new("gpt-4o");

// With explicit provider
let llm = LLM::with_provider("my-model", "anthropic");

// Builder pattern
let llm = LLM::new("gpt-4o")
    .temperature(0.7)
    .max_tokens(2000)
    .api_key("sk-...")
    .base_url("https://api.example.com/v1")
    .timeout(30.0)
    .stream(true)
    .stop(vec!["STOP".to_string()])
    .reasoning_effort(ReasoningEffort::High);
```

### LLM Fields

| Field | Type | Default | Description |
|-------|------|---------|-------------|
| `model` | `String` | Required | Model identifier (e.g., "gpt-4o") |
| `temperature` | `Option<f64>` | `None` | Sampling temperature |
| `top_p` | `Option<f64>` | `None` | Nucleus sampling parameter |
| `n` | `Option<i32>` | `None` | Number of completions |
| `stop` | `Vec<String>` | `[]` | Stop sequences |
| `max_tokens` | `Option<i64>` | `None` | Max tokens to generate |
| `max_completion_tokens` | `Option<i64>` | `None` | Max completion tokens (OpenAI) |
| `presence_penalty` | `Option<f64>` | `None` | Presence penalty (-2 to 2) |
| `frequency_penalty` | `Option<f64>` | `None` | Frequency penalty (-2 to 2) |
| `logit_bias` | `Option<HashMap<i64, f64>>` | `None` | Token logit biases |
| `response_format` | `Option<Value>` | `None` | Structured output format |
| `seed` | `Option<i64>` | `None` | Random seed for reproducibility |
| `timeout` | `Option<f64>` | `None` | API call timeout (seconds) |
| `base_url` | `Option<String>` | `None` | Custom API base URL |
| `api_key` | `Option<String>` | `None` | API key (not serialized) |
| `api_version` | `Option<String>` | `None` | API version (Azure) |
| `stream` | `bool` | `false` | Enable streaming |
| `reasoning_effort` | `Option<ReasoningEffort>` | `None` | Reasoning level |
| `context_window_size` | `i64` | `0` | Override context window |
| `provider` | `Option<String>` | `None` | Explicit provider |
| `additional_params` | `HashMap<String, Value>` | `{}` | Extra provider params |

### Reasoning Effort

```rust
use crewai::llm::ReasoningEffort;

let llm = LLM::new("o3-mini")
    .reasoning_effort(ReasoningEffort::High);

// Variants: None, Low, Medium, High
```

## Provider Inference

The LLM infers its provider from the model name:

```rust
let llm = LLM::new("gpt-4o");
assert_eq!(llm.infer_provider(), "openai");

let llm = LLM::new("claude-3-5-sonnet-20241022");
assert_eq!(llm.infer_provider(), "anthropic");

let llm = LLM::new("gemini-2.0-flash");
assert_eq!(llm.infer_provider(), "gemini");

let llm = LLM::new("openai/gpt-4o"); // prefix-based
assert_eq!(llm.infer_provider(), "openai");

let llm = LLM::new("bedrock/anthropic.claude-3");
assert_eq!(llm.infer_provider(), "bedrock");
```

### Provider Resolution Order

1. Explicit `provider` field
2. Model string prefix (e.g., `openai/gpt-4`)
3. Model name pattern matching (e.g., `gpt-` -> OpenAI)
4. Default: `"openai"`

### Supported Native Providers

| Provider | Prefixes/Patterns |
|----------|-------------------|
| `openai` | `gpt-`, `o1`, `o3`, `o4`, `whisper-` |
| `anthropic` | `claude-`, `anthropic.`, `anthropic/` |
| `gemini` / `google` | `gemini-`, `gemma-`, `learnlm-` |
| `azure` / `azure_openai` | `gpt-`, `azure-`, via `azure/` prefix |
| `bedrock` / `aws` | Model names containing `.` (e.g., `anthropic.claude-3`) |
| `mistral` | `mistral-` |

## Context Window Sizes

The LLM automatically looks up context window sizes:

```rust
let llm = LLM::new("gpt-4o");
assert_eq!(llm.get_context_window_size(), 128_000);

let llm = LLM::new("gemini-1.5-pro");
assert_eq!(llm.get_context_window_size(), 2_097_152);

// Usable context (85% of total)
let usable = llm.get_usable_context_window_size();
// 128_000 * 0.85 = 108_800

// Override
let mut llm = LLM::new("gpt-4");
llm.context_window_size = 16384;
assert_eq!(llm.get_context_window_size(), 16384);
```

### Constants

| Constant | Value | Description |
|----------|-------|-------------|
| `MIN_CONTEXT` | 1,024 | Minimum context window |
| `MAX_CONTEXT` | 2,097,152 | Maximum context window |
| `DEFAULT_CONTEXT_WINDOW_SIZE` | 8,192 | Fallback for unknown models |
| `CONTEXT_WINDOW_USAGE_RATIO` | 0.85 | 85% of window is usable |

## Calling an LLM

```rust
use std::collections::HashMap;

let llm = LLM::new("gpt-4o").api_key("sk-...");

let mut messages = vec![];
let mut msg = HashMap::new();
msg.insert("role".to_string(), "user".to_string());
msg.insert("content".to_string(), "What is Rust?".to_string());
messages.push(msg);

// Synchronous call
let response = llm.call(&messages, None)?;

// Async call
let response = llm.acall(&messages, None).await?;

// With tools
let tools = vec![serde_json::json!({
    "type": "function",
    "function": {
        "name": "search",
        "description": "Search the web",
        "parameters": {"type": "object", "properties": {"query": {"type": "string"}}}
    }
})];
let response = llm.call(&messages, Some(&tools))?;
```

**Note**: `LLM::call()` is currently a stub pending provider integration. See the [Technical Debt Report](../../TECHNICAL_DEBT.md).

## Completion Parameters

Get all configured parameters as a map:

```rust
let llm = LLM::new("gpt-4o")
    .temperature(0.5)
    .max_tokens(500)
    .stream(true);

let params = llm.prepare_completion_params();
// {"model": "gpt-4o", "temperature": 0.5, "max_tokens": 500, "stream": true}
```

## Capability Checks

```rust
let llm = LLM::new("gpt-4o");
assert!(llm.supports_function_calling());

let llm = LLM::new("claude-3-5-sonnet-20241022");
assert!(llm.supports_function_calling());
```

## BaseLLM Trait

The provider-level trait for full LLM implementations:

```rust
use crewai::llms::base_llm::BaseLLM;

// Required methods:
// fn model(&self) -> &str
// fn temperature(&self) -> Option<f64>
// fn stop(&self) -> &[String]
// fn set_stop(&mut self, stop: Vec<String>)
// fn call(...) -> Result<Value, Error>
// fn get_token_usage_summary(&self) -> UsageMetrics
// fn track_token_usage(&mut self, usage_data: &HashMap<String, Value>)

// Optional methods with defaults:
// fn provider(&self) -> &str                      // "openai"
// fn is_litellm(&self) -> bool                    // false
// fn supports_function_calling(&self) -> bool     // false
// fn supports_stop_words(&self) -> bool           // true
// fn get_context_window_size(&self) -> usize      // 4096
// fn supports_multimodal(&self) -> bool           // false
// fn format_text_content(&self, text: &str) -> Value
// fn convert_tools_for_inference(tools: Vec<Value>) -> Vec<Value>
// async fn acall(...) -> Result<Value, Error>
```

## BaseLLMState

Shared state embedded in provider implementations:

```rust
use crewai::llms::base_llm::BaseLLMState;

let mut state = BaseLLMState::new("gpt-4o");

// Apply stop words
state.stop = vec!["Observation:".to_string()];
let truncated = state.apply_stop_words("Text before\nObservation: result");
assert_eq!(truncated, "Text before");

// Track token usage
let mut usage = HashMap::new();
usage.insert("prompt_tokens".to_string(), serde_json::json!(100));
usage.insert("completion_tokens".to_string(), serde_json::json!(50));
state.track_token_usage_internal(&usage);

let summary = state.get_token_usage_summary();
assert_eq!(summary.total_tokens, 150);

// Validate structured output
let parsed = BaseLLMState::validate_structured_output(
    r#"Here is the JSON: {"key": "value"}"#
)?;
```

## Token Usage

```rust
use crewai::types::usage_metrics::UsageMetrics;

// UsageMetrics fields:
// - total_tokens: i64
// - prompt_tokens: i64
// - cached_prompt_tokens: i64
// - completion_tokens: i64
// - successful_requests: i64
```
