//! Task-run DTO types.
//!
//! Wire-format types for task runs: local execution records (runner-side),
//! backend/API representations (web-side), session and finding entities,
//! create/update request payloads, filters, and verification result responses.
//!
//! Ported from `qontinui-schemas/ts/src/task-run/index.ts`. Rust is the source
//! of truth; JSON Schema emitted from these types drives the TS and Python
//! bindings.
//!
//! This module is wire-format only: no business logic, no `impl` blocks, no
//! tests. Dates, times, and UUIDs are `String`s — see the crate-level docs.
//! Free-form step-config maps use `HashMap<String, serde_json::Value>`.

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
/// Mirrors `TaskRun` in the runner's `taskRun.ts`. Carries the in-flight
/// output log, optional workflow/config references, and AI-generated summary
/// fields filled in after completion.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct TaskRun {
    /// Unique identifier (UUID v4 string).
    pub id: String,
    /// Display name of the task.
    pub task_name: String,
    /// Original prompt text, if applicable.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub prompt: Option<String>,
    /// Kind of task (one-off, automation, or scheduled).
    pub task_type: TaskType,
    /// ID of the workflow config used to run this task, if any.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub config_id: Option<String>,
    /// Name of the workflow used to run this task, if any.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub workflow_name: Option<String>,
    /// Current lifecycle status.
    pub status: TaskRunStatus,
    /// Number of AI sessions that have been run.
    pub sessions_count: u64,
    /// Optional cap on AI sessions.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max_sessions: Option<u64>,
    /// Whether the task will auto-continue into another session on exit.
    pub auto_continue: bool,
    /// Accumulated output log for the task run.
    pub output_log: String,
    /// Error message if the task failed.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub error_message: Option<String>,
    /// AI-generated summary of the task run.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub summary: Option<String>,
    /// Whether the task's goal was achieved (AI assessment).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub goal_achieved: Option<bool>,
    /// Description of any remaining work (AI assessment).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub remaining_work: Option<String>,
    /// ISO 8601 timestamp when the summary was generated.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub summary_generated_at: Option<String>,
    /// ISO 8601 timestamp when the task was created.
    pub created_at: String,
    /// ISO 8601 timestamp when the task record was last updated.
    pub updated_at: String,
    /// ISO 8601 timestamp when the task completed.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub completed_at: Option<String>,
}

// ============================================================================
// Web Backend Task Run (API responses)
// ============================================================================

/// A task run as returned by the backend API.
///
/// Mirrors `TaskRunBackend` in the web app's `task-runs.ts`. Nullable
/// ownership fields (`project_id`, `created_by_user_id`, `runner_id`) are
/// always present on the wire but may be `null`; they remain `Option<String>`
/// with default-and-skip so round-tripping is stable.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct TaskRunBackend {
    /// Unique identifier (UUID v4 string).
    pub id: String,
    /// Owning project ID, if scoped to a project.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub project_id: Option<String>,
    /// User who created the task run, if known.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub created_by_user_id: Option<String>,
    /// Runner instance that executed the task, if known.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub runner_id: Option<String>,
    /// Display name.
    pub task_name: String,
    /// Original prompt text.
    pub prompt: String,
    /// Current lifecycle status.
    pub status: TaskRunStatus,
    /// Number of AI sessions that have been run.
    pub sessions_count: u64,
    /// Optional cap on AI sessions.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max_sessions: Option<u64>,
    /// Whether the task will auto-continue into another session on exit.
    pub auto_continue: bool,
    /// Short summary of the run output, if stored.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub output_summary: Option<String>,
    /// Whether the full output log was persisted.
    pub full_output_stored: bool,
    /// Error message if the task failed.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub error_message: Option<String>,
    /// Total duration in seconds.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub duration_seconds: Option<i64>,
    /// ISO 8601 timestamp when the task was created.
    pub created_at: String,
    /// ISO 8601 timestamp when the record was last updated.
    pub updated_at: String,
    /// ISO 8601 timestamp when the task completed.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub completed_at: Option<String>,
}

