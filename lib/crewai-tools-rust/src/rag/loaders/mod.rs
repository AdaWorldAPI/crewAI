//! Document loaders for the RAG framework.
//!
//! Each loader implements the [`BaseLoader`](super::core::BaseLoader) trait and
//! handles loading documents from a specific file format or data source.
//! These correspond to loader classes in the Python `crewai_tools.rag` module.

use super::core::{BaseLoader, Document};

// ── CsvLoader ────────────────────────────────────────────────────────────────

/// Load documents from CSV files.
///
/// Corresponds to Python CSV loader in `crewai_tools.rag`.
#[derive(Debug, Clone)]
pub struct CsvLoader {
    /// Path to the CSV file.
    pub file_path: String,
    /// Column to use as document content. If `None`, all columns are concatenated.
    pub content_column: Option<String>,
}

impl CsvLoader {
    pub fn new(file_path: impl Into<String>) -> Self {
        Self {
            file_path: file_path.into(),
            content_column: None,
        }
    }

    pub fn with_content_column(mut self, column: impl Into<String>) -> Self {
        self.content_column = Some(column.into());
        self
    }
}

impl BaseLoader for CsvLoader {
    fn load(&self) -> Result<Vec<Document>, anyhow::Error> {
        anyhow::bail!("CsvLoader: not yet implemented - requires CSV parsing integration")
    }

    fn loader_name(&self) -> &str {
        "CsvLoader"
    }
}

// ── JsonLoader ───────────────────────────────────────────────────────────────

/// Load documents from JSON files.
///
/// Corresponds to Python JSON loader in `crewai_tools.rag`.
#[derive(Debug, Clone)]
pub struct JsonLoader {
    /// Path to the JSON file.
    pub file_path: String,
    /// JSON path expression for extracting content.
    pub content_path: Option<String>,
}

impl JsonLoader {
    pub fn new(file_path: impl Into<String>) -> Self {
        Self {
            file_path: file_path.into(),
            content_path: None,
        }
    }

    pub fn with_content_path(mut self, path: impl Into<String>) -> Self {
        self.content_path = Some(path.into());
        self
    }
}

impl BaseLoader for JsonLoader {
    fn load(&self) -> Result<Vec<Document>, anyhow::Error> {
        anyhow::bail!("JsonLoader: not yet implemented - requires JSON parsing integration")
    }

    fn loader_name(&self) -> &str {
        "JsonLoader"
    }
}

// ── PdfLoader ────────────────────────────────────────────────────────────────

/// Load documents from PDF files.
///
/// Corresponds to Python PDF loader in `crewai_tools.rag`.
#[derive(Debug, Clone)]
pub struct PdfLoader {
    /// Path to the PDF file.
    pub file_path: String,
    /// Whether to split by pages (each page becomes a separate document).
    pub split_by_page: bool,
}

impl PdfLoader {
    pub fn new(file_path: impl Into<String>) -> Self {
        Self {
            file_path: file_path.into(),
            split_by_page: true,
        }
    }

    pub fn with_split_by_page(mut self, split: bool) -> Self {
        self.split_by_page = split;
        self
    }
}

impl BaseLoader for PdfLoader {
    fn load(&self) -> Result<Vec<Document>, anyhow::Error> {
        anyhow::bail!("PdfLoader: not yet implemented - requires PDF parsing library")
    }

    fn loader_name(&self) -> &str {
        "PdfLoader"
    }
}

// ── TextLoader ───────────────────────────────────────────────────────────────

/// Load documents from plain text files.
///
/// Corresponds to Python text loader in `crewai_tools.rag`.
#[derive(Debug, Clone)]
pub struct TextLoader {
    /// Path to the text file.
    pub file_path: String,
    /// Encoding of the file (default: "utf-8").
    pub encoding: String,
}

impl TextLoader {
    pub fn new(file_path: impl Into<String>) -> Self {
        Self {
            file_path: file_path.into(),
            encoding: "utf-8".to_string(),
        }
    }

    pub fn with_encoding(mut self, encoding: impl Into<String>) -> Self {
        self.encoding = encoding.into();
        self
    }
}

impl BaseLoader for TextLoader {
    fn load(&self) -> Result<Vec<Document>, anyhow::Error> {
        anyhow::bail!("TextLoader: not yet implemented - requires file I/O integration")
    }

    fn loader_name(&self) -> &str {
        "TextLoader"
    }
}

// ── WebpageLoader ────────────────────────────────────────────────────────────

/// Load documents from web pages by fetching and parsing HTML.
///
/// Corresponds to Python webpage loader in `crewai_tools.rag`.
#[derive(Debug, Clone)]
pub struct WebpageLoader {
    /// URL of the web page to load.
    pub url: String,
    /// Whether to extract only the main content (strip navigation, etc.).
    pub extract_main_content: bool,
}

impl WebpageLoader {
    pub fn new(url: impl Into<String>) -> Self {
        Self {
            url: url.into(),
            extract_main_content: true,
        }
    }

    pub fn with_extract_main_content(mut self, extract: bool) -> Self {
        self.extract_main_content = extract;
        self
    }
}

impl BaseLoader for WebpageLoader {
    fn load(&self) -> Result<Vec<Document>, anyhow::Error> {
        anyhow::bail!(
            "WebpageLoader: not yet implemented - requires HTTP client and HTML parsing"
        )
    }

    fn loader_name(&self) -> &str {
        "WebpageLoader"
    }
}

// ── DirectoryLoader ──────────────────────────────────────────────────────────

