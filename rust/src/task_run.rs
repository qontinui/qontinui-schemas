//! Task-run DTO types.
//!
//! Wire-format types for task runs: local execution records (runner-side),
//! backend/API representations (web-side), session and finding entities,
//! create/update request payloads, filters, and verification result responses.
//!
//! Ported from `qontinui-schemas/ts/src/task-run/_api.ts`. Rust is the
//! source of truth; JSON Schema emitted from these types drives the TS and
//! Python bindings.
//!
//! ## Conventions
//!
//! - Optional TS fields (`?`) map to `Option<T>` with
//!   `#[serde(default, skip_serializing_if = "Option::is_none")]`.
//! - Required-nullable TS fields (`T | null`) map to `Option<T>` with
//!   `#[serde(default)]` but **without** `skip_serializing_if` — the wire
//!   always includes the key (as `null` when absent); serde treats missing
//!   and `null` identically on deserialize.
//! - Timestamps are ISO 8601 `String`s (see crate docs).
//! - Counts and indices are `u32`. Millisecond durations are `u64`. Second
//!   durations are `i64` to preserve a sentinel range if ever needed.
//! - `Record<string, number>` ↔ `HashMap<String, u32>`.
//! - Free-form step-config index signatures use
//!   `HashMap<String, serde_json::Value>` on a `#[serde(flatten)]` `extra` field.

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;

// ============================================================================
// Status Types
// ============================================================================

/// Lifecycle status of a task run.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum TaskRunStatus {
    /// The task is currently executing.
    Running,
    /// The task finished successfully.
    Complete,
    /// The task finished with an error.
    Failed,
    /// The task was stopped before natural completion.
    Stopped,
}

/// Kind of task being tracked.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum TaskType {
    /// A one-off AI task launched from a prompt.
    Task,
    /// A UI automation task.
    Automation,
    /// A task executed by the scheduler.
    Scheduled,
}

// ============================================================================
// Runner Task Run (local execution)
// ============================================================================

/// A task run as tracked by the local runner during execution.
///
/// Mirrors `TaskRun` in the runner's `taskRun.ts`. Optional fields here use
/// `?` in TypeScript, so they are omitted on the wire when missing.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
#[schemars(deny_unknown_fields)]
pub struct TaskRun {
    /// Unique identifier (UUID v4 string).
    #[serde(alias = "id")]
    pub id: String,
    /// Display name of the task.
    #[serde(alias = "task_name")]
    pub task_name: String,
    /// Original prompt text, if applicable.
    #[serde(default, skip_serializing_if = "Option::is_none", alias = "prompt")]
    pub prompt: Option<String>,
    /// Kind of task (one-off, automation, or scheduled).
    #[serde(alias = "task_type")]
    pub task_type: TaskType,
    /// ID of the workflow config used to run this task, if any.
    #[serde(default, skip_serializing_if = "Option::is_none", alias = "config_id")]
    pub config_id: Option<String>,
    /// Name of the workflow used to run this task, if any.
    #[serde(default, skip_serializing_if = "Option::is_none", alias = "workflow_name")]
    pub workflow_name: Option<String>,
    /// Current lifecycle status.
    #[serde(alias = "status")]
    pub status: TaskRunStatus,
    /// Number of AI sessions that have been run.
    #[serde(alias = "sessions_count")]
    pub sessions_count: u32,
    /// Optional cap on AI sessions.
    #[serde(default, skip_serializing_if = "Option::is_none", alias = "max_sessions")]
    pub max_sessions: Option<u32>,
    /// Whether the task will auto-continue into another session on exit.
    #[serde(alias = "auto_continue")]
    pub auto_continue: bool,
    /// Accumulated output log for the task run.
    #[serde(alias = "output_log")]
    pub output_log: String,
    /// Error message if the task failed.
    #[serde(default, skip_serializing_if = "Option::is_none", alias = "error_message")]
    pub error_message: Option<String>,
    /// AI-generated summary of the task run.
    #[serde(default, skip_serializing_if = "Option::is_none", alias = "summary")]
    pub summary: Option<String>,
    /// Whether the task's goal was achieved (AI assessment).
    #[serde(default, skip_serializing_if = "Option::is_none", alias = "goal_achieved")]
    pub goal_achieved: Option<bool>,
    /// Description of any remaining work (AI assessment).
    #[serde(default, skip_serializing_if = "Option::is_none", alias = "remaining_work")]
    pub remaining_work: Option<String>,
    /// ISO 8601 timestamp when the summary was generated.
    #[serde(default, skip_serializing_if = "Option::is_none", alias = "summary_generated_at")]
    pub summary_generated_at: Option<String>,
    /// ISO 8601 timestamp when the task was created.
    #[serde(alias = "created_at")]
    pub created_at: String,
    /// ISO 8601 timestamp when the task record was last updated.
    #[serde(alias = "updated_at")]
    pub updated_at: String,
    /// ISO 8601 timestamp when the task completed.
    #[serde(default, skip_serializing_if = "Option::is_none", alias = "completed_at")]
    pub completed_at: Option<String>,
}

// ============================================================================
// Web Backend Task Run (API responses)
// ============================================================================