// ============================================================================
// Session Types
// ============================================================================

/// A single AI session within a task run.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct TaskRunSession {
    /// Unique identifier (UUID v4 string).
    pub id: String,
    /// Parent task run ID.
    pub task_id: String,
    /// 1-based session index within the parent task run.
    pub session_number: u64,
    /// ISO 8601 timestamp when the session started.
    pub started_at: String,
    /// ISO 8601 timestamp when the session ended.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ended_at: Option<String>,
    /// Duration of the session in seconds.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub duration_seconds: Option<i64>,
    /// Short summary of the session output, if stored.
    #[serde(default, skip_serializing_if = "Option::is_none")]
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
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct TaskRunFinding {
    /// Unique identifier (UUID v4 string).
    pub id: String,
    /// Parent task run ID.
    pub task_id: String,
    /// Category (bug, security, performance, etc.).
    pub category: TaskRunFindingCategory,
    /// Severity of the finding.
    pub severity: TaskRunFindingSeverity,
    /// Current lifecycle status.
    pub status: TaskRunFindingStatus,
    /// How the finding should be acted upon.
    pub action_type: TaskRunFindingActionType,
    /// Hash used to deduplicate findings across runs.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub signature_hash: Option<String>,
    /// Short human-readable title.
    pub title: String,
    /// Full description.
    pub description: String,
    /// How the finding was resolved, if applicable.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub resolution: Option<String>,
    /// File path where the issue was found.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub file_path: Option<String>,
    /// Line number where the issue was found.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub line_number: Option<i64>,
    /// Column number where the issue was found.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub column_number: Option<i64>,
    /// Snippet of code illustrating the issue.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub code_snippet: Option<String>,
    /// Session number in which the finding was detected.
    pub detected_in_session: u64,
    /// Session number in which the finding was resolved.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub resolved_in_session: Option<u64>,
    /// Whether this finding requires user input.
    pub needs_input: bool,
    /// Question posed to the user, if input is needed.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub question: Option<String>,
    /// Suggested response options for the user, if input is needed.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub input_options: Option<Vec<String>>,
    /// The user's response, if any.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub user_response: Option<String>,
    /// ISO 8601 timestamp when the finding was detected.
    pub detected_at: String,
    /// ISO 8601 timestamp when the finding was resolved.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub resolved_at: Option<String>,
    /// ISO 8601 timestamp when the record was last updated.
    pub updated_at: String,
}

/// Response-shaped alias of [`TaskRunFinding`] used by the backend API.
pub type TaskRunFindingResponse = TaskRunFinding;

// ============================================================================
// Detail Types
// ============================================================================

/// Aggregated finding counts grouped along each axis.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct TaskRunFindingSummary {
    /// Count of findings by category.
    pub by_category: HashMap<String, u64>,
    /// Count of findings by severity.
    pub by_severity: HashMap<String, u64>,
    /// Count of findings by status.
    pub by_status: HashMap<String, u64>,
    /// Total number of findings.
    pub total: u64,
}

/// Detailed view of a backend task run, including its sessions and findings.
///
/// The TypeScript `extends TaskRunBackend` is flattened here by inlining the
/// fields of [`TaskRunBackend`] via `#[serde(flatten)]` on a nested struct —
/// but since JSON Schema emission does not round-trip `flatten` cleanly, we
/// instead carry a typed `task` field holding the base record. Consumers
/// producing API payloads merge this at the application layer.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct TaskRunBackendDetail {
    /// Base task run record. Flattened at the JSON layer by the backend.
    #[serde(flatten)]
    pub task: TaskRunBackend,
    /// AI sessions associated with this task run.
    #[serde(default)]
    pub sessions: Vec<TaskRunSession>,
    /// Findings surfaced during this task run.
    #[serde(default)]
    pub findings: Vec<TaskRunFinding>,
    /// Aggregated counts of findings.
    pub finding_summary: TaskRunFindingSummary,
}

// ============================================================================
// Request / Update Types
// ============================================================================

