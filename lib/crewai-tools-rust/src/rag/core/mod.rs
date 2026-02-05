//! Core traits and types for the RAG framework.
//!
//! This module defines the foundational abstractions used by loaders, chunkers,
//! and embedding services in the RAG pipeline.

use serde::{Deserialize, Serialize};
use serde_json::Value;

/// A single document loaded from a data source.
///
/// Corresponds to document representations in the Python `crewai_tools` RAG system.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Document {
    /// The text content of the document.
    pub content: String,
    /// Metadata associated with the document (source, page number, etc.).
    pub metadata: std::collections::HashMap<String, Value>,
}

impl Document {
    /// Create a new document with the given content.
    pub fn new(content: impl Into<String>) -> Self {
        Self {
            content: content.into(),
            metadata: std::collections::HashMap::new(),
        }
    }

    /// Add a metadata key-value pair.
    pub fn with_metadata(mut self, key: impl Into<String>, value: Value) -> Self {
        self.metadata.insert(key.into(), value);
        self
    }
}

/// A chunk of text produced by splitting a document.
///
/// Corresponds to chunk representations in the Python `crewai_tools` RAG system.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Chunk {
    /// The text content of the chunk.
    pub content: String,
    /// Metadata inherited from the source document plus chunk-specific info.
    pub metadata: std::collections::HashMap<String, Value>,
    /// Index of this chunk within the source document.
    pub index: usize,
}

/// Trait for loading documents from various data sources.
///
/// Corresponds to the Python `BaseLoader` class in `crewai_tools.rag`.
pub trait BaseLoader: Send + Sync {
    /// Load documents from the configured data source.
    ///
    /// Returns a list of documents, or an error if loading fails.
    fn load(&self) -> Result<Vec<Document>, anyhow::Error>;

    /// Returns the name of this loader.
    fn loader_name(&self) -> &str;
}

/// Trait for splitting documents into smaller chunks.
///
/// Corresponds to the Python `BaseChunker` class in `crewai_tools.rag`.
pub trait BaseChunker: Send + Sync {
    /// Split a document into chunks.
    ///
    /// Returns a list of chunks, or an error if chunking fails.
    fn chunk(&self, document: &Document) -> Result<Vec<Chunk>, anyhow::Error>;

    /// Returns the name of this chunker.
    fn chunker_name(&self) -> &str;
}

/// Trait for generating text embeddings.
///
/// Corresponds to the Python `EmbeddingService` / embedding config in `crewai_tools.rag`.
pub trait EmbeddingService: Send + Sync {
    /// Generate an embedding vector for the given text.
    ///
    /// Returns a vector of f32 values, or an error if embedding fails.
    fn embed(&self, text: &str) -> Result<Vec<f32>, anyhow::Error>;

    /// Generate embedding vectors for a batch of texts.
    ///
    /// Returns a list of embedding vectors, or an error if embedding fails.
    fn embed_batch(&self, texts: &[&str]) -> Result<Vec<Vec<f32>>, anyhow::Error>;

    /// Returns the name of the embedding model.
    fn model_name(&self) -> &str;

    /// Returns the dimensionality of the embedding vectors.
    fn dimensions(&self) -> usize;
}