/// A task run as returned by the backend API.
///
/// Mirrors `TaskRunBackend` in the web app's `task-runs.ts`. Nullable
/// ownership fields (`project_id`, `created_by_user_id`, `runner_id`,
/// `max_sessions`, `output_summary`, `error_message`, `duration_seconds`,
/// `completed_at`) are required on the wire but may be `null`; they are
/// `Option<T>` with `serde(default)` so deserialize tolerates missing, but
/// are always serialized (including as `null`) to preserve the wire shape.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
#[schemars(deny_unknown_fields)]
pub struct TaskRunBackend {
    /// Unique identifier (UUID v4 string).
    #[serde(alias = "id")]
    pub id: String,
    /// Owning project ID, if scoped to a project.
    #[serde(default, alias = "project_id")]
    pub project_id: Option<String>,
    /// User who created the task run, if known.
    #[serde(default, alias = "created_by_user_id")]
    pub created_by_user_id: Option<String>,
    /// Runner instance that executed the task, if known.
    #[serde(default, alias = "runner_id")]
    pub runner_id: Option<String>,
    /// Display name.
    #[serde(alias = "task_name")]
    pub task_name: String,
    /// Original prompt text.
    #[serde(alias = "prompt")]
    pub prompt: String,
    /// Current lifecycle status.
    #[serde(alias = "status")]
    pub status: TaskRunStatus,
    /// Number of AI sessions that have been run.
    #[serde(alias = "sessions_count")]
    pub sessions_count: u32,
    /// Optional cap on AI sessions.
    #[serde(default, alias = "max_sessions")]
    pub max_sessions: Option<u32>,
    /// Whether the task will auto-continue into another session on exit.
    #[serde(alias = "auto_continue")]
    pub auto_continue: bool,
    /// Short summary of the run output, if stored.
    #[serde(default, alias = "output_summary")]
    pub output_summary: Option<String>,
    /// Whether the full output log was persisted.
    #[serde(alias = "full_output_stored")]
    pub full_output_stored: bool,
    /// Error message if the task failed.
    #[serde(default, alias = "error_message")]
    pub error_message: Option<String>,
    /// Total duration in seconds.
    #[serde(default, alias = "duration_seconds")]
    pub duration_seconds: Option<i64>,
    /// ISO 8601 timestamp when the task was created.
    #[serde(alias = "created_at")]
    pub created_at: String,
    /// ISO 8601 timestamp when the record was last updated.
    #[serde(alias = "updated_at")]
    pub updated_at: String,
    /// ISO 8601 timestamp when the task completed.
    #[serde(default, alias = "completed_at")]
    pub completed_at: Option<String>,
}

// ============================================================================
// Session Types
// ============================================================================

/// A single AI session within a task run.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
#[schemars(deny_unknown_fields)]
pub struct TaskRunSession {
    /// Unique identifier (UUID v4 string).
    #[serde(alias = "id")]
    pub id: String,
    /// Parent task run ID.
    #[serde(alias = "task_id", alias = "task_run_id")]
    pub task_run_id: String,
    /// 1-based session index within the parent task run.
    #[serde(alias = "session_number")]
    pub session_number: u32,
    /// ISO 8601 timestamp when the session started.
    #[serde(alias = "started_at")]
    pub started_at: String,
    /// ISO 8601 timestamp when the session ended.
    #[serde(default, alias = "ended_at")]
    pub ended_at: Option<String>,
    /// Duration of the session in seconds.
    #[serde(default, alias = "duration_seconds")]
    pub duration_seconds: Option<i64>,
    /// Short summary of the session output, if stored.
    #[serde(default, alias = "output_summary")]
    pub output_summary: Option<String>,
}

// ============================================================================
// Finding Types
// ============================================================================

/// Category of a finding surfaced during a task run.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum TaskRunFindingCategory {
    /// A bug in application code.
    CodeBug,
    /// A security issue (e.g., vulnerability, exposed secret).
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
    /// A data migration issue.
    DataMigration,
    /// A non-blocking warning.
    Warning,
}

/// Severity of a finding.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum TaskRunFindingSeverity {
    /// Critical severity — blocks release or execution.
    Critical,
    /// High severity.
    High,
    /// Medium severity.
    Medium,
    /// Low severity.
    Low,
    /// Informational only.
    Info,
}

/// Lifecycle status of a finding.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum TaskRunFindingStatus {
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

/// How a finding should be acted upon.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum TaskRunFindingActionType {
    /// Can be fixed automatically.
    AutoFix,
    /// Requires user input to proceed.
    NeedsUserInput,
    /// Requires manual intervention.
    Manual,
    /// Informational; no action required.
    Informational,
}