/// Request payload for creating a task run (runner-side fields combined with
/// backend-side fields).
#[derive(Debug, Clone, Default, Serialize, Deserialize, JsonSchema)]
pub struct TaskRunCreate {
    /// Optional client-generated ID.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    /// Optional owning project.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub project_id: Option<String>,
    /// Optional runner that will execute the task.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub runner_id: Option<String>,
    /// Display name.
    pub task_name: String,
    /// Prompt text, if applicable.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub prompt: Option<String>,
    /// Optional cap on AI sessions.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max_sessions: Option<u64>,
    /// Whether the task should auto-continue.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub auto_continue: Option<bool>,
    /// Task type.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub task_type: Option<TaskType>,
    /// Workflow config ID to use.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub config_id: Option<String>,
    /// Workflow name to use.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub workflow_name: Option<String>,
    /// Serialized execution steps, if provided ad-hoc.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub execution_steps_json: Option<String>,
    /// Serialized log-sources configuration.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub log_sources_json: Option<String>,
}

/// Request payload for updating an existing task run. All fields are optional;
/// only those supplied are applied.
#[derive(Debug, Clone, Default, Serialize, Deserialize, JsonSchema)]
pub struct TaskRunUpdate {
    /// New lifecycle status.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub status: Option<TaskRunStatus>,
    /// Updated session count.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sessions_count: Option<u64>,
    /// Updated output summary.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub output_summary: Option<String>,
    /// Full output log to persist.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub full_output: Option<String>,
    /// Whether the full output log has been stored.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub full_output_stored: Option<bool>,
    /// Error message to attach.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub error_message: Option<String>,
    /// Total duration in seconds.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub duration_seconds: Option<i64>,
    /// ISO 8601 timestamp when the task completed.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub completed_at: Option<String>,
}

/// Request payload for creating a finding.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct TaskRunFindingCreate {
    /// Optional client-generated ID.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    /// Category.
    pub category: TaskRunFindingCategory,
    /// Severity.
    pub severity: TaskRunFindingSeverity,
    /// Initial status (defaults server-side if omitted).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub status: Option<TaskRunFindingStatus>,
    /// Action type (defaults server-side if omitted).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub action_type: Option<TaskRunFindingActionType>,
    /// Deduplication hash.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub signature_hash: Option<String>,
    /// Short title.
    pub title: String,
    /// Full description.
    pub description: String,
    /// Resolution text, if already resolved.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub resolution: Option<String>,
    /// File path where the issue was found.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub file_path: Option<String>,
    /// Line number where the issue was found.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub line_number: Option<i64>,
    /// Column number where the issue was found.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub column_number: Option<i64>,
    /// Snippet of code illustrating the issue.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub code_snippet: Option<String>,
    /// Session number in which the finding was detected.
    pub detected_in_session: u64,
    /// Whether this finding requires user input.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub needs_input: Option<bool>,
    /// Question to pose to the user, if input is needed.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub question: Option<String>,
    /// Suggested response options for the user.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub input_options: Option<Vec<String>>,
}

/// Request payload for updating a finding. All fields are optional.
#[derive(Debug, Clone, Default, Serialize, Deserialize, JsonSchema)]
pub struct TaskRunFindingUpdate {
    /// New lifecycle status.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub status: Option<TaskRunFindingStatus>,
    /// Resolution text.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub resolution: Option<String>,
    /// Session number in which the finding was resolved.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub resolved_in_session: Option<u64>,
    /// ISO 8601 timestamp of resolution.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub resolved_at: Option<String>,
    /// User response, if the finding needed input.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub user_response: Option<String>,
}

// ============================================================================
// Runner-Specific Request Types
// ============================================================================

/// Inline `data` payload on a [`RunPromptResponse`].
#[derive(Debug, Clone, Default, Serialize, Deserialize, JsonSchema)]
pub struct RunPromptResponseData {
    /// AI output text.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub output: Option<String>,
    /// Final response text.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub response: Option<String>,
}

