//! File operation tools for crewAI.
//!
//! This module contains tools for reading, writing, compressing, and listing
//! files and directories. Each struct corresponds to a Python tool class
//! in `crewai_tools`.

use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use serde_json::Value;

// ── FileReadTool ─────────────────────────────────────────────────────────────

/// Read the contents of a file from the local filesystem.
///
/// Corresponds to Python `FileReadTool` in `crewai_tools`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileReadTool {
    /// Path to the file to read (can also be provided at runtime).
    pub file_path: Option<String>,
}

impl FileReadTool {
    pub fn new() -> Self {
        Self { file_path: None }
    }

    pub fn with_file_path(mut self, path: impl Into<String>) -> Self {
        self.file_path = Some(path.into());
        self
    }

    pub fn run(&self, _args: HashMap<String, Value>) -> Result<Value, anyhow::Error> {
        anyhow::bail!("FileReadTool: not yet implemented - requires filesystem integration")
    }
}

impl Default for FileReadTool {
    fn default() -> Self {
        Self::new()
    }
}

// ── FileWriterTool ───────────────────────────────────────────────────────────

/// Write content to a file on the local filesystem.
///
/// Corresponds to Python `FileWriterTool` in `crewai_tools`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileWriterTool {
    /// Directory where files will be written.
    pub directory: Option<String>,
    /// Default filename to use if not provided at runtime.
    pub filename: Option<String>,
    /// Whether to overwrite existing files.
    pub overwrite: bool,
}

impl FileWriterTool {
    pub fn new() -> Self {
        Self {
            directory: None,
            filename: None,
            overwrite: false,
        }
    }

    pub fn with_directory(mut self, dir: impl Into<String>) -> Self {
        self.directory = Some(dir.into());
        self
    }

    pub fn with_filename(mut self, name: impl Into<String>) -> Self {
        self.filename = Some(name.into());
        self
    }

    pub fn with_overwrite(mut self, overwrite: bool) -> Self {
        self.overwrite = overwrite;
        self
    }

    pub fn run(&self, _args: HashMap<String, Value>) -> Result<Value, anyhow::Error> {
        anyhow::bail!("FileWriterTool: not yet implemented - requires filesystem integration")
    }
}

impl Default for FileWriterTool {
    fn default() -> Self {
        Self::new()
    }
}

// ── FileCompressorTool ───────────────────────────────────────────────────────

/// Compress files or directories into archive formats (zip, tar.gz, etc.).
///
/// Corresponds to Python `FileCompressorTool` in `crewai_tools`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileCompressorTool {
    /// Compression format: "zip", "tar.gz", "tar.bz2".
    pub format: String,
    /// Output path for the compressed file.
    pub output_path: Option<String>,
}

impl FileCompressorTool {
    pub fn new() -> Self {
        Self {
            format: "zip".to_string(),
            output_path: None,
        }
    }

    pub fn with_format(mut self, format: impl Into<String>) -> Self {
        self.format = format.into();
        self
    }

    pub fn with_output_path(mut self, path: impl Into<String>) -> Self {
        self.output_path = Some(path.into());
        self
    }

    pub fn run(&self, _args: HashMap<String, Value>) -> Result<Value, anyhow::Error> {
        anyhow::bail!(
            "FileCompressorTool: not yet implemented - requires archive/compression integration"
        )
    }
}

impl Default for FileCompressorTool {
    fn default() -> Self {
        Self::new()
    }
}

// ── DirectoryReadTool ────────────────────────────────────────────────────────

/// List and read the contents of a directory on the local filesystem.
///
/// Corresponds to Python `DirectoryReadTool` in `crewai_tools`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DirectoryReadTool {
    /// Path to the directory to read.
    pub directory: Option<String>,
}

impl DirectoryReadTool {
    pub fn new() -> Self {
        Self { directory: None }
    }

    pub fn with_directory(mut self, dir: impl Into<String>) -> Self {
        self.directory = Some(dir.into());
        self
    }

    pub fn run(&self, _args: HashMap<String, Value>) -> Result<Value, anyhow::Error> {
        anyhow::bail!("DirectoryReadTool: not yet implemented - requires filesystem integration")
    }
}

impl Default for DirectoryReadTool {
    fn default() -> Self {
        Self::new()
    }
}