/// A finding surfaced during a task run (bug, enhancement, TODO, etc.).
///
/// All nullable fields here are required-nullable on the wire (always present,
/// possibly `null`), so they use `serde(default)` without `skip_serializing_if`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
#[schemars(deny_unknown_fields)]
pub struct TaskRunFinding {
    /// Unique identifier (UUID v4 string).
    #[serde(alias = "id")]
    pub id: String,
    /// Parent task run ID.
    #[serde(alias = "task_id", alias = "task_run_id")]
    pub task_run_id: String,
    /// Category (bug, security, performance, etc.).
    #[serde(alias = "category")]
    pub category: TaskRunFindingCategory,
    /// Severity of the finding.
    #[serde(alias = "severity")]
    pub severity: TaskRunFindingSeverity,
    /// Current lifecycle status.
    #[serde(alias = "status")]
    pub status: TaskRunFindingStatus,
    /// How the finding should be acted upon.
    #[serde(alias = "action_type")]
    pub action_type: TaskRunFindingActionType,
    /// Hash used to deduplicate findings across runs.
    #[serde(default, alias = "signature_hash")]
    pub signature_hash: Option<String>,
    /// Short human-readable title.
    #[serde(alias = "title")]
    pub title: String,
    /// Full description.
    #[serde(alias = "description")]
    pub description: String,
    /// How the finding was resolved, if applicable.
    #[serde(default, alias = "resolution")]
    pub resolution: Option<String>,
    /// File path where the issue was found.
    #[serde(default, alias = "file_path")]
    pub file_path: Option<String>,
    /// Line number where the issue was found.
    #[serde(default, alias = "line_number")]
    pub line_number: Option<u32>,
    /// Column number where the issue was found.
    #[serde(default, alias = "column_number")]
    pub column_number: Option<u32>,
    /// Snippet of code illustrating the issue.
    #[serde(default, alias = "code_snippet")]
    pub code_snippet: Option<String>,
    /// Session number in which the finding was detected.
    #[serde(alias = "detected_in_session")]
    pub detected_in_session: u32,
    /// Session number in which the finding was resolved.
    #[serde(default, alias = "resolved_in_session")]
    pub resolved_in_session: Option<u32>,
    /// Whether this finding requires user input.
    #[serde(alias = "needs_input")]
    pub needs_input: bool,
    /// Question posed to the user, if input is needed.
    #[serde(default, alias = "question")]
    pub question: Option<String>,
    /// Suggested response options for the user, if input is needed.
    #[serde(default, alias = "input_options")]
    pub input_options: Option<Vec<String>>,
    /// The user's response, if any.
    #[serde(default, alias = "user_response")]
    pub user_response: Option<String>,
    /// ISO 8601 timestamp when the finding was detected.
    #[serde(alias = "detected_at")]
    pub detected_at: String,
    /// ISO 8601 timestamp when the finding was resolved.
    #[serde(default, alias = "resolved_at")]
    pub resolved_at: Option<String>,
    /// ISO 8601 timestamp when the record was last updated.
    #[serde(alias = "updated_at")]
    pub updated_at: String,
}

/// Response-shape alias for [`TaskRunFinding`].
///
/// The TS source declares `export type TaskRunFindingResponse = TaskRunFinding;`
/// — a transparent alias. Exposing it as a Rust `type` alias keeps downstream
/// code (tests, handlers) referring to the name they expect.
pub type TaskRunFindingResponse = TaskRunFinding;

// ============================================================================
// Detail Types
// ============================================================================

/// Aggregated finding counts grouped along each axis.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
#[schemars(deny_unknown_fields)]
pub struct TaskRunFindingSummary {
    /// Count of findings by category.
    #[serde(alias = "by_category")]
    pub by_category: HashMap<String, u32>,
    /// Count of findings by severity.
    #[serde(alias = "by_severity")]
    pub by_severity: HashMap<String, u32>,
    /// Count of findings by status.
    #[serde(alias = "by_status")]
    pub by_status: HashMap<String, u32>,
    /// Total number of findings.
    #[serde(alias = "total")]
    pub total: u32,
}

/// Detailed view of a backend task run, including its sessions and findings.
///
/// The TypeScript `TaskRunBackendDetail extends TaskRunBackend` is modeled in
/// Rust by flattening a [`TaskRunBackend`] base so the wire shape stays flat.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct TaskRunBackendDetail {
    /// Base task run record, flattened so its fields appear inline.
    #[serde(flatten)]
    pub base: TaskRunBackend,
    /// AI sessions associated with this task run.
    #[serde(default, alias = "sessions")]
    pub sessions: Vec<TaskRunSession>,
    /// Findings surfaced during this task run.
    #[serde(default, alias = "findings")]
    pub findings: Vec<TaskRunFinding>,
    /// Aggregated counts of findings.
    #[serde(alias = "finding_summary")]
    pub finding_summary: TaskRunFindingSummary,
}

// ============================================================================
// Request / Update Types
// ============================================================================

/// Request payload for creating a task run.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
#[schemars(deny_unknown_fields)]
pub struct TaskRunCreate {
    /// Optional client-generated ID.
    #[serde(default, skip_serializing_if = "Option::is_none", alias = "id")]
    pub id: Option<String>,
    /// Optional owning project.
    #[serde(default, skip_serializing_if = "Option::is_none", alias = "project_id")]
    pub project_id: Option<String>,
    /// Optional runner that will execute the task.
    #[serde(default, skip_serializing_if = "Option::is_none", alias = "runner_id")]
    pub runner_id: Option<String>,
    /// Display name.
    #[serde(alias = "task_name")]
    pub task_name: String,
    /// Prompt text, if applicable.
    #[serde(default, skip_serializing_if = "Option::is_none", alias = "prompt")]
    pub prompt: Option<String>,
    /// Optional cap on AI sessions.
    #[serde(default, skip_serializing_if = "Option::is_none", alias = "max_sessions")]
    pub max_sessions: Option<u32>,
    /// Whether the task should auto-continue.
    #[serde(default, skip_serializing_if = "Option::is_none", alias = "auto_continue")]
    pub auto_continue: Option<bool>,
    /// Task type.
    #[serde(default, skip_serializing_if = "Option::is_none", alias = "task_type")]
    pub task_type: Option<TaskType>,
    /// Workflow config ID to use.
    #[serde(default, skip_serializing_if = "Option::is_none", alias = "config_id")]
    pub config_id: Option<String>,
    /// Workflow name to use.
    #[serde(default, skip_serializing_if = "Option::is_none", alias = "workflow_name")]
    pub workflow_name: Option<String>,
    /// Serialized execution steps, if provided ad-hoc.
    #[serde(default, skip_serializing_if = "Option::is_none", alias = "execution_steps_json")]
    pub execution_steps_json: Option<String>,
    /// Serialized log-sources configuration.
    #[serde(default, skip_serializing_if = "Option::is_none", alias = "log_sources_json")]
    pub log_sources_json: Option<String>,
}

