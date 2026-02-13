//! Search tools for crewAI.
//!
//! This module contains tools for searching the web, documents, databases,
//! and various data sources. Each struct corresponds to a Python tool class
//! in `crewai_tools`.

use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use serde_json::Value;

// ── BraveSearchTool ──────────────────────────────────────────────────────────

/// Search the web using the Brave Search API.
///
/// Corresponds to Python `BraveSearchTool` in `crewai_tools`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BraveSearchTool {
    /// Brave Search API key.
    pub api_key: Option<String>,
    /// Maximum number of results to return.
    pub max_results: usize,
    /// Country code for localized results.
    pub country: Option<String>,
}

impl BraveSearchTool {
    pub fn new() -> Self {
        Self {
            api_key: None,
            max_results: 10,
            country: None,
        }
    }

    pub fn with_api_key(mut self, key: impl Into<String>) -> Self {
        self.api_key = Some(key.into());
        self
    }

    pub fn with_max_results(mut self, n: usize) -> Self {
        self.max_results = n;
        self
    }

    pub fn with_country(mut self, country: impl Into<String>) -> Self {
        self.country = Some(country.into());
        self
    }

    /// Run a Brave Search query.
    ///
    /// # Arguments (in `args`)
    /// * `search_query` - The search query string.
    pub fn run(&self, args: HashMap<String, Value>) -> Result<Value, anyhow::Error> {
        let query = args
            .get("search_query")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing required argument: search_query"))?;

        let api_key = self
            .api_key
            .clone()
            .or_else(|| std::env::var("BRAVE_API_KEY").ok())
            .ok_or_else(|| anyhow::anyhow!("Missing BRAVE_API_KEY"))?;

        let client = reqwest::blocking::Client::new();
        let mut request = client
            .get("https://api.search.brave.com/res/v1/web/search")
            .header("Accept", "application/json")
            .header("Accept-Encoding", "gzip")
            .header("X-Subscription-Token", &api_key)
            .query(&[("q", query), ("count", &self.max_results.to_string())]);

        if let Some(ref country) = self.country {
            request = request.query(&[("country", country.as_str())]);
        }

        let resp = request.send()?.json::<Value>()?;
        Ok(resp)
    }
}

impl Default for BraveSearchTool {
    fn default() -> Self {
        Self::new()
    }
}

// ── SerperDevTool ────────────────────────────────────────────────────────────

/// Search the web using the Serper.dev Google Search API.
///
/// Corresponds to Python `SerperDevTool` in `crewai_tools`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SerperDevTool {
    /// Serper.dev API key.
    pub api_key: Option<String>,
    /// Search type (e.g., "search", "news", "images").
    pub search_type: String,
    /// Maximum number of results.
    pub max_results: usize,
    /// Country code.
    pub country: Option<String>,
    /// Language code.
    pub language: Option<String>,
}

impl SerperDevTool {
    pub fn new() -> Self {
        Self {
            api_key: None,
            search_type: "search".to_string(),
            max_results: 10,
            country: None,
            language: None,
        }
    }

    pub fn with_api_key(mut self, key: impl Into<String>) -> Self {
        self.api_key = Some(key.into());
        self
    }

    pub fn with_search_type(mut self, search_type: impl Into<String>) -> Self {
        self.search_type = search_type.into();
        self
    }

    pub fn with_max_results(mut self, n: usize) -> Self {
        self.max_results = n;
        self
    }

    pub fn with_country(mut self, country: impl Into<String>) -> Self {
        self.country = Some(country.into());
        self
    }

    pub fn with_language(mut self, lang: impl Into<String>) -> Self {
        self.language = Some(lang.into());
        self
    }

    /// Run a Serper.dev Google Search query.
    ///
    /// # Arguments (in `args`)
    /// * `search_query` - The search query string.
    pub fn run(&self, args: HashMap<String, Value>) -> Result<Value, anyhow::Error> {
        let query = args
            .get("search_query")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing required argument: search_query"))?;

        let api_key = self
            .api_key
            .clone()
            .or_else(|| std::env::var("SERPER_API_KEY").ok())
            .ok_or_else(|| anyhow::anyhow!("Missing SERPER_API_KEY"))?;

        let mut body = serde_json::json!({
            "q": query,
            "num": self.max_results,
        });
        if let Some(ref country) = self.country {
            body["gl"] = Value::String(country.clone());
        }
        if let Some(ref lang) = self.language {
            body["hl"] = Value::String(lang.clone());
        }

        let endpoint = match self.search_type.as_str() {
            "news" => "https://google.serper.dev/news",
            "images" => "https://google.serper.dev/images",
            "places" => "https://google.serper.dev/places",
            _ => "https://google.serper.dev/search",
        };

        let client = reqwest::blocking::Client::new();
        let resp = client
            .post(endpoint)
            .header("X-API-KEY", &api_key)
            .header("Content-Type", "application/json")
            .json(&body)
            .send()?
            .json::<Value>()?;

        Ok(resp)
    }
}