/// Response from the runner's `run_prompt` endpoint.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct RunPromptResponse {
    /// Whether the prompt was accepted and started successfully.
    pub success: bool,
    /// ID of the created task run, if any.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub task_run_id: Option<String>,
    /// ID of the created AI session, if any.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub session_id: Option<String>,
    /// Path to the state file tracking the session.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub state_file: Option<String>,
    /// Path to the log file for the session.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub log_file: Option<String>,
    /// OS process ID of the spawned AI session, if any.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub pid: Option<u64>,
    /// Error message if the call failed.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
    /// Immediate output if the call ran synchronously.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub output: Option<String>,
    /// Structured data payload from a synchronous prompt run.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub data: Option<RunPromptResponseData>,
}

/// Request body for the runner's `run_prompt` endpoint.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct RunPromptRequest {
    /// Display name for the task.
    pub name: String,
    /// Prompt content (the actual text sent to the AI).
    pub content: String,
    /// Optional cap on AI sessions.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max_sessions: Option<u64>,
    /// Display-only version of the prompt (shown in the UI).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub display_prompt: Option<String>,
    /// Hard timeout in seconds.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub timeout_seconds: Option<u64>,
    /// Optional free-form context string appended to the prompt.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub context: Option<String>,
    /// Attached image paths.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub image_paths: Option<Vec<String>>,
    /// Attached video paths.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub video_paths: Option<Vec<String>>,
    /// Optional path to a trace file.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub trace_path: Option<String>,
    /// Cap on video frames to extract for the prompt.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max_video_frames: Option<u64>,
    /// Cap on trace screenshots to include.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max_trace_screenshots: Option<u64>,
}

/// Request body for creating a task run (simplified shape used by the runner's
/// create-task endpoint).
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct CreateTaskRunRequest {
    /// Display name.
    pub task_name: String,
    /// Prompt text, if applicable.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub prompt: Option<String>,
    /// Task type.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub task_type: Option<TaskType>,
    /// Workflow config ID to use.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub config_id: Option<String>,
    /// Workflow name to use.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub workflow_name: Option<String>,
    /// Optional cap on AI sessions.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max_sessions: Option<u64>,
    /// Whether the task should auto-continue.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub auto_continue: Option<bool>,
    /// Serialized execution steps, if provided ad-hoc.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub execution_steps_json: Option<String>,
    /// Serialized log-sources configuration.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub log_sources_json: Option<String>,
}

// ============================================================================
// Filter Types
// ============================================================================

/// Filter parameters for listing task runs.
#[derive(Debug, Clone, Default, Serialize, Deserialize, JsonSchema)]
pub struct TaskRunFilters {
    /// Restrict to a given project.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub project_id: Option<String>,
    /// Restrict to a given status.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub status: Option<TaskRunStatus>,
    /// Include only runs after this ISO 8601 timestamp.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub start_date: Option<String>,
    /// Include only runs before this ISO 8601 timestamp.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub end_date: Option<String>,
    /// Pagination offset.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub offset: Option<u64>,
    /// Pagination limit.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub limit: Option<u64>,
}

/// Filter parameters for listing findings.
#[derive(Debug, Clone, Default, Serialize, Deserialize, JsonSchema)]
pub struct TaskRunFindingFilters {
    /// Restrict to a given category.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub category: Option<TaskRunFindingCategory>,
    /// Restrict to a given severity.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub severity: Option<TaskRunFindingSeverity>,
    /// Restrict to a given status.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub status: Option<TaskRunFindingStatus>,
}

// ============================================================================
// Response Types
// ============================================================================

/// Pagination envelope attached to list responses.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct Pagination {
    /// Total number of matching records.
    pub total: u64,
    /// Maximum number of records returned per page.
    pub limit: u64,
    /// Offset into the full result set.
    pub offset: u64,
    /// Whether additional records are available after this page.
    pub has_more: bool,
}

/// Response for `GET /task-runs`.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct TaskRunListResponse {
    /// Page of matching task runs.
    pub tasks: Vec<TaskRunBackend>,
    /// Pagination envelope.
    pub pagination: Pagination,
}