/// Request payload for updating an existing task run. All fields are optional;
/// only those supplied are applied.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
#[schemars(deny_unknown_fields)]
pub struct TaskRunUpdate {
    /// New lifecycle status.
    #[serde(default, skip_serializing_if = "Option::is_none", alias = "status")]
    pub status: Option<TaskRunStatus>,
    /// Updated session count.
    #[serde(default, skip_serializing_if = "Option::is_none", alias = "sessions_count")]
    pub sessions_count: Option<u32>,
    /// Updated output summary.
    #[serde(default, skip_serializing_if = "Option::is_none", alias = "output_summary")]
    pub output_summary: Option<String>,
    /// Full output log to persist.
    #[serde(default, skip_serializing_if = "Option::is_none", alias = "full_output")]
    pub full_output: Option<String>,
    /// Whether the full output log has been stored.
    #[serde(default, skip_serializing_if = "Option::is_none", alias = "full_output_stored")]
    pub full_output_stored: Option<bool>,
    /// Error message to attach.
    #[serde(default, skip_serializing_if = "Option::is_none", alias = "error_message")]
    pub error_message: Option<String>,
    /// Total duration in seconds.
    #[serde(default, skip_serializing_if = "Option::is_none", alias = "duration_seconds")]
    pub duration_seconds: Option<i64>,
    /// ISO 8601 timestamp when the task completed.
    #[serde(default, skip_serializing_if = "Option::is_none", alias = "completed_at")]
    pub completed_at: Option<String>,
}

/// Request payload for creating a finding.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
#[schemars(deny_unknown_fields)]
pub struct TaskRunFindingCreate {
    /// Optional client-generated ID.
    #[serde(default, skip_serializing_if = "Option::is_none", alias = "id")]
    pub id: Option<String>,
    /// Category.
    #[serde(alias = "category")]
    pub category: TaskRunFindingCategory,
    /// Severity.
    #[serde(alias = "severity")]
    pub severity: TaskRunFindingSeverity,
    /// Initial status (defaults server-side if omitted).
    #[serde(default, skip_serializing_if = "Option::is_none", alias = "status")]
    pub status: Option<TaskRunFindingStatus>,
    /// Action type (defaults server-side if omitted).
    #[serde(default, skip_serializing_if = "Option::is_none", alias = "action_type")]
    pub action_type: Option<TaskRunFindingActionType>,
    /// Deduplication hash.
    #[serde(default, skip_serializing_if = "Option::is_none", alias = "signature_hash")]
    pub signature_hash: Option<String>,
    /// Short title.
    #[serde(alias = "title")]
    pub title: String,
    /// Full description.
    #[serde(alias = "description")]
    pub description: String,
    /// Resolution text, if already resolved.
    #[serde(default, skip_serializing_if = "Option::is_none", alias = "resolution")]
    pub resolution: Option<String>,
    /// File path where the issue was found.
    #[serde(default, skip_serializing_if = "Option::is_none", alias = "file_path")]
    pub file_path: Option<String>,
    /// Line number where the issue was found.
    #[serde(default, skip_serializing_if = "Option::is_none", alias = "line_number")]
    pub line_number: Option<u32>,
    /// Column number where the issue was found.
    #[serde(default, skip_serializing_if = "Option::is_none", alias = "column_number")]
    pub column_number: Option<u32>,
    /// Snippet of code illustrating the issue.
    #[serde(default, skip_serializing_if = "Option::is_none", alias = "code_snippet")]
    pub code_snippet: Option<String>,
    /// Session number in which the finding was detected.
    #[serde(alias = "detected_in_session")]
    pub detected_in_session: u32,
    /// Whether this finding requires user input.
    #[serde(default, skip_serializing_if = "Option::is_none", alias = "needs_input")]
    pub needs_input: Option<bool>,
    /// Question to pose to the user, if input is needed.
    #[serde(default, skip_serializing_if = "Option::is_none", alias = "question")]
    pub question: Option<String>,
    /// Suggested response options for the user.
    #[serde(default, skip_serializing_if = "Option::is_none", alias = "input_options")]
    pub input_options: Option<Vec<String>>,
}

