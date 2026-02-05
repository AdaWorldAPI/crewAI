//! Automation tools for crewAI.
//!
//! This module contains tools for workflow automation, third-party service
//! integrations, and crew orchestration helpers. Each struct corresponds to
//! a Python tool class in `crewai_tools`.

use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use serde_json::Value;

// ── ComposioTool ─────────────────────────────────────────────────────────────

/// Execute actions through the Composio platform (400+ app integrations).
///
/// Corresponds to Python `ComposioTool` in `crewai_tools`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComposioTool {
    /// Composio API key.
    pub api_key: Option<String>,
    /// Specific action to execute.
    pub action: Option<String>,
    /// App name for the action.
    pub app_name: Option<String>,
}

impl ComposioTool {
    pub fn new() -> Self {
        Self {
            api_key: None,
            action: None,
            app_name: None,
        }
    }

    pub fn with_api_key(mut self, key: impl Into<String>) -> Self {
        self.api_key = Some(key.into());
        self
    }

    pub fn with_action(mut self, action: impl Into<String>) -> Self {
        self.action = Some(action.into());
        self
    }

    pub fn with_app_name(mut self, app: impl Into<String>) -> Self {
        self.app_name = Some(app.into());
        self
    }

    pub fn run(&self, _args: HashMap<String, Value>) -> Result<Value, anyhow::Error> {
        anyhow::bail!("ComposioTool: not yet implemented - requires Composio API integration")
    }
}

impl Default for ComposioTool {
    fn default() -> Self {
        Self::new()
    }
}

// ── ApifyActorsTool ──────────────────────────────────────────────────────────

/// Run Apify actors (web scraping, automation, data processing).
///
/// Corresponds to Python `ApifyActorsTool` in `crewai_tools`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApifyActorsTool {
    /// Apify API token.
    pub api_token: Option<String>,
    /// Actor ID to run (e.g., "apify/web-scraper").
    pub actor_id: Option<String>,
}

impl ApifyActorsTool {
    pub fn new() -> Self {
        Self {
            api_token: None,
            actor_id: None,
        }
    }

    pub fn with_api_token(mut self, token: impl Into<String>) -> Self {
        self.api_token = Some(token.into());
        self
    }

    pub fn with_actor_id(mut self, id: impl Into<String>) -> Self {
        self.actor_id = Some(id.into());
        self
    }

    pub fn run(&self, _args: HashMap<String, Value>) -> Result<Value, anyhow::Error> {
        anyhow::bail!("ApifyActorsTool: not yet implemented - requires Apify API integration")
    }
}

impl Default for ApifyActorsTool {
    fn default() -> Self {
        Self::new()
    }
}

// ── ZapierActionTools ────────────────────────────────────────────────────────

/// Execute multiple Zapier actions via the Natural Language Actions API.
///
/// Corresponds to Python `ZapierActionTools` (plural) in `crewai_tools`.
/// This differs from the adapter `ZapierActionTool` (singular) by supporting
/// multi-action orchestration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ZapierActionTools {
    /// Zapier NLA API key.
    pub api_key: Option<String>,
    /// List of allowed action IDs. If empty, all actions are allowed.
    pub allowed_actions: Vec<String>,
}

impl ZapierActionTools {
    pub fn new() -> Self {
        Self {
            api_key: None,
            allowed_actions: Vec::new(),
        }
    }

    pub fn with_api_key(mut self, key: impl Into<String>) -> Self {
        self.api_key = Some(key.into());
        self
    }

    pub fn with_allowed_actions(mut self, actions: Vec<String>) -> Self {
        self.allowed_actions = actions;
        self
    }

    pub fn run(&self, _args: HashMap<String, Value>) -> Result<Value, anyhow::Error> {
        anyhow::bail!(
            "ZapierActionTools: not yet implemented - requires Zapier NLA API integration"
        )
    }
}

impl Default for ZapierActionTools {
    fn default() -> Self {
        Self::new()
    }
}

// ── GenerateCrewaiAutomationTool ─────────────────────────────────────────────

/// Generate crewAI automation configurations (crews, agents, tasks) from
/// natural language descriptions.
///
/// Corresponds to Python `GenerateCrewaiAutomationTool` in `crewai_tools`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenerateCrewaiAutomationTool {
    /// Output format: "yaml", "python", "json".
    pub output_format: String,
}

impl GenerateCrewaiAutomationTool {
    pub fn new() -> Self {
        Self {
            output_format: "yaml".to_string(),
        }
    }

    pub fn with_output_format(mut self, format: impl Into<String>) -> Self {
        self.output_format = format.into();
        self
    }

    pub fn run(&self, _args: HashMap<String, Value>) -> Result<Value, anyhow::Error> {
        anyhow::bail!(
            "GenerateCrewaiAutomationTool: not yet implemented - requires LLM code generation integration"
        )
    }
}

impl Default for GenerateCrewaiAutomationTool {
    fn default() -> Self {
        Self::new()
    }
}

// ── InvokeCrewaiAutomationTool ───────────────────────────────────────────────

/// Invoke a previously generated or defined crewAI automation pipeline.
///
/// Corresponds to Python `InvokeCrewAIAutomationTool` in `crewai_tools`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InvokeCrewaiAutomationTool {
    /// Path to the automation configuration file.
    pub config_path: Option<String>,
}

impl InvokeCrewaiAutomationTool {
    pub fn new() -> Self {
        Self { config_path: None }
    }

    pub fn with_config_path(mut self, path: impl Into<String>) -> Self {
        self.config_path = Some(path.into());
        self
    }

    pub fn run(&self, _args: HashMap<String, Value>) -> Result<Value, anyhow::Error> {
        anyhow::bail!(
            "InvokeCrewaiAutomationTool: not yet implemented - requires crew runtime integration"
        )
    }
}

impl Default for InvokeCrewaiAutomationTool {
    fn default() -> Self {
        Self::new()
    }
}

// ── MergeAgentHandlerTool ────────────────────────────────────────────────────

/// Handle merging of agent outputs and coordinate multi-agent workflows.
///
/// Corresponds to Python `MergeAgentHandlerTool` in `crewai_tools`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MergeAgentHandlerTool {
    /// Merge strategy: "concat", "summarize", "vote".
    pub merge_strategy: String,
}

impl MergeAgentHandlerTool {
    pub fn new() -> Self {
        Self {
            merge_strategy: "concat".to_string(),
        }
    }

    pub fn with_merge_strategy(mut self, strategy: impl Into<String>) -> Self {
        self.merge_strategy = strategy.into();
        self
    }

    pub fn run(&self, _args: HashMap<String, Value>) -> Result<Value, anyhow::Error> {
        anyhow::bail!(
            "MergeAgentHandlerTool: not yet implemented - requires multi-agent coordination"
        )
    }
}

impl Default for MergeAgentHandlerTool {
    fn default() -> Self {
        Self::new()
    }
}