impl Default for SerperDevTool {
    fn default() -> Self {
        Self::new()
    }
}

// ── TavilySearchTool ─────────────────────────────────────────────────────────

/// Search the web using the Tavily Search API.
///
/// Corresponds to Python `TavilySearchTool` in `crewai_tools`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TavilySearchTool {
    /// Tavily API key.
    pub api_key: Option<String>,
    /// Search depth: "basic" or "advanced".
    pub search_depth: String,
    /// Maximum number of results.
    pub max_results: usize,
}

impl TavilySearchTool {
    pub fn new() -> Self {
        Self {
            api_key: None,
            search_depth: "basic".to_string(),
            max_results: 10,
        }
    }

    pub fn with_api_key(mut self, key: impl Into<String>) -> Self {
        self.api_key = Some(key.into());
        self
    }

    pub fn with_search_depth(mut self, depth: impl Into<String>) -> Self {
        self.search_depth = depth.into();
        self
    }

    pub fn with_max_results(mut self, n: usize) -> Self {
        self.max_results = n;
        self
    }

    pub fn run(&self, _args: HashMap<String, Value>) -> Result<Value, anyhow::Error> {
        anyhow::bail!("TavilySearchTool: not yet implemented - requires Tavily API integration")
    }
}

impl Default for TavilySearchTool {
    fn default() -> Self {
        Self::new()
    }
}

// ── ExaSearchTool ────────────────────────────────────────────────────────────

/// Search using the EXA (formerly Metaphor) neural search API.
///
/// Corresponds to Python `EXASearchTool` in `crewai_tools`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExaSearchTool {
    /// EXA API key.
    pub api_key: Option<String>,
    /// Maximum number of results.
    pub max_results: usize,
    /// Whether to include page contents in results.
    pub include_contents: bool,
}

impl ExaSearchTool {
    pub fn new() -> Self {
        Self {
            api_key: None,
            max_results: 10,
            include_contents: true,
        }
    }

    pub fn with_api_key(mut self, key: impl Into<String>) -> Self {
        self.api_key = Some(key.into());
        self
    }

    pub fn with_max_results(mut self, n: usize) -> Self {
        self.max_results = n;
        self
    }

    pub fn with_include_contents(mut self, include: bool) -> Self {
        self.include_contents = include;
        self
    }

    pub fn run(&self, _args: HashMap<String, Value>) -> Result<Value, anyhow::Error> {
        anyhow::bail!("ExaSearchTool: not yet implemented - requires EXA API integration")
    }
}

impl Default for ExaSearchTool {
    fn default() -> Self {
        Self::new()
    }
}

// ── ArxivPaperTool ───────────────────────────────────────────────────────────

/// Search and retrieve academic papers from arXiv.
///
/// Corresponds to Python `ArxivPaperTool` in `crewai_tools`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArxivPaperTool {
    /// Maximum number of papers to return.
    pub max_results: usize,
    /// Sort order: "relevance" or "submittedDate".
    pub sort_by: String,
}

impl ArxivPaperTool {
    pub fn new() -> Self {
        Self {
            max_results: 5,
            sort_by: "relevance".to_string(),
        }
    }

    pub fn with_max_results(mut self, n: usize) -> Self {
        self.max_results = n;
        self
    }

    pub fn with_sort_by(mut self, sort: impl Into<String>) -> Self {
        self.sort_by = sort.into();
        self
    }

    pub fn run(&self, _args: HashMap<String, Value>) -> Result<Value, anyhow::Error> {
        anyhow::bail!("ArxivPaperTool: not yet implemented - requires arXiv API integration")
    }
}

impl Default for ArxivPaperTool {
    fn default() -> Self {
        Self::new()
    }
}

// ── CsvSearchTool ────────────────────────────────────────────────────────────

/// Search within CSV files using semantic or keyword search.
///
/// Corresponds to Python `CSVSearchTool` in `crewai_tools`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CsvSearchTool {
    /// Path to the CSV file.
    pub file_path: Option<String>,
}

impl CsvSearchTool {
    pub fn new() -> Self {
        Self { file_path: None }
    }

    pub fn with_file_path(mut self, path: impl Into<String>) -> Self {
        self.file_path = Some(path.into());
        self
    }

