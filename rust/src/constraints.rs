//! Constraint engine DTO types.
//!
//! These types mirror the runtime constraint engine in
//! `qontinui-runner/src-tauri/src/constraint_engine/` and the TypeScript
//! counterpart in `qontinui-schemas/ts/src/constraints/`. Rust is the source of
//! truth; JSON Schema emitted from these types drives TS and Python bindings.
//!
//! This module is wire-format only: no business logic, no impl blocks beyond
//! `Default`, no tests. Dates, times, and UUIDs are represented as `String`.

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

// ============================================================================
// Severity
// ============================================================================

/// Severity of a constraint violation.
///
/// - `Block`: Reject the fix, inject violation context, re-run agentic phase
///   without consuming an iteration. After `max_retries`, consume the iteration.
/// - `Warn`: Apply the fix, but inject violation context for the next iteration.
/// - `Log`: Record only, don't affect execution.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ConstraintSeverity {
    /// Reject the fix and re-run the agentic phase with injected violation
    /// context. After `max_retries`, the iteration is consumed.
    Block,
    /// Apply the fix, but inject violation context for the next iteration.
    Warn,
    /// Record the violation only; do not affect execution.
    Log,
}

// ============================================================================
// Check Types (internally tagged discriminated union)
// ============================================================================

/// What to check and how.
///
/// Internally tagged by the `type` field. Variants correspond to the four
/// check kinds implemented by the runner's constraint engine.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ConstraintCheck {
    /// Grep modified files for a regex pattern.
    /// Violation if the pattern IS found (use for secrets, debug statements, etc.)
    GrepForbidden {
        /// Regex pattern to search for.
        pattern: String,
        /// Optional glob to limit which modified files are checked.
        /// Default: all modified files.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        file_glob: Option<String>,
    },

    /// Grep modified files for a regex pattern.
    /// Violation if the pattern is NOT found (use for required headers, licenses, etc.)
    GrepRequired {
        /// Regex pattern that must be present.
        pattern: String,
        /// Optional glob to limit which modified files are checked.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        file_glob: Option<String>,
    },

    /// Check that all modified files are within allowed paths.
    /// Violation if any modified file is outside the allowed directories.
    FileScope {
        /// Allowed directory prefixes (relative to project root).
        /// e.g., `["src/", "tests/", "config/"]`.
        #[serde(default)]
        allowed_paths: Vec<String>,
    },

    /// Run a shell command. Violation if exit code is non-zero.
    /// Useful for quick compilation checks, linting, etc.
    Command {
        /// The command to run.
        cmd: String,
        /// Working directory (relative to project root). Default: project root.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        cwd: Option<String>,
        /// Timeout in seconds. Default: 30.
        #[serde(default = "default_timeout_secs")]
        timeout_secs: u64,
    },
}

/// Default timeout for `ConstraintCheck::Command` (30 seconds).
fn default_timeout_secs() -> u64 {
    30
}

// ============================================================================
// Core Constraint Definition
// ============================================================================

/// A constraint definition.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct Constraint {
    /// Unique identifier (e.g., `"builtin:no-secrets"`, `"project:no-todos"`).
    pub id: String,
    /// Human-readable name.
    pub name: String,
    /// Why this constraint exists (shown to the AI on violation).
    pub description: String,
    /// What to check.
    pub check: ConstraintCheck,
    /// How severe a violation is.
    pub severity: ConstraintSeverity,
    /// Whether this constraint is enabled. Default: true.
    #[serde(default = "default_true")]
    pub enabled: bool,
}

/// Default for `Constraint::enabled` — a constraint is enabled unless explicitly disabled.
fn default_true() -> bool {
    true
}

// ============================================================================
// Evaluation Results
// ============================================================================

/// A specific violation found during constraint evaluation.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ConstraintViolation {
    /// File where the violation was found (if applicable).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub file: Option<String>,
    /// Line number (if applicable).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub line: Option<u32>,
    /// What was found / what went wrong.
    pub detail: String,
}

