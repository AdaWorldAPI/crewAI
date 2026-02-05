//! # crewai-tools
//!
//! Specialized tool integrations for crewAI-rust, porting the Python crewai-tools package.
//!
//! This crate provides 90+ specialized tools organized into feature-gated modules:
//!
//! - **search** - Web search, document search, and data source search tools
//! - **web_scraping** - Website scraping and content extraction tools
//! - **database** - Vector database and SQL query tools
//! - **file_ops** - File reading, writing, and compression tools
//! - **ai_ml** - AI/ML service integration tools (DALL-E, vision, OCR, RAG)
//! - **automation** - Workflow automation tools (Composio, Zapier, Apify)
//! - **cloud_storage** - Cloud storage tools (S3, Bedrock)
//! - **browser** - Browser automation tools (Browserbase, Hyperbrowser)
//! - **rag** - Retrieval-Augmented Generation framework (loaders, chunkers, embeddings)
//!
//! ## Feature Flags
//!
//! By default, the `search` and `file_ops` features are enabled. Use the `all` feature
//! to enable every module, or pick individual features as needed.

/// Crate version, matching the Python crewai-tools package version.
pub const VERSION: &str = "1.9.3";

// ── Adapters (always available) ──────────────────────────────────────────────
pub mod adapters;

// ── Tool modules (feature-gated) ────────────────────────────────────────────
pub mod tools;

// ── RAG framework (feature-gated) ───────────────────────────────────────────
#[cfg(feature = "rag")]
pub mod rag;

// ── Re-exports ──────────────────────────────────────────────────────────────

// Adapters
pub use adapters::{
    EnterpriseActionTool, McpServerAdapter, RagAdapter, LanceDbAdapter, ZapierActionTool,
};

// Search tools
#[cfg(feature = "search")]
pub use tools::search::{
    ArxivPaperTool, BraveSearchTool, CodeDocsSearchTool, CsvSearchTool, DirectorySearchTool,
    DocxSearchTool, ExaSearchTool, GithubSearchTool, JsonSearchTool, LinkupSearchTool,
    MdxSearchTool, MySqlSearchTool, ParallelSearchTool, PdfSearchTool, SerperDevTool,
    TavilySearchTool, TxtSearchTool, WebsiteSearchTool, XmlSearchTool,
    YoutubeChannelSearchTool, YoutubeVideoSearchTool,
};

// Web scraping tools
#[cfg(feature = "web_scraping")]
pub use tools::web_scraping::{
    FirecrawlCrawlWebsiteTool, FirecrawlScrapeWebsiteTool, FirecrawlSearchTool,
    JinaScrapeWebsiteTool, ScrapeElementFromWebsiteTool, ScrapeWebsiteTool,
    ScrapegraphScrapeTool, ScrapflyScrapeWebsiteTool, SeleniumScrapingTool,
    SerperScrapeWebsiteTool, SpiderTool,
};

// Database tools
#[cfg(feature = "database")]
pub use tools::database::{
    CouchbaseFtsVectorSearchTool, DatabricksQueryTool, MongoDbVectorSearchTool, Nl2SqlTool,
    QdrantVectorSearchTool, SingleStoreSearchTool, SnowflakeSearchTool,
    WeaviateVectorSearchTool,
};

// File operation tools
#[cfg(feature = "file_ops")]
pub use tools::file_ops::{DirectoryReadTool, FileCompressorTool, FileReadTool, FileWriterTool};

// AI/ML tools
#[cfg(feature = "ai_ml")]
pub use tools::ai_ml::{AiMindTool, DalleTool, LlamaIndexTool, OcrTool, RagTool, VisionTool};

// Automation tools
#[cfg(feature = "automation")]
pub use tools::automation::{
    ApifyActorsTool, ComposioTool, GenerateCrewaiAutomationTool, InvokeCrewaiAutomationTool,
    MergeAgentHandlerTool, ZapierActionTools,
};

// Cloud storage tools
#[cfg(feature = "cloud_storage")]
pub use tools::cloud_storage::{
    BedrockInvokeAgentTool, BedrockKbRetrieverTool, S3ReaderTool, S3WriterTool,
};

// Browser tools
#[cfg(feature = "browser")]
pub use tools::browser::{BrowserbaseLoadTool, HyperbrowserLoadTool, MultiOnTool, StagehandTool};

// RAG framework
#[cfg(feature = "rag")]
pub use rag::{
    chunkers::{DefaultChunker, StructuredChunker, TextChunker, WebChunker},
    core::{BaseChunker, BaseLoader, EmbeddingService},
    loaders::{
        CsvLoader, DirectoryLoader, DocxLoader, GithubLoader, JsonLoader, PdfLoader, TextLoader,
        WebpageLoader, XmlLoader, YoutubeVideoLoader,
    },
};
