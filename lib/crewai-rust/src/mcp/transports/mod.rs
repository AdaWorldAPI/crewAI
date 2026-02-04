//! Transport layer implementations for MCP connections.
//!
//! Port of crewai/mcp/transports/

pub mod stdio;
pub mod http;
pub mod sse;

use async_trait::async_trait;

pub use stdio::StdioTransport;
pub use http::HTTPTransport;
pub use sse::SSETransport;

/// MCP transport types.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TransportType {
    /// Standard I/O transport (local process).
    Stdio,
    /// HTTP transport.
    Http,
    /// Streamable HTTP transport.
    StreamableHttp,
    /// Server-Sent Events transport.
    Sse,
}

impl std::fmt::Display for TransportType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TransportType::Stdio => write!(f, "stdio"),
            TransportType::Http => write!(f, "http"),
            TransportType::StreamableHttp => write!(f, "streamable-http"),
            TransportType::Sse => write!(f, "sse"),
        }
    }
}

/// Base trait for MCP transport implementations.
///
/// Defines the interface that all transport implementations must follow.
/// Transports handle the low-level communication with MCP servers.
#[async_trait]
pub trait BaseTransport: Send + Sync {
    /// Return the transport type.
    fn transport_type(&self) -> TransportType;

    /// Check if transport is connected.
    fn connected(&self) -> bool;

    /// Establish connection to MCP server.
    async fn connect(&mut self) -> Result<(), anyhow::Error>;

    /// Close connection to MCP server.
    async fn disconnect(&mut self) -> Result<(), anyhow::Error>;

    /// Return a string identifier for this server (used for caching/logging).
    fn server_identifier(&self) -> String;
}
