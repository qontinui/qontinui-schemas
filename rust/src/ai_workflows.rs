//! AI workflow template DTOs.
//!
//! Wire-format types for saved AI Builder workflows — the pre-authored workflow
//! configurations that users create, organize, and run from the Automation
//! Builder tab. These are **template types**, distinct from `UnifiedWorkflow`
//! which is the runtime execution format.
//!
//! Extracted from `qontinui-runner/src-tauri/src/ai_workflows.rs`. Runtime
//! behaviour (file-system persistence, CRUD helpers, search/filter functions)
//! stays in the runner. This module is data-only.
//!
//! ## Wire-format notes
//!
//! - Field names use camelCase renames to match the existing persisted JSON
//!   contract (`ai_workflows.json`) and the TypeScript frontend.
//! - `ExecutionStep` here is the **legacy step shape** from the AI Builder.
//!   It is intentionally separate from `ExecutionStepConfig` in
//!   `step_executor/executor_types.rs` and from `CanonicalStep` /
//!   `FullRunnerStep` in `workflow_step`.
//! - Dates/times are ISO 8601 strings (see crate-level docs).

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

// ============================================================================
// Helper — skip_serializing_if predicates
// ============================================================================

fn is_false(v: &bool) -> bool {
    !(*v)
}

fn is_empty_string(s: &str) -> bool {
    s.is_empty()
}

// ============================================================================
// ExecutionStep — a single step in a saved AI Builder sequence
// ============================================================================

/// A single step in an AI Builder execution sequence.
///
/// This is the **legacy step shape** persisted in `ai_workflows.json`. It
/// carries fields for every step flavour (`workflow`, `state`, `playwright`,
/// `prompt`, `action`, `screenshot`) as optional branches. Steps that don't
/// apply to the given `step_type` have their branch fields set to `None`.
#[derive(Debug, Clone, Default, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
#[schemars(deny_unknown_fields)]
pub struct ExecutionStep {
    /// Unique identifier for this step.
    #[serde(alias = "id")]
    pub id: String,

    /// Type of step: `"workflow"`, `"state"`, `"playwright"`, `"prompt"`,
    /// `"action"`, or `"screenshot"`.
    #[serde(rename = "type", alias = "step_type")]
    pub step_type: String,

    /// Display name for the step.
    #[serde(alias = "name")]
    pub name: String,

    /// Whether to capture a screenshot after this step.
    #[serde(default, skip_serializing_if = "is_false", alias = "take_screenshot")]
    pub take_screenshot: bool,

    /// Delay in seconds before taking screenshot (default 0).
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        alias = "screenshot_delay"
    )]
    pub screenshot_delay: Option<f64>,

    /// For playwright steps: the script ID.
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        alias = "playwright_script_id"
    )]
    pub playwright_script_id: Option<String>,

    /// For playwright steps: the script content.
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        alias = "playwright_script_content"
    )]
    pub playwright_script_content: Option<String>,

    /// For playwright steps: the target URL.
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        alias = "playwright_target_url"
    )]
    pub playwright_target_url: Option<String>,

    /// For prompt steps: the prompt ID from the library.
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        alias = "prompt_id"
    )]
    pub prompt_id: Option<String>,

    /// For prompt steps: the actual prompt content.
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        alias = "prompt_content"
    )]
    pub prompt_content: Option<String>,

    /// For action steps: the action type (`"click"`, `"double_click"`,
    /// `"right_click"`).
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        alias = "action_type"
    )]
    pub action_type: Option<String>,

    /// For action steps: the target image ID.
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        alias = "target_image_id"
    )]
    pub target_image_id: Option<String>,

    /// For action steps: the target image name (for display).
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        alias = "target_image_name"
    )]
    pub target_image_name: Option<String>,

    /// Monitor for screenshot capture (number for specific monitor, `"all"`
    /// for all monitors).
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        alias = "screenshot_monitor"
    )]
    pub screenshot_monitor: Option<serde_json::Value>,
}

// ============================================================================
// AiWorkflow — a saved AI Builder workflow template
// ============================================================================

/// Default value for `auto_include_contexts` — `true`.
fn default_auto_include_contexts() -> bool {
    true
}

/// A saved AI Builder workflow template.
///
/// Users create these in the Automation Builder tab. Each workflow carries an
/// ordered list of `ExecutionStep`s plus metadata (goal, category, tags,
/// context references). The runner persists them in `ai_workflows.json` and
/// surfaces them through Tauri commands and the HTTP API.
#[derive(Debug, Clone, Default, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
#[schemars(deny_unknown_fields)]
pub struct AiWorkflow {
    /// Unique identifier (UUID v4).
    #[serde(alias = "id")]
    pub id: String,

    /// Display name for the workflow.
    #[serde(alias = "name")]
    pub name: String,

    /// Optional description of what this workflow does.
    #[serde(default, skip_serializing_if = "is_empty_string", alias = "description")]
    pub description: String,

    /// The ordered list of execution steps.
    #[serde(default, skip_serializing_if = "Vec::is_empty", alias = "steps")]
    pub steps: Vec<ExecutionStep>,

    /// The goal/objective for this workflow.
    #[serde(default, skip_serializing_if = "is_empty_string", alias = "goal")]
    pub goal: String,

    /// Maximum iterations for the AI loop.
    /// `None` (omitted) means no cap — loop until success or explicit stop.
    #[serde(default, skip_serializing_if = "Option::is_none", alias = "max_iterations")]
    pub max_iterations: Option<u32>,

    /// Whether to capture input for coordinate validation.
    #[serde(default, skip_serializing_if = "is_false", alias = "capture_input_validation")]
    pub capture_input_validation: bool,

    /// Category for organization (e.g., `"Testing"`, `"Development"`).
    #[serde(default, skip_serializing_if = "is_empty_string", alias = "category")]
    pub category: String,

    /// Tags for filtering/searching.
    #[serde(default, skip_serializing_if = "Vec::is_empty", alias = "tags")]
    pub tags: Vec<String>,

    /// Manually added context IDs.
    #[serde(default, skip_serializing_if = "Vec::is_empty", alias = "context_ids")]
    pub context_ids: Vec<String>,

    /// Disabled context IDs (excluded from auto-include).
    #[serde(default, skip_serializing_if = "Vec::is_empty", alias = "disabled_context_ids")]
    pub disabled_context_ids: Vec<String>,

    /// Whether to auto-include contexts based on task mentions (default: true).
    #[serde(default = "default_auto_include_contexts", alias = "auto_include_contexts")]
    pub auto_include_contexts: bool,

    /// ISO 8601 timestamp of creation.
    #[serde(alias = "created_at")]
    pub created_at: String,

    /// ISO 8601 timestamp of last modification.
    #[serde(alias = "modified_at")]
    pub modified_at: String,
}
