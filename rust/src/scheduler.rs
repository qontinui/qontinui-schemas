//! Scheduler DTO types.
//!
//! Wire-format types for the CI/CD scheduler: schedule expressions, task
//! definitions, execution records, and the API request/response envelopes used
//! to create and update scheduled tasks.
//!
//! These types are a port of the shape-bearing portion of
//! `qontinui-runner/src-tauri/src/scheduler.rs`. Behavior (constructors,
//! condition evaluation, rearm checks, next-run computation) stays in the
//! runner — this crate is data-only.

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

// ============================================================================
// Schedule Expression Types
// ============================================================================

/// How a task should be scheduled.
///
/// Serialized with the external tag `type` and payload under `value` so that
/// `Once("...")`, `Cron("...")`, and `Interval(60)` round-trip as
/// `{ "type": "Once", "value": "..." }` etc. The `Condition` variant wraps a
/// [`ConditionScheduleConfig`] rather than a scalar, but uses the same
/// `{ type, value }` envelope.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "type", content = "value")]
pub enum ScheduleExpression {
    /// Run once at a specific datetime (ISO 8601).
    Once(String),
    /// Cron expression (e.g., `"0 9 * * *"` for 9 AM daily). The runner
    /// accepts both 5-field and 6/7-field cron forms and normalizes internally.
    Cron(String),
    /// Interval in seconds between runs (for testing/debugging).
    Interval(u64),
    /// Condition-only schedule: no time trigger, runs whenever the attached
    /// [`ScheduleConditions`] on the task are met.
    Condition(ConditionScheduleConfig),
}

/// Configuration for condition-only schedules.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
#[schemars(deny_unknown_fields)]
pub struct ConditionScheduleConfig {
    /// Minutes to wait after an execution completes before re-evaluating
    /// conditions for another run.
    #[serde(default = "default_rearm_delay", alias = "rearm_delay_minutes")]
    pub rearm_delay_minutes: u32,
}

impl Default for ConditionScheduleConfig {
    fn default() -> Self {
        Self {
            rearm_delay_minutes: default_rearm_delay(),
        }
    }
}

fn default_rearm_delay() -> u32 {
    60
}

// ============================================================================
// Schedule Conditions
// ============================================================================

/// Condition that requires the runner to be idle (not executing workflows or
/// AI tasks) before the task may run.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, Default)]
#[serde(rename_all = "camelCase")]
#[schemars(deny_unknown_fields)]
pub struct IdleCondition {
    /// Whether this condition is active.
    #[serde(alias = "enabled")]
    pub enabled: bool,
}

/// A single repository to monitor for inactivity.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
#[schemars(deny_unknown_fields)]
pub struct RepositoryWatch {
    /// Path to the repository directory.
    #[serde(alias = "path")]
    pub path: String,
    /// Minutes of inactivity required before the watch is considered met.
    #[serde(alias = "inactive_minutes")]
    pub inactive_minutes: u32,
}

/// Condition that requires repositories to have no file modifications for a
/// period. ALL configured repositories must be inactive for the overall
/// condition to be met.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, Default)]
#[serde(rename_all = "camelCase")]
#[schemars(deny_unknown_fields)]
pub struct RepositoryInactiveCondition {
    /// Whether this condition is active.
    #[serde(alias = "enabled")]
    pub enabled: bool,
    /// List of repositories to watch. ALL must be inactive simultaneously.
    #[serde(default, alias = "repositories")]
    pub repositories: Vec<RepositoryWatch>,
}

/// Conditions that must ALL be met before task execution.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, Default)]
#[serde(rename_all = "camelCase")]
#[schemars(deny_unknown_fields)]
pub struct ScheduleConditions {
    /// Require the runner to be idle.
    #[serde(default, skip_serializing_if = "Option::is_none", alias = "require_idle")]
    pub require_idle: Option<IdleCondition>,
    /// Require repository file inactivity across one or more paths.
    #[serde(default, skip_serializing_if = "Option::is_none", alias = "require_repo_inactive")]
    pub require_repo_inactive: Option<RepositoryInactiveCondition>,
    /// Maximum time to wait for conditions (minutes). `None` = wait
    /// indefinitely.
    #[serde(default, skip_serializing_if = "Option::is_none", alias = "timeout_minutes")]
    pub timeout_minutes: Option<u32>,
}

