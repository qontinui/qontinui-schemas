//! Findings models — AI-analysis result payloads.
//!
//! Rust is the source of truth. Ported from
//! `src/qontinui_schemas/findings/models.py` + `enums.py`. TS and Python
//! bindings regenerate from the JSON Schemas emitted here.
//!
//! A *finding* represents an issue, observation, or recommendation produced
//! by an AI analysis session. Findings flow Runner → Backend (create/update)
//! and Backend → Frontend (detail/list/summary).
//!
//! Wire-format notes:
//! - UUIDs are serialized as plain strings (see crate-level docs).
//! - Dates are ISO 8601 strings (no `chrono` dependency).
//! - Enum string values are lowercase `snake_case` to match the Python
//!   `str | Enum` base classes in `enums.py`.

use std::collections::HashMap;

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

// ============================================================================
// Enums (ported from findings/enums.py)
// ============================================================================

/// Category of a detected finding.
///
/// Determines the kind of issue or observation surfaced during analysis.
/// Mirrors Python `FindingCategory(str, Enum)`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum FindingCategory {
    /// A bug in application code.
    CodeBug,
    /// A security concern (vulnerability, exposed secret, etc.).
    Security,
    /// A performance concern.
    Performance,
    /// A TODO or open task.
    Todo,
    /// A proposed enhancement.
    Enhancement,
    /// A configuration issue.
    ConfigIssue,
    /// A test-related issue.
    TestIssue,
    /// A documentation issue.
    Documentation,
    /// A runtime issue observed during execution.
    RuntimeIssue,
    /// An issue that was already fixed.
    AlreadyFixed,
    /// Behavior that looked suspicious but is expected.
    ExpectedBehavior,
}

/// Severity level of a finding.
///
/// Lifecycle (ordered, most-severe first): `CRITICAL` → `HIGH` → `MEDIUM` →
/// `LOW` → `INFO`. Mirrors Python `FindingSeverity(str, Enum)`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum FindingSeverity {
    /// Blocks functionality — immediate attention required.
    Critical,
    /// Major issue — should be fixed soon.
    High,
    /// Moderate issue — should be fixed.
    Medium,
    /// Minor issue — fix when convenient.
    Low,
    /// Informational — no action required.
    Info,
}

/// Status of a finding.
///
/// Lifecycle: `DETECTED` → `IN_PROGRESS` → (`RESOLVED` | `WONT_FIX` |
/// `DEFERRED`). `NEEDS_INPUT` is a special state requiring user decision.
/// Mirrors Python `FindingStatus(str, Enum)`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum FindingStatus {
    /// Newly detected.
    Detected,
    /// Being worked on.
    InProgress,
    /// Needs user input before it can proceed.
    NeedsInput,
    /// Resolved.
    Resolved,
    /// Acknowledged but won't be fixed.
    WontFix,
    /// Deferred to a later time.
    Deferred,
}

/// Type of action recommended for a finding.
///
/// Mirrors Python `FindingActionType(str, Enum)`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum FindingActionType {
    /// Can be automatically fixed without user intervention.
    AutoFix,
    /// Requires user decision or input to resolve.
    NeedsUserInput,
    /// No action needed — for awareness only.
    Informational,
}

// ============================================================================
// Supporting structs
// ============================================================================

/// Code context for a finding.
///
/// Provides location (file/line/column) and an optional snippet for findings
/// that relate to specific code.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct FindingCodeContext {
    /// File path where the finding was detected.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub file: Option<String>,
    /// Line number where the finding was detected.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub line: Option<i64>,
    /// Column number where the finding was detected.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub column: Option<i64>,
    /// Code snippet related to the finding (max 1000 chars on the Python side).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub snippet: Option<String>,
}

/// User-input request attached to a finding.
///
/// Defines the question to pose and the expected input format when a finding
/// requires a user decision.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct FindingUserInput {
    /// Question to present to the user.
    pub question: String,
    /// Type of input expected — typically `"text"` or `"choice"`.
    #[serde(default = "default_input_type")]
    pub input_type: String,
    /// Options for choice-type input.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub options: Option<Vec<String>>,
}

fn default_input_type() -> String {
    "text".to_string()
}

// ============================================================================
// Create / Update / Detail payloads
// ============================================================================