/// Response for `GET /task-runs/{id}/findings`.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct TaskRunFindingsListResponse {
    /// Findings for the task run.
    pub findings: Vec<TaskRunFinding>,
    /// Aggregated counts of findings.
    pub summary: TaskRunFindingSummary,
}

/// Compact findings summary including the most recent findings.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct FindingsSummary {
    /// Total number of findings.
    pub total: u64,
    /// Count by severity.
    pub by_severity: HashMap<String, u64>,
    /// Count by category.
    pub by_category: HashMap<String, u64>,
    /// Count by status.
    pub by_status: HashMap<String, u64>,
    /// Most recent findings.
    pub recent: Vec<TaskRunFindingResponse>,
}

// ============================================================================
// Verification Result Types
// ============================================================================

/// A specific issue detail from an individual verification check.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct CheckIssueDetail {
    /// File path where the issue was detected.
    pub file: String,
    /// Line number, if applicable.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub line: Option<i64>,
    /// Column number, if applicable.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub column: Option<i64>,
    /// Error code or lint rule, if applicable.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub code: Option<String>,
    /// Human-readable message.
    pub message: String,
    /// Severity label (free-form string from the upstream check).
    pub severity: String,
    /// Whether the check can auto-fix this issue.
    pub fixable: bool,
}

/// Result of a single named verification check (e.g., `"eslint"`, `"mypy"`).
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct IndividualCheckResult {
    /// Name of the check.
    pub name: String,
    /// Free-form status string (e.g., `"passed"`, `"failed"`, `"skipped"`).
    pub status: String,
    /// How long the check took, in milliseconds.
    pub duration_ms: u64,
    /// Number of issues surfaced by this check.
    pub issues_found: u64,
    /// Number of issues auto-fixed by this check.
    pub issues_fixed: u64,
    /// Number of files the check inspected.
    pub files_checked: u64,
    /// Error message, if the check itself failed to run.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub error_message: Option<String>,
    /// Raw check output, if captured.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub output: Option<String>,
    /// Specific issue details.
    #[serde(default)]
    pub issues: Vec<CheckIssueDetail>,
}

/// Detailed output captured for a single verification step.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct VerificationStepDetails {
    /// ID of the step this detail belongs to.
    pub step_id: String,
    /// Phase the step belongs to (e.g., `"setup"`, `"verification"`).
    pub phase: String,
    /// Captured stdout, if any.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub stdout: Option<String>,
    /// Captured stderr, if any.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub stderr: Option<String>,
    /// Number of assertions that passed.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub assertions_passed: Option<u64>,
    /// Total number of assertions.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub assertions_total: Option<u64>,
    /// Captured browser/console output.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub console_output: Option<String>,
    /// Captured page snapshot (HTML or serialized representation).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub page_snapshot: Option<String>,
    /// Exit code of the spawned process.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub exit_code: Option<i64>,
    /// Results of individual named checks (e.g., lint, type, test).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub check_results: Option<Vec<IndividualCheckResult>>,
}

/// Execution config captured for a verification step.
///
/// The TypeScript type includes an index signature `[key: string]: unknown`,
/// so extra arbitrary fields are captured in `extra` via `serde(flatten)` and
/// passed through opaquely.
#[derive(Debug, Clone, Default, Serialize, Deserialize, JsonSchema)]
pub struct StepExecutionConfig {
    /// Action type (e.g., click, type, wait).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub action_type: Option<String>,
    /// Target image ID, if the action references an image.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub target_image_id: Option<String>,
    /// Target image name, if the action references an image.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub target_image_name: Option<String>,
    /// Check type for verification steps.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub check_type: Option<String>,
    /// Timeout in seconds.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub timeout_seconds: Option<u64>,
    /// Additional step-specific configuration fields. Mirrors the TS index
    /// signature `[key: string]: unknown`.
    #[serde(flatten)]
    pub extra: HashMap<String, Value>,
}