/// Request payload for updating a finding. All fields are optional.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
#[schemars(deny_unknown_fields)]
pub struct TaskRunFindingUpdate {
    /// New lifecycle status.
    #[serde(default, skip_serializing_if = "Option::is_none", alias = "status")]
    pub status: Option<TaskRunFindingStatus>,
    /// Resolution text.
    #[serde(default, skip_serializing_if = "Option::is_none", alias = "resolution")]
    pub resolution: Option<String>,
    /// Session number in which the finding was resolved.
    #[serde(default, skip_serializing_if = "Option::is_none", alias = "resolved_in_session")]
    pub resolved_in_session: Option<u32>,
    /// ISO 8601 timestamp of resolution.
    #[serde(default, skip_serializing_if = "Option::is_none", alias = "resolved_at")]
    pub resolved_at: Option<String>,
    /// User response, if the finding needed input.
    #[serde(default, skip_serializing_if = "Option::is_none", alias = "user_response")]
    pub user_response: Option<String>,
}

// ============================================================================
// Runner-Specific Request Types
// ============================================================================

/// Inline `data` payload on a [`RunPromptResponse`].
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
#[schemars(deny_unknown_fields)]
pub struct RunPromptResponseData {
    /// AI output text.
    #[serde(default, skip_serializing_if = "Option::is_none", alias = "output")]
    pub output: Option<String>,
    /// Final response text.
    #[serde(default, skip_serializing_if = "Option::is_none", alias = "response")]
    pub response: Option<String>,
}

/// Response from the runner's `run_prompt` endpoint.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
#[schemars(deny_unknown_fields)]
pub struct RunPromptResponse {
    /// Whether the prompt was accepted and started successfully.
    #[serde(alias = "success")]
    pub success: bool,
    /// ID of the created task run, if any.
    #[serde(default, skip_serializing_if = "Option::is_none", alias = "task_run_id")]
    pub task_run_id: Option<String>,
    /// ID of the created AI session, if any.
    #[serde(default, skip_serializing_if = "Option::is_none", alias = "session_id")]
    pub session_id: Option<String>,
    /// Path to the state file tracking the session.
    #[serde(default, skip_serializing_if = "Option::is_none", alias = "state_file")]
    pub state_file: Option<String>,
    /// Path to the log file for the session.
    #[serde(default, skip_serializing_if = "Option::is_none", alias = "log_file")]
    pub log_file: Option<String>,
    /// OS process ID of the spawned AI session, if any.
    #[serde(default, skip_serializing_if = "Option::is_none", alias = "pid")]
    pub pid: Option<u32>,
    /// Error message if the call failed.
    #[serde(default, skip_serializing_if = "Option::is_none", alias = "error")]
    pub error: Option<String>,
    /// Immediate output if the call ran synchronously.
    #[serde(default, skip_serializing_if = "Option::is_none", alias = "output")]
    pub output: Option<String>,
    /// Structured data payload from a synchronous prompt run.
    #[serde(default, skip_serializing_if = "Option::is_none", alias = "data")]
    pub data: Option<RunPromptResponseData>,
}

/// Request body for the runner's `run_prompt` endpoint.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
#[schemars(deny_unknown_fields)]
pub struct RunPromptRequest {
    /// Display name for the task.
    #[serde(alias = "name")]
    pub name: String,
    /// Prompt content (the actual text sent to the AI).
    #[serde(alias = "content")]
    pub content: String,
    /// Optional cap on AI sessions.
    #[serde(default, skip_serializing_if = "Option::is_none", alias = "max_sessions")]
    pub max_sessions: Option<u32>,
    /// Display-only version of the prompt (shown in the UI).
    #[serde(default, skip_serializing_if = "Option::is_none", alias = "display_prompt")]
    pub display_prompt: Option<String>,
    /// Hard timeout in seconds.
    #[serde(default, skip_serializing_if = "Option::is_none", alias = "timeout_seconds")]
    pub timeout_seconds: Option<u64>,
    /// Optional free-form context string appended to the prompt.
    #[serde(default, skip_serializing_if = "Option::is_none", alias = "context")]
    pub context: Option<String>,
    /// Attached image paths.
    #[serde(default, skip_serializing_if = "Option::is_none", alias = "image_paths")]
    pub image_paths: Option<Vec<String>>,
    /// Attached video paths.
    #[serde(default, skip_serializing_if = "Option::is_none", alias = "video_paths")]
    pub video_paths: Option<Vec<String>>,
    /// Optional path to a trace file.
    #[serde(default, skip_serializing_if = "Option::is_none", alias = "trace_path")]
    pub trace_path: Option<String>,
    /// Cap on video frames to extract for the prompt.
    #[serde(default, skip_serializing_if = "Option::is_none", alias = "max_video_frames")]
    pub max_video_frames: Option<u32>,
    /// Cap on trace screenshots to include.
    #[serde(default, skip_serializing_if = "Option::is_none", alias = "max_trace_screenshots")]
    pub max_trace_screenshots: Option<u32>,
}