    pub fn run(&self, _args: HashMap<String, Value>) -> Result<Value, anyhow::Error> {
        anyhow::bail!("CsvSearchTool: not yet implemented - requires CSV parsing and search")
    }
}

impl Default for CsvSearchTool {
    fn default() -> Self {
        Self::new()
    }
}

// ── CodeDocsSearchTool ───────────────────────────────────────────────────────

/// Search through code documentation sites.
///
/// Corresponds to Python `CodeDocsSearchTool` in `crewai_tools`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodeDocsSearchTool {
    /// URL of the documentation site.
    pub docs_url: Option<String>,
}

impl CodeDocsSearchTool {
    pub fn new() -> Self {
        Self { docs_url: None }
    }

    pub fn with_docs_url(mut self, url: impl Into<String>) -> Self {
        self.docs_url = Some(url.into());
        self
    }

    pub fn run(&self, _args: HashMap<String, Value>) -> Result<Value, anyhow::Error> {
        anyhow::bail!(
            "CodeDocsSearchTool: not yet implemented - requires documentation scraping and search"
        )
    }
}

impl Default for CodeDocsSearchTool {
    fn default() -> Self {
        Self::new()
    }
}

// ── DirectorySearchTool ──────────────────────────────────────────────────────

/// Search through files in a directory using semantic search.
///
/// Corresponds to Python `DirectorySearchTool` in `crewai_tools`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DirectorySearchTool {
    /// Path to the directory to search.
    pub directory: Option<String>,
}

impl DirectorySearchTool {
    pub fn new() -> Self {
        Self { directory: None }
    }

    pub fn with_directory(mut self, dir: impl Into<String>) -> Self {
        self.directory = Some(dir.into());
        self
    }

    pub fn run(&self, _args: HashMap<String, Value>) -> Result<Value, anyhow::Error> {
        anyhow::bail!(
            "DirectorySearchTool: not yet implemented - requires directory traversal and semantic search"
        )
    }
}

impl Default for DirectorySearchTool {
    fn default() -> Self {
        Self::new()
    }
}

// ── DocxSearchTool ───────────────────────────────────────────────────────────

/// Search within DOCX (Microsoft Word) files.
///
/// Corresponds to Python `DOCXSearchTool` in `crewai_tools`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocxSearchTool {
    /// Path to the DOCX file.
    pub file_path: Option<String>,
}

impl DocxSearchTool {
    pub fn new() -> Self {
        Self { file_path: None }
    }

    pub fn with_file_path(mut self, path: impl Into<String>) -> Self {
        self.file_path = Some(path.into());
        self
    }

    pub fn run(&self, _args: HashMap<String, Value>) -> Result<Value, anyhow::Error> {
        anyhow::bail!("DocxSearchTool: not yet implemented - requires DOCX parsing and search")
    }
}

impl Default for DocxSearchTool {
    fn default() -> Self {
        Self::new()
    }
}

// ── GithubSearchTool ─────────────────────────────────────────────────────────

/// Search within GitHub repositories (code, issues, PRs, etc.).
///
/// Corresponds to Python `GithubSearchTool` in `crewai_tools`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GithubSearchTool {
    /// GitHub API token.
    pub github_token: Option<String>,
    /// Repository in "owner/repo" format.
    pub repository: Option<String>,
    /// Content types to search: "code", "issues", "pulls", "discussions".
    pub content_types: Vec<String>,
}

impl GithubSearchTool {
    pub fn new() -> Self {
        Self {
            github_token: None,
            repository: None,
            content_types: vec!["code".to_string()],
        }
    }

    pub fn with_github_token(mut self, token: impl Into<String>) -> Self {
        self.github_token = Some(token.into());
        self
    }

    pub fn with_repository(mut self, repo: impl Into<String>) -> Self {
        self.repository = Some(repo.into());
        self
    }

    pub fn with_content_types(mut self, types: Vec<String>) -> Self {
        self.content_types = types;
        self
    }

    pub fn run(&self, _args: HashMap<String, Value>) -> Result<Value, anyhow::Error> {
        anyhow::bail!("GithubSearchTool: not yet implemented - requires GitHub API integration")
    }
}

impl Default for GithubSearchTool {
    fn default() -> Self {
        Self::new()
    }
}

// ── JsonSearchTool ───────────────────────────────────────────────────────────

/// Search within JSON files using semantic or keyword search.
///
/// Corresponds to Python `JSONSearchTool` in `crewai_tools`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonSearchTool {
    /// Path to the JSON file.
    pub file_path: Option<String>,
}

impl JsonSearchTool {
    pub fn new() -> Self {
        Self { file_path: None }
    }