/// Status of condition checking for a deferred task.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
#[schemars(deny_unknown_fields)]
pub struct ConditionStatus {
    /// ISO 8601 timestamp when conditions began being evaluated.
    #[serde(alias = "waiting_since")]
    pub waiting_since: String,
    /// Current idle-condition result. `None` if not yet checked,
    /// `Some(true)` if idle, `Some(false)` if busy.
    #[serde(default, skip_serializing_if = "Option::is_none", alias = "idle_met")]
    pub idle_met: Option<bool>,
    /// Current repository-inactive status per repository: `(path, is_inactive)`.
    #[serde(default, skip_serializing_if = "Option::is_none", alias = "repo_inactive_met")]
    pub repo_inactive_met: Option<Vec<(String, bool)>>,
    /// Whether the overall condition-wait timeout has been exceeded.
    #[serde(default, alias = "timed_out")]
    pub timed_out: bool,
}

// ============================================================================
// Task Type Definitions
// ============================================================================

/// Type of task to schedule.
///
/// Internally tagged by `task_type`: the variant fields are inlined alongside
/// the discriminator rather than nested under a `value` key.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "task_type")]
pub enum ScheduledTaskType {
    /// Run a workflow — either a legacy workflow-by-name from a loaded config
    /// or a unified workflow by ID.
    Workflow {
        /// Display name (also used to look up legacy workflows).
        workflow_name: String,
        /// Optional path to a workflow config file.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        config_path: Option<String>,
        /// Optional monitor index to target.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        monitor_index: Option<i32>,
        /// If set, run a unified workflow by ID instead of a legacy workflow
        /// by name.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        workflow_id: Option<String>,
    },
    /// Run a prompt from the Prompt Library.
    Prompt {
        /// ID of the prompt to run.
        prompt_id: String,
        /// Optional override for `max_sessions`. `None` uses the prompt's own
        /// setting.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        max_sessions: Option<u32>,
    },
    /// Trigger auto-fix: check findings and fix any auto-fixable items.
    AutoFix {
        /// Whether to check the findings queue before running.
        #[serde(default = "default_true")]
        check_findings: bool,
        /// Force a run even if no findings are present.
        #[serde(default)]
        force_run: bool,
    },
    /// Execute a watcher (screenpipe-inspired reactive AI agent). Queries the
    /// activity timeline, reasons with AI, and triggers an action.
    Watcher {
        /// ID of the watcher definition in PostgreSQL.
        watcher_id: String,
    },
    /// Continuous background capture (screenpipe-style). Periodically
    /// captures screen state and stores it in the activity timeline.
    BackgroundCapture {
        /// Optional monitor index to capture.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        monitor_index: Option<i32>,
        /// Seconds between successive captures.
        #[serde(default = "default_capture_interval")]
        capture_interval_secs: u64,
        /// Whether to also trigger a capture on window focus change.
        #[serde(default = "default_true")]
        capture_on_focus_change: bool,
    },
}

fn default_true() -> bool {
    true
}

fn default_capture_interval() -> u64 {
    30
}

// ============================================================================
// Task Status
// ============================================================================

/// Status of a scheduled task execution.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq, Default)]
#[serde(rename_all = "snake_case")]
pub enum ScheduledTaskStatus {
    /// Scheduled and waiting to run.
    #[default]
    Pending,
    /// Currently executing.
    Running,
    /// Finished successfully.
    Completed,
    /// Finished with an error.
    Failed,
    /// Skipped because the task had already completed successfully and
    /// `skip_if_completed` is `true`.
    Skipped,
    /// Cancelled before or during execution.
    Cancelled,
}

// ============================================================================
// Execution Record
// ============================================================================

/// Record of a single task execution.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
#[schemars(deny_unknown_fields)]
pub struct TaskExecutionRecord {
    /// Unique ID for this execution (UUID v4 string).
    #[serde(alias = "execution_id")]
    pub execution_id: String,
    /// Session ID if this execution triggered an AI session, used for
    /// downstream success tracking.
    #[serde(default, skip_serializing_if = "Option::is_none", alias = "session_id")]
    pub session_id: Option<String>,
    /// ISO 8601 timestamp when execution started.
    #[serde(alias = "started_at")]
    pub started_at: String,
    /// ISO 8601 timestamp when execution ended.
    #[serde(default, skip_serializing_if = "Option::is_none", alias = "ended_at")]
    pub ended_at: Option<String>,
    /// Current status of this execution.
    #[serde(alias = "status")]
    pub status: ScheduledTaskStatus,
    /// Whether the task succeeded, read from the session checkpoint.
    #[serde(default, alias = "success")]
    pub success: bool,
    /// Error message if the execution failed.
    #[serde(default, skip_serializing_if = "Option::is_none", alias = "error_message")]
    pub error_message: Option<String>,
    /// Whether auto-fix was triggered after this execution.
    #[serde(default, alias = "triggered_auto_fix")]
    pub triggered_auto_fix: bool,
    /// Session ID of the auto-fix session, if one was triggered.
    #[serde(default, skip_serializing_if = "Option::is_none", alias = "auto_fix_session_id")]
    pub auto_fix_session_id: Option<String>,
}