/// Request body for creating a task run (simplified shape used by the runner's
/// create-task endpoint).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
#[schemars(deny_unknown_fields)]
pub struct CreateTaskRunRequest {
    /// Display name.
    #[serde(alias = "task_name")]
    pub task_name: String,
    /// Prompt text, if applicable.
    #[serde(default, skip_serializing_if = "Option::is_none", alias = "prompt")]
    pub prompt: Option<String>,
    /// Task type.
    #[serde(default, skip_serializing_if = "Option::is_none", alias = "task_type")]
    pub task_type: Option<TaskType>,
    /// Workflow config ID to use.
    #[serde(default, skip_serializing_if = "Option::is_none", alias = "config_id")]
    pub config_id: Option<String>,
    /// Workflow name to use.
    #[serde(default, skip_serializing_if = "Option::is_none", alias = "workflow_name")]
    pub workflow_name: Option<String>,
    /// Optional cap on AI sessions.
    #[serde(default, skip_serializing_if = "Option::is_none", alias = "max_sessions")]
    pub max_sessions: Option<u32>,
    /// Whether the task should auto-continue.
    #[serde(default, skip_serializing_if = "Option::is_none", alias = "auto_continue")]
    pub auto_continue: Option<bool>,
    /// Serialized execution steps, if provided ad-hoc.
    #[serde(default, skip_serializing_if = "Option::is_none", alias = "execution_steps_json")]
    pub execution_steps_json: Option<String>,
    /// Serialized log-sources configuration.
    #[serde(default, skip_serializing_if = "Option::is_none", alias = "log_sources_json")]
    pub log_sources_json: Option<String>,
}

// ============================================================================
// Filter Types
// ============================================================================

/// Filter parameters for listing task runs.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
#[schemars(deny_unknown_fields)]
pub struct TaskRunFilters {
    /// Restrict to a given project.
    #[serde(default, skip_serializing_if = "Option::is_none", alias = "project_id")]
    pub project_id: Option<String>,
    /// Restrict to a given status.
    #[serde(default, skip_serializing_if = "Option::is_none", alias = "status")]
    pub status: Option<TaskRunStatus>,
    /// Include only runs after this ISO 8601 timestamp.
    #[serde(default, skip_serializing_if = "Option::is_none", alias = "start_date")]
    pub start_date: Option<String>,
    /// Include only runs before this ISO 8601 timestamp.
    #[serde(default, skip_serializing_if = "Option::is_none", alias = "end_date")]
    pub end_date: Option<String>,
    /// Pagination offset.
    #[serde(default, skip_serializing_if = "Option::is_none", alias = "offset")]
    pub offset: Option<u32>,
    /// Pagination limit.
    #[serde(default, skip_serializing_if = "Option::is_none", alias = "limit")]
    pub limit: Option<u32>,
}

/// Filter parameters for listing findings.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
#[schemars(deny_unknown_fields)]
pub struct TaskRunFindingFilters {
    /// Restrict to a given category.
    #[serde(default, skip_serializing_if = "Option::is_none", alias = "category")]
    pub category: Option<TaskRunFindingCategory>,
    /// Restrict to a given severity.
    #[serde(default, skip_serializing_if = "Option::is_none", alias = "severity")]
    pub severity: Option<TaskRunFindingSeverity>,
    /// Restrict to a given status.
    #[serde(default, skip_serializing_if = "Option::is_none", alias = "status")]
    pub status: Option<TaskRunFindingStatus>,
}

// ============================================================================
// Response Types
// ============================================================================

/// Pagination envelope attached to list responses.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
#[schemars(deny_unknown_fields)]
pub struct Pagination {
    /// Total number of matching records.
    #[serde(alias = "total")]
    pub total: u32,
    /// Maximum number of records returned per page.
    #[serde(alias = "limit")]
    pub limit: u32,
    /// Offset into the full result set.
    #[serde(alias = "offset")]
    pub offset: u32,
    /// Whether additional records are available after this page.
    #[serde(alias = "has_more")]
    pub has_more: bool,
}

/// Response for `GET /task-runs`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
#[schemars(deny_unknown_fields)]
pub struct TaskRunListResponse {
    /// Page of matching task runs.
    #[serde(alias = "tasks")]
    pub tasks: Vec<TaskRunBackend>,
    /// Pagination envelope.
    #[serde(alias = "pagination")]
    pub pagination: Pagination,
}

/// Response for `GET /task-runs/{id}/findings`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
#[schemars(deny_unknown_fields)]
pub struct TaskRunFindingsListResponse {
    /// Findings for the task run.
    #[serde(alias = "findings")]
    pub findings: Vec<TaskRunFinding>,
    /// Aggregated counts of findings.
    #[serde(alias = "summary")]
    pub summary: TaskRunFindingSummary,
}

/// Compact findings summary including the most recent findings.
///
/// The TS type `TaskRunFindingResponse` is a type alias for `TaskRunFinding`;
/// in Rust we use `TaskRunFinding` directly.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
#[schemars(deny_unknown_fields)]
pub struct FindingsSummary {
    /// Total number of findings.
    #[serde(alias = "total")]
    pub total: u32,
    /// Count by severity.
    #[serde(alias = "by_severity")]
    pub by_severity: HashMap<String, u32>,
    /// Count by category.
    #[serde(alias = "by_category")]
    pub by_category: HashMap<String, u32>,
    /// Count by status.
    #[serde(alias = "by_status")]
    pub by_status: HashMap<String, u32>,
    /// Most recent findings.
    #[serde(alias = "recent")]
    pub recent: Vec<TaskRunFinding>,
}

// ============================================================================
// Verification Result Types
// ============================================================================

/// A specific issue detail from an individual verification check.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
#[schemars(deny_unknown_fields)]
pub struct CheckIssueDetail {
    /// File path where the issue was detected.
    #[serde(alias = "file")]
    pub file: String,
    /// Line number, if applicable.
    #[serde(default, alias = "line")]
    pub line: Option<u32>,
    /// Column number, if applicable.
    #[serde(default, alias = "column")]
    pub column: Option<u32>,
    /// Error code or lint rule, if applicable.
    #[serde(default, alias = "code")]
    pub code: Option<String>,
    /// Human-readable message.
    #[serde(alias = "message")]
    pub message: String,
    /// Severity label (free-form string from the upstream check).
    #[serde(alias = "severity")]
    pub severity: String,
    /// Whether the check can auto-fix this issue.
    #[serde(alias = "fixable")]
    pub fixable: bool,
}