    pub fn with_file_path(mut self, path: impl Into<String>) -> Self {
        self.file_path = Some(path.into());
        self
    }

    pub fn run(&self, _args: HashMap<String, Value>) -> Result<Value, anyhow::Error> {
        anyhow::bail!("JsonSearchTool: not yet implemented - requires JSON parsing and search")
    }
}

impl Default for JsonSearchTool {
    fn default() -> Self {
        Self::new()
    }
}

// ── MdxSearchTool ────────────────────────────────────────────────────────────

/// Search within MDX (Markdown with JSX) files.
///
/// Corresponds to Python `MDXSearchTool` in `crewai_tools`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MdxSearchTool {
    /// Path to the MDX file or directory.
    pub file_path: Option<String>,
}

impl MdxSearchTool {
    pub fn new() -> Self {
        Self { file_path: None }
    }

    pub fn with_file_path(mut self, path: impl Into<String>) -> Self {
        self.file_path = Some(path.into());
        self
    }

    pub fn run(&self, _args: HashMap<String, Value>) -> Result<Value, anyhow::Error> {
        anyhow::bail!("MdxSearchTool: not yet implemented - requires MDX parsing and search")
    }
}

impl Default for MdxSearchTool {
    fn default() -> Self {
        Self::new()
    }
}

// ── PdfSearchTool ────────────────────────────────────────────────────────────

/// Search within PDF documents.
///
/// Corresponds to Python `PDFSearchTool` in `crewai_tools`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PdfSearchTool {
    /// Path to the PDF file.
    pub file_path: Option<String>,
}

impl PdfSearchTool {
    pub fn new() -> Self {
        Self { file_path: None }
    }

    pub fn with_file_path(mut self, path: impl Into<String>) -> Self {
        self.file_path = Some(path.into());
        self
    }

    pub fn run(&self, _args: HashMap<String, Value>) -> Result<Value, anyhow::Error> {
        anyhow::bail!("PdfSearchTool: not yet implemented - requires PDF parsing and search")
    }
}

impl Default for PdfSearchTool {
    fn default() -> Self {
        Self::new()
    }
}

// ── TxtSearchTool ────────────────────────────────────────────────────────────

/// Search within plain text files.
///
/// Corresponds to Python `TXTSearchTool` in `crewai_tools`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TxtSearchTool {
    /// Path to the text file.
    pub file_path: Option<String>,
}

impl TxtSearchTool {
    pub fn new() -> Self {
        Self { file_path: None }
    }

    pub fn with_file_path(mut self, path: impl Into<String>) -> Self {
        self.file_path = Some(path.into());
        self
    }

    pub fn run(&self, _args: HashMap<String, Value>) -> Result<Value, anyhow::Error> {
        anyhow::bail!("TxtSearchTool: not yet implemented - requires text file search")
    }
}

impl Default for TxtSearchTool {
    fn default() -> Self {
        Self::new()
    }
}

// ── XmlSearchTool ────────────────────────────────────────────────────────────

/// Search within XML files.
///
/// Corresponds to Python `XMLSearchTool` in `crewai_tools`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct XmlSearchTool {
    /// Path to the XML file.
    pub file_path: Option<String>,
}

impl XmlSearchTool {
    pub fn new() -> Self {
        Self { file_path: None }
    }

    pub fn with_file_path(mut self, path: impl Into<String>) -> Self {
        self.file_path = Some(path.into());
        self
    }

    pub fn run(&self, _args: HashMap<String, Value>) -> Result<Value, anyhow::Error> {
        anyhow::bail!("XmlSearchTool: not yet implemented - requires XML parsing and search")
    }
}

impl Default for XmlSearchTool {
    fn default() -> Self {
        Self::new()
    }
}

// ── WebsiteSearchTool ────────────────────────────────────────────────────────

/// Search within the content of a specific website.
///
/// Corresponds to Python `WebsiteSearchTool` in `crewai_tools`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebsiteSearchTool {
    /// URL of the website to search.
    pub website_url: Option<String>,
}

impl WebsiteSearchTool {
    pub fn new() -> Self {
        Self { website_url: None }
    }

    pub fn with_website_url(mut self, url: impl Into<String>) -> Self {
        self.website_url = Some(url.into());
        self
    }

    pub fn run(&self, _args: HashMap<String, Value>) -> Result<Value, anyhow::Error> {
        anyhow::bail!(
            "WebsiteSearchTool: not yet implemented - requires website scraping and semantic search"
        )
    }
}

impl Default for WebsiteSearchTool {
    fn default() -> Self {
        Self::new()
    }
}