// ============================================================================
// Scheduled Task
// ============================================================================

/// A scheduled task definition — the full persisted frame, including computed
/// fields (`last_run`, `next_run`) and condition-evaluation state
/// (`conditions`, `condition_status`).
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
#[schemars(deny_unknown_fields)]
pub struct ScheduledTask {
    /// Unique identifier (UUID v4 string).
    #[serde(alias = "id")]
    pub id: String,
    /// Display name for the task.
    #[serde(alias = "name")]
    pub name: String,
    /// Optional human-readable description.
    #[serde(default, skip_serializing_if = "Option::is_none", alias = "description")]
    pub description: Option<String>,
    /// Whether the task is enabled and eligible to run.
    #[serde(default = "default_true", alias = "enabled")]
    pub enabled: bool,
    /// Schedule configuration.
    #[serde(alias = "schedule")]
    pub schedule: ScheduleExpression,
    /// Task type and its per-type configuration.
    #[serde(alias = "task")]
    pub task: ScheduledTaskType,
    /// Skip future runs once the task has succeeded at least once.
    #[serde(default, alias = "skip_if_completed")]
    pub skip_if_completed: bool,
    /// Automatically trigger auto-fix when this task fails.
    #[serde(default, alias = "auto_fix_on_failure")]
    pub auto_fix_on_failure: bool,
    /// Free-form description of success criteria, for human reference.
    #[serde(default, skip_serializing_if = "Option::is_none", alias = "success_criteria")]
    pub success_criteria: Option<String>,
    /// ISO 8601 timestamp of creation.
    #[serde(alias = "created_at")]
    pub created_at: String,
    /// ISO 8601 timestamp of last modification.
    #[serde(alias = "modified_at")]
    pub modified_at: String,
    /// Record of the most recent execution.
    #[serde(default, skip_serializing_if = "Option::is_none", alias = "last_run")]
    pub last_run: Option<TaskExecutionRecord>,
    /// Next scheduled run time (ISO 8601), computed by the runner.
    #[serde(default, skip_serializing_if = "Option::is_none", alias = "next_run")]
    pub next_run: Option<String>,
    /// Optional conditions that must be met before execution.
    #[serde(default, skip_serializing_if = "Option::is_none", alias = "conditions")]
    pub conditions: Option<ScheduleConditions>,
    /// Present while the task is waiting for its conditions to be met.
    #[serde(default, skip_serializing_if = "Option::is_none", alias = "condition_status")]
    pub condition_status: Option<ConditionStatus>,
}

// ============================================================================
// Scheduler Settings
// ============================================================================

/// Global scheduler settings.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
#[schemars(deny_unknown_fields)]
pub struct SchedulerSettings {
    /// Whether the scheduler is enabled globally.
    #[serde(default = "default_true", alias = "enabled")]
    pub enabled: bool,
    /// Maximum number of scheduled tasks allowed to run concurrently.
    #[serde(default = "default_max_concurrent", alias = "max_concurrent")]
    pub max_concurrent: u32,
    /// Default `auto_fix_on_failure` value for newly created tasks.
    #[serde(default, alias = "default_auto_fix_on_failure")]
    pub default_auto_fix_on_failure: bool,
    /// Timezone for schedule interpretation (IANA name). `None` = local time.
    #[serde(default, skip_serializing_if = "Option::is_none", alias = "timezone")]
    pub timezone: Option<String>,
}

fn default_max_concurrent() -> u32 {
    1
}

impl Default for SchedulerSettings {
    fn default() -> Self {
        Self {
            enabled: true,
            max_concurrent: default_max_concurrent(),
            default_auto_fix_on_failure: false,
            timezone: None,
        }
    }
}

