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
pub struct WorkerOutput {
    /// Summary of work performed in this iteration.
    pub work_summary: String,
    /// Signals for orchestrator control flow.
    #[serde(default)]
    pub signals: Vec<StructuredSignal>,
    /// Findings discovered during work.
    #[serde(default)]
    pub findings: Vec<StructuredFinding>,
    /// Files that were modified in this iteration.
    #[serde(default)]
    pub files_modified: Vec<String>,
    /// Criterion overrides with justifications.
    #[serde(default)]
    pub criterion_overrides: Vec<StructuredOverride>,
    /// Confidence level in the work quality.
    #[serde(default)]
    pub confidence: ConfidenceLevel,
    /// Optional suggestion for next action if work continues.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub next_action_suggestion: Option<String>,
    /// Optional progress estimate (0.0 to 1.0).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub progress_estimate: Option<f32>,
    /// Optional notes for debugging or context.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub notes: Option<String>,
}

/// A signal from the worker to the orchestrator.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct StructuredSignal {
    /// Signal type (e.g., "complete", "blocked", "needs_input").
    pub signal_type: String,
    /// Optional message providing context for the signal.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
}

/// A finding discovered during work.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct StructuredFinding {
    /// Finding category (e.g., "bug", "security", "performance").
    pub category: String,
    /// Severity level.
    pub severity: String,
    /// Short title describing the finding.
    pub title: String,
    /// Detailed description.
    #[serde(default)]
    pub description: String,
    /// Whether this finding requires human input.
    #[serde(default)]
    pub needs_input: bool,
}

/// A criterion override with justification.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct StructuredOverride {
    /// The criterion being overridden.
    pub criterion: String,
    /// The new status.
    pub status: String,
    /// Justification for the override.
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
pub struct FileChange {
    /// Path of the changed file.
    pub path: String,
    /// Action performed on the file (e.g., "modified", "created", "deleted").
    pub action: String,
}

/// A finding reported by the AI.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct FindingOutput {
    /// Finding category.
    pub category: String,
    /// Severity level.
    pub severity: String,
    /// Short title describing the finding.
    pub title: String,
    /// Detailed description.
    #[serde(default)]
    pub description: String,
    /// Whether this finding requires human input.
    #[serde(default)]
    pub needs_input: bool,
    /// Whether this finding has been resolved.
    #[serde(default)]
    pub resolved: bool,
}

/// A reflection fix reported by the AI.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ReflectionFixOutput {
    /// Identifier for the error being fixed.
    pub error_id: String,
    /// Description of the fix applied.
    pub description: String,
}

/// Canonical structured output from the agentic phase.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct AgenticPhaseOutput {
    /// Overall status of the agentic phase.
    pub status: AgenticStatus,
    /// Human-readable summary of what was done.
    pub summary: String,
    /// AI's confidence that the fixes will pass verification (0.0-1.0).
    #[serde(default)]
    pub confidence: Option<f64>,
    /// Whether the AI determined errors are unfixable.
    #[serde(default)]
    pub unfixable: bool,
    /// Reason why errors are unfixable (if unfixable is true).
    #[serde(default)]
    pub unfixable_reason: Option<String>,
    /// Files modified during this agentic phase.
    #[serde(default)]
    pub files_modified: Vec<FileChange>,
    /// Dynamically injected verification steps.
    #[serde(default)]
    pub injected_steps: Vec<Value>,
    /// Reflection fixes (when reflection_mode is enabled).
    #[serde(default)]
    pub reflection_fixes: Vec<ReflectionFixOutput>,
    /// Findings reported by the AI.
    #[serde(default)]
    pub findings: Vec<FindingOutput>,
}
