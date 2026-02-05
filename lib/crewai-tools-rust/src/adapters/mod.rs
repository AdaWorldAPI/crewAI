//! Adapter modules for integrating crewAI tools with external platforms.
//!
//! These adapters bridge crewAI's tool system with protocols and services such as
//! MCP (Model Context Protocol), enterprise action frameworks, Zapier, and
//! vector-database-backed RAG pipelines.

use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use serde_json::Value;

// ── McpServerAdapter ─────────────────────────────────────────────────────────

/// Adapter that exposes crewAI tools over the Model Context Protocol (MCP).
///
/// Corresponds to the Python `MCPServerAdapter` class in `crewai_tools`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpServerAdapter {
    /// Human-readable name for this MCP server instance.
    pub server_name: String,
    /// Optional description surfaced to MCP clients.
    pub description: Option<String>,
}

impl McpServerAdapter {
    /// Create a new MCP server adapter with the given name.
    pub fn new(server_name: impl Into<String>) -> Self {
        Self {
            server_name: server_name.into(),
            description: None,
        }
    }

    /// Set an optional description.
    pub fn with_description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }

    /// Returns the adapter name.
    pub fn name(&self) -> &str {
        &self.server_name
    }

    /// Returns the adapter description.
    pub fn description(&self) -> Option<&str> {
        self.description.as_deref()
    }

    /// Run the MCP server adapter.
    pub fn run(&self, _args: HashMap<String, Value>) -> Result<Value, anyhow::Error> {
        anyhow::bail!("McpServerAdapter: not yet implemented - requires MCP protocol integration")
    }
}

// ── EnterpriseActionTool ─────────────────────────────────────────────────────

/// Enterprise-grade action tool for executing business workflow actions.
///
/// Corresponds to the Python `EnterpriseActionTool` class in `crewai_tools`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnterpriseActionTool {
    /// Name of the enterprise action.
    pub action_name: String,
    /// Optional API endpoint for the enterprise service.
    pub api_endpoint: Option<String>,
    /// Optional API key for authentication.
    pub api_key: Option<String>,
}

impl EnterpriseActionTool {
    /// Create a new enterprise action tool.
    pub fn new(action_name: impl Into<String>) -> Self {
        Self {
            action_name: action_name.into(),
            api_endpoint: None,
            api_key: None,
        }
    }

    /// Set the API endpoint.
    pub fn with_api_endpoint(mut self, endpoint: impl Into<String>) -> Self {
        self.api_endpoint = Some(endpoint.into());
        self
    }

    /// Set the API key.
    pub fn with_api_key(mut self, key: impl Into<String>) -> Self {
        self.api_key = Some(key.into());
        self
    }

    /// Returns the tool name.
    pub fn name(&self) -> &str {
        "EnterpriseActionTool"
    }

    /// Returns a description of the tool.
    pub fn description(&self) -> &str {
        "Execute enterprise workflow actions through external service integrations"
    }

    /// Run the enterprise action.
    pub fn run(&self, _args: HashMap<String, Value>) -> Result<Value, anyhow::Error> {
        anyhow::bail!(
            "EnterpriseActionTool: not yet implemented - requires enterprise service integration"
        )
    }
}

// ── ZapierActionTool ─────────────────────────────────────────────────────────

/// Tool for executing Zapier actions via the Zapier Natural Language Actions API.
///
/// Corresponds to the Python `ZapierActionTool` class in `crewai_tools`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ZapierActionTool {
    /// Zapier NLA API key.
    pub api_key: Option<String>,
    /// Specific action ID to execute, if pre-configured.
    pub action_id: Option<String>,
}

impl ZapierActionTool {
    /// Create a new Zapier action tool.
    pub fn new() -> Self {
        Self {
            api_key: None,
            action_id: None,
        }
    }

    /// Set the API key.
    pub fn with_api_key(mut self, key: impl Into<String>) -> Self {
        self.api_key = Some(key.into());
        self
    }

