//! Web scraping tools for crewAI.
//!
//! This module contains tools for scraping website content, crawling sites,
//! and extracting specific elements from web pages. Each struct corresponds
//! to a Python tool class in `crewai_tools`.

use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use serde_json::Value;

// ── ScrapeWebsiteTool ────────────────────────────────────────────────────────

/// Scrape the full content of a website page.
///
/// Corresponds to Python `ScrapeWebsiteTool` in `crewai_tools`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScrapeWebsiteTool {
    /// URL of the website to scrape (can also be provided at runtime).
    pub website_url: Option<String>,
}

impl ScrapeWebsiteTool {
    pub fn new() -> Self {
        Self { website_url: None }
    }

    pub fn with_website_url(mut self, url: impl Into<String>) -> Self {
        self.website_url = Some(url.into());
        self
    }

    pub fn run(&self, _args: HashMap<String, Value>) -> Result<Value, anyhow::Error> {
        anyhow::bail!("ScrapeWebsiteTool: not yet implemented - requires HTTP client and HTML parsing")
    }
}

impl Default for ScrapeWebsiteTool {
    fn default() -> Self {
        Self::new()
    }
}

// ── ScrapeElementFromWebsiteTool ─────────────────────────────────────────────

/// Scrape a specific HTML element from a website using CSS selectors.
///
/// Corresponds to Python `ScrapeElementFromWebsiteTool` in `crewai_tools`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScrapeElementFromWebsiteTool {
    /// URL of the website to scrape.
    pub website_url: Option<String>,
    /// CSS selector for the element to extract.
    pub css_selector: Option<String>,
}

impl ScrapeElementFromWebsiteTool {
    pub fn new() -> Self {
        Self {
            website_url: None,
            css_selector: None,
        }
    }

    pub fn with_website_url(mut self, url: impl Into<String>) -> Self {
        self.website_url = Some(url.into());
        self
    }

    pub fn with_css_selector(mut self, selector: impl Into<String>) -> Self {
        self.css_selector = Some(selector.into());
        self
    }

    pub fn run(&self, _args: HashMap<String, Value>) -> Result<Value, anyhow::Error> {
        anyhow::bail!(
            "ScrapeElementFromWebsiteTool: not yet implemented - requires CSS selector-based scraping"
        )
    }
}

impl Default for ScrapeElementFromWebsiteTool {
    fn default() -> Self {
        Self::new()
    }
}

// ── FirecrawlCrawlWebsiteTool ────────────────────────────────────────────────

/// Crawl an entire website using the Firecrawl API.
///
/// Corresponds to Python `FirecrawlCrawlWebsiteTool` in `crewai_tools`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FirecrawlCrawlWebsiteTool {
    /// Firecrawl API key.
    pub api_key: Option<String>,
    /// URL to start crawling from.
    pub url: Option<String>,
    /// Maximum number of pages to crawl.
    pub max_pages: usize,
}

impl FirecrawlCrawlWebsiteTool {
    pub fn new() -> Self {
        Self {
            api_key: None,
            url: None,
            max_pages: 100,
        }
    }

    pub fn with_api_key(mut self, key: impl Into<String>) -> Self {
        self.api_key = Some(key.into());
        self
    }

    pub fn with_url(mut self, url: impl Into<String>) -> Self {
        self.url = Some(url.into());
        self
    }

    pub fn with_max_pages(mut self, n: usize) -> Self {
        self.max_pages = n;
        self
    }

    pub fn run(&self, _args: HashMap<String, Value>) -> Result<Value, anyhow::Error> {
        anyhow::bail!(
            "FirecrawlCrawlWebsiteTool: not yet implemented - requires Firecrawl API integration"
        )
    }
}

impl Default for FirecrawlCrawlWebsiteTool {
    fn default() -> Self {
        Self::new()
    }
}

// ── FirecrawlScrapeWebsiteTool ───────────────────────────────────────────────

/// Scrape a single website page using the Firecrawl API.
///
/// Corresponds to Python `FirecrawlScrapeWebsiteTool` in `crewai_tools`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FirecrawlScrapeWebsiteTool {
    /// Firecrawl API key.
    pub api_key: Option<String>,
    /// URL to scrape.
    pub url: Option<String>,
}

impl FirecrawlScrapeWebsiteTool {
    pub fn new() -> Self {
        Self {
            api_key: None,
            url: None,
        }
    }