/// Result of a single named verification check (e.g., `"eslint"`, `"mypy"`).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
#[schemars(deny_unknown_fields)]
pub struct IndividualCheckResult {
    /// Name of the check.
    #[serde(alias = "name")]
    pub name: String,
    /// Free-form status string (e.g., `"passed"`, `"failed"`, `"skipped"`).
    #[serde(alias = "status")]
    pub status: String,
    /// How long the check took, in milliseconds.
    #[serde(alias = "duration_ms")]
    pub duration_ms: u64,
    /// Number of issues surfaced by this check.
    #[serde(alias = "issues_found")]
    pub issues_found: u32,
    /// Number of issues auto-fixed by this check.
    #[serde(alias = "issues_fixed")]
    pub issues_fixed: u32,
    /// Number of files the check inspected.
    #[serde(alias = "files_checked")]
    pub files_checked: u32,
    /// Error message, if the check itself failed to run.
    #[serde(default, alias = "error_message")]
    pub error_message: Option<String>,
    /// Raw check output, if captured.
    #[serde(default, alias = "output")]
    pub output: Option<String>,
    /// Specific issue details.
    #[serde(default, alias = "issues")]
    pub issues: Vec<CheckIssueDetail>,
}

/// Detailed output captured for a single verification step.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
#[schemars(deny_unknown_fields)]
pub struct VerificationStepDetails {
    /// ID of the step this detail belongs to.
    #[serde(alias = "step_id")]
    pub step_id: String,
    /// Phase the step belongs to (e.g., `"setup"`, `"verification"`).
    #[serde(alias = "phase")]
    pub phase: String,
    /// Captured stdout, if any.
    #[serde(default, alias = "stdout")]
    pub stdout: Option<String>,
    /// Captured stderr, if any.
    #[serde(default, alias = "stderr")]
    pub stderr: Option<String>,
    /// Number of assertions that passed.
    #[serde(default, alias = "assertions_passed")]
    pub assertions_passed: Option<u32>,
    /// Total number of assertions.
    #[serde(default, alias = "assertions_total")]
    pub assertions_total: Option<u32>,
    /// Captured browser/console output.
    #[serde(default, alias = "console_output")]
    pub console_output: Option<String>,
    /// Captured page snapshot (HTML or serialized representation).
    #[serde(default, alias = "page_snapshot")]
    pub page_snapshot: Option<String>,
    /// Exit code of the spawned process.
    #[serde(default, alias = "exit_code")]
    pub exit_code: Option<i32>,
    /// Results of individual named checks (e.g., lint, type, test).
    #[serde(default, alias = "check_results")]
    pub check_results: Option<Vec<IndividualCheckResult>>,
}

/// Execution config captured for a verification step.
///
/// The TypeScript type includes an index signature `[key: string]: unknown`,
/// so extra arbitrary fields are captured in `extra` via `serde(flatten)` and
/// passed through opaquely.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct StepExecutionConfig {
    /// Action type (e.g., click, type, wait).
    #[serde(default, skip_serializing_if = "Option::is_none", alias = "action_type")]
    pub action_type: Option<String>,
    /// Target image ID, if the action references an image.
    #[serde(default, skip_serializing_if = "Option::is_none", alias = "target_image_id")]
    pub target_image_id: Option<String>,
    /// Target image name, if the action references an image.
    #[serde(default, skip_serializing_if = "Option::is_none", alias = "target_image_name")]
    pub target_image_name: Option<String>,
    /// Check type for verification steps.
    #[serde(default, skip_serializing_if = "Option::is_none", alias = "check_type")]
    pub check_type: Option<String>,
    /// Timeout in seconds.
    #[serde(default, skip_serializing_if = "Option::is_none", alias = "timeout_seconds")]
    pub timeout_seconds: Option<u64>,
    /// Additional step-specific configuration fields. Mirrors the TS index
    /// signature `[key: string]: unknown`.
    #[serde(flatten)]
    pub extra: HashMap<String, Value>,
}

/// Result of a single step within a verification phase.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
#[schemars(deny_unknown_fields)]
pub struct VerificationStepResult {
    /// Zero-based index of the step within the phase.
    #[serde(alias = "step_index")]
    pub step_index: u32,
    /// Free-form step type label.
    #[serde(alias = "step_type")]
    pub step_type: String,
    /// Display name of the step.
    #[serde(alias = "step_name")]
    pub step_name: String,
    /// ID of the step, if assigned.
    #[serde(default, alias = "step_id")]
    pub step_id: Option<String>,
    /// Whether the step succeeded.
    #[serde(alias = "success")]
    pub success: bool,
    /// Error message if the step failed.
    #[serde(default, alias = "error")]
    pub error: Option<String>,
    /// Path to a screenshot captured for the step.
    #[serde(default, alias = "screenshot_path")]
    pub screenshot_path: Option<String>,
    /// ISO 8601 timestamp when the step started.
    #[serde(default, alias = "started_at")]
    pub started_at: Option<String>,
    /// ISO 8601 timestamp when the step ended.
    #[serde(default, alias = "ended_at")]
    pub ended_at: Option<String>,
    /// Step duration in milliseconds.
    #[serde(alias = "duration_ms")]
    pub duration_ms: u64,
    /// Execution config for the step.
    #[serde(alias = "config")]
    pub config: StepExecutionConfig,
    /// Detailed captured output, if any.
    #[serde(default, alias = "verification_details")]
    pub verification_details: Option<VerificationStepDetails>,
    /// Arbitrary structured output produced by the step.
    #[serde(default, alias = "output_data")]
    pub output_data: Option<HashMap<String, Value>>,
}

