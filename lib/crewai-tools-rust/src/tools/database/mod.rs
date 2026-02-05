//! Database tools for crewAI.
//!
//! This module contains tools for querying vector databases, SQL databases,
//! and data warehouses. Each struct corresponds to a Python tool class
//! in `crewai_tools`.

use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use serde_json::Value;

// ── QdrantVectorSearchTool ───────────────────────────────────────────────────

/// Search a Qdrant vector database for semantically similar documents.
///
/// Corresponds to Python `QdrantVectorSearchTool` in `crewai_tools`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QdrantVectorSearchTool {
    /// Qdrant server URL.
    pub qdrant_url: Option<String>,
    /// Qdrant API key (for cloud instances).
    pub api_key: Option<String>,
    /// Collection name to search.
    pub collection_name: String,
    /// Number of results to return.
    pub top_k: usize,
}

impl QdrantVectorSearchTool {
    pub fn new(collection_name: impl Into<String>) -> Self {
        Self {
            qdrant_url: None,
            api_key: None,
            collection_name: collection_name.into(),
            top_k: 5,
        }
    }

    pub fn with_qdrant_url(mut self, url: impl Into<String>) -> Self {
        self.qdrant_url = Some(url.into());
        self
    }

    pub fn with_api_key(mut self, key: impl Into<String>) -> Self {
        self.api_key = Some(key.into());
        self
    }

    pub fn with_top_k(mut self, k: usize) -> Self {
        self.top_k = k;
        self
    }

    pub fn run(&self, _args: HashMap<String, Value>) -> Result<Value, anyhow::Error> {
        anyhow::bail!(
            "QdrantVectorSearchTool: not yet implemented - requires Qdrant client integration"
        )
    }
}

// ── MongoDbVectorSearchTool ──────────────────────────────────────────────────

/// Search a MongoDB Atlas vector index for similar documents.
///
/// Corresponds to Python `MongoDBVectorSearchTool` in `crewai_tools`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MongoDbVectorSearchTool {
    /// MongoDB connection string.
    pub connection_string: Option<String>,
    /// Database name.
    pub database: String,
    /// Collection name.
    pub collection: String,
    /// Vector index name.
    pub index_name: String,
    /// Number of results to return.
    pub top_k: usize,
}

impl MongoDbVectorSearchTool {
    pub fn new(database: impl Into<String>, collection: impl Into<String>) -> Self {
        Self {
            connection_string: None,
            database: database.into(),
            collection: collection.into(),
            index_name: "vector_index".to_string(),
            top_k: 5,
        }
    }

    pub fn with_connection_string(mut self, conn: impl Into<String>) -> Self {
        self.connection_string = Some(conn.into());
        self
    }

    pub fn with_index_name(mut self, name: impl Into<String>) -> Self {
        self.index_name = name.into();
        self
    }

    pub fn with_top_k(mut self, k: usize) -> Self {
        self.top_k = k;
        self
    }

    pub fn run(&self, _args: HashMap<String, Value>) -> Result<Value, anyhow::Error> {
        anyhow::bail!(
            "MongoDbVectorSearchTool: not yet implemented - requires MongoDB driver integration"
        )
    }
}

// ── WeaviateVectorSearchTool ─────────────────────────────────────────────────

/// Search a Weaviate vector database for similar objects.
///
/// Corresponds to Python `WeaviateVectorSearchTool` in `crewai_tools`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WeaviateVectorSearchTool {
    /// Weaviate server URL.
    pub weaviate_url: Option<String>,
    /// Weaviate API key.
    pub api_key: Option<String>,
    /// Class name to search.
    pub class_name: String,
    /// Number of results to return.
    pub top_k: usize,
}

impl WeaviateVectorSearchTool {
    pub fn new(class_name: impl Into<String>) -> Self {
        Self {
            weaviate_url: None,
            api_key: None,
            class_name: class_name.into(),
            top_k: 5,
        }
    }

    pub fn with_weaviate_url(mut self, url: impl Into<String>) -> Self {
        self.weaviate_url = Some(url.into());
        self
    }