    pub fn with_api_key(mut self, key: impl Into<String>) -> Self {
        self.api_key = Some(key.into());
        self
    }

    pub fn with_url(mut self, url: impl Into<String>) -> Self {
        self.url = Some(url.into());
        self
    }

    pub fn run(&self, _args: HashMap<String, Value>) -> Result<Value, anyhow::Error> {
        anyhow::bail!(
            "FirecrawlScrapeWebsiteTool: not yet implemented - requires Firecrawl API integration"
        )
    }
}

impl Default for FirecrawlScrapeWebsiteTool {
    fn default() -> Self {
        Self::new()
    }
}

// ── FirecrawlSearchTool ──────────────────────────────────────────────────────

/// Search for content across websites using the Firecrawl API.
///
/// Corresponds to Python `FirecrawlSearchTool` in `crewai_tools`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FirecrawlSearchTool {
    /// Firecrawl API key.
    pub api_key: Option<String>,
    /// Maximum number of results.
    pub max_results: usize,
}

impl FirecrawlSearchTool {
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
        anyhow::bail!(
            "FirecrawlSearchTool: not yet implemented - requires Firecrawl API integration"
        )
    }
}

impl Default for FirecrawlSearchTool {
    fn default() -> Self {
        Self::new()
    }
}

// ── JinaScrapeWebsiteTool ────────────────────────────────────────────────────

/// Scrape website content using the Jina Reader API (returns clean markdown).
///
/// Corresponds to Python `JinaScrapeWebsiteTool` in `crewai_tools`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JinaScrapeWebsiteTool {
    /// URL to scrape.
    pub url: Option<String>,
}

impl JinaScrapeWebsiteTool {
    pub fn new() -> Self {
        Self { url: None }
    }

    pub fn with_url(mut self, url: impl Into<String>) -> Self {
        self.url = Some(url.into());
        self
    }

    pub fn run(&self, _args: HashMap<String, Value>) -> Result<Value, anyhow::Error> {
        anyhow::bail!(
            "JinaScrapeWebsiteTool: not yet implemented - requires Jina Reader API integration"
        )
    }
}

impl Default for JinaScrapeWebsiteTool {
    fn default() -> Self {
        Self::new()
    }
}

// ── SeleniumScrapingTool ─────────────────────────────────────────────────────

/// Scrape JavaScript-rendered websites using Selenium WebDriver.
///
/// Corresponds to Python `SeleniumScrapingTool` in `crewai_tools`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SeleniumScrapingTool {
    /// URL to scrape.
    pub website_url: Option<String>,
    /// CSS selector for element extraction.
    pub css_selector: Option<String>,
    /// Wait timeout in seconds for page load.
    pub wait_timeout: u64,
}

impl SeleniumScrapingTool {
    pub fn new() -> Self {
        Self {
            website_url: None,
            css_selector: None,
            wait_timeout: 10,
        }
    }

    pub fn with_website_url(mut self, url: impl Into<String>) -> Self {
        self.website_url = Some(url.into());
        self
    }

    pub fn with_css_selector(mut self, selector: impl Into<String>) -> Self {
        self.css_selector = Some(selector.into());
        self
    }

    pub fn with_wait_timeout(mut self, timeout: u64) -> Self {
        self.wait_timeout = timeout;
        self
    }

    pub fn run(&self, _args: HashMap<String, Value>) -> Result<Value, anyhow::Error> {
        anyhow::bail!(
            "SeleniumScrapingTool: not yet implemented - requires Selenium WebDriver integration"
        )
    }
}

impl Default for SeleniumScrapingTool {
    fn default() -> Self {
        Self::new()
    }
}

// ── ScrapflyScrapeWebsiteTool ────────────────────────────────────────────────

/// Scrape websites using the Scrapfly API with anti-bot bypass.
///
/// Corresponds to Python `ScrapflyScrapeWebsiteTool` in `crewai_tools`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScrapflyScrapeWebsiteTool {
    /// Scrapfly API key.
    pub api_key: Option<String>,
    /// URL to scrape.
    pub url: Option<String>,
    /// Whether to enable anti-scraping protection bypass.
    pub anti_scraping: bool,
}

impl ScrapflyScrapeWebsiteTool {
    pub fn new() -> Self {
        Self {
            api_key: None,
            url: None,
            anti_scraping: true,
        }
    }

