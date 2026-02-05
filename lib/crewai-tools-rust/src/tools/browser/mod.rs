//! Browser tools for crewAI.
//!
//! This module contains tools for browser automation and web interaction.
//! Each struct corresponds to a Python tool class in `crewai_tools`.

use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use serde_json::Value;

// ── BrowserbaseLoadTool ──────────────────────────────────────────────────────

/// Load and interact with web pages using the Browserbase cloud browser service.
///
/// Corresponds to Python `BrowserbaseLoadTool` in `crewai_tools`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BrowserbaseLoadTool {
    /// Browserbase API key.
    pub api_key: Option<String>,
    /// Browserbase project ID.
    pub project_id: Option<String>,
    /// Whether to enable text-only mode (no images).
    pub text_only: bool,
}

impl BrowserbaseLoadTool {
    pub fn new() -> Self {
        Self {
            api_key: None,
            project_id: None,
            text_only: false,
        }
    }

    pub fn with_api_key(mut self, key: impl Into<String>) -> Self {
        self.api_key = Some(key.into());
        self
    }

    pub fn with_project_id(mut self, id: impl Into<String>) -> Self {
        self.project_id = Some(id.into());
        self
    }

    pub fn with_text_only(mut self, text_only: bool) -> Self {
        self.text_only = text_only;
        self
    }

    pub fn run(&self, _args: HashMap<String, Value>) -> Result<Value, anyhow::Error> {
        anyhow::bail!(
            "BrowserbaseLoadTool: not yet implemented - requires Browserbase API integration"
        )
    }
}

impl Default for BrowserbaseLoadTool {
    fn default() -> Self {
        Self::new()
    }
}

// ── HyperbrowserLoadTool ─────────────────────────────────────────────────────

/// Load and interact with web pages using the Hyperbrowser service.
///
/// Corresponds to Python `HyperbrowserLoadTool` in `crewai_tools`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HyperbrowserLoadTool {
    /// Hyperbrowser API key.
    pub api_key: Option<String>,
    /// Session timeout in seconds.
    pub timeout: u64,
}

impl HyperbrowserLoadTool {
    pub fn new() -> Self {
        Self {
            api_key: None,
            timeout: 30,
        }
    }

    pub fn with_api_key(mut self, key: impl Into<String>) -> Self {
        self.api_key = Some(key.into());
        self
    }

    pub fn with_timeout(mut self, timeout: u64) -> Self {
        self.timeout = timeout;
        self
    }

    pub fn run(&self, _args: HashMap<String, Value>) -> Result<Value, anyhow::Error> {
        anyhow::bail!(
            "HyperbrowserLoadTool: not yet implemented - requires Hyperbrowser API integration"
        )
    }
}

impl Default for HyperbrowserLoadTool {
    fn default() -> Self {
        Self::new()
    }
}

// ── StagehandTool ────────────────────────────────────────────────────────────

/// AI-powered browser automation using the Stagehand framework.
///
/// Corresponds to Python `StagehandTool` in `crewai_tools`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StagehandTool {
    /// Stagehand API key or configuration.
    pub api_key: Option<String>,
    /// Whether to run in headless mode.
    pub headless: bool,
}

impl StagehandTool {
    pub fn new() -> Self {
        Self {
            api_key: None,
            headless: true,
        }
    }

    pub fn with_api_key(mut self, key: impl Into<String>) -> Self {
        self.api_key = Some(key.into());
        self
    }

    pub fn with_headless(mut self, headless: bool) -> Self {
        self.headless = headless;
        self
    }

    pub fn run(&self, _args: HashMap<String, Value>) -> Result<Value, anyhow::Error> {
        anyhow::bail!(
            "StagehandTool: not yet implemented - requires Stagehand framework integration"
        )
    }
}

impl Default for StagehandTool {
    fn default() -> Self {
        Self::new()
    }
}

// ── MultiOnTool ──────────────────────────────────────────────────────────────

/// Autonomous web browsing and task completion using the MultiOn API.
///
/// Corresponds to Python `MultiOnTool` in `crewai_tools`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MultiOnTool {
    /// MultiOn API key.
    pub api_key: Option<String>,
    /// Whether to run in local mode.
    pub local: bool,
}

impl MultiOnTool {
    pub fn new() -> Self {
        Self {
            api_key: None,
            local: false,
        }
    }

    pub fn with_api_key(mut self, key: impl Into<String>) -> Self {
        self.api_key = Some(key.into());
        self
    }

    pub fn with_local(mut self, local: bool) -> Self {
        self.local = local;
        self
    }

    pub fn run(&self, _args: HashMap<String, Value>) -> Result<Value, anyhow::Error> {
        anyhow::bail!("MultiOnTool: not yet implemented - requires MultiOn API integration")
    }
}

impl Default for MultiOnTool {
    fn default() -> Self {
        Self::new()
    }
}