// ============================================================================
// Scheduler Status (API responses)
// ============================================================================

/// Summary of the scheduler's current runtime state, returned from the status
/// API.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
#[schemars(deny_unknown_fields)]
pub struct SchedulerStatus {
    /// Whether the scheduler is enabled.
    #[serde(alias = "enabled")]
    pub enabled: bool,
    /// Number of tasks currently running.
    #[serde(alias = "running_tasks")]
    pub running_tasks: u32,
    /// Number of tasks scheduled but not yet running.
    #[serde(alias = "pending_tasks")]
    pub pending_tasks: u32,
    /// The next task scheduled to run, if any.
    #[serde(default, skip_serializing_if = "Option::is_none", alias = "next_task")]
    pub next_task: Option<NextTaskInfo>,
}

/// Minimal description of the next task due to run.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
#[schemars(deny_unknown_fields)]
pub struct NextTaskInfo {
    /// Task ID.
    #[serde(alias = "id")]
    pub id: String,
    /// Task display name.
    #[serde(alias = "name")]
    pub name: String,
    /// ISO 8601 timestamp of the next scheduled run.
    #[serde(alias = "next_run")]
    pub next_run: String,
}

// ============================================================================
// API Request/Response Types
// ============================================================================

/// Request body for creating a new scheduled task.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
#[schemars(deny_unknown_fields)]
pub struct CreateScheduledTaskRequest {
    /// Display name.
    #[serde(alias = "name")]
    pub name: String,
    /// Optional description.
    #[serde(default, skip_serializing_if = "Option::is_none", alias = "description")]
    pub description: Option<String>,
    /// Schedule configuration.
    #[serde(alias = "schedule")]
    pub schedule: ScheduleExpression,
    /// Task type and its per-type configuration.
    #[serde(alias = "task")]
    pub task: ScheduledTaskType,
    /// Skip future runs once the task has succeeded.
    #[serde(default, skip_serializing_if = "Option::is_none", alias = "skip_if_completed")]
    pub skip_if_completed: Option<bool>,
    /// Trigger auto-fix on failure.
    #[serde(default, skip_serializing_if = "Option::is_none", alias = "auto_fix_on_failure")]
    pub auto_fix_on_failure: Option<bool>,
    /// Free-form success criteria description.
    #[serde(default, skip_serializing_if = "Option::is_none", alias = "success_criteria")]
    pub success_criteria: Option<String>,
    /// Optional conditions that must be met before execution.
    #[serde(default, skip_serializing_if = "Option::is_none", alias = "conditions")]
    pub conditions: Option<ScheduleConditions>,
}

/// Request body for updating an existing scheduled task. All fields are
/// optional; only those supplied are applied.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, Default)]
#[serde(rename_all = "camelCase")]
#[schemars(deny_unknown_fields)]
pub struct UpdateScheduledTaskRequest {
    /// New display name.
    #[serde(default, skip_serializing_if = "Option::is_none", alias = "name")]
    pub name: Option<String>,
    /// New description (pass `null` to clear).
    #[serde(default, skip_serializing_if = "Option::is_none", alias = "description")]
    pub description: Option<String>,
    /// Enable/disable the task.
    #[serde(default, skip_serializing_if = "Option::is_none", alias = "enabled")]
    pub enabled: Option<bool>,
    /// Replace the schedule expression.
    #[serde(default, skip_serializing_if = "Option::is_none", alias = "schedule")]
    pub schedule: Option<ScheduleExpression>,
    /// Replace the task definition.
    #[serde(default, skip_serializing_if = "Option::is_none", alias = "task")]
    pub task: Option<ScheduledTaskType>,
    /// Update `skip_if_completed`.
    #[serde(default, skip_serializing_if = "Option::is_none", alias = "skip_if_completed")]
    pub skip_if_completed: Option<bool>,
    /// Update `auto_fix_on_failure`.
    #[serde(default, skip_serializing_if = "Option::is_none", alias = "auto_fix_on_failure")]
    pub auto_fix_on_failure: Option<bool>,
    /// Update the success criteria (pass `null` to clear).
    #[serde(default, skip_serializing_if = "Option::is_none", alias = "success_criteria")]
    pub success_criteria: Option<String>,
    /// Replace the conditions block (pass `null` to clear).
    #[serde(default, skip_serializing_if = "Option::is_none", alias = "conditions")]
    pub conditions: Option<ScheduleConditions>,
}
