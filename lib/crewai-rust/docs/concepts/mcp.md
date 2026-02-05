# MCP (Model Context Protocol)

MCP integration allows crewAI agents to discover and invoke tools from external MCP-compatible servers. The module provides server configuration types for three transport mechanisms, a client abstraction, and tool filtering.

**Source**: `src/mcp/` (corresponds to Python `crewai/mcp/`)

---

## Architecture

```
Agent
  |
  +-- mcps: Vec<MCPServerConfig>
        |
        +-- MCPServerStdio  --> StdioTransport
        +-- MCPServerHTTP   --> StreamableHTTPTransport
        +-- MCPServerSSE    --> SSETransport
              |
              +-- MCPClient
                    |
                    +-- list_tools()
                    +-- call_tool(name, args)
                          |
                          +-- MCPNativeTool / MCPToolWrapper
```

## Server Configuration

### MCPServerStdio

Connect to a local MCP server that runs as a child process:

```rust
use crewai::mcp::MCPServerStdio;
use std::collections::HashMap;

let config = MCPServerStdio::new("python")
    .with_args(vec!["-m".to_string(), "mcp_server".to_string()])
    .with_cache_tools_list(true);

// With environment variables
let mut env = HashMap::new();
env.insert("API_KEY".to_string(), "secret123".to_string());

let config = MCPServerStdio::new("npx")
    .with_args(vec!["-y".to_string(), "@mcp/weather-server".to_string()])
    .with_env(env)
    .with_cache_tools_list(true);
```

**Fields**:

| Field | Type | Default | Description |
|-------|------|---------|-------------|
| `command` | `String` | Required | Executable to run |
| `args` | `Vec<String>` | `[]` | Command arguments |
| `env` | `Option<HashMap<String, String>>` | `None` | Environment variables |
| `tool_filter` | `Option<ArcToolFilter>` | `None` | Tool filter function |
| `cache_tools_list` | `bool` | `false` | Cache discovered tools |

### MCPServerHTTP

Connect to a remote MCP server over HTTP/HTTPS (streamable HTTP transport):

```rust
use crewai::mcp::MCPServerHTTP;
use std::collections::HashMap;

let mut headers = HashMap::new();
headers.insert("Authorization".to_string(), "Bearer token123".to_string());

let config = MCPServerHTTP::new("https://api.example.com/mcp")
    .with_headers(headers)
    .with_streamable(true)    // default: true
    .with_cache_tools_list(true);
```

**Fields**:

| Field | Type | Default | Description |
|-------|------|---------|-------------|
| `url` | `String` | Required | Server URL |
| `headers` | `Option<HashMap<String, String>>` | `None` | HTTP headers |
| `streamable` | `bool` | `true` | Use streamable HTTP transport |
| `tool_filter` | `Option<ArcToolFilter>` | `None` | Tool filter function |
| `cache_tools_list` | `bool` | `false` | Cache discovered tools |

### MCPServerSSE

Connect to a remote MCP server using Server-Sent Events:

```rust
use crewai::mcp::MCPServerSSE;

let config = MCPServerSSE::new("https://api.example.com/mcp/sse")
    .with_cache_tools_list(true);

// With authentication headers
let mut headers = HashMap::new();
headers.insert("Authorization".to_string(), "Bearer token".to_string());

let config = MCPServerSSE::new("https://api.example.com/sse")
    .with_headers(headers);
```

**Fields**:

| Field | Type | Default | Description |
|-------|------|---------|-------------|
| `url` | `String` | Required | Server URL |
| `headers` | `Option<HashMap<String, String>>` | `None` | HTTP headers |
| `tool_filter` | `Option<ArcToolFilter>` | `None` | Tool filter function |
| `cache_tools_list` | `bool` | `false` | Cache discovered tools |

## MCPServerConfig Enum

A union type for any server configuration:

```rust
use crewai::mcp::{MCPServerConfig, MCPServerStdio, MCPServerHTTP, MCPServerSSE};

// Create from specific types using From
let config: MCPServerConfig = MCPServerStdio::new("python").into();
let config: MCPServerConfig = MCPServerHTTP::new("https://example.com").into();
let config: MCPServerConfig = MCPServerSSE::new("https://example.com/sse").into();

// Or construct directly
let config = MCPServerConfig::Stdio(MCPServerStdio::new("node"));

// Common methods on the enum
config.tool_filter();       // &Option<ArcToolFilter>
config.cache_tools_list();  // bool
config.server_identifier(); // String for logging
```