/// Result of a single step within a verification phase.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct VerificationStepResult {
    /// Zero-based index of the step within the phase.
    pub step_index: u64,
    /// Free-form step type label.
    pub step_type: String,
    /// Display name of the step.
    pub step_name: String,
    /// ID of the step, if assigned.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub step_id: Option<String>,
    /// Whether the step succeeded.
    pub success: bool,
    /// Error message if the step failed.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
    /// Path to a screenshot captured for the step.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub screenshot_path: Option<String>,
    /// ISO 8601 timestamp when the step started.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub started_at: Option<String>,
    /// ISO 8601 timestamp when the step ended.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ended_at: Option<String>,
    /// Step duration in milliseconds.
    pub duration_ms: u64,
    /// Execution config for the step.
    pub config: StepExecutionConfig,
    /// Detailed captured output, if any.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub verification_details: Option<VerificationStepDetails>,
    /// Arbitrary structured output produced by the step.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub output_data: Option<HashMap<String, Value>>,
}

/// Result of evaluating a named gate across a set of steps.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct GateEvaluationResult {
    /// Name of the gate.
    pub gate_name: String,
    /// IDs of steps the gate required.
    #[serde(default)]
    pub required_step_ids: Vec<String>,
    /// IDs of required steps that passed.
    #[serde(default)]
    pub passed_step_ids: Vec<String>,
    /// IDs of required steps that failed.
    #[serde(default)]
    pub failed_step_ids: Vec<String>,
    /// IDs of required steps that were missing.
    #[serde(default)]
    pub missing_step_ids: Vec<String>,
    /// Whether the gate passed overall.
    pub passed: bool,
}

/// Result of a single iteration of the verification phase.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct VerificationPhaseResult {
    /// 1-based iteration index within the workflow run.
    pub iteration: u64,
    /// Whether all steps passed.
    pub all_passed: bool,
    /// Total number of steps executed.
    pub total_steps: u64,
    /// Number of steps that passed.
    pub passed_steps: u64,
    /// Number of steps that failed.
    pub failed_steps: u64,
    /// Number of steps that were skipped.
    pub skipped_steps: u64,
    /// Total duration of the phase in milliseconds.
    pub total_duration_ms: u64,
    /// Per-step results.
    #[serde(default)]
    pub step_results: Vec<VerificationStepResult>,
    /// Whether a critical step failure short-circuited the phase.
    pub critical_failure: bool,
    /// Per-gate evaluation results.
    #[serde(default)]
    pub gate_results: Vec<GateEvaluationResult>,
    /// Whether pass/fail is determined by gates rather than overall step counts.
    pub gate_based_evaluation: bool,
}

/// Response record for a single stored verification result.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct VerificationResultResponse {
    /// Unique identifier (UUID v4 string).
    pub id: String,
    /// ID of the owning task run.
    pub task_run_id: String,
    /// 1-based iteration index within the workflow run.
    pub iteration: u64,
    /// Whether all steps passed.
    pub all_passed: bool,
    /// Total number of steps executed.
    pub total_steps: u64,
    /// Number of steps that passed.
    pub passed_steps: u64,
    /// Number of steps that failed.
    pub failed_steps: u64,
    /// Number of steps that were skipped.
    pub skipped_steps: u64,
    /// Total duration of the phase in milliseconds.
    pub total_duration_ms: u64,
    /// Whether a critical step failure short-circuited the phase.
    pub critical_failure: bool,
    /// Full typed verification result payload.
    pub result_json: VerificationPhaseResult,
    /// ISO 8601 timestamp when the record was created.
    pub created_at: String,
}

/// Response for listing verification results for a task run.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct VerificationResultsListResponse {
    /// ID of the owning task run.
    pub task_run_id: String,
    /// All verification results for the task run.
    #[serde(default)]
    pub results: Vec<VerificationResultResponse>,
    /// Total number of verification results.
    pub count: u64,
    /// Number of iterations that passed.
    pub passed_iterations: u64,
    /// Number of iterations that failed.
    pub failed_iterations: u64,
}