/// Result of evaluating a single constraint.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ConstraintResult {
    /// The id of the constraint that was evaluated.
    pub constraint_id: String,
    /// The human-readable name of the constraint that was evaluated.
    pub constraint_name: String,
    /// Whether the constraint passed.
    pub passed: bool,
    /// Severity of the constraint (for quick filtering).
    pub severity: ConstraintSeverity,
    /// Details about the violation (empty if passed).
    #[serde(default)]
    pub violations: Vec<ConstraintViolation>,
}

// ============================================================================
// Resource Limits
// ============================================================================

/// Resource limits for workflow execution.
///
/// When a limit is approached (within the warning threshold), the tracker
/// emits context injection actions. When exceeded, it emits stronger actions.
#[derive(Debug, Clone, Default, Serialize, Deserialize, JsonSchema)]
pub struct ResourceLimits {
    /// Maximum wall-clock time for the entire workflow (seconds).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max_wall_time_secs: Option<u64>,
    /// Maximum number of unique files modified across all iterations.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max_files_modified: Option<u64>,
    /// Maximum agentic phase durations summed (milliseconds).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max_agentic_time_ms: Option<u64>,
    /// Warning threshold as a fraction (0.0-1.0). When resource usage exceeds
    /// this fraction of the limit, a warning is injected.
    /// Default: 0.75 (warn at 75% of limit).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub warning_threshold: Option<f64>,
}

// ============================================================================
// AI Constraint Proposals
// ============================================================================

/// A new constraint proposed by the AI during an agentic phase.
///
/// Serialized with `"type": "new_constraint"` via the `ConstraintProposal`
/// enum's internal tag.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct NewConstraintProposal {
    /// The proposed constraint definition.
    pub constraint: Constraint,
}

/// A proposal to enable or disable a builtin constraint.
///
/// Serialized with `"type": "builtin_override"` via the `ConstraintProposal`
/// enum's internal tag.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct BuiltinOverrideProposal {
    /// Builtin suffix (e.g., `"no-secrets"`, `"no-debug-statements"`).
    pub builtin_suffix: String,
    /// Whether the builtin should be enabled.
    pub enabled: bool,
    /// Human-readable justification for the override.
    pub reason: String,
}

/// A constraint proposal from the AI.
///
/// Internally tagged by `type` so the on-the-wire shape matches the TypeScript
/// discriminated union `{ type: "new_constraint" | "builtin_override", ... }`.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ConstraintProposal {
    /// The AI proposes a new project-level constraint.
    NewConstraint(NewConstraintProposal),
    /// The AI proposes to enable or disable a builtin constraint.
    BuiltinOverride(BuiltinOverrideProposal),
}

// ============================================================================
// API Request / Response Types
// ============================================================================

/// Request body for `POST /constraints/validate`.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ValidateConfigRequest {
    /// Raw TOML content to validate.
    pub toml: String,
}

/// Response for `POST /constraints/validate`.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ValidateConfigResponse {
    /// Whether the config is fully valid (parseable with no errors or warnings).
    pub valid: bool,
    /// Parse errors or non-fatal warnings (e.g., constraints skipped due to bad regex).
    #[serde(default)]
    pub errors: Vec<String>,
    /// Successfully parsed constraints (may be partial if some were skipped).
    #[serde(default)]
    pub constraints: Vec<Constraint>,
}

/// Response for `GET /constraints/config`.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ReadConfigResponse {
    /// Raw TOML content of the `constraints.toml` file (empty string if not found).
    pub toml: String,
    /// Resolved file path, if a config file was found.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub path: Option<String>,
}

/// Request body for `POST /constraints/config`.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct WriteConfigRequest {
    /// Project path for the `constraints.toml`. Defaults to workspace root.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub project_path: Option<String>,
    /// Raw TOML content to validate and write.
    pub toml: String,
}

/// Response for `POST /constraints/config`.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct WriteConfigResponse {
    /// Whether the config is fully valid (parseable with no errors or warnings).
    pub valid: bool,
    /// Parse errors or non-fatal warnings.
    #[serde(default)]
    pub errors: Vec<String>,
    /// The file path that was written to.
    pub path: String,
}