    pub fn with_api_key(mut self, key: impl Into<String>) -> Self {
        self.api_key = Some(key.into());
        self
    }

    pub fn with_top_k(mut self, k: usize) -> Self {
        self.top_k = k;
        self
    }

    pub fn run(&self, _args: HashMap<String, Value>) -> Result<Value, anyhow::Error> {
        anyhow::bail!(
            "WeaviateVectorSearchTool: not yet implemented - requires Weaviate client integration"
        )
    }
}

// ── CouchbaseFtsVectorSearchTool ─────────────────────────────────────────────

/// Search a Couchbase full-text search (FTS) vector index.
///
/// Corresponds to Python `CouchbaseFTSVectorSearchTool` in `crewai_tools`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CouchbaseFtsVectorSearchTool {
    /// Couchbase connection string.
    pub connection_string: Option<String>,
    /// Bucket name.
    pub bucket: String,
    /// Scope name.
    pub scope: Option<String>,
    /// FTS index name.
    pub index_name: String,
    /// Number of results to return.
    pub top_k: usize,
}

impl CouchbaseFtsVectorSearchTool {
    pub fn new(bucket: impl Into<String>, index_name: impl Into<String>) -> Self {
        Self {
            connection_string: None,
            bucket: bucket.into(),
            scope: None,
            index_name: index_name.into(),
            top_k: 5,
        }
    }

    pub fn with_connection_string(mut self, conn: impl Into<String>) -> Self {
        self.connection_string = Some(conn.into());
        self
    }

    pub fn with_scope(mut self, scope: impl Into<String>) -> Self {
        self.scope = Some(scope.into());
        self
    }

    pub fn with_top_k(mut self, k: usize) -> Self {
        self.top_k = k;
        self
    }

    pub fn run(&self, _args: HashMap<String, Value>) -> Result<Value, anyhow::Error> {
        anyhow::bail!(
            "CouchbaseFtsVectorSearchTool: not yet implemented - requires Couchbase SDK integration"
        )
    }
}

// ── SingleStoreSearchTool ────────────────────────────────────────────────────

/// Search a SingleStore database using vector similarity or full-text search.
///
/// Corresponds to Python `SingleStoreSearchTool` in `crewai_tools`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SingleStoreSearchTool {
    /// SingleStore connection string.
    pub connection_string: Option<String>,
    /// Database name.
    pub database: Option<String>,
    /// Table name.
    pub table: Option<String>,
    /// Number of results to return.
    pub top_k: usize,
}

impl SingleStoreSearchTool {
    pub fn new() -> Self {
        Self {
            connection_string: None,
            database: None,
            table: None,
            top_k: 5,
        }
    }

    pub fn with_connection_string(mut self, conn: impl Into<String>) -> Self {
        self.connection_string = Some(conn.into());
        self
    }

    pub fn with_database(mut self, db: impl Into<String>) -> Self {
        self.database = Some(db.into());
        self
    }

    pub fn with_table(mut self, table: impl Into<String>) -> Self {
        self.table = Some(table.into());
        self
    }

    pub fn with_top_k(mut self, k: usize) -> Self {
        self.top_k = k;
        self
    }

    pub fn run(&self, _args: HashMap<String, Value>) -> Result<Value, anyhow::Error> {
        anyhow::bail!(
            "SingleStoreSearchTool: not yet implemented - requires SingleStore driver integration"
        )
    }
}

impl Default for SingleStoreSearchTool {
    fn default() -> Self {
        Self::new()
    }
}

// ── SnowflakeSearchTool ──────────────────────────────────────────────────────

/// Query a Snowflake data warehouse using natural language or SQL.
///
/// Corresponds to Python `SnowflakeSearchTool` in `crewai_tools`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SnowflakeSearchTool {
    /// Snowflake account identifier.
    pub account: Option<String>,
    /// Snowflake username.
    pub username: Option<String>,
    /// Snowflake password.
    pub password: Option<String>,
    /// Warehouse name.
    pub warehouse: Option<String>,
    /// Database name.
    pub database: Option<String>,
    /// Schema name.
    pub schema: Option<String>,
}