    /// Set a specific action ID.
    pub fn with_action_id(mut self, id: impl Into<String>) -> Self {
        self.action_id = Some(id.into());
        self
    }

    /// Returns the tool name.
    pub fn name(&self) -> &str {
        "ZapierActionTool"
    }

    /// Returns a description of the tool.
    pub fn description(&self) -> &str {
        "Execute Zapier actions via the Natural Language Actions API"
    }

    /// Run the Zapier action.
    pub fn run(&self, _args: HashMap<String, Value>) -> Result<Value, anyhow::Error> {
        anyhow::bail!(
            "ZapierActionTool: not yet implemented - requires Zapier NLA API integration"
        )
    }
}

impl Default for ZapierActionTool {
    fn default() -> Self {
        Self::new()
    }
}

// ── RagAdapter ───────────────────────────────────────────────────────────────

/// Adapter for connecting RAG (Retrieval-Augmented Generation) pipelines to crewAI tools.
///
/// Corresponds to the Python `RagAdapter` class in `crewai_tools`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RagAdapter {
    /// Name of the RAG pipeline.
    pub pipeline_name: String,
    /// Embedding model to use.
    pub embedding_model: Option<String>,
    /// Number of results to retrieve.
    pub top_k: usize,
}

impl RagAdapter {
    /// Create a new RAG adapter.
    pub fn new(pipeline_name: impl Into<String>) -> Self {
        Self {
            pipeline_name: pipeline_name.into(),
            embedding_model: None,
            top_k: 5,
        }
    }

    /// Set the embedding model.
    pub fn with_embedding_model(mut self, model: impl Into<String>) -> Self {
        self.embedding_model = Some(model.into());
        self
    }

    /// Set the number of results to retrieve.
    pub fn with_top_k(mut self, k: usize) -> Self {
        self.top_k = k;
        self
    }

    /// Returns the adapter name.
    pub fn name(&self) -> &str {
        "RagAdapter"
    }

    /// Returns a description of the adapter.
    pub fn description(&self) -> &str {
        "Connect RAG pipelines to crewAI for retrieval-augmented generation"
    }

    /// Run a RAG query.
    pub fn run(&self, _args: HashMap<String, Value>) -> Result<Value, anyhow::Error> {
        anyhow::bail!("RagAdapter: not yet implemented - requires RAG pipeline integration")
    }
}

// ── LanceDbAdapter ───────────────────────────────────────────────────────────

/// Adapter for LanceDB vector database integration.
///
/// Corresponds to the Python `LanceDbAdapter` / LanceDB integration in `crewai_tools`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LanceDbAdapter {
    /// Path to the LanceDB database.
    pub db_path: String,
    /// Table name within the database.
    pub table_name: Option<String>,
    /// Number of results to return.
    pub top_k: usize,
}

impl LanceDbAdapter {
    /// Create a new LanceDB adapter.
    pub fn new(db_path: impl Into<String>) -> Self {
        Self {
            db_path: db_path.into(),
            table_name: None,
            top_k: 5,
        }
    }

    /// Set the table name.
    pub fn with_table_name(mut self, table: impl Into<String>) -> Self {
        self.table_name = Some(table.into());
        self
    }

    /// Set the number of results to return.
    pub fn with_top_k(mut self, k: usize) -> Self {
        self.top_k = k;
        self
    }

    /// Returns the adapter name.
    pub fn name(&self) -> &str {
        "LanceDbAdapter"
    }

    /// Returns a description of the adapter.
    pub fn description(&self) -> &str {
        "Vector search and storage via LanceDB"
    }

    /// Run a vector search query.
    pub fn run(&self, _args: HashMap<String, Value>) -> Result<Value, anyhow::Error> {
        anyhow::bail!("LanceDbAdapter: not yet implemented - requires LanceDB integration")
    }
}