/// Load documents from all supported files in a directory.
///
/// Corresponds to Python directory loader in `crewai_tools.rag`.
#[derive(Debug, Clone)]
pub struct DirectoryLoader {
    /// Path to the directory.
    pub directory: String,
    /// File extensions to include (e.g., ["txt", "pdf", "md"]).
    pub extensions: Vec<String>,
    /// Whether to search subdirectories recursively.
    pub recursive: bool,
}

impl DirectoryLoader {
    pub fn new(directory: impl Into<String>) -> Self {
        Self {
            directory: directory.into(),
            extensions: Vec::new(),
            recursive: true,
        }
    }

    pub fn with_extensions(mut self, exts: Vec<String>) -> Self {
        self.extensions = exts;
        self
    }

    pub fn with_recursive(mut self, recursive: bool) -> Self {
        self.recursive = recursive;
        self
    }
}

impl BaseLoader for DirectoryLoader {
    fn load(&self) -> Result<Vec<Document>, anyhow::Error> {
        anyhow::bail!(
            "DirectoryLoader: not yet implemented - requires directory traversal and multi-format loading"
        )
    }

    fn loader_name(&self) -> &str {
        "DirectoryLoader"
    }
}

// ── DocxLoader ───────────────────────────────────────────────────────────────

/// Load documents from DOCX (Microsoft Word) files.
///
/// Corresponds to Python DOCX loader in `crewai_tools.rag`.
#[derive(Debug, Clone)]
pub struct DocxLoader {
    /// Path to the DOCX file.
    pub file_path: String,
}

impl DocxLoader {
    pub fn new(file_path: impl Into<String>) -> Self {
        Self {
            file_path: file_path.into(),
        }
    }
}

impl BaseLoader for DocxLoader {
    fn load(&self) -> Result<Vec<Document>, anyhow::Error> {
        anyhow::bail!("DocxLoader: not yet implemented - requires DOCX parsing library")
    }

    fn loader_name(&self) -> &str {
        "DocxLoader"
    }
}

// ── XmlLoader ────────────────────────────────────────────────────────────────

/// Load documents from XML files.
///
/// Corresponds to Python XML loader in `crewai_tools.rag`.
#[derive(Debug, Clone)]
pub struct XmlLoader {
    /// Path to the XML file.
    pub file_path: String,
    /// XPath expression for selecting content nodes.
    pub content_xpath: Option<String>,
}

impl XmlLoader {
    pub fn new(file_path: impl Into<String>) -> Self {
        Self {
            file_path: file_path.into(),
            content_xpath: None,
        }
    }

    pub fn with_content_xpath(mut self, xpath: impl Into<String>) -> Self {
        self.content_xpath = Some(xpath.into());
        self
    }
}

impl BaseLoader for XmlLoader {
    fn load(&self) -> Result<Vec<Document>, anyhow::Error> {
        anyhow::bail!("XmlLoader: not yet implemented - requires XML parsing library")
    }

    fn loader_name(&self) -> &str {
        "XmlLoader"
    }
}

// ── GithubLoader ─────────────────────────────────────────────────────────────

/// Load documents from a GitHub repository.
///
/// Corresponds to Python GitHub loader in `crewai_tools.rag`.
#[derive(Debug, Clone)]
pub struct GithubLoader {
    /// Repository in "owner/repo" format.
    pub repository: String,
    /// GitHub API token.
    pub github_token: Option<String>,
    /// Branch to load from (default: "main").
    pub branch: String,
    /// File extensions to include.
    pub extensions: Vec<String>,
}

impl GithubLoader {
    pub fn new(repository: impl Into<String>) -> Self {
        Self {
            repository: repository.into(),
            github_token: None,
            branch: "main".to_string(),
            extensions: Vec::new(),
        }
    }

    pub fn with_github_token(mut self, token: impl Into<String>) -> Self {
        self.github_token = Some(token.into());
        self
    }

    pub fn with_branch(mut self, branch: impl Into<String>) -> Self {
        self.branch = branch.into();
        self
    }

    pub fn with_extensions(mut self, exts: Vec<String>) -> Self {
        self.extensions = exts;
        self
    }
}

impl BaseLoader for GithubLoader {
    fn load(&self) -> Result<Vec<Document>, anyhow::Error> {
        anyhow::bail!("GithubLoader: not yet implemented - requires GitHub API integration")
    }

    fn loader_name(&self) -> &str {
        "GithubLoader"
    }
}

// ── YoutubeVideoLoader ───────────────────────────────────────────────────────

/// Load transcripts from YouTube videos.
///
/// Corresponds to Python YouTube video loader in `crewai_tools.rag`.
#[derive(Debug, Clone)]
pub struct YoutubeVideoLoader {
    /// YouTube video URL or ID.
    pub video_url: String,
    /// Preferred language for transcripts.
    pub language: String,
}

impl YoutubeVideoLoader {
    pub fn new(video_url: impl Into<String>) -> Self {
        Self {
            video_url: video_url.into(),
            language: "en".to_string(),
        }
    }

    pub fn with_language(mut self, lang: impl Into<String>) -> Self {
        self.language = lang.into();
        self
    }
}

impl BaseLoader for YoutubeVideoLoader {
    fn load(&self) -> Result<Vec<Document>, anyhow::Error> {
        anyhow::bail!(
            "YoutubeVideoLoader: not yet implemented - requires YouTube transcript API integration"
        )
    }

    fn loader_name(&self) -> &str {
        "YoutubeVideoLoader"
    }
}
