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

    /// Read the contents of a file.
    ///
    /// # Arguments (in `args`)
    /// * `file_path` - Path to the file to read.
    pub fn run(&self, args: HashMap<String, Value>) -> Result<Value, anyhow::Error> {
        let path = args
            .get("file_path")
            .and_then(|v| v.as_str())
            .or(self.file_path.as_deref())
            .ok_or_else(|| anyhow::anyhow!("Missing required argument: file_path"))?;

        let content = std::fs::read_to_string(path)
            .map_err(|e| anyhow::anyhow!("Failed to read file '{}': {}", path, e))?;

        Ok(Value::String(content))
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

    /// Write content to a file.
    ///
    /// # Arguments (in `args`)
    /// * `content` - The content to write.
    /// * `filename` - The filename (optional if set on struct).
    /// * `directory` - The directory (optional if set on struct).
    pub fn run(&self, args: HashMap<String, Value>) -> Result<Value, anyhow::Error> {
        let content = args
            .get("content")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing required argument: content"))?;

        let filename = args
            .get("filename")
            .and_then(|v| v.as_str())
            .map(String::from)
            .or_else(|| self.filename.clone())
            .ok_or_else(|| anyhow::anyhow!("Missing required argument: filename"))?;

        let directory = args
            .get("directory")
            .and_then(|v| v.as_str())
            .map(String::from)
            .or_else(|| self.directory.clone())
            .unwrap_or_else(|| ".".to_string());

        let dir_path = std::path::Path::new(&directory);
        if !dir_path.exists() {
            std::fs::create_dir_all(dir_path)
                .map_err(|e| anyhow::anyhow!("Failed to create directory '{}': {}", directory, e))?;
        }

        let file_path = dir_path.join(&filename);

        if file_path.exists() && !self.overwrite {
            anyhow::bail!(
                "File '{}' already exists and overwrite is disabled",
                file_path.display()
            );
        }

        std::fs::write(&file_path, content)
            .map_err(|e| anyhow::anyhow!("Failed to write file '{}': {}", file_path.display(), e))?;

        Ok(Value::String(format!(
            "Successfully wrote to {}",
            file_path.display()
        )))
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

    /// List the contents of a directory.
    ///
    /// # Arguments (in `args`)
    /// * `directory` - Path to the directory to list.
    pub fn run(&self, args: HashMap<String, Value>) -> Result<Value, anyhow::Error> {
        let dir = args
            .get("directory")
            .and_then(|v| v.as_str())
            .or(self.directory.as_deref())
            .ok_or_else(|| anyhow::anyhow!("Missing required argument: directory"))?;

        let entries: Vec<String> = std::fs::read_dir(dir)
            .map_err(|e| anyhow::anyhow!("Failed to read directory '{}': {}", dir, e))?
            .filter_map(|entry| {
                entry.ok().map(|e| {
                    let path = e.path();
                    let name = path.file_name().unwrap_or_default().to_string_lossy().to_string();
                    if path.is_dir() {
                        format!("{}/", name)
                    } else {
                        name
                    }
                })
            })
            .collect();

        Ok(serde_json::json!({
            "directory": dir,
            "entries": entries,
            "count": entries.len(),
        }))
    }
}

impl Default for DirectoryReadTool {
    fn default() -> Self {
        Self::new()
    }
}