## Tool Filtering

Filter which tools from an MCP server are exposed to agents:

```rust
use crewai::mcp::config::ArcToolFilter;
use std::sync::Arc;

// Only include tools that start with "allowed_"
let filter: ArcToolFilter = Arc::new(|tool: &serde_json::Value| {
    tool.get("name")
        .and_then(|n| n.as_str())
        .map(|name| name.starts_with("allowed_"))
        .unwrap_or(false)
});

let config = MCPServerStdio::new("python")
    .with_tool_filter(filter);
```

### StaticToolFilter

For simple include/exclude lists:

```rust
use crewai::mcp::StaticToolFilter;

// Include only specific tools
let filter = StaticToolFilter::include(vec![
    "get_weather".to_string(),
    "search_files".to_string(),
]);

// Exclude specific tools
let filter = StaticToolFilter::exclude(vec![
    "dangerous_tool".to_string(),
]);
```

### ToolFilterContext

Provides additional context to the filter function:

```rust
use crewai::mcp::ToolFilterContext;

// ToolFilterContext fields:
// - agent_role: Option<String>
// - task_description: Option<String>
// - server_name: Option<String>
```

## MCPClient

The client handles communication with MCP servers:

```rust
use crewai::mcp::MCPClient;

// MCPClient wraps a transport and provides:
// - list_tools() -> Vec<Value>    -- discover available tools
// - call_tool(name, args) -> Value -- invoke a tool
// - close()                        -- clean up connection
```

## Transport Types

```rust
use crewai::mcp::TransportType;

pub enum TransportType {
    Stdio,          // Local process via stdin/stdout
    Http,           // HTTP/HTTPS with streamable transport
    Sse,            // Server-Sent Events
}
```

### BaseTransport Trait

All transports implement this trait:

```rust
pub trait BaseTransport: Send + Sync {
    fn connect(&mut self) -> Result<(), Error>;
    fn send(&self, message: &Value) -> Result<Value, Error>;
    async fn asend(&self, message: &Value) -> Result<Value, Error>;
    fn close(&mut self) -> Result<(), Error>;
}
```

## Server Identification

Each server config provides a unique identifier for caching and logging:

```rust
let stdio = MCPServerStdio::new("python")
    .with_args(vec!["server.py".to_string()]);
assert_eq!(stdio.server_identifier(), "stdio:python:server.py");

let http = MCPServerHTTP::new("https://example.com/mcp");
assert_eq!(http.server_identifier(), "http:https://example.com/mcp");

let sse = MCPServerSSE::new("https://example.com/sse");
assert_eq!(sse.server_identifier(), "sse:https://example.com/sse");
```

## Debug Security

HTTP and SSE configs mask header values in debug output:

```rust
let mut headers = HashMap::new();
headers.insert("Authorization".to_string(), "Bearer secret_token".to_string());

let config = MCPServerHTTP::new("https://example.com")
    .with_headers(headers);

println!("{:?}", config);
// Output shows "Authorization=<masked>" -- never exposes secrets
```

## Serialization

All server configs support serde serialization (tool_filter is skipped):

```rust
let config = MCPServerHTTP::new("https://example.com/mcp")
    .with_cache_tools_list(true);

let json = serde_json::to_string(&config)?;
let deserialized: MCPServerHTTP = serde_json::from_str(&json)?;
```

## Using MCP with Agents

```rust
use crewai::mcp::{MCPServerStdio, MCPServerHTTP};

let mut agent = Agent::new(
    "Research Assistant".to_string(),
    "Research topics using external tools".to_string(),
    "Expert researcher with access to MCP tool servers".to_string(),
);

// Agents reference MCP servers by config
// (actual integration is handled during agent execution)
agent.mcps = Some(vec![
    "https://tools.example.com/mcp".to_string(),
]);
```

## Implementation Status

Server configuration types, the MCPServerConfig enum, tool filtering, and serialization are fully implemented. Transport layer I/O (actual stdio/HTTP/SSE communication) requires additional integration. See the [Technical Debt Report](../../TECHNICAL_DEBT.md).
