//! Structured AI worker output types.
//!
//! These types define the structured output contracts for AI worker agents and
//! agentic phase execution. They are used for JSON Schema generation and
//! cross-language type bindings (TypeScript, Python).
//!
//! - `WorkerOutput` and its supporting types (`StructuredSignal`,
//!   `StructuredFinding`, `StructuredOverride`, `ConfidenceLevel`) model the
//!   per-iteration output from an AI worker agent.
//! - `AgenticPhaseOutput` and its supporting types (`AgenticStatus`,
//!   `FileChange`, `FindingOutput`, `ReflectionFixOutput`) model the canonical
//!   output from the agentic phase of a unified workflow execution.

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::Value;

// ── WorkerOutput (mirrors orchestrator::structured_output::WorkerOutput) ──

/// Structured output from an AI worker agent.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
#[schemars(deny_unknown_fields)]
pub struct WorkerOutput {
    /// Summary of work performed in this iteration.
    #[serde(alias = "work_summary")]
    pub work_summary: String,
    /// Signals for orchestrator control flow.
    #[serde(default, alias = "signals")]
    pub signals: Vec<StructuredSignal>,
    /// Findings discovered during work.
    #[serde(default, alias = "findings")]
    pub findings: Vec<StructuredFinding>,
    /// Files that were modified in this iteration.
    #[serde(default, alias = "files_modified")]
    pub files_modified: Vec<String>,
    /// Criterion overrides with justifications.
    #[serde(default, alias = "criterion_overrides")]
    pub criterion_overrides: Vec<StructuredOverride>,
    /// Confidence level in the work quality.
    #[serde(default, alias = "confidence")]
    pub confidence: ConfidenceLevel,
    /// Optional suggestion for next action if work continues.
    #[serde(skip_serializing_if = "Option::is_none", alias = "next_action_suggestion")]
    pub next_action_suggestion: Option<String>,
    /// Optional progress estimate (0.0 to 1.0).
    #[serde(skip_serializing_if = "Option::is_none", alias = "progress_estimate")]
    pub progress_estimate: Option<f32>,
    /// Optional notes for debugging or context.
    #[serde(skip_serializing_if = "Option::is_none", alias = "notes")]
    pub notes: Option<String>,
}

/// A signal from the worker to the orchestrator.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
#[schemars(deny_unknown_fields)]
pub struct StructuredSignal {
    /// Signal type (e.g., "complete", "blocked", "needs_input").
    #[serde(alias = "signal_type")]
    pub signal_type: String,
    /// Optional message providing context for the signal.
    #[serde(skip_serializing_if = "Option::is_none", alias = "message")]
    pub message: Option<String>,
}

/// A finding discovered during work.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
#[schemars(deny_unknown_fields)]
pub struct StructuredFinding {
    /// Finding category (e.g., "bug", "security", "performance").
    #[serde(alias = "category")]
    pub category: String,
    /// Severity level.
    #[serde(alias = "severity")]
    pub severity: String,
    /// Short title describing the finding.
    #[serde(alias = "title")]
    pub title: String,
    /// Detailed description.
    #[serde(default, alias = "description")]
    pub description: String,
    /// Whether this finding requires human input.
    #[serde(default, alias = "needs_input")]
    pub needs_input: bool,
}

/// A criterion override with justification.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
#[schemars(deny_unknown_fields)]
pub struct StructuredOverride {
    /// The criterion being overridden.
    #[serde(alias = "criterion")]
    pub criterion: String,
    /// The new status.
    #[serde(alias = "status")]
    pub status: String,
    /// Justification for the override.
    #[serde(alias = "justification")]
    pub justification: String,
}

/// Confidence level enum.
#[derive(Debug, Clone, Default, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "lowercase")]
pub enum ConfidenceLevel {
    /// High confidence in work quality.
    High,
    /// Medium confidence in work quality.
    #[default]
    Medium,
    /// Low confidence in work quality.
    Low,
}

// ── AgenticPhaseOutput (mirrors unified_workflow_executor::agentic_output) ──

/// Status of the agentic phase execution.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum AgenticStatus {
    /// Agentic phase completed successfully.
    Success,
    /// Agentic phase partially succeeded.
    PartialSuccess,
    /// Agentic phase failed.
    Failed,
    /// Errors determined to be unfixable.
    Unfixable,
}

/// A file change made during the agentic phase.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
#[schemars(deny_unknown_fields)]
pub struct FileChange {
    /// Path of the changed file.
    #[serde(alias = "path")]
    pub path: String,
    /// Action performed on the file (e.g., "modified", "created", "deleted").
    #[serde(alias = "action")]
    pub action: String,
}

/// A finding reported by the AI.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
#[schemars(deny_unknown_fields)]
pub struct FindingOutput {
    /// Finding category.
    #[serde(alias = "category")]
    pub category: String,
    /// Severity level.
    #[serde(alias = "severity")]
    pub severity: String,
    /// Short title describing the finding.
    #[serde(alias = "title")]
    pub title: String,
    /// Detailed description.
    #[serde(default, alias = "description")]
    pub description: String,
    /// Whether this finding requires human input.
    #[serde(default, alias = "needs_input")]
    pub needs_input: bool,
    /// Whether this finding has been resolved.
    #[serde(default, alias = "resolved")]
    pub resolved: bool,
}

/// A reflection fix reported by the AI.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
#[schemars(deny_unknown_fields)]
pub struct ReflectionFixOutput {
    /// Identifier for the error being fixed.
    #[serde(alias = "error_id")]
    pub error_id: String,
    /// Description of the fix applied.
    #[serde(alias = "description")]
    pub description: String,
}

/// Canonical structured output from the agentic phase.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
#[schemars(deny_unknown_fields)]
pub struct AgenticPhaseOutput {
    /// Overall status of the agentic phase.
    #[serde(alias = "status")]
    pub status: AgenticStatus,
    /// Human-readable summary of what was done.
    #[serde(alias = "summary")]
    pub summary: String,
    /// AI's confidence that the fixes will pass verification (0.0-1.0).
    #[serde(default, alias = "confidence")]
    pub confidence: Option<f64>,
    /// Whether the AI determined errors are unfixable.
    #[serde(default, alias = "unfixable")]
    pub unfixable: bool,
    /// Reason why errors are unfixable (if unfixable is true).
    #[serde(default, alias = "unfixable_reason")]
    pub unfixable_reason: Option<String>,
    /// Files modified during this agentic phase.
    #[serde(default, alias = "files_modified")]
    pub files_modified: Vec<FileChange>,
    /// Dynamically injected verification steps.
    #[serde(default, alias = "injected_steps")]
    pub injected_steps: Vec<Value>,
    /// Reflection fixes (when reflection_mode is enabled).
    #[serde(default, alias = "reflection_fixes")]
    pub reflection_fixes: Vec<ReflectionFixOutput>,
    /// Findings reported by the AI.
    #[serde(default, alias = "findings")]
    pub findings: Vec<FindingOutput>,
}