// ── YoutubeChannelSearchTool ─────────────────────────────────────────────────

/// Search within a YouTube channel's videos and transcripts.
///
/// Corresponds to Python `YoutubeChannelSearchTool` in `crewai_tools`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct YoutubeChannelSearchTool {
    /// YouTube channel URL or ID.
    pub channel: Option<String>,
}

impl YoutubeChannelSearchTool {
    pub fn new() -> Self {
        Self { channel: None }
    }

    pub fn with_channel(mut self, channel: impl Into<String>) -> Self {
        self.channel = Some(channel.into());
        self
    }

    pub fn run(&self, _args: HashMap<String, Value>) -> Result<Value, anyhow::Error> {
        anyhow::bail!(
            "YoutubeChannelSearchTool: not yet implemented - requires YouTube API integration"
        )
    }
}

impl Default for YoutubeChannelSearchTool {
    fn default() -> Self {
        Self::new()
    }
}

// ── YoutubeVideoSearchTool ───────────────────────────────────────────────────

/// Search within a specific YouTube video's transcript.
///
/// Corresponds to Python `YoutubeVideoSearchTool` in `crewai_tools`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct YoutubeVideoSearchTool {
    /// YouTube video URL or ID.
    pub video_url: Option<String>,
}

impl YoutubeVideoSearchTool {
    pub fn new() -> Self {
        Self { video_url: None }
    }

    pub fn with_video_url(mut self, url: impl Into<String>) -> Self {
        self.video_url = Some(url.into());
        self
    }

    pub fn run(&self, _args: HashMap<String, Value>) -> Result<Value, anyhow::Error> {
        anyhow::bail!(
            "YoutubeVideoSearchTool: not yet implemented - requires YouTube transcript API integration"
        )
    }
}

impl Default for YoutubeVideoSearchTool {
    fn default() -> Self {
        Self::new()
    }
}

// ── MySqlSearchTool ──────────────────────────────────────────────────────────

/// Search within a MySQL database using natural language queries.
///
/// Corresponds to Python `MySQLSearchTool` in `crewai_tools`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MySqlSearchTool {
    /// MySQL connection string.
    pub connection_string: Option<String>,
    /// Database name.
    pub database: Option<String>,
}

impl MySqlSearchTool {
    pub fn new() -> Self {
        Self {
            connection_string: None,
            database: None,
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

    pub fn run(&self, _args: HashMap<String, Value>) -> Result<Value, anyhow::Error> {
        anyhow::bail!("MySqlSearchTool: not yet implemented - requires MySQL driver integration")
    }
}

impl Default for MySqlSearchTool {
    fn default() -> Self {
        Self::new()
    }
}

// ── LinkupSearchTool ─────────────────────────────────────────────────────────

/// Search using the Linkup API for enriched link previews and metadata.
///
/// Corresponds to Python `LinkupSearchTool` in `crewai_tools`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LinkupSearchTool {
    /// Linkup API key.
    pub api_key: Option<String>,
    /// Maximum number of results.
    pub max_results: usize,
}

impl LinkupSearchTool {
    pub fn new() -> Self {
        Self {
            api_key: None,
            max_results: 10,
        }
    }

    pub fn with_api_key(mut self, key: impl Into<String>) -> Self {
        self.api_key = Some(key.into());
        self
    }

    pub fn with_max_results(mut self, n: usize) -> Self {
        self.max_results = n;
        self
    }

    pub fn run(&self, _args: HashMap<String, Value>) -> Result<Value, anyhow::Error> {
        anyhow::bail!("LinkupSearchTool: not yet implemented - requires Linkup API integration")
    }
}

impl Default for LinkupSearchTool {
    fn default() -> Self {
        Self::new()
    }
}

// ── ParallelSearchTool ───────────────────────────────────────────────────────

/// Execute multiple search queries in parallel across different search tools.
///
/// Corresponds to Python `ParallelSearchTool` in `crewai_tools`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParallelSearchTool {
    /// Maximum concurrency for parallel searches.
    pub max_concurrency: usize,
}

impl ParallelSearchTool {
    pub fn new() -> Self {
        Self {
            max_concurrency: 5,
        }
    }

    pub fn with_max_concurrency(mut self, n: usize) -> Self {
        self.max_concurrency = n;
        self
    }

    pub fn run(&self, _args: HashMap<String, Value>) -> Result<Value, anyhow::Error> {
        anyhow::bail!(
            "ParallelSearchTool: not yet implemented - requires parallel search orchestration"
        )
    }
}

impl Default for ParallelSearchTool {
    fn default() -> Self {
        Self::new()
    }
}