/// Result of evaluating a named gate across a set of steps.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
#[schemars(deny_unknown_fields)]
pub struct GateEvaluationResult {
    /// Name of the gate.
    #[serde(alias = "gate_name")]
    pub gate_name: String,
    /// IDs of steps the gate required.
    #[serde(default, alias = "required_step_ids")]
    pub required_step_ids: Vec<String>,
    /// IDs of required steps that passed.
    #[serde(default, alias = "passed_step_ids")]
    pub passed_step_ids: Vec<String>,
    /// IDs of required steps that failed.
    #[serde(default, alias = "failed_step_ids")]
    pub failed_step_ids: Vec<String>,
    /// IDs of required steps that were missing.
    #[serde(default, alias = "missing_step_ids")]
    pub missing_step_ids: Vec<String>,
    /// Whether the gate passed overall.
    #[serde(alias = "passed")]
    pub passed: bool,
}

/// Result of a single iteration of the verification phase.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
#[schemars(deny_unknown_fields)]
pub struct VerificationPhaseResult {
    /// 1-based iteration index within the workflow run.
    #[serde(alias = "iteration")]
    pub iteration: u32,
    /// Whether all steps passed.
    #[serde(alias = "all_passed")]
    pub all_passed: bool,
    /// Total number of steps executed.
    #[serde(alias = "total_steps")]
    pub total_steps: u32,
    /// Number of steps that passed.
    #[serde(alias = "passed_steps")]
    pub passed_steps: u32,
    /// Number of steps that failed.
    #[serde(alias = "failed_steps")]
    pub failed_steps: u32,
    /// Number of steps that were skipped.
    #[serde(alias = "skipped_steps")]
    pub skipped_steps: u32,
    /// Total duration of the phase in milliseconds.
    #[serde(alias = "total_duration_ms")]
    pub total_duration_ms: u64,
    /// Per-step results.
    #[serde(default, alias = "step_results")]
    pub step_results: Vec<VerificationStepResult>,
    /// Whether a critical step failure short-circuited the phase.
    #[serde(alias = "critical_failure")]
    pub critical_failure: bool,
    /// Per-gate evaluation results.
    #[serde(default, alias = "gate_results")]
    pub gate_results: Vec<GateEvaluationResult>,
    /// Whether pass/fail is determined by gates rather than overall step counts.
    #[serde(alias = "gate_based_evaluation")]
    pub gate_based_evaluation: bool,
}

/// Response record for a single stored verification result.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
#[schemars(deny_unknown_fields)]
pub struct VerificationResultResponse {
    /// Unique identifier (UUID v4 string).
    #[serde(alias = "id")]
    pub id: String,
    /// ID of the owning task run.
    #[serde(alias = "task_run_id")]
    pub task_run_id: String,
    /// 1-based iteration index within the workflow run.
    #[serde(alias = "iteration")]
    pub iteration: u32,
    /// Whether all steps passed.
    #[serde(alias = "all_passed")]
    pub all_passed: bool,
    /// Total number of steps executed.
    #[serde(alias = "total_steps")]
    pub total_steps: u32,
    /// Number of steps that passed.
    #[serde(alias = "passed_steps")]
    pub passed_steps: u32,
    /// Number of steps that failed.
    #[serde(alias = "failed_steps")]
    pub failed_steps: u32,
    /// Number of steps that were skipped.
    #[serde(alias = "skipped_steps")]
    pub skipped_steps: u32,
    /// Total duration of the phase in milliseconds.
    #[serde(alias = "total_duration_ms")]
    pub total_duration_ms: u64,
    /// Whether a critical step failure short-circuited the phase.
    #[serde(alias = "critical_failure")]
    pub critical_failure: bool,
    /// Full typed verification result payload.
    #[serde(alias = "result_json")]
    pub result_json: VerificationPhaseResult,
    /// ISO 8601 timestamp when the record was created.
    #[serde(alias = "created_at")]
    pub created_at: String,
}

/// Response for listing verification results for a task run.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
#[schemars(deny_unknown_fields)]
pub struct VerificationResultsListResponse {
    /// ID of the owning task run.
    #[serde(alias = "task_run_id")]
    pub task_run_id: String,
    /// All verification results for the task run.
    #[serde(default, alias = "results")]
    pub results: Vec<VerificationResultResponse>,
    /// Total number of verification results.
    #[serde(alias = "count")]
    pub count: u32,
    /// Number of iterations that passed.
    #[serde(alias = "passed_iterations")]
    pub passed_iterations: u32,
    /// Number of iterations that failed.
    #[serde(alias = "failed_iterations")]
    pub failed_iterations: u32,
}
