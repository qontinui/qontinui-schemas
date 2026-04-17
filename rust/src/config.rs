//! Config models — AI context and workflow category.
//!
//! Mirrors `src/qontinui_schemas/config/models/context.py` and the
//! `Category` class from `config_root.py`. Rust is the source of truth; TS
//! and Python bindings regenerate from the JSON Schemas emitted here.
//!
//! Wire convention: these types predate the snake_case migration and use
//! camelCase field aliases on the wire (`taskMentions`, `autoInclude`,
//! `createdAt`, …). Rust code uses snake_case; `#[serde(rename = ...)]`
//! renames each field on the wire to match the existing consumers.

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

// ============================================================================
// ContextAutoInclude
// ============================================================================

/// Rules for automatically including a context in AI tasks.
///
/// When an AI task is created, the runner evaluates these rules to decide
/// which contexts should be auto-included. Multiple rules are OR'd together
/// (any match triggers inclusion).
#[derive(Debug, Clone, Default, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct ContextAutoInclude {
    /// Keywords in the task prompt that trigger inclusion
    /// (case-insensitive).
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        rename = "taskMentions",
        alias = "task_mentions"
    )]
    pub task_mentions: Option<Vec<String>>,
    /// Action types in the loaded config that trigger inclusion
    /// (e.g., `CLICK`, `FIND`).
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        rename = "actionTypes",
        alias = "action_types"
    )]
    pub action_types: Option<Vec<String>>,
    /// Regex patterns in recent logs that trigger inclusion.
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        rename = "errorPatterns",
        alias = "error_patterns"
    )]
    pub error_patterns: Option<Vec<String>>,
    /// Glob patterns for files being worked on (e.g., `*.rs`, `src/api/**`).
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        rename = "filePatterns",
        alias = "file_patterns"
    )]
    pub file_patterns: Option<Vec<String>>,
}

// ============================================================================
// Context
// ============================================================================

/// AI context — a markdown document injected into AI task prompts to
/// provide background knowledge, coding standards, architectural guidance,
/// or debugging tips.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct Context {
    /// Unique identifier (UUID v4 or a prefixed string like
    /// `"ctx-schema-flow"`).
    #[serde(alias = "id")]
    pub id: String,
    /// Human-readable name for display.
    #[serde(alias = "name")]
    pub name: String,
    /// Markdown content injected into AI prompts.
    #[serde(alias = "content")]
    pub content: String,
    /// Category for organization (e.g., `"architecture"`, `"debugging"`,
    /// `"philosophy"`).
    #[serde(default, skip_serializing_if = "Option::is_none", alias = "category")]
    pub category: Option<String>,
    /// Tags for flexible grouping and search.
    #[serde(default, alias = "tags")]
    pub tags: Vec<String>,
    /// Rules for automatic inclusion in AI tasks.
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        rename = "autoInclude",
        alias = "auto_include"
    )]
    pub auto_include: Option<ContextAutoInclude>,
    /// ISO 8601 creation timestamp.
    #[serde(rename = "createdAt", alias = "created_at")]
    pub created_at: String,
    /// ISO 8601 last-modification timestamp.
    #[serde(rename = "modifiedAt", alias = "modified_at")]
    pub modified_at: String,
}

// ============================================================================
// Category
// ============================================================================

/// Workflow category for organization and automation control.
///
/// Categories organize workflows and control which are available for
/// automation in the runner. Only workflows in categories with
/// `automationEnabled = true` appear in the runner's workflow list.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct Category {
    /// Category name (e.g., `"Main"`, `"Testing"`).
    #[serde(alias = "name")]
    pub name: String,
    /// Whether workflows in this category are available for automation.
    #[serde(default = "default_true", rename = "automationEnabled", alias = "automation_enabled")]
    pub automation_enabled: bool,
}

fn default_true() -> bool {
    true
}