/// Schema for creating a finding (Runner → Backend).
///
/// Sent by the runner when an AI analysis session detects an issue or
/// observation.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct FindingCreate {
    /// Parent task run ID.
    pub task_run_id: String,
    /// Session number where the finding was detected.
    pub session_num: i64,
    /// Category of the finding.
    pub category: FindingCategory,
    /// Severity level of the finding.
    pub severity: FindingSeverity,
    /// Brief title describing the finding (max 500 chars on the Python side).
    pub title: String,
    /// Detailed description of the finding.
    pub description: String,
    /// Code context, if the finding relates to specific code.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub code_context: Option<FindingCodeContext>,
    /// Hash used to deduplicate findings across sessions.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub signature_hash: Option<String>,
    /// Type of action for this finding.
    pub action_type: FindingActionType,
    /// User-input request, if `action_type` requires a user decision.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub user_input: Option<FindingUserInput>,
}

/// Request schema for batch finding creation.
///
/// Allows creating multiple findings in a single request. The Python side
/// enforces `1 <= len(findings) <= 50`; validators on the Rust side are
/// intentionally omitted to keep this a pure wire-format layer.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct FindingBatchCreate {
    /// Findings to create (1–50 items on the Python side).
    pub findings: Vec<FindingCreate>,
}

/// Schema for updating a finding.
///
/// Used to update status, record a resolution, or capture a user response.
/// All fields are optional; only those supplied are applied.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct FindingUpdate {
    /// New status for the finding.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub status: Option<FindingStatus>,
    /// Resolution description.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub resolution: Option<String>,
    /// Session number where the finding was resolved.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub resolved_in_session: Option<i64>,
    /// User's response to a finding requiring input.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub user_response: Option<String>,
}

/// Detailed finding information (Backend → Frontend).
///
/// Used when retrieving individual finding details. The `id` is a UUID v4
/// string (see crate-level wire-format note).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct FindingDetail {
    /// Finding ID (UUID v4 string).
    pub id: String,
    /// Parent task run ID.
    pub task_run_id: String,
    /// Session number where the finding was detected.
    pub session_num: i64,
    /// Category of the finding.
    pub category: FindingCategory,
    /// Severity level of the finding.
    pub severity: FindingSeverity,
    /// Current status of the finding.
    pub status: FindingStatus,
    /// Brief title describing the finding.
    pub title: String,
    /// Detailed description of the finding.
    pub description: String,
    /// Resolution description if resolved.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub resolution: Option<String>,
    /// Code context, if the finding relates to specific code.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub code_context: Option<FindingCodeContext>,
    /// Hash used to deduplicate findings across sessions.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub signature_hash: Option<String>,
    /// Type of action for this finding.
    pub action_type: FindingActionType,
    /// User-input request, if `action_type` requires a user decision.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub user_input: Option<FindingUserInput>,
    /// User's response, if input was requested.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub user_response: Option<String>,
    /// ISO 8601 timestamp (UTC) when the finding was detected.
    pub detected_at: String,
    /// ISO 8601 timestamp (UTC) when the finding was resolved.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub resolved_at: Option<String>,
    /// Session number where the finding was resolved.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub resolved_in_session: Option<i64>,
}

/// Response schema for a paginated finding list.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct FindingListResponse {
    /// Findings on this page.
    #[serde(default)]
    pub findings: Vec<FindingDetail>,
    /// Total count of findings matching the query.
    pub total: i64,
    /// Maximum items per page.
    pub limit: i64,
    /// Number of items skipped.
    pub offset: i64,
    /// Whether more items exist beyond this page.
    pub has_more: bool,
}

/// Summary statistics for findings in a task run.
///
/// Aggregated counts grouped along each axis (category, severity, status)
/// plus roll-up counts for UI dashboards.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct FindingSummary {
    /// Task run ID.
    pub task_run_id: String,
    /// Total number of findings.
    #[serde(default)]
    pub total: i64,
    /// Count of findings by category.
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub by_category: HashMap<String, i64>,
    /// Count of findings by severity.
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub by_severity: HashMap<String, i64>,
    /// Count of findings by status.
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub by_status: HashMap<String, i64>,
    /// Number of findings awaiting user input.
    #[serde(default)]
    pub needs_input_count: i64,
    /// Number of resolved findings.
    #[serde(default)]
    pub resolved_count: i64,
    /// Number of unresolved findings.
    #[serde(default)]
    pub outstanding_count: i64,
}