impl SnowflakeSearchTool {
    pub fn new() -> Self {
        Self {
            account: None,
            username: None,
            password: None,
            warehouse: None,
            database: None,
            schema: None,
        }
    }

    pub fn with_account(mut self, account: impl Into<String>) -> Self {
        self.account = Some(account.into());
        self
    }

    pub fn with_username(mut self, username: impl Into<String>) -> Self {
        self.username = Some(username.into());
        self
    }

    pub fn with_password(mut self, password: impl Into<String>) -> Self {
        self.password = Some(password.into());
        self
    }

    pub fn with_warehouse(mut self, warehouse: impl Into<String>) -> Self {
        self.warehouse = Some(warehouse.into());
        self
    }

    pub fn with_database(mut self, database: impl Into<String>) -> Self {
        self.database = Some(database.into());
        self
    }

    pub fn with_schema(mut self, schema: impl Into<String>) -> Self {
        self.schema = Some(schema.into());
        self
    }

    pub fn run(&self, _args: HashMap<String, Value>) -> Result<Value, anyhow::Error> {
        anyhow::bail!(
            "SnowflakeSearchTool: not yet implemented - requires Snowflake driver integration"
        )
    }
}

impl Default for SnowflakeSearchTool {
    fn default() -> Self {
        Self::new()
    }
}

// ── DatabricksQueryTool ──────────────────────────────────────────────────────

/// Query Databricks SQL warehouses or notebooks.
///
/// Corresponds to Python `DatabricksQueryTool` in `crewai_tools`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabricksQueryTool {
    /// Databricks workspace URL.
    pub workspace_url: Option<String>,
    /// Databricks access token.
    pub access_token: Option<String>,
    /// SQL warehouse ID.
    pub warehouse_id: Option<String>,
}

impl DatabricksQueryTool {
    pub fn new() -> Self {
        Self {
            workspace_url: None,
            access_token: None,
            warehouse_id: None,
        }
    }

    pub fn with_workspace_url(mut self, url: impl Into<String>) -> Self {
        self.workspace_url = Some(url.into());
        self
    }

    pub fn with_access_token(mut self, token: impl Into<String>) -> Self {
        self.access_token = Some(token.into());
        self
    }

    pub fn with_warehouse_id(mut self, id: impl Into<String>) -> Self {
        self.warehouse_id = Some(id.into());
        self
    }

    pub fn run(&self, _args: HashMap<String, Value>) -> Result<Value, anyhow::Error> {
        anyhow::bail!(
            "DatabricksQueryTool: not yet implemented - requires Databricks SQL API integration"
        )
    }
}

impl Default for DatabricksQueryTool {
    fn default() -> Self {
        Self::new()
    }
}

// ── Nl2SqlTool ───────────────────────────────────────────────────────────────

/// Convert natural language questions to SQL queries and execute them.
///
/// Corresponds to Python `NL2SQLTool` in `crewai_tools`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Nl2SqlTool {
    /// Database connection string.
    pub connection_string: Option<String>,
    /// Database dialect (e.g., "postgresql", "mysql", "sqlite").
    pub dialect: String,
    /// List of tables to include in schema context.
    pub tables: Vec<String>,
}

impl Nl2SqlTool {
    pub fn new() -> Self {
        Self {
            connection_string: None,
            dialect: "postgresql".to_string(),
            tables: Vec::new(),
        }
    }

    pub fn with_connection_string(mut self, conn: impl Into<String>) -> Self {
        self.connection_string = Some(conn.into());
        self
    }

    pub fn with_dialect(mut self, dialect: impl Into<String>) -> Self {
        self.dialect = dialect.into();
        self
    }

    pub fn with_tables(mut self, tables: Vec<String>) -> Self {
        self.tables = tables;
        self
    }

    pub fn run(&self, _args: HashMap<String, Value>) -> Result<Value, anyhow::Error> {
        anyhow::bail!(
            "Nl2SqlTool: not yet implemented - requires LLM SQL generation and database driver integration"
        )
    }
}

impl Default for Nl2SqlTool {
    fn default() -> Self {
        Self::new()
    }
}