    pub fn with_api_key(mut self, key: impl Into<String>) -> Self {
        self.api_key = Some(key.into());
        self
    }

    pub fn with_url(mut self, url: impl Into<String>) -> Self {
        self.url = Some(url.into());
        self
    }

    pub fn with_anti_scraping(mut self, enabled: bool) -> Self {
        self.anti_scraping = enabled;
        self
    }

    pub fn run(&self, _args: HashMap<String, Value>) -> Result<Value, anyhow::Error> {
        anyhow::bail!(
            "ScrapflyScrapeWebsiteTool: not yet implemented - requires Scrapfly API integration"
        )
    }
}

impl Default for ScrapflyScrapeWebsiteTool {
    fn default() -> Self {
        Self::new()
    }
}

// ── ScrapegraphScrapeTool ────────────────────────────────────────────────────

/// Scrape websites using the ScrapeGraph AI-powered scraping service.
///
/// Corresponds to Python `ScrapegraphScrapeTool` in `crewai_tools`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScrapegraphScrapeTool {
    /// ScrapeGraph API key.
    pub api_key: Option<String>,
    /// URL to scrape.
    pub url: Option<String>,
}

impl ScrapegraphScrapeTool {
    pub fn new() -> Self {
        Self {
            api_key: None,
            url: None,
        }
    }

    pub fn with_api_key(mut self, key: impl Into<String>) -> Self {
        self.api_key = Some(key.into());
        self
    }

    pub fn with_url(mut self, url: impl Into<String>) -> Self {
        self.url = Some(url.into());
        self
    }

    pub fn run(&self, _args: HashMap<String, Value>) -> Result<Value, anyhow::Error> {
        anyhow::bail!(
            "ScrapegraphScrapeTool: not yet implemented - requires ScrapeGraph API integration"
        )
    }
}

impl Default for ScrapegraphScrapeTool {
    fn default() -> Self {
        Self::new()
    }
}

// ── SerperScrapeWebsiteTool ──────────────────────────────────────────────────

/// Scrape websites using the Serper.dev web scraping endpoint.
///
/// Corresponds to Python `SerperScrapeWebsiteTool` in `crewai_tools`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SerperScrapeWebsiteTool {
    /// Serper.dev API key.
    pub api_key: Option<String>,
    /// URL to scrape.
    pub url: Option<String>,
}

impl SerperScrapeWebsiteTool {
    pub fn new() -> Self {
        Self {
            api_key: None,
            url: None,
        }
    }

    pub fn with_api_key(mut self, key: impl Into<String>) -> Self {
        self.api_key = Some(key.into());
        self
    }

    pub fn with_url(mut self, url: impl Into<String>) -> Self {
        self.url = Some(url.into());
        self
    }

    pub fn run(&self, _args: HashMap<String, Value>) -> Result<Value, anyhow::Error> {
        anyhow::bail!(
            "SerperScrapeWebsiteTool: not yet implemented - requires Serper.dev API integration"
        )
    }
}

impl Default for SerperScrapeWebsiteTool {
    fn default() -> Self {
        Self::new()
    }
}

// ── SpiderTool ───────────────────────────────────────────────────────────────

/// Web crawling and scraping tool using the Spider API.
///
/// Corresponds to Python `SpiderTool` in `crewai_tools`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpiderTool {
    /// Spider API key.
    pub api_key: Option<String>,
    /// URL to crawl.
    pub url: Option<String>,
    /// Maximum depth for crawling.
    pub max_depth: usize,
}

impl SpiderTool {
    pub fn new() -> Self {
        Self {
            api_key: None,
            url: None,
            max_depth: 3,
        }
    }

    pub fn with_api_key(mut self, key: impl Into<String>) -> Self {
        self.api_key = Some(key.into());
        self
    }

    pub fn with_url(mut self, url: impl Into<String>) -> Self {
        self.url = Some(url.into());
        self
    }

    pub fn with_max_depth(mut self, depth: usize) -> Self {
        self.max_depth = depth;
        self
    }

    pub fn run(&self, _args: HashMap<String, Value>) -> Result<Value, anyhow::Error> {
        anyhow::bail!("SpiderTool: not yet implemented - requires Spider API integration")
    }
}

impl Default for SpiderTool {
    fn default() -> Self {
        Self::new()
    }
}
