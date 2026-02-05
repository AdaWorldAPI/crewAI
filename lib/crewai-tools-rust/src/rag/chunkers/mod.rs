//! Text chunkers for the RAG framework.
//!
//! Each chunker implements the [`BaseChunker`](super::core::BaseChunker) trait and
//! handles splitting documents into smaller chunks using different strategies.
//! These correspond to chunker classes in the Python `crewai_tools.rag` module.

use super::core::{BaseChunker, Chunk, Document};

// ── DefaultChunker ───────────────────────────────────────────────────────────

/// Default chunker using fixed-size character-based splitting with overlap.
///
/// Corresponds to the Python default chunker in `crewai_tools.rag`.
#[derive(Debug, Clone)]
pub struct DefaultChunker {
    /// Maximum number of characters per chunk.
    pub chunk_size: usize,
    /// Number of overlapping characters between consecutive chunks.
    pub chunk_overlap: usize,
}

impl DefaultChunker {
    pub fn new() -> Self {
        Self {
            chunk_size: 1000,
            chunk_overlap: 200,
        }
    }

    pub fn with_chunk_size(mut self, size: usize) -> Self {
        self.chunk_size = size;
        self
    }

    pub fn with_chunk_overlap(mut self, overlap: usize) -> Self {
        self.chunk_overlap = overlap;
        self
    }
}

impl Default for DefaultChunker {
    fn default() -> Self {
        Self::new()
    }
}

impl BaseChunker for DefaultChunker {
    fn chunk(&self, _document: &Document) -> Result<Vec<Chunk>, anyhow::Error> {
        anyhow::bail!("DefaultChunker: not yet implemented - requires text splitting logic")
    }

    fn chunker_name(&self) -> &str {
        "DefaultChunker"
    }
}

// ── TextChunker ──────────────────────────────────────────────────────────────

/// Smart text chunker that respects sentence and paragraph boundaries.
///
/// Corresponds to the Python text chunker in `crewai_tools.rag`.
#[derive(Debug, Clone)]
pub struct TextChunker {
    /// Maximum number of characters per chunk.
    pub chunk_size: usize,
    /// Number of overlapping characters between consecutive chunks.
    pub chunk_overlap: usize,
    /// Separator pattern for splitting (e.g., "\n\n" for paragraphs).
    pub separator: String,
}

impl TextChunker {
    pub fn new() -> Self {
        Self {
            chunk_size: 1000,
            chunk_overlap: 200,
            separator: "\n\n".to_string(),
        }
    }

    pub fn with_chunk_size(mut self, size: usize) -> Self {
        self.chunk_size = size;
        self
    }

    pub fn with_chunk_overlap(mut self, overlap: usize) -> Self {
        self.chunk_overlap = overlap;
        self
    }

    pub fn with_separator(mut self, sep: impl Into<String>) -> Self {
        self.separator = sep.into();
        self
    }
}

impl Default for TextChunker {
    fn default() -> Self {
        Self::new()
    }
}

impl BaseChunker for TextChunker {
    fn chunk(&self, _document: &Document) -> Result<Vec<Chunk>, anyhow::Error> {
        anyhow::bail!(
            "TextChunker: not yet implemented - requires sentence/paragraph-aware splitting"
        )
    }

    fn chunker_name(&self) -> &str {
        "TextChunker"
    }
}

// ── StructuredChunker ────────────────────────────────────────────────────────

/// Chunker for structured documents (JSON, XML, CSV) that preserves structure.
///
/// Corresponds to the Python structured chunker in `crewai_tools.rag`.
#[derive(Debug, Clone)]
pub struct StructuredChunker {
    /// Maximum number of elements per chunk.
    pub max_elements: usize,
}

impl StructuredChunker {
    pub fn new() -> Self {
        Self { max_elements: 50 }
    }

    pub fn with_max_elements(mut self, n: usize) -> Self {
        self.max_elements = n;
        self
    }
}

impl Default for StructuredChunker {
    fn default() -> Self {
        Self::new()
    }
}

impl BaseChunker for StructuredChunker {
    fn chunk(&self, _document: &Document) -> Result<Vec<Chunk>, anyhow::Error> {
        anyhow::bail!(
            "StructuredChunker: not yet implemented - requires structure-aware splitting"
        )
    }

    fn chunker_name(&self) -> &str {
        "StructuredChunker"
    }
}

// ── WebChunker ───────────────────────────────────────────────────────────────

/// Chunker optimized for web page content that respects HTML semantic sections.
///
/// Corresponds to the Python web chunker in `crewai_tools.rag`.
#[derive(Debug, Clone)]
pub struct WebChunker {
    /// Maximum number of characters per chunk.
    pub chunk_size: usize,
    /// Whether to strip HTML tags before chunking.
    pub strip_html: bool,
}

impl WebChunker {
    pub fn new() -> Self {
        Self {
            chunk_size: 1000,
            strip_html: true,
        }
    }

    pub fn with_chunk_size(mut self, size: usize) -> Self {
        self.chunk_size = size;
        self
    }

    pub fn with_strip_html(mut self, strip: bool) -> Self {
        self.strip_html = strip;
        self
    }
}

impl Default for WebChunker {
    fn default() -> Self {
        Self::new()
    }
}

impl BaseChunker for WebChunker {
    fn chunk(&self, _document: &Document) -> Result<Vec<Chunk>, anyhow::Error> {
        anyhow::bail!(
            "WebChunker: not yet implemented - requires HTML-aware splitting"
        )
    }

    fn chunker_name(&self) -> &str {
        "WebChunker"
    }
}
