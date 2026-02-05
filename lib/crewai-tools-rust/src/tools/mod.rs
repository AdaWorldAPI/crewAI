//! Tool modules for crewAI.
//!
//! Each sub-module is feature-gated and contains tool structs that correspond
//! to the Python `crewai_tools` package classes.

/// Search tools: web search engines, document search, data source search.
#[cfg(feature = "search")]
pub mod search;

/// Web scraping tools: website content extraction, crawling, element scraping.
#[cfg(feature = "web_scraping")]
pub mod web_scraping;

/// Database tools: vector databases, SQL queries, data warehouses.
#[cfg(feature = "database")]
pub mod database;

/// File operation tools: reading, writing, compressing, and listing files.
#[cfg(feature = "file_ops")]
pub mod file_ops;

/// AI/ML tools: image generation, vision, OCR, RAG, and LLM integrations.
#[cfg(feature = "ai_ml")]
pub mod ai_ml;

/// Automation tools: Composio, Zapier, Apify, and crewAI automation helpers.
#[cfg(feature = "automation")]
pub mod automation;

/// Cloud storage tools: S3, Bedrock, and other cloud service integrations.
#[cfg(feature = "cloud_storage")]
pub mod cloud_storage;

/// Browser tools: headless browser automation and web interaction.
#[cfg(feature = "browser")]
pub mod browser;
