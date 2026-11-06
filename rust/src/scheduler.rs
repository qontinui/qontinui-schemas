//! Scheduler DTO types.
//!
//! Wire-format types for the CI/CD scheduler: schedule expressions, task
//! definitions, execution records, and the API request/response envelopes used
//! to create and update scheduled tasks.
//!
//! These types are a port of the shape-bearing portion of
//! `qontinui-runner/src-tauri/src/scheduler.rs`. Behavior (constructors,
//! condition evaluation, rearm checks, next-run computation) stays in the
//! runner — this crate is data-only, apart from the shape invariants the
//! recurring-agent types check in their `validate()` methods.
//!
//! The recurring-agent section at the end of this file is the wire contract of
//! plan `2026-10-29-recurring-agents-declared-in-coord-reconciled-by-runner`:
//! coord serves [`RecurringAgentSpec`]s, each runner reconciles them into
//! managed [`ScheduledTask`]s (see [`ManagedBy`]) and reports back a
//! [`RecurringAgentStatusReport`]. Those types use snake_case field names,
//! mirroring the `coord.recurring_agents*` columns they are stored in.

use std::collections::{BTreeSet, HashSet};

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

/// Condition that runs an external command and is met iff it exits 0.
///
/// The runner execs `command` directly — argv, never a shell — so `command[0]`
/// is the program and the rest are its arguments; wrap it in `sh -c` yourself
/// if you want shell syntax. The condition is **NOT met** on a non-zero exit,
/// on a timeout (the probe is killed at `timeout_seconds`) and on a spawn
/// failure (including an empty `command`); each is logged distinctly. An
/// unobserved or failed probe never reads as met.
///
/// The probe is rate-limited to one run per `poll_seconds` per task: between
/// polls the runner reuses the last result. `poll_seconds` is floored at 60
/// because the scheduler re-evaluates conditions once per 60 s tick — a shorter
/// interval could not be honoured. The runner's create/update API REFUSES a
/// value below the floor (and an empty `command`, and a `timeout_seconds` of
/// 0 or above 3600) rather than silently rewriting it; a stored row that
/// predates that validation is clamped up to 60 at evaluation time.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
#[schemars(deny_unknown_fields)]
pub struct ProbeCondition {
    /// Whether this condition is active.
    #[serde(alias = "enabled")]
    pub enabled: bool,
    /// Program and arguments to exec (no shell). Must be non-empty.
    #[serde(alias = "command")]
    pub command: Vec<String>,
    /// Minimum seconds between two probe runs for the same task. Minimum 60.
    #[serde(alias = "poll_seconds")]
    #[schemars(range(min = 60))]
    pub poll_seconds: u32,
    /// Seconds a single probe run may take before it is killed and counted as
    /// NOT met. 1..=3600.
    #[serde(alias = "timeout_seconds")]
    #[schemars(range(min = 1, max = 3600))]
    pub timeout_seconds: u32,
}

/// Conditions that must ALL be met before task execution.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, Default)]
#[serde(rename_all = "camelCase")]
#[schemars(deny_unknown_fields)]
pub struct ScheduleConditions {
    /// Require the runner to be idle.
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        alias = "require_idle"
    )]
    pub require_idle: Option<IdleCondition>,
    /// Require repository file inactivity across one or more paths.
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        alias = "require_repo_inactive"
    )]
    pub require_repo_inactive: Option<RepositoryInactiveCondition>,
    /// Require an external probe command to exit 0 (see [`ProbeCondition`]).
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        alias = "require_probe"
    )]
    pub require_probe: Option<ProbeCondition>,
    /// Maximum time to wait for conditions (minutes). `None` = wait
    /// indefinitely.
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        alias = "timeout_minutes"
    )]
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
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        alias = "repo_inactive_met"
    )]
    pub repo_inactive_met: Option<Vec<(String, bool)>>,
    /// Current probe-condition result. `None` if no probe is configured or
    /// none has been evaluated yet, `Some(true)` if the last probe exited 0.
    #[serde(default, skip_serializing_if = "Option::is_none", alias = "probe_met")]
    pub probe_met: Option<bool>,
    /// Human-readable outcome of the last probe evaluation — e.g. `exit 0`,
    /// `exit 3`, `timed out after 30s`, `spawn failed: ...`, `probe running;
    /// awaiting its result`, `not run: another condition is not met` — with a
    /// bounded stderr tail appended when the probe wrote one.
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        alias = "probe_detail"
    )]
    pub probe_detail: Option<String>,
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
    /// Run an ad-hoc Claude agent prompt without pre-registering it in the
    /// Prompt Library. The runner launches it as one headless `claude`
    /// session through its own spawn seam; the recorded outcome is that
    /// session's exit status (success iff exit code 0).
    RemoteAgent {
        /// The prompt content to send to the Claude session.
        prompt: String,
        /// Working directory for the spawned session. `None` means the
        /// runner's project root. Stored as a string (not `PathBuf`) to keep
        /// this crate JSON-Schema-friendly.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        working_directory: Option<String>,
        /// Allowed tool names (e.g. `["Bash", "Read", "Write", "Edit"]`).
        /// Empty = use the runner's default tool allow-list.
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        allowed_tools: Vec<String>,
        /// Optional model override. `None` = runner default
        /// (typically `claude-sonnet-4-6`).
        #[serde(default, skip_serializing_if = "Option::is_none")]
        model: Option<String>,
        /// Optional reasoning-effort level for the spawned session: one of
        /// `low`, `medium`, `high`, `xhigh` or `max`. `None` leaves the level
        /// to the runner. A plain string rather than an enum, so a stored
        /// task carrying a level this crate does not list still loads; the
        /// consumer validates it.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        effort: Option<String>,
        /// Optional ultracode switch (standing dynamic-workflow orchestration)
        /// for the spawned session. `Some(true)` turns it on, `Some(false)`
        /// keeps it off, and `None` leaves the choice to the runner.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        ultracode: Option<bool>,
        /// Optional kebab-case name of a spawn-effort profile (e.g.
        /// `research`, `implementation`) that the runner resolves to an
        /// effort level and an ultracode setting. `None` leaves the choice
        /// to the runner.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        effort_profile: Option<String>,
        /// Optional MCP connection references resolved at dispatch time
        /// against the runner's existing MCP config. Empty = inherit
        /// whatever the runner currently has configured.
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        mcp_connections: Vec<McpConnectionRef>,
        /// Hard cap on Claude turns per run (safety bound). `None` =
        /// runner's default cap (typically 50).
        #[serde(default, skip_serializing_if = "Option::is_none")]
        max_turns: Option<u32>,
        /// Wall-clock timeout in seconds. `None` = runner default
        /// (typically 600s = 10 min).
        #[serde(default, skip_serializing_if = "Option::is_none")]
        timeout_seconds: Option<u64>,
    },
    /// Run a shell command in-binary (no Claude session). The recorded
    /// outcome is the command's exit status: success iff exit code 0; a
    /// timeout is a failure.
    Script {
        /// Shell command line, run via the platform shell (`sh -c`; on
        /// Windows `bash -c`, then `cmd /C` as the fallback).
        command: String,
        /// Working directory for the command. `None` = the runner's project
        /// root.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        working_directory: Option<String>,
        /// Wall-clock timeout in seconds. `None` (or 0) = runner default
        /// (600s = 10 min).
        #[serde(default, skip_serializing_if = "Option::is_none")]
        timeout_seconds: Option<u64>,
    },
}

/// Lightweight reference to an MCP connection for [`ScheduledTaskType::RemoteAgent`].
///
/// `name` is the MCP server name as registered in the runner's MCP config.
/// `url` is an optional override; when omitted the runner resolves the URL
/// from its existing MCP config at dispatch time.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
#[schemars(deny_unknown_fields)]
pub struct McpConnectionRef {
    /// MCP server name as registered in the runner's MCP config.
    pub name: String,
    /// Optional URL override; falls back to the runner's MCP config when
    /// `None`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
}

/// Catch-up policy applied when the runner restarts after missing one or
/// more scheduled slots.
#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum CatchUpPolicy {
    /// Run the missed slot once for each occurrence that was skipped.
    Run,
    /// Skip all missed slots; emit `MissedRunnerDown` history rows for
    /// audit but do not execute.
    Skip,
    /// Collapse multiple missed slots into a single catch-up run.
    #[default]
    RunOnce,
}

fn default_catch_up_policy() -> CatchUpPolicy {
    CatchUpPolicy::RunOnce
}

fn default_catch_up_grace_seconds() -> u32 {
    300
}

fn default_consecutive_launch_failures() -> u32 {
    0
}

fn default_launch_failure_backoff_seconds() -> u32 {
    60
}

fn default_true() -> bool {
    true
}

fn default_catch_up_run() -> bool {
    false
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
    /// Finished with an error after the task had started running.
    Failed,
    /// The task could not even start (workflow file missing, prompt resolve
    /// failure, ai_session spawn error, etc.). Distinct from `Failed` because
    /// it drives the launch-failure backoff in
    /// [`ScheduledTask::launch_failure_backoff_seconds`].
    LaunchFailed,
    /// Skipped because the task had already completed successfully and
    /// `skip_if_completed` is `true`.
    Skipped,
    /// Cancelled before or during execution.
    Cancelled,
    /// Slot detected as missed during catch-up reconciliation while the
    /// runner was offline. Recorded as a history row when
    /// [`CatchUpPolicy::Skip`] is in effect; never returned for currently
    /// queued tasks.
    MissedRunnerDown,
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
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        alias = "error_message"
    )]
    pub error_message: Option<String>,
    /// Whether auto-fix was triggered after this execution.
    #[serde(default, alias = "triggered_auto_fix")]
    pub triggered_auto_fix: bool,
    /// Session ID of the auto-fix session, if one was triggered.
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        alias = "auto_fix_session_id"
    )]
    pub auto_fix_session_id: Option<String>,
    /// ISO 8601 timestamp of the *scheduled* slot this execution covers,
    /// distinct from `started_at` (which is the actual launch time). Allows
    /// detecting "ran X minutes late" and is consumed by the missed-run
    /// reconciler. Optional for backward compatibility with execution rows
    /// written before this field existed.
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        alias = "scheduled_for"
    )]
    pub scheduled_for: Option<String>,
    /// Whether this execution record was created by the catch-up reconciler
    /// (rather than the normal scheduling tick).
    #[serde(default = "default_catch_up_run", alias = "catch_up_run")]
    pub catch_up_run: bool,
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
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        alias = "description"
    )]
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
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        alias = "success_criteria"
    )]
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
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        alias = "condition_status"
    )]
    pub condition_status: Option<ConditionStatus>,
    /// Catch-up policy applied when the runner restarts after missing one
    /// or more scheduled slots.
    #[serde(default = "default_catch_up_policy", alias = "catch_up_policy")]
    pub catch_up_policy: CatchUpPolicy,
    /// Slots within this number of seconds of "now" are not treated as
    /// missed (the normal scheduler will pick them up). Default 300s.
    #[serde(
        default = "default_catch_up_grace_seconds",
        alias = "catch_up_grace_seconds"
    )]
    pub catch_up_grace_seconds: u32,
    /// Number of consecutive launch failures since the task last started
    /// successfully. Drives exponential backoff. Reset to 0 on the next
    /// successful launch.
    #[serde(
        default = "default_consecutive_launch_failures",
        alias = "consecutive_launch_failures"
    )]
    pub consecutive_launch_failures: u32,
    /// **Base** backoff in seconds for launch failures (not the current
    /// accumulated backoff). The runner computes the effective delay as
    /// `min(base * 2^(consecutive_launch_failures - 1), 86400)`.
    /// Default 60s.
    #[serde(
        default = "default_launch_failure_backoff_seconds",
        alias = "launch_failure_backoff_seconds"
    )]
    pub launch_failure_backoff_seconds: u32,
    /// IANA time zone this task's schedule is interpreted in (e.g.
    /// `Europe/Berlin`). Takes precedence over the global
    /// [`SchedulerSettings::timezone`]; `None` = the scheduler's zone. Lets one
    /// fleet-wide declaration mean either "local night on every device" or a
    /// fixed instant.
    #[serde(default, skip_serializing_if = "Option::is_none", alias = "timezone")]
    pub timezone: Option<String>,
    /// Set when the task was created by the runner's reconciler from a
    /// fleet-wide recurring-agent declaration rather than by a user. `None` =
    /// an unmanaged, user-created task, which the reconciler never touches.
    #[serde(default, skip_serializing_if = "Option::is_none", alias = "managed")]
    pub managed: Option<ManagedBy>,
}

/// The only [`ManagedBy::source`] value today: the task was reconciled from a
/// coord recurring-agent declaration.
pub const MANAGED_SOURCE_COORD: &str = "coord";

/// Provenance of a [`ScheduledTask`] that the runner reconciles from a
/// fleet-wide declaration (plan
/// `2026-10-29-recurring-agents-declared-in-coord-reconciled-by-runner`, D5).
///
/// `(source, tenant_id, agent)` is the managed key: the runner holds at most
/// one task per key. What the reconciler compares to decide whether a task is
/// current is `digest`, never `version` — a catalog fix in coord's code moves
/// no row version, and a deleted-and-recreated agent restarts its version at 1.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
#[schemars(deny_unknown_fields)]
pub struct ManagedBy {
    /// Which declaration store the task came from — [`MANAGED_SOURCE_COORD`].
    /// A plain string rather than an enum, so a stored task from a newer
    /// source still loads; the consumer decides what it recognises.
    #[serde(alias = "source")]
    pub source: String,
    /// The coord tenant (UUID) whose declaration this task applies. A device
    /// bound to several tenants holds one managed task per tenant and agent.
    #[serde(alias = "tenant_id")]
    pub tenant_id: String,
    /// The recurring agent's name — its join key across coord's status, run
    /// requests and this managed task (coord forbids renames).
    #[serde(alias = "agent")]
    pub agent: String,
    /// The declaration's row version when applied. History and display only.
    #[serde(alias = "version")]
    pub version: u32,
    /// sha256 (64 lowercase hex) of the resolved spec coord served when this
    /// task was applied. `None` = applied from a declaration that carried no
    /// digest.
    #[serde(default, skip_serializing_if = "Option::is_none", alias = "digest")]
    pub digest: Option<String>,
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
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        alias = "description"
    )]
    pub description: Option<String>,
    /// Schedule configuration.
    #[serde(alias = "schedule")]
    pub schedule: ScheduleExpression,
    /// Task type and its per-type configuration.
    #[serde(alias = "task")]
    pub task: ScheduledTaskType,
    /// Skip future runs once the task has succeeded.
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        alias = "skip_if_completed"
    )]
    pub skip_if_completed: Option<bool>,
    /// Trigger auto-fix on failure.
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        alias = "auto_fix_on_failure"
    )]
    pub auto_fix_on_failure: Option<bool>,
    /// Free-form success criteria description.
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        alias = "success_criteria"
    )]
    pub success_criteria: Option<String>,
    /// Optional conditions that must be met before execution.
    #[serde(default, skip_serializing_if = "Option::is_none", alias = "conditions")]
    pub conditions: Option<ScheduleConditions>,
    /// Optional catch-up policy override. `None` = runner default
    /// ([`CatchUpPolicy::RunOnce`]).
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        alias = "catch_up_policy"
    )]
    pub catch_up_policy: Option<CatchUpPolicy>,
    /// Optional catch-up grace window override (seconds). `None` =
    /// runner default (300s).
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        alias = "catch_up_grace_seconds"
    )]
    pub catch_up_grace_seconds: Option<u32>,
    /// Optional override for the launch-failure backoff base (seconds).
    /// `None` = runner default (60s).
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        alias = "launch_failure_backoff_seconds"
    )]
    pub launch_failure_backoff_seconds: Option<u32>,
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
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        alias = "description"
    )]
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
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        alias = "skip_if_completed"
    )]
    pub skip_if_completed: Option<bool>,
    /// Update `auto_fix_on_failure`.
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        alias = "auto_fix_on_failure"
    )]
    pub auto_fix_on_failure: Option<bool>,
    /// Update the success criteria (pass `null` to clear).
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        alias = "success_criteria"
    )]
    pub success_criteria: Option<String>,
    /// Replace the conditions block (pass `null` to clear).
    #[serde(default, skip_serializing_if = "Option::is_none", alias = "conditions")]
    pub conditions: Option<ScheduleConditions>,
    /// Update the catch-up policy.
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        alias = "catch_up_policy"
    )]
    pub catch_up_policy: Option<CatchUpPolicy>,
    /// Update the catch-up grace window (seconds).
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        alias = "catch_up_grace_seconds"
    )]
    pub catch_up_grace_seconds: Option<u32>,
    /// Update the launch-failure backoff base (seconds).
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        alias = "launch_failure_backoff_seconds"
    )]
    pub launch_failure_backoff_seconds: Option<u32>,
}

// ============================================================================
// Recurring Agents — declared fleet-wide in coord, reconciled by each runner
// ============================================================================
//
// Plan `2026-10-29-recurring-agents-declared-in-coord-reconciled-by-runner`.
// coord stores these in `coord.recurring_agents` (declarations) and
// `coord.recurring_agent_device_status` (what each device reports); the
// `validate()` methods below enforce the same invariants that table's CHECK
// constraints do, so a violating value is refused before it reaches the
// database rather than by it.

/// `unapplied_reason`: the runner has no `paths.workspace_root`, so a template
/// whose working directory is `{workspace_root}` cannot be resolved.
pub const UNAPPLIED_WORKSPACE_ROOT_UNSET: &str = "workspace_root_unset";
/// `unapplied_reason` (state `refused`): a `Script` command or a `requireProbe`
/// command differs from the runner's own bundled pin for the catalog entry, or
/// a custom agent carried one at all.
pub const REFUSED_COMMAND: &str = "refused_command";
/// `unapplied_reason`: the device could not prove it is bound to the tenant.
pub const UNAPPLIED_TENANT_UNPROVEN: &str = "tenant_unproven";
/// `unapplied_reason`: the device's "accept managed tasks" setting is off.
pub const UNAPPLIED_MANAGED_TASKS_REFUSED: &str = "managed_tasks_refused";

/// A shape violation in a recurring-agent value. Every variant names the
/// field it is about, so the message can be shown to whoever wrote the value.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum RecurringAgentError {
    /// A required string or list is empty (or only whitespace).
    #[error("{field} must not be empty")]
    Empty {
        /// The offending field.
        field: &'static str,
    },
    /// A digest is not a sha256 in 64 lowercase hex characters.
    #[error("{field} must be a sha256 digest (64 lowercase hex characters), got {value:?}")]
    BadDigest {
        /// The offending field.
        field: &'static str,
        /// The value that was supplied.
        value: String,
    },
    /// A version is 0 or does not fit the `INTEGER` column it is stored in;
    /// versions run from 1 to `i32::MAX`.
    #[error("{field} must be between 1 and 2147483647")]
    VersionOutOfRange {
        /// The offending field.
        field: &'static str,
    },
    /// A value stored in a `UUID` column does not parse as a UUID.
    #[error("{field} must be a UUID, got {value:?}")]
    BadUuid {
        /// The offending field.
        field: &'static str,
        /// The value that was supplied.
        value: String,
    },
    /// A value stored in a `TIMESTAMPTZ` column is not an RFC 3339 timestamp.
    #[error("{field} must be an RFC 3339 timestamp, got {value:?}")]
    BadTimestamp {
        /// The offending field.
        field: &'static str,
        /// The value that was supplied.
        value: String,
    },
    /// A tenant-row (stored) spec names a catalog entry and also carries a
    /// `task_spec` (`ck_recurring_agents_task_spec_only_custom`).
    #[error("a stored catalog agent (catalog_ref set) must not carry a task_spec")]
    CatalogWithTaskSpec,
    /// The report says the device refuses managed tasks, yet an entry is not
    /// `unapplied` with [`UNAPPLIED_MANAGED_TASKS_REFUSED`].
    #[error("accepts_managed is false, so the entry must be unapplied with reason \"managed_tasks_refused\"")]
    NotRefusedWhileManagedOff,
    /// The state is `unapplied` or `refused` but no reason was given.
    #[error("state {state:?} requires a non-empty unapplied_reason")]
    MissingReason {
        /// The status entry's state.
        state: RecurringAgentState,
    },
    /// A reason was given for a state that is neither `unapplied` nor
    /// `refused`.
    #[error("state {state:?} must not carry an unapplied_reason")]
    UnexpectedReason {
        /// The status entry's state.
        state: RecurringAgentState,
    },
    /// The state is `overridden` but no override record was given.
    #[error("state overridden requires an override record")]
    MissingOverride,
    /// An override record that sets none of the fields it may override.
    #[error("override record overrides nothing (set schedule, timezone, task or conditions)")]
    EmptyOverride,
    /// A custom agent (no `catalog_ref`) without a `task_spec`.
    #[error("a custom agent (no catalog_ref) must carry a task_spec")]
    CustomWithoutTask,
    /// A custom agent whose task is not a `RemoteAgent` (D9: `Script` tasks
    /// come only from the catalog).
    #[error("a custom agent may only run a RemoteAgent task, got {task_type}")]
    CustomTaskNotRemoteAgent {
        /// The task type that was supplied.
        task_type: &'static str,
    },
    /// A custom agent carrying a `requireProbe` condition (D9: a probe command
    /// runs as the device user, so it comes only from the catalog).
    #[error("a custom agent may not carry a requireProbe condition")]
    CustomWithProbe,
    /// The same value appears twice where values must be unique.
    #[error("duplicate {what} {value:?}")]
    Duplicate {
        /// What kind of value was duplicated.
        what: &'static str,
        /// The duplicated value.
        value: String,
    },
    /// A catalog parameter definition, or a value supplied for one, is
    /// malformed.
    #[error("parameter {name:?}: {problem}")]
    Param {
        /// The parameter's name.
        name: String,
        /// What is wrong with it.
        problem: String,
    },
    /// A status entry in a report is invalid.
    #[error("agent {name:?}: {error}")]
    Agent {
        /// The status entry's agent name.
        name: String,
        /// What is wrong with it.
        error: Box<RecurringAgentError>,
    },
}

fn require_non_empty(field: &'static str, value: &str) -> Result<(), RecurringAgentError> {
    if value.trim().is_empty() {
        return Err(RecurringAgentError::Empty { field });
    }
    Ok(())
}

fn require_optional_non_empty(
    field: &'static str,
    value: Option<&str>,
) -> Result<(), RecurringAgentError> {
    value.map_or(Ok(()), |v| require_non_empty(field, v))
}

fn require_version(field: &'static str, version: u32) -> Result<(), RecurringAgentError> {
    if version == 0 || version > i32::MAX as u32 {
        return Err(RecurringAgentError::VersionOutOfRange { field });
    }
    Ok(())
}

/// The value is stored in a `UUID` column, so it must parse as one.
fn require_uuid(field: &'static str, value: &str) -> Result<(), RecurringAgentError> {
    uuid::Uuid::parse_str(value)
        .map(|_| ())
        .map_err(|_| RecurringAgentError::BadUuid {
            field,
            value: value.to_string(),
        })
}

/// The value is stored in a `TIMESTAMPTZ` column, so it must be RFC 3339.
fn require_timestamp(field: &'static str, value: &str) -> Result<(), RecurringAgentError> {
    chrono::DateTime::parse_from_rfc3339(value)
        .map(|_| ())
        .map_err(|_| RecurringAgentError::BadTimestamp {
            field,
            value: value.to_string(),
        })
}

fn require_optional_timestamp(
    field: &'static str,
    value: Option<&str>,
) -> Result<(), RecurringAgentError> {
    value.map_or(Ok(()), |v| require_timestamp(field, v))
}

/// Whether `value` is a sha256 digest as coord writes it: exactly 64
/// lowercase hex characters (the `^[0-9a-f]{64}$` CHECK on
/// `coord.recurring_agent_device_status.applied_digest`).
pub fn is_sha256_hex(value: &str) -> bool {
    value.len() == 64
        && value
            .bytes()
            .all(|b| matches!(b, b'0'..=b'9' | b'a'..=b'f'))
}

fn require_digest(field: &'static str, value: Option<&str>) -> Result<(), RecurringAgentError> {
    match value {
        Some(v) if !is_sha256_hex(v) => Err(RecurringAgentError::BadDigest {
            field,
            value: v.to_string(),
        }),
        _ => Ok(()),
    }
}

fn require_unique_entries<'a>(
    what: &'static str,
    values: impl IntoIterator<Item = &'a str>,
) -> Result<(), RecurringAgentError> {
    let mut seen = HashSet::new();
    for value in values {
        if !seen.insert(value) {
            return Err(RecurringAgentError::Duplicate {
                what,
                value: value.to_string(),
            });
        }
    }
    Ok(())
}

/// Check a list of ids/names: no blanks, no duplicates, and non-empty unless
/// `allow_empty`.
fn require_name_list(
    field: &'static str,
    names: &[String],
    allow_empty: bool,
) -> Result<(), RecurringAgentError> {
    if names.is_empty() && !allow_empty {
        return Err(RecurringAgentError::Empty { field });
    }
    for name in names {
        require_non_empty(field, name)?;
    }
    require_unique_entries(field, names.iter().map(String::as_str))
}

impl ManagedBy {
    /// Check the managed key and digest are well-formed.
    pub fn validate(&self) -> Result<(), RecurringAgentError> {
        require_non_empty("managed.source", &self.source)?;
        require_uuid("managed.tenantId", &self.tenant_id)?;
        require_non_empty("managed.agent", &self.agent)?;
        require_version("managed.version", self.version)?;
        require_digest("managed.digest", self.digest.as_deref())
    }
}

/// A device's dispatch role, in coord's vocabulary (`dispatch_role.rs`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum RecurringAgentDispatchRole {
    /// Every lane open.
    Workhorse,
    /// Takes nothing coord sends.
    Bench,
    /// CI only.
    CiNode,
}

/// An operating system a [`RecurringAgentTarget::Os`] target selects.
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize, JsonSchema,
)]
#[serde(rename_all = "snake_case")]
pub enum RecurringAgentOs {
    /// Linux.
    Linux,
    /// Windows.
    Windows,
    /// macOS.
    Macos,
}

/// Which devices a recurring agent applies to (plan D4).
///
/// Always serialized as a JSON object tagged by `kind` — `{"kind":"all"}`,
/// never the bare string `"all"` — because coord's
/// `ck_recurring_agents_target_object` refuses a non-object. coord resolves the
/// target per device on its read route, so a runner never evaluates it.
///
/// Unknown fields are refused on deserialization (not only in the schema): a
/// misspelt `exclude_devices` silently ignored would widen the target to
/// devices the author meant to exclude. A runner that cannot parse a target
/// keeps its last-applied set (the reconciler's fail-safe), which is the safe
/// side of that failure.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum RecurringAgentTarget {
    /// Every device with the runner capability.
    All {
        /// Device ids (UUIDs) removed from the target.
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        exclude_devices: Vec<String>,
    },
    /// An explicit list of devices.
    Devices {
        /// Device ids (UUIDs). May be empty: coord's catalog defaults for a
        /// one-box agent are `{"kind":"devices","devices":[]}` — the agent
        /// targets no device until an operator picks one.
        devices: Vec<String>,
        /// Device ids (UUIDs) removed from the target.
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        exclude_devices: Vec<String>,
    },
    /// Every device holding the given dispatch role.
    DispatchRole {
        /// The role a device must hold.
        role: RecurringAgentDispatchRole,
        /// Device ids (UUIDs) removed from the target.
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        exclude_devices: Vec<String>,
    },
    /// Every device whose capabilities contain ALL of these.
    Capabilities {
        /// Required capabilities (containment). Must be non-empty.
        capabilities: Vec<String>,
        /// Device ids (UUIDs) removed from the target.
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        exclude_devices: Vec<String>,
    },
    /// Every device running one of these operating systems.
    Os {
        /// Selected operating systems. Must be non-empty.
        os: Vec<RecurringAgentOs>,
        /// Device ids (UUIDs) removed from the target.
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        exclude_devices: Vec<String>,
    },
}

impl RecurringAgentTarget {
    /// The devices removed from this target, whatever its kind.
    pub fn exclude_devices(&self) -> &[String] {
        match self {
            Self::All { exclude_devices }
            | Self::Devices {
                exclude_devices, ..
            }
            | Self::DispatchRole {
                exclude_devices, ..
            }
            | Self::Capabilities {
                exclude_devices, ..
            }
            | Self::Os {
                exclude_devices, ..
            } => exclude_devices,
        }
    }

    /// Check every selector list is free of blanks and duplicates, and
    /// non-empty except `devices` (an empty device list targets nothing).
    pub fn validate(&self) -> Result<(), RecurringAgentError> {
        match self {
            Self::All { .. } | Self::DispatchRole { .. } => {}
            Self::Devices { devices, .. } => require_name_list("target.devices", devices, true)?,
            Self::Capabilities { capabilities, .. } => {
                require_name_list("target.capabilities", capabilities, false)?
            }
            Self::Os { os, .. } => {
                if os.is_empty() {
                    return Err(RecurringAgentError::Empty { field: "target.os" });
                }
                let mut seen = BTreeSet::new();
                for system in os {
                    if !seen.insert(*system) {
                        return Err(RecurringAgentError::Duplicate {
                            what: "target.os",
                            value: format!("{system:?}"),
                        });
                    }
                }
            }
        }
        require_name_list("target.exclude_devices", self.exclude_devices(), true)
    }
}

/// The type, default and bounds of one catalog parameter — exactly the shape
/// coord serves in each catalog entry's `params` array
/// (`recurring_agents::param_schema_json` in qontinui-coord).
///
/// Tagged by `type` and FLATTENED into its [`RecurringAgentParamDef`], so one
/// served parameter reads
/// `{"name":"mode","description":"…","required":false,"type":"enum","values":["reap","report"],"default":"reap"}`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum RecurringAgentParamKind {
    /// A checkbox.
    Boolean {
        /// The value used when the tenant sets none.
        default: bool,
    },
    /// A whole number within `minimum..=maximum`.
    Integer {
        /// Smallest allowed value.
        minimum: i64,
        /// Largest allowed value.
        maximum: i64,
        /// The value used when the tenant sets none. Within the bounds.
        default: i64,
    },
    /// One of a fixed set of strings.
    Enum {
        /// The allowed values, in display order. Non-empty, no duplicates.
        values: Vec<String>,
        /// The value used when the tenant sets none. One of `values`.
        default: String,
    },
    /// A non-empty set of integers drawn from `allowed`, rendered ascending
    /// (e.g. the dossier-exit-check's lookback windows).
    IntegerSet {
        /// The values a tenant may pick from. Non-empty, no duplicates.
        allowed: Vec<i64>,
        /// Values that exist but this entry refuses, each with why — shown
        /// greyed out in the form rather than hidden.
        #[serde(default)]
        excluded: Vec<RecurringAgentExcludedValue>,
        /// The set used when the tenant sets none. Non-empty, drawn from
        /// `allowed`, none of it excluded.
        default: Vec<i64>,
    },
    /// A UUID, lower-cased on render. Coord serves no default (`null`).
    Uuid {
        /// The value used when the tenant sets none.
        #[serde(default)]
        default: Option<String>,
    },
    /// A lower-case slug matching `^[a-z0-9]([a-z0-9-]{0,61}[a-z0-9])?$`.
    Slug {
        /// The value used when the tenant sets none. `None` = no default.
        #[serde(default)]
        default: Option<String>,
    },
}

impl RecurringAgentParamKind {
    /// The value a parameter takes when the tenant sets none, as coord renders
    /// it. `None` = no default (a required parameter must then be supplied).
    pub fn default_value(&self) -> Option<serde_json::Value> {
        use serde_json::Value;
        match self {
            Self::Boolean { default } => Some(Value::Bool(*default)),
            Self::Integer { default, .. } => Some(Value::from(*default)),
            Self::Enum { default, .. } => Some(Value::String(default.clone())),
            Self::IntegerSet { default, .. } => Some(Value::from(default.clone())),
            Self::Uuid { default } | Self::Slug { default } => default.clone().map(Value::String),
        }
    }
}

/// An [`RecurringAgentParamKind::IntegerSet`] value the entry refuses.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[schemars(deny_unknown_fields)]
pub struct RecurringAgentExcludedValue {
    /// The refused value.
    pub value: i64,
    /// Why it is refused.
    pub reason: String,
}

/// One parameter a catalog entry's prompt is rendered from (plan D2), e.g.
/// cleanup-steward's `mode` enum or return-to-main's `daily_cap` integer.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct RecurringAgentParamDef {
    /// The key a value is supplied under in [`RecurringAgentSpec::params`].
    pub name: String,
    /// Human-readable explanation, shown beside the form control.
    pub description: String,
    /// A required parameter with no default must be supplied by the tenant
    /// row before the entry renders at all (strategic-pulse's tenant slug).
    #[serde(default)]
    pub required: bool,
    /// Type, default and bounds, flattened beside the fields above.
    #[serde(flatten)]
    pub kind: RecurringAgentParamKind,
}

/// The parameter form of one catalog entry: the typed parameters a tenant may
/// set instead of editing prompt text (plan D2, D10). Serialized as the bare
/// array coord serves under a catalog entry's `params` key.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(transparent)]
pub struct RecurringAgentParamSchema {
    /// The parameters, in display order. Names are unique.
    pub params: Vec<RecurringAgentParamDef>,
}

impl RecurringAgentParamSchema {
    /// Look up a parameter definition by name.
    pub fn get(&self, name: &str) -> Option<&RecurringAgentParamDef> {
        self.params.iter().find(|p| p.name == name)
    }

    /// Check the schema itself: names non-empty and unique, every default
    /// satisfies its own kind, enum values and integer-set choices non-empty
    /// and unique, integer bounds ordered.
    pub fn validate(&self) -> Result<(), RecurringAgentError> {
        for def in &self.params {
            require_non_empty("params[].name", &def.name)?;
            let problem = |problem: &str| RecurringAgentError::Param {
                name: def.name.clone(),
                problem: problem.to_string(),
            };
            match &def.kind {
                RecurringAgentParamKind::Boolean { .. } => {}
                RecurringAgentParamKind::Integer {
                    minimum,
                    maximum,
                    default,
                } => {
                    if minimum > maximum {
                        return Err(problem("minimum is greater than maximum"));
                    }
                    if !(minimum..=maximum).contains(&default) {
                        return Err(problem("default is outside [minimum, maximum]"));
                    }
                }
                RecurringAgentParamKind::Enum { values, default } => {
                    if values.is_empty() {
                        return Err(problem("enum has no values"));
                    }
                    require_unique_entries("enum value", values.iter().map(String::as_str))?;
                    if !values.contains(default) {
                        return Err(problem("default is not one of the values"));
                    }
                }
                RecurringAgentParamKind::IntegerSet {
                    allowed,
                    excluded,
                    default,
                } => {
                    if allowed.is_empty() {
                        return Err(problem("integer set allows no values"));
                    }
                    if !all_distinct(allowed) {
                        return Err(problem("allowed lists a value twice"));
                    }
                    if !all_distinct(&excluded.iter().map(|e| e.value).collect::<Vec<_>>()) {
                        return Err(problem("excluded lists a value twice"));
                    }
                    if excluded.iter().any(|e| e.reason.trim().is_empty()) {
                        return Err(problem("an excluded value has no reason"));
                    }
                    check_integer_set(default, allowed, excluded).map_err(|why| {
                        RecurringAgentError::Param {
                            name: def.name.clone(),
                            problem: format!("default: {why}"),
                        }
                    })?;
                }
                RecurringAgentParamKind::Uuid { default } => {
                    if default
                        .as_deref()
                        .is_some_and(|d| uuid::Uuid::parse_str(d).is_err())
                    {
                        return Err(problem("default is not a UUID"));
                    }
                }
                RecurringAgentParamKind::Slug { default } => {
                    if default.as_deref().is_some_and(|d| !is_param_slug(d)) {
                        return Err(problem("default is not a slug"));
                    }
                }
            }
        }
        require_unique_entries("parameter", self.params.iter().map(|p| p.name.as_str()))
    }

    /// Check tenant-supplied parameter values against this schema, the way
    /// coord resolves them: every key is a known parameter, every non-null
    /// value has its parameter's type and lies within its values or bounds,
    /// and a required parameter with no default has a value. An absent or
    /// `null` key inherits the default.
    pub fn validate_params(
        &self,
        params: &serde_json::Map<String, serde_json::Value>,
    ) -> Result<(), RecurringAgentError> {
        for name in params.keys() {
            if self.get(name).is_none() {
                return Err(RecurringAgentError::Param {
                    name: name.clone(),
                    problem: "unknown parameter".to_string(),
                });
            }
        }
        for def in &self.params {
            let problem = |problem: &str| RecurringAgentError::Param {
                name: def.name.clone(),
                problem: problem.to_string(),
            };
            let Some(value) = params.get(&def.name).filter(|v| !v.is_null()) else {
                if def.required && def.kind.default_value().is_none() {
                    return Err(problem("required parameter has no value and no default"));
                }
                continue;
            };
            match &def.kind {
                RecurringAgentParamKind::Boolean { .. } => {
                    if !value.is_boolean() {
                        return Err(problem("expected a boolean"));
                    }
                }
                RecurringAgentParamKind::Integer {
                    minimum, maximum, ..
                } => match value.as_i64() {
                    Some(v) if (minimum..=maximum).contains(&&v) => {}
                    Some(_) => return Err(problem("value is outside [minimum, maximum]")),
                    None => return Err(problem("expected a whole number")),
                },
                RecurringAgentParamKind::Enum { values, .. } => match value.as_str() {
                    Some(v) if values.iter().any(|o| o == v) => {}
                    Some(_) => return Err(problem("value is not one of the values")),
                    None => return Err(problem("expected a string")),
                },
                RecurringAgentParamKind::IntegerSet {
                    allowed, excluded, ..
                } => {
                    let Some(items) = value.as_array() else {
                        return Err(problem("expected an array of whole numbers"));
                    };
                    let mut set = Vec::with_capacity(items.len());
                    for item in items {
                        match item.as_i64() {
                            Some(n) => set.push(n),
                            None => return Err(problem("expected whole numbers")),
                        }
                    }
                    check_integer_set(&set, allowed, excluded).map_err(|why| problem(&why))?;
                }
                RecurringAgentParamKind::Uuid { .. } => {
                    if value
                        .as_str()
                        .is_none_or(|s| uuid::Uuid::parse_str(s).is_err())
                    {
                        return Err(problem("expected a UUID string"));
                    }
                }
                RecurringAgentParamKind::Slug { .. } => {
                    if value.as_str().is_none_or(|s| !is_param_slug(s)) {
                        return Err(problem(
                            "expected a slug matching ^[a-z0-9]([a-z0-9-]{0,61}[a-z0-9])?$",
                        ));
                    }
                }
            }
        }
        Ok(())
    }
}

fn all_distinct(values: &[i64]) -> bool {
    let mut seen = BTreeSet::new();
    values.iter().all(|v| seen.insert(*v))
}

/// A non-empty, duplicate-free set drawn from `allowed` with nothing excluded —
/// coord's rule for an integer-set value.
fn check_integer_set(
    set: &[i64],
    allowed: &[i64],
    excluded: &[RecurringAgentExcludedValue],
) -> Result<(), String> {
    if set.is_empty() {
        return Err("the set may not be empty".to_string());
    }
    let mut seen = BTreeSet::new();
    for n in set {
        if let Some(e) = excluded.iter().find(|e| e.value == *n) {
            return Err(format!("{n} is refused: {}", e.reason));
        }
        if !allowed.contains(n) {
            return Err(format!("{n} is not one of {allowed:?}"));
        }
        if !seen.insert(*n) {
            return Err(format!("{n} is listed twice"));
        }
    }
    Ok(())
}

/// `^[a-z0-9]([a-z0-9-]{0,61}[a-z0-9])?$` — coord's slug parameter grammar.
fn is_param_slug(s: &str) -> bool {
    let bytes = s.as_bytes();
    let edge = |b: &u8| b.is_ascii_lowercase() || b.is_ascii_digit();
    (1..=63).contains(&bytes.len())
        && bytes.first().is_some_and(edge)
        && bytes.last().is_some_and(edge)
        && bytes.iter().all(|b| edge(b) || *b == b'-')
}

/// One recurring agent as coord serves it: a catalog entry merged with the
/// tenant's overrides, or a custom agent's full spec (plan D2).
///
/// On coord's device read route `task_spec` carries the resolved task (for a
/// catalog agent, the template rendered from `params`; a working directory may
/// still be the `{workspace_root}` token, which the runner resolves). A
/// custom agent (no `catalog_ref`) must carry a `RemoteAgent` task and no
/// `requireProbe` condition (D9): `Script` and probe commands come only from
/// the catalog, and the runner applies them only when they equal its own
/// bundled pin.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[schemars(deny_unknown_fields)]
pub struct RecurringAgentSpec {
    /// The agent's name — unique per tenant, and the join key for device
    /// status, run requests and managed tasks (coord forbids renames).
    pub name: String,
    /// The catalog entry this agent is, or overrides. `None` = a custom agent.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub catalog_ref: Option<String>,
    /// Whether the agent is enabled for the tenant.
    pub enabled: bool,
    /// When the agent runs.
    pub schedule: ScheduleExpression,
    /// IANA time zone for `schedule`. `None` = each device's scheduler zone.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub timezone: Option<String>,
    /// What to do about slots missed while a runner was down.
    #[serde(default = "default_catch_up_policy")]
    pub catch_up_policy: CatchUpPolicy,
    /// Values for the catalog entry's [`RecurringAgentParamSchema`]. Always a
    /// JSON object; empty = every parameter at its default.
    #[serde(default)]
    pub params: serde_json::Map<String, serde_json::Value>,
    /// The task to run. Required for a custom agent; for a catalog agent, the
    /// resolved template when coord serves it to a device.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub task_spec: Option<ScheduledTaskType>,
    /// Conditions the task waits for before running.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub conditions: Option<ScheduleConditions>,
    /// Which devices the agent applies to.
    pub target: RecurringAgentTarget,
    /// The declaration's row version (starts at 1). History and display only —
    /// compare `digest`, never `version`, to decide whether a device is current.
    pub version: u32,
    /// sha256 (64 lowercase hex) of the device-independent resolved spec,
    /// computed by coord. What a runner stores as `managed.digest` and echoes
    /// back as `applied_digest`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub digest: Option<String>,
}

impl RecurringAgentSpec {
    /// Check the SERVED (resolved) shape — what coord's device read route
    /// returns — including D9's custom-agent rules. In that shape a catalog
    /// agent legitimately carries a `task_spec` (its rendered template). For a
    /// tenant row as stored in `coord.recurring_agents`, use
    /// [`Self::validate_stored`]. Parameter values are checked against the
    /// catalog entry separately, with
    /// [`RecurringAgentParamSchema::validate_params`].
    pub fn validate(&self) -> Result<(), RecurringAgentError> {
        require_non_empty("name", &self.name)?;
        require_optional_non_empty("catalog_ref", self.catalog_ref.as_deref())?;
        require_optional_non_empty("timezone", self.timezone.as_deref())?;
        require_version("version", self.version)?;
        require_digest("digest", self.digest.as_deref())?;
        self.target.validate()?;
        if self.catalog_ref.is_none() {
            match &self.task_spec {
                None => return Err(RecurringAgentError::CustomWithoutTask),
                Some(ScheduledTaskType::RemoteAgent { .. }) => {}
                Some(other) => {
                    return Err(RecurringAgentError::CustomTaskNotRemoteAgent {
                        task_type: task_type_name(other),
                    })
                }
            }
            if self
                .conditions
                .as_ref()
                .is_some_and(|c| c.require_probe.is_some())
            {
                return Err(RecurringAgentError::CustomWithProbe);
            }
        }
        Ok(())
    }

    /// Check the STORED shape of a tenant row: everything [`Self::validate`]
    /// checks, plus `ck_recurring_agents_task_spec_only_custom` — a row with a
    /// `catalog_ref` carries no `task_spec`, because a catalog agent's task
    /// comes from the catalog in code (D2, D9).
    pub fn validate_stored(&self) -> Result<(), RecurringAgentError> {
        self.validate()?;
        if self.catalog_ref.is_some() && self.task_spec.is_some() {
            return Err(RecurringAgentError::CatalogWithTaskSpec);
        }
        Ok(())
    }

    /// The managed key a runner stamps on the [`ScheduledTask`] it reconciles
    /// from this spec, for the tenant that served it.
    pub fn managed_by(&self, tenant_id: impl Into<String>) -> ManagedBy {
        ManagedBy {
            source: MANAGED_SOURCE_COORD.to_string(),
            tenant_id: tenant_id.into(),
            agent: self.name.clone(),
            version: self.version,
            digest: self.digest.clone(),
        }
    }
}

/// The `task_type` tag a [`ScheduledTaskType`] serializes with.
fn task_type_name(task: &ScheduledTaskType) -> &'static str {
    match task {
        ScheduledTaskType::Workflow { .. } => "Workflow",
        ScheduledTaskType::Prompt { .. } => "Prompt",
        ScheduledTaskType::AutoFix { .. } => "AutoFix",
        ScheduledTaskType::Watcher { .. } => "Watcher",
        ScheduledTaskType::BackgroundCapture { .. } => "BackgroundCapture",
        ScheduledTaskType::RemoteAgent { .. } => "RemoteAgent",
        ScheduledTaskType::Script { .. } => "Script",
    }
}

/// What a device did with one recurring agent — the
/// `coord.recurring_agent_device_status.state` vocabulary.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum RecurringAgentState {
    /// The current spec is applied as a managed task.
    Applied,
    /// Applied, but paused on this machine (D6). Status, never an alert.
    Paused,
    /// Applied with a local override record (D6). Requires `override`.
    Overridden,
    /// Not applied; `unapplied_reason` says why.
    Unapplied,
    /// Refused by the runner's own checks (D9); `unapplied_reason` says why.
    Refused,
}

impl RecurringAgentState {
    /// Whether this state requires a non-empty `unapplied_reason` — and, by
    /// the `ck_..._reason_iff_unapplied` CHECK, whether it permits one.
    pub fn requires_reason(self) -> bool {
        matches!(self, Self::Unapplied | Self::Refused)
    }
}

/// A local override of a managed task on one machine (plan D6), stored by the
/// runner and reported to coord so the web shows it. Only the fields set are
/// overridden; the rest follow the fleet declaration.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[schemars(deny_unknown_fields)]
pub struct RecurringAgentOverride {
    /// ISO 8601 timestamp when the override was made.
    pub created_at: String,
    /// Replacement schedule.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub schedule: Option<ScheduleExpression>,
    /// Replacement IANA time zone.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub timezone: Option<String>,
    /// Replacement task.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub task: Option<ScheduledTaskType>,
    /// Replacement conditions (e.g. require idle / repo inactive).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub conditions: Option<ScheduleConditions>,
    /// Free-form note from whoever made the override.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub note: Option<String>,
}

impl RecurringAgentOverride {
    /// Check the record is timestamped and overrides at least one field.
    pub fn validate(&self) -> Result<(), RecurringAgentError> {
        require_non_empty("override.created_at", &self.created_at)?;
        require_optional_non_empty("override.timezone", self.timezone.as_deref())?;
        if self.schedule.is_none()
            && self.timezone.is_none()
            && self.task.is_none()
            && self.conditions.is_none()
        {
            return Err(RecurringAgentError::EmptyOverride);
        }
        Ok(())
    }
}

/// One device's status for one recurring agent — a
/// `coord.recurring_agent_device_status` row (plan D7).
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[schemars(deny_unknown_fields)]
pub struct RecurringAgentStatus {
    /// The agent's name.
    pub name: String,
    /// The declaration version applied. Display only.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub applied_version: Option<u32>,
    /// The digest of the spec applied. `None` = none applied.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub applied_digest: Option<String>,
    /// What the device did with the agent.
    pub state: RecurringAgentState,
    /// Why the agent is `unapplied` or `refused` (e.g.
    /// [`UNAPPLIED_WORKSPACE_ROOT_UNSET`], [`REFUSED_COMMAND`]). Present
    /// exactly for those two states.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub unapplied_reason: Option<String>,
    /// The local override record. Required when `state` is `overridden`; a
    /// `paused` task may also carry one.
    #[serde(default, rename = "override", skip_serializing_if = "Option::is_none")]
    pub override_record: Option<RecurringAgentOverride>,
    /// The newest run.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_run: Option<TaskExecutionRecord>,
    /// The last N runs, newest first.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub recent_runs: Vec<TaskExecutionRecord>,
    /// ISO 8601 end of the newest completed run. `None` = none known.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_completed_at: Option<String>,
    /// ISO 8601 end of the newest successful run. `None` = none known.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_success_at: Option<String>,
    /// ISO 8601 next scheduled run.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub next_run: Option<String>,
    /// The `request_id` of the newest "Run now" request the device ran,
    /// echoed so coord can stop serving it as pending (plan D10).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub acknowledged_request_id: Option<String>,
}

impl RecurringAgentStatus {
    /// A status entry with no applied spec and no run history, validated.
    /// Fill the remaining fields as they become known.
    pub fn new(
        name: impl Into<String>,
        state: RecurringAgentState,
        unapplied_reason: Option<String>,
        override_record: Option<RecurringAgentOverride>,
    ) -> Result<Self, RecurringAgentError> {
        let status = Self {
            name: name.into(),
            applied_version: None,
            applied_digest: None,
            state,
            unapplied_reason,
            override_record,
            last_run: None,
            recent_runs: Vec::new(),
            last_completed_at: None,
            last_success_at: None,
            next_run: None,
            acknowledged_request_id: None,
        };
        status.validate()?;
        Ok(status)
    }

    /// Check the invariants coord's table enforces: a non-empty reason exactly
    /// when the state is `unapplied` or `refused`; `overridden` carries an
    /// override record; the digest is a sha256; the version is at least 1.
    pub fn validate(&self) -> Result<(), RecurringAgentError> {
        require_non_empty("name", &self.name)?;
        match (
            self.unapplied_reason.as_deref(),
            self.state.requires_reason(),
        ) {
            (None, true) => return Err(RecurringAgentError::MissingReason { state: self.state }),
            (Some(reason), true) if reason.trim().is_empty() => {
                return Err(RecurringAgentError::MissingReason { state: self.state })
            }
            (Some(_), false) => {
                return Err(RecurringAgentError::UnexpectedReason { state: self.state })
            }
            _ => {}
        }
        match &self.override_record {
            Some(record) => record.validate()?,
            None if self.state == RecurringAgentState::Overridden => {
                return Err(RecurringAgentError::MissingOverride)
            }
            None => {}
        }
        if let Some(version) = self.applied_version {
            require_version("applied_version", version)?;
        }
        require_digest("applied_digest", self.applied_digest.as_deref())?;
        require_optional_timestamp("last_completed_at", self.last_completed_at.as_deref())?;
        require_optional_timestamp("last_success_at", self.last_success_at.as_deref())?;
        require_optional_timestamp("next_run", self.next_run.as_deref())?;
        if let Some(request_id) = &self.acknowledged_request_id {
            require_uuid("acknowledged_request_id", request_id)?;
        }
        Ok(())
    }
}

/// What a runner posts to coord after every reconcile and every run
/// (`POST /coord/recurring-agents/status/:device_id`, plan D7): one entry per
/// recurring agent the tenant's declaration targets at this device.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[schemars(deny_unknown_fields)]
pub struct RecurringAgentStatusReport {
    /// The tenant (UUID) the reported agents were declared by. A device bound
    /// to several tenants posts one report per proven tenant.
    pub tenant_id: String,
    /// RFC 3339 timestamp when the device built this report.
    pub reported_at: String,
    /// The device's "accept managed tasks" setting (D9; default on). When
    /// off, the runner applies nothing and reports each agent `unapplied`
    /// with [`UNAPPLIED_MANAGED_TASKS_REFUSED`].
    pub accepts_managed: bool,
    /// One entry per agent, names unique.
    #[serde(default)]
    pub agents: Vec<RecurringAgentStatus>,
}

impl RecurringAgentStatusReport {
    /// Check the report and every entry; entry errors name their agent. When
    /// `accepts_managed` is false, every entry must be `unapplied` with
    /// [`UNAPPLIED_MANAGED_TASKS_REFUSED`].
    pub fn validate(&self) -> Result<(), RecurringAgentError> {
        require_uuid("tenant_id", &self.tenant_id)?;
        require_timestamp("reported_at", &self.reported_at)?;
        for agent in &self.agents {
            let entry = agent.validate().and_then(|()| {
                let refused = agent.state == RecurringAgentState::Unapplied
                    && agent.unapplied_reason.as_deref() == Some(UNAPPLIED_MANAGED_TASKS_REFUSED);
                if self.accepts_managed || refused {
                    Ok(())
                } else {
                    Err(RecurringAgentError::NotRefusedWhileManagedOff)
                }
            });
            entry.map_err(|e| RecurringAgentError::Agent {
                name: agent.name.clone(),
                error: Box::new(e),
            })?;
        }
        require_unique_entries("agent", self.agents.iter().map(|a| a.name.as_str()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::{json, Value};

    const TENANT: &str = "7d6c1c2e-1b7a-4e43-9a43-6f1f0f3c2a11";
    const DIGEST: &str = "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef";

    /// Serialize, deserialize, re-serialize: the two JSON strings must match.
    fn round_trip<T>(value: &T) -> Value
    where
        T: Serialize + for<'de> Deserialize<'de>,
    {
        let json = serde_json::to_string(value).unwrap();
        let back: T = serde_json::from_str(&json).unwrap();
        assert_eq!(json, serde_json::to_string(&back).unwrap());
        serde_json::from_str(&json).unwrap()
    }

    /// A ScheduledTask exactly as stored before `timezone` / `managed` existed.
    /// The same task as one compact JSON string, keys in declaration order —
    /// the exact bytes the runner wrote before this change.
    const LEGACY_TASK: &str = concat!(
        r#"{"id":"t-1","name":"Nightly cleanup","enabled":true,"#,
        r#""schedule":{"type":"Cron","value":"0 3 * * *"},"#,
        r#""task":{"task_type":"Script","command":"bash sweep.sh"},"#,
        r#""skipIfCompleted":false,"autoFixOnFailure":false,"#,
        r#""createdAt":"2026-10-01T00:00:00Z","modifiedAt":"2026-10-01T00:00:00Z","#,
        r#""catchUpPolicy":"run_once","catchUpGraceSeconds":300,"#,
        r#""consecutiveLaunchFailures":0,"launchFailureBackoffSeconds":60}"#,
    );

    fn legacy_task_json() -> Value {
        json!({
            "id": "t-1",
            "name": "Nightly cleanup",
            "enabled": true,
            "schedule": {"type": "Cron", "value": "0 3 * * *"},
            "task": {"task_type": "Script", "command": "bash sweep.sh"},
            "skipIfCompleted": false,
            "autoFixOnFailure": false,
            "createdAt": "2026-10-01T00:00:00Z",
            "modifiedAt": "2026-10-01T00:00:00Z",
            "catchUpPolicy": "run_once",
            "catchUpGraceSeconds": 300,
            "consecutiveLaunchFailures": 0,
            "launchFailureBackoffSeconds": 60
        })
    }

    fn managed() -> ManagedBy {
        ManagedBy {
            source: MANAGED_SOURCE_COORD.to_string(),
            tenant_id: "7d6c1c2e-1b7a-4e43-9a43-6f1f0f3c2a11".to_string(),
            agent: "cleanup-steward".to_string(),
            version: 2,
            digest: Some(DIGEST.to_string()),
        }
    }

    fn remote_agent() -> ScheduledTaskType {
        ScheduledTaskType::RemoteAgent {
            prompt: "Summarise yesterday's findings.".to_string(),
            working_directory: Some("{workspace_root}".to_string()),
            allowed_tools: vec![],
            model: None,
            effort: None,
            ultracode: None,
            effort_profile: None,
            mcp_connections: vec![],
            max_turns: None,
            timeout_seconds: None,
        }
    }

    fn custom_spec() -> RecurringAgentSpec {
        RecurringAgentSpec {
            name: "daily-digest".to_string(),
            catalog_ref: None,
            enabled: true,
            schedule: ScheduleExpression::Cron("0 7 * * *".to_string()),
            timezone: Some("Europe/Berlin".to_string()),
            catch_up_policy: CatchUpPolicy::Skip,
            params: serde_json::Map::new(),
            task_spec: Some(remote_agent()),
            conditions: None,
            target: RecurringAgentTarget::All {
                exclude_devices: vec![],
            },
            version: 1,
            digest: Some(DIGEST.to_string()),
        }
    }

    fn override_record() -> RecurringAgentOverride {
        RecurringAgentOverride {
            created_at: "2026-10-30T12:00:00Z".to_string(),
            schedule: Some(ScheduleExpression::Cron("0 4 * * *".to_string())),
            timezone: None,
            task: None,
            conditions: None,
            note: Some("later on this box".to_string()),
        }
    }

    // ---- ScheduledTask: managed + timezone -------------------------------

    #[test]
    fn legacy_scheduled_task_json_still_deserializes_and_round_trips_unchanged() {
        let legacy = legacy_task_json();
        let task: ScheduledTask = serde_json::from_value(legacy.clone()).unwrap();
        assert!(task.timezone.is_none());
        assert!(task.managed.is_none());
        assert_eq!(serde_json::to_value(&task).unwrap(), legacy);
        // Re-serializing adds neither field: an unmanaged task is byte-for-byte
        // what it was.
        let from_bytes: ScheduledTask = serde_json::from_str(LEGACY_TASK).unwrap();
        assert_eq!(serde_json::to_string(&from_bytes).unwrap(), LEGACY_TASK);
        assert_eq!(serde_json::from_str::<Value>(LEGACY_TASK).unwrap(), legacy);
    }

    #[test]
    fn scheduled_task_round_trips_managed_and_timezone() {
        let mut legacy = legacy_task_json();
        legacy["timezone"] = json!("America/New_York");
        legacy["managed"] = json!({
            "source": "coord",
            "tenantId": "7d6c1c2e-1b7a-4e43-9a43-6f1f0f3c2a11",
            "agent": "cleanup-steward",
            "version": 2,
            "digest": DIGEST
        });
        let task: ScheduledTask = serde_json::from_value(legacy.clone()).unwrap();
        assert_eq!(task.timezone.as_deref(), Some("America/New_York"));
        assert_eq!(task.managed, Some(managed()));
        assert_eq!(round_trip(&task), legacy);
    }

    #[test]
    fn scheduled_task_accepts_snake_case_managed_aliases() {
        let mut legacy = legacy_task_json();
        legacy["managed"] = json!({
            "source": "coord",
            "tenant_id": "7d6c1c2e-1b7a-4e43-9a43-6f1f0f3c2a11",
            "agent": "cleanup-steward",
            "version": 2,
            "digest": DIGEST
        });
        let task: ScheduledTask = serde_json::from_value(legacy).unwrap();
        assert_eq!(task.managed, Some(managed()));
    }

    #[test]
    fn managed_by_validate() {
        assert_eq!(managed().validate(), Ok(()));
        let no_digest = ManagedBy {
            digest: None,
            ..managed()
        };
        assert_eq!(no_digest.validate(), Ok(()));
        let bad = ManagedBy {
            digest: Some("ABC".to_string()),
            ..managed()
        };
        assert!(matches!(
            bad.validate(),
            Err(RecurringAgentError::BadDigest { .. })
        ));
        let zero = ManagedBy {
            version: 0,
            ..managed()
        };
        assert!(matches!(
            zero.validate(),
            Err(RecurringAgentError::VersionOutOfRange { .. })
        ));
        let no_tenant = ManagedBy {
            tenant_id: " ".to_string(),
            ..managed()
        };
        assert!(matches!(
            no_tenant.validate(),
            Err(RecurringAgentError::BadUuid { .. })
        ));
        let too_big = ManagedBy {
            version: i32::MAX as u32 + 1,
            ..managed()
        };
        assert!(matches!(
            too_big.validate(),
            Err(RecurringAgentError::VersionOutOfRange { .. })
        ));
        let max = ManagedBy {
            version: i32::MAX as u32,
            ..managed()
        };
        assert_eq!(max.validate(), Ok(()));
    }

    // ---- RecurringAgentTarget ---------------------------------------------

    #[test]
    fn target_all_serializes_as_an_object_never_a_bare_string() {
        let all = RecurringAgentTarget::All {
            exclude_devices: vec![],
        };
        let v = round_trip(&all);
        assert!(v.is_object(), "target must be a JSON object, got {v}");
        assert_eq!(v, json!({"kind": "all"}));
        assert!(serde_json::from_value::<RecurringAgentTarget>(json!("all")).is_err());
    }

    #[test]
    fn every_target_variant_round_trips_as_a_tagged_object() {
        let cases = [
            (
                RecurringAgentTarget::All {
                    exclude_devices: vec!["d-9".to_string()],
                },
                json!({"kind": "all", "exclude_devices": ["d-9"]}),
            ),
            (
                RecurringAgentTarget::Devices {
                    devices: vec!["d-1".to_string(), "d-2".to_string()],
                    exclude_devices: vec![],
                },
                json!({"kind": "devices", "devices": ["d-1", "d-2"]}),
            ),
            (
                RecurringAgentTarget::DispatchRole {
                    role: RecurringAgentDispatchRole::CiNode,
                    exclude_devices: vec![],
                },
                json!({"kind": "dispatch_role", "role": "ci_node"}),
            ),
            (
                RecurringAgentTarget::Capabilities {
                    capabilities: vec!["runner".to_string()],
                    exclude_devices: vec!["d-3".to_string()],
                },
                json!({"kind": "capabilities", "capabilities": ["runner"], "exclude_devices": ["d-3"]}),
            ),
            (
                RecurringAgentTarget::Os {
                    os: vec![RecurringAgentOs::Linux, RecurringAgentOs::Macos],
                    exclude_devices: vec![],
                },
                json!({"kind": "os", "os": ["linux", "macos"]}),
            ),
        ];
        for (target, expected) in cases {
            assert_eq!(round_trip(&target), expected);
            assert_eq!(target.validate(), Ok(()));
            assert_eq!(
                serde_json::from_value::<RecurringAgentTarget>(expected).unwrap(),
                target
            );
        }
    }

    #[test]
    fn target_refuses_unknown_fields_so_a_typo_cannot_widen_it() {
        let typo = json!({"kind": "all", "exclude_device": ["d-1"]});
        assert!(serde_json::from_value::<RecurringAgentTarget>(typo).is_err());
        // A field from ANOTHER variant is refused too: `all` with a `devices`
        // list must not quietly mean "every device".
        let wrong_variant = json!({"kind": "all", "devices": ["d-1"]});
        assert!(serde_json::from_value::<RecurringAgentTarget>(wrong_variant).is_err());
        let unknown_kind = json!({"kind": "everyone"});
        assert!(serde_json::from_value::<RecurringAgentTarget>(unknown_kind).is_err());
        let no_kind = json!({"devices": ["d-1"]});
        assert!(serde_json::from_value::<RecurringAgentTarget>(no_kind).is_err());
    }

    #[test]
    fn target_validate_refuses_empty_blank_and_duplicate_selectors() {
        // coord's one-box catalog default (`TARGET_NONE`) is legal: it targets
        // no device until an operator picks one.
        let none: RecurringAgentTarget =
            serde_json::from_value(json!({"kind": "devices", "devices": []})).unwrap();
        assert_eq!(none.validate(), Ok(()));
        let empty_caps = RecurringAgentTarget::Capabilities {
            capabilities: vec![],
            exclude_devices: vec![],
        };
        assert!(matches!(
            empty_caps.validate(),
            Err(RecurringAgentError::Empty {
                field: "target.capabilities"
            })
        ));
        let blank = RecurringAgentTarget::Capabilities {
            capabilities: vec![" ".to_string()],
            exclude_devices: vec![],
        };
        assert!(matches!(
            blank.validate(),
            Err(RecurringAgentError::Empty { .. })
        ));
        let dup_os = RecurringAgentTarget::Os {
            os: vec![RecurringAgentOs::Windows, RecurringAgentOs::Windows],
            exclude_devices: vec![],
        };
        assert!(matches!(
            dup_os.validate(),
            Err(RecurringAgentError::Duplicate { .. })
        ));
        let dup_exclude = RecurringAgentTarget::All {
            exclude_devices: vec!["d-1".to_string(), "d-1".to_string()],
        };
        assert!(matches!(
            dup_exclude.validate(),
            Err(RecurringAgentError::Duplicate { .. })
        ));
    }

    // ---- Parameter schema --------------------------------------------------

    /// A catalog entry's `params` array exactly as coord's
    /// `param_schema_json` serves it (shapes from coord's catalog: the
    /// cleanup-steward's enum and integer set, return-to-main's bounded
    /// integer and boolean, findings-steward's UUID, strategic-pulse's
    /// required slug).
    fn served_params() -> Value {
        json!([
            {"name": "mode", "description": "report or reap", "required": false,
             "type": "enum", "values": ["reap", "report"], "default": "reap"},
            {"name": "classes", "description": "the run set", "required": false,
             "type": "integer_set", "allowed": [1, 2, 3, 10, 12, 14, 16],
             "excluded": [{"value": 6, "reason": "it has its own sweep"}],
             "default": [3, 12, 14, 16]},
            {"name": "act", "description": "move checkouts", "required": false,
             "type": "boolean", "default": false},
            {"name": "daily_cap", "description": "runs per day", "required": false,
             "type": "integer", "minimum": 1, "maximum": 24, "default": 3},
            {"name": "tenant", "description": "tenant UUID", "required": false,
             "type": "uuid", "default": null},
            {"name": "tenant_slug", "description": "tenant slug", "required": true,
             "type": "slug", "default": null}
        ])
    }

    fn param_schema() -> RecurringAgentParamSchema {
        serde_json::from_value(served_params()).unwrap()
    }

    #[test]
    fn param_schema_reads_and_writes_coords_served_array_byte_for_byte() {
        let schema = param_schema();
        assert_eq!(schema.params.len(), 6);
        assert!(schema.get("tenant_slug").unwrap().required);
        assert_eq!(
            schema.get("classes").unwrap().kind,
            RecurringAgentParamKind::IntegerSet {
                allowed: vec![1, 2, 3, 10, 12, 14, 16],
                excluded: vec![RecurringAgentExcludedValue {
                    value: 6,
                    reason: "it has its own sweep".to_string(),
                }],
                default: vec![3, 12, 14, 16],
            }
        );
        // coord's real shape: an excluded value need not be in `allowed`.
        // Re-serialized, it is the served array itself — no wrapper object,
        // no nested `kind`, `default: null` kept for uuid/slug.
        assert_eq!(round_trip(&schema), served_params());
        assert_eq!(schema.validate(), Ok(()));
    }

    #[test]
    fn a_param_def_without_required_defaults_to_optional() {
        let def: RecurringAgentParamDef = serde_json::from_value(
            json!({"name": "act", "description": "d", "type": "boolean", "default": true}),
        )
        .unwrap();
        assert!(!def.required);
        assert_eq!(def.kind.default_value(), Some(json!(true)));
    }

    #[test]
    fn param_schema_validate_refuses_bad_definitions() {
        let mut bad_default = param_schema();
        bad_default.params[0].kind = RecurringAgentParamKind::Enum {
            values: vec!["report".to_string()],
            default: "reap".to_string(),
        };
        assert!(matches!(
            bad_default.validate(),
            Err(RecurringAgentError::Param { .. })
        ));

        let mut inverted = param_schema();
        inverted.params[3].kind = RecurringAgentParamKind::Integer {
            minimum: 10,
            maximum: 1,
            default: 5,
        };
        assert!(matches!(
            inverted.validate(),
            Err(RecurringAgentError::Param { .. })
        ));

        for (allowed, excluded, default) in [
            (vec![], vec![], vec![1]),
            (vec![1, 1], vec![], vec![1]),
            (vec![1, 2], vec![], vec![]),
            (vec![1, 2], vec![(2, "no")], vec![2]),
            (vec![1, 2], vec![(2, " ")], vec![1]),
            (vec![1, 2], vec![], vec![3]),
        ] {
            let mut bad = param_schema();
            bad.params[1].kind = RecurringAgentParamKind::IntegerSet {
                allowed,
                excluded: excluded
                    .into_iter()
                    .map(|(value, reason)| RecurringAgentExcludedValue {
                        value,
                        reason: reason.to_string(),
                    })
                    .collect(),
                default,
            };
            assert!(
                matches!(bad.validate(), Err(RecurringAgentError::Param { .. })),
                "expected {:?} to be refused",
                bad.params[1].kind
            );
        }

        let mut bad_uuid = param_schema();
        bad_uuid.params[4].kind = RecurringAgentParamKind::Uuid {
            default: Some("not-a-uuid".to_string()),
        };
        assert!(bad_uuid.validate().is_err());
        let mut bad_slug = param_schema();
        bad_slug.params[5].kind = RecurringAgentParamKind::Slug {
            default: Some("Upper_Case".to_string()),
        };
        assert!(bad_slug.validate().is_err());

        let mut dup = param_schema();
        dup.params.push(dup.params[2].clone());
        assert!(matches!(
            dup.validate(),
            Err(RecurringAgentError::Duplicate { .. })
        ));
    }

    #[test]
    fn param_values_are_checked_the_way_coord_resolves_them() {
        let schema = param_schema();
        let ok = json!({
            "mode": "report", "classes": [16, 3], "act": true, "daily_cap": 24,
            "tenant": TENANT, "tenant_slug": "acme-co"
        });
        assert_eq!(schema.validate_params(ok.as_object().unwrap()), Ok(()));
        // null inherits the default, as coord treats it.
        let nulls = json!({"tenant_slug": "a", "mode": null, "tenant": null});
        assert_eq!(schema.validate_params(nulls.as_object().unwrap()), Ok(()));
        for bad in [
            json!({}), // tenant_slug is required with no default
            json!({"tenant_slug": null}),
            json!({"tenant_slug": "a", "unknown": 1}),
            json!({"tenant_slug": "a", "mode": "delete-everything"}),
            json!({"tenant_slug": "a", "mode": 1}),
            json!({"tenant_slug": "a", "act": "yes"}),
            json!({"tenant_slug": "a", "daily_cap": 25}),
            json!({"tenant_slug": "a", "daily_cap": 2.5}),
            json!({"tenant_slug": "a", "classes": []}),
            json!({"tenant_slug": "a", "classes": [6]}),
            json!({"tenant_slug": "a", "classes": [7]}),
            json!({"tenant_slug": "a", "classes": [3, 3]}),
            json!({"tenant_slug": "a", "classes": 3}),
            json!({"tenant_slug": "a", "tenant": "nope"}),
            json!({"tenant_slug": "-a"}),
            json!({"tenant_slug": "A"}),
            json!({"tenant_slug": "a".repeat(64)}),
        ] {
            assert!(
                matches!(
                    schema.validate_params(bad.as_object().unwrap()),
                    Err(RecurringAgentError::Param { .. })
                ),
                "expected {bad} to be refused"
            );
        }
    }

    #[test]
    fn param_slug_grammar_matches_coords() {
        for good in ["a", "a1", "acme-co", &"a".repeat(63)] {
            assert!(is_param_slug(good), "{good}");
        }
        for bad in ["", "-a", "a-", "A", "a_b", "a b", &"a".repeat(64)] {
            assert!(!is_param_slug(bad), "{bad}");
        }
    }

    // ---- RecurringAgentSpec ------------------------------------------------

    #[test]
    fn spec_round_trips_snake_case_with_target_object() {
        let mut spec = custom_spec();
        spec.params.insert("mode".to_string(), json!("report"));
        let v = round_trip(&spec);
        assert_eq!(v["catch_up_policy"], json!("skip"));
        assert_eq!(v["target"], json!({"kind": "all"}));
        assert_eq!(v["task_spec"]["task_type"], json!("RemoteAgent"));
        assert_eq!(v["params"], json!({"mode": "report"}));
        assert_eq!(spec.validate(), Ok(()));
    }

    #[test]
    fn spec_defaults_params_and_catch_up_policy() {
        let spec: RecurringAgentSpec = serde_json::from_value(json!({
            "name": "cleanup-steward",
            "catalog_ref": "cleanup-steward",
            "enabled": true,
            "schedule": {"type": "Cron", "value": "0 2 * * *"},
            "target": {"kind": "all"},
            "version": 1
        }))
        .unwrap();
        assert!(spec.params.is_empty());
        assert_eq!(spec.catch_up_policy, CatchUpPolicy::RunOnce);
        // A catalog agent may omit task_spec (a tenant row carries none).
        assert_eq!(spec.validate(), Ok(()));
        // params must be an object.
        assert!(serde_json::from_value::<RecurringAgentSpec>(json!({
            "name": "x", "enabled": true,
            "schedule": {"type": "Interval", "value": 60},
            "target": {"kind": "all"}, "version": 1, "params": []
        }))
        .is_err());
    }

    #[test]
    fn a_stored_catalog_row_with_coords_target_none_validates() {
        let row: RecurringAgentSpec = serde_json::from_value(json!({
            "name": "findings-steward",
            "catalog_ref": "findings-steward",
            "enabled": false,
            "schedule": {"type": "Cron", "value": "13 3 * * *"},
            "target": {"kind": "devices", "devices": []},
            "version": 1
        }))
        .unwrap();
        assert_eq!(row.validate_stored(), Ok(()));
    }

    #[test]
    fn custom_spec_must_be_a_remote_agent_without_a_probe() {
        let no_task = RecurringAgentSpec {
            task_spec: None,
            ..custom_spec()
        };
        assert_eq!(
            no_task.validate(),
            Err(RecurringAgentError::CustomWithoutTask)
        );

        let script = RecurringAgentSpec {
            task_spec: Some(ScheduledTaskType::Script {
                command: "rm -rf /".to_string(),
                working_directory: None,
                timeout_seconds: None,
            }),
            ..custom_spec()
        };
        assert_eq!(
            script.validate(),
            Err(RecurringAgentError::CustomTaskNotRemoteAgent {
                task_type: "Script"
            })
        );

        let probe = RecurringAgentSpec {
            conditions: Some(ScheduleConditions {
                require_probe: Some(ProbeCondition {
                    enabled: true,
                    command: vec!["sh".to_string(), "-c".to_string(), "true".to_string()],
                    poll_seconds: 60,
                    timeout_seconds: 10,
                }),
                ..ScheduleConditions::default()
            }),
            ..custom_spec()
        };
        assert_eq!(probe.validate(), Err(RecurringAgentError::CustomWithProbe));

        // The same Script task is allowed on a catalog agent: the runner checks
        // it against its own bundled pin instead.
        let catalog_script = RecurringAgentSpec {
            catalog_ref: Some("return-to-main".to_string()),
            ..script
        };
        assert_eq!(catalog_script.validate(), Ok(()));
    }

    #[test]
    fn spec_validate_refuses_bad_scalars() {
        let zero = RecurringAgentSpec {
            version: 0,
            ..custom_spec()
        };
        assert!(matches!(
            zero.validate(),
            Err(RecurringAgentError::VersionOutOfRange { .. })
        ));
        let digest = RecurringAgentSpec {
            digest: Some(DIGEST.to_uppercase()),
            ..custom_spec()
        };
        assert!(matches!(
            digest.validate(),
            Err(RecurringAgentError::BadDigest { .. })
        ));
        let name = RecurringAgentSpec {
            name: String::new(),
            ..custom_spec()
        };
        assert!(matches!(
            name.validate(),
            Err(RecurringAgentError::Empty { field: "name" })
        ));
    }

    #[test]
    fn managed_by_is_derived_from_the_spec() {
        let spec = RecurringAgentSpec {
            name: "cleanup-steward".to_string(),
            version: 2,
            ..custom_spec()
        };
        assert_eq!(
            spec.managed_by("7d6c1c2e-1b7a-4e43-9a43-6f1f0f3c2a11"),
            managed()
        );
    }

    // ---- Status report -----------------------------------------------------

    #[test]
    fn reason_is_required_exactly_for_unapplied_and_refused() {
        for state in [RecurringAgentState::Unapplied, RecurringAgentState::Refused] {
            assert_eq!(
                RecurringAgentStatus::new("a", state, None, None).unwrap_err(),
                RecurringAgentError::MissingReason { state }
            );
            assert_eq!(
                RecurringAgentStatus::new("a", state, Some("  ".to_string()), None).unwrap_err(),
                RecurringAgentError::MissingReason { state }
            );
            assert!(RecurringAgentStatus::new(
                "a",
                state,
                Some(UNAPPLIED_WORKSPACE_ROOT_UNSET.to_string()),
                None
            )
            .is_ok());
        }
        for state in [RecurringAgentState::Applied, RecurringAgentState::Paused] {
            assert!(RecurringAgentStatus::new("a", state, None, None).is_ok());
            assert_eq!(
                RecurringAgentStatus::new("a", state, Some(REFUSED_COMMAND.to_string()), None)
                    .unwrap_err(),
                RecurringAgentError::UnexpectedReason { state }
            );
        }
    }

    #[test]
    fn overridden_requires_an_override_record_and_paused_may_carry_one() {
        assert_eq!(
            RecurringAgentStatus::new("a", RecurringAgentState::Overridden, None, None)
                .unwrap_err(),
            RecurringAgentError::MissingOverride
        );
        assert!(RecurringAgentStatus::new(
            "a",
            RecurringAgentState::Overridden,
            None,
            Some(override_record())
        )
        .is_ok());
        assert!(RecurringAgentStatus::new(
            "a",
            RecurringAgentState::Paused,
            None,
            Some(override_record())
        )
        .is_ok());
        let empty = RecurringAgentOverride {
            schedule: None,
            ..override_record()
        };
        assert_eq!(
            RecurringAgentStatus::new("a", RecurringAgentState::Overridden, None, Some(empty))
                .unwrap_err(),
            RecurringAgentError::EmptyOverride
        );
    }

    #[test]
    fn status_report_round_trips_with_override_key_and_run_history() {
        let run = TaskExecutionRecord {
            execution_id: "e-1".to_string(),
            session_id: None,
            started_at: "2026-10-31T02:00:00Z".to_string(),
            ended_at: Some("2026-10-31T02:04:00Z".to_string()),
            status: ScheduledTaskStatus::Completed,
            success: true,
            error_message: None,
            triggered_auto_fix: false,
            auto_fix_session_id: None,
            scheduled_for: Some("2026-10-31T02:00:00Z".to_string()),
            catch_up_run: false,
        };
        let mut overridden = RecurringAgentStatus::new(
            "cleanup-steward",
            RecurringAgentState::Overridden,
            None,
            Some(override_record()),
        )
        .unwrap();
        overridden.applied_version = Some(2);
        overridden.applied_digest = Some(DIGEST.to_string());
        overridden.last_run = Some(run.clone());
        overridden.recent_runs = vec![run];
        overridden.last_completed_at = Some("2026-10-31T02:04:00Z".to_string());
        overridden.last_success_at = Some("2026-10-31T02:04:00Z".to_string());
        overridden.next_run = Some("2026-11-01T04:00:00Z".to_string());
        overridden.acknowledged_request_id =
            Some("0e7c5a52-4b5e-4a9c-9a0e-1d1d2f3a4b5c".to_string());
        let refused = RecurringAgentStatus::new(
            "return-to-main",
            RecurringAgentState::Refused,
            Some(REFUSED_COMMAND.to_string()),
            None,
        )
        .unwrap();
        let report = RecurringAgentStatusReport {
            tenant_id: "7d6c1c2e-1b7a-4e43-9a43-6f1f0f3c2a11".to_string(),
            reported_at: "2026-11-01T00:00:00Z".to_string(),
            accepts_managed: true,
            agents: vec![overridden, refused],
        };
        assert_eq!(report.validate(), Ok(()));
        let v = round_trip(&report);
        assert_eq!(v["agents"][0]["state"], json!("overridden"));
        assert!(v["agents"][0]["override"].is_object());
        assert!(v["agents"][0].get("override_record").is_none());
        assert!(v["agents"][0]["recent_runs"].is_array());
        assert_eq!(v["agents"][1]["unapplied_reason"], json!("refused_command"));
        assert!(v["agents"][1].get("override").is_none());
    }

    #[test]
    fn status_report_validate_names_the_bad_agent_and_refuses_duplicates() {
        let mut bad =
            RecurringAgentStatus::new("a", RecurringAgentState::Applied, None, None).unwrap();
        bad.applied_digest = Some("nope".to_string());
        let report = RecurringAgentStatusReport {
            tenant_id: TENANT.to_string(),
            reported_at: "2026-11-01T00:00:00Z".to_string(),
            accepts_managed: true,
            agents: vec![bad],
        };
        assert!(matches!(
            report.validate(),
            Err(RecurringAgentError::Agent { ref name, .. }) if name == "a"
        ));

        let applied =
            RecurringAgentStatus::new("a", RecurringAgentState::Applied, None, None).unwrap();
        let dup = RecurringAgentStatusReport {
            agents: vec![applied.clone(), applied],
            ..report
        };
        assert!(matches!(
            dup.validate(),
            Err(RecurringAgentError::Duplicate { what: "agent", .. })
        ));
    }

    #[test]
    fn status_state_vocabulary_matches_the_database_check() {
        let states = [
            (RecurringAgentState::Applied, "applied"),
            (RecurringAgentState::Paused, "paused"),
            (RecurringAgentState::Overridden, "overridden"),
            (RecurringAgentState::Unapplied, "unapplied"),
            (RecurringAgentState::Refused, "refused"),
        ];
        for (state, wire) in states {
            assert_eq!(serde_json::to_value(state).unwrap(), json!(wire));
        }
    }

    #[test]
    fn an_empty_reason_on_a_reasonless_state_is_refused() {
        for state in [RecurringAgentState::Applied, RecurringAgentState::Paused] {
            assert_eq!(
                RecurringAgentStatus::new("a", state, Some(String::new()), None).unwrap_err(),
                RecurringAgentError::UnexpectedReason { state }
            );
        }
    }

    #[test]
    fn status_validate_checks_column_types() {
        let base =
            RecurringAgentStatus::new("a", RecurringAgentState::Applied, None, None).unwrap();
        type Mutation = fn(&mut RecurringAgentStatus);
        let cases: [(Mutation, &str); 5] = [
            (
                |s| s.last_completed_at = Some("yesterday".to_string()),
                "timestamp",
            ),
            (
                |s| s.last_success_at = Some("2026-10-31".to_string()),
                "timestamp",
            ),
            (
                |s| s.next_run = Some("2026-10-31 02:00".to_string()),
                "timestamp",
            ),
            (
                |s| s.acknowledged_request_id = Some("req-1".to_string()),
                "uuid",
            ),
            (|s| s.applied_version = Some(i32::MAX as u32 + 1), "version"),
        ];
        for (mutate, what) in cases {
            let mut status = base.clone();
            mutate(&mut status);
            let err = status.validate().unwrap_err();
            let ok = match what {
                "timestamp" => matches!(err, RecurringAgentError::BadTimestamp { .. }),
                "uuid" => matches!(err, RecurringAgentError::BadUuid { .. }),
                _ => matches!(err, RecurringAgentError::VersionOutOfRange { .. }),
            };
            assert!(ok, "expected a {what} error, got {err:?}");
        }
        let mut good = base;
        good.next_run = Some("2026-11-01T04:00:00+01:00".to_string());
        good.acknowledged_request_id = Some("0e7c5a52-4b5e-4a9c-9a0e-1d1d2f3a4b5c".to_string());
        assert_eq!(good.validate(), Ok(()));
    }

    #[test]
    fn report_validate_checks_tenant_uuid_and_reported_at() {
        let report = RecurringAgentStatusReport {
            tenant_id: TENANT.to_string(),
            reported_at: "2026-11-01T00:00:00Z".to_string(),
            accepts_managed: true,
            agents: vec![],
        };
        assert_eq!(report.validate(), Ok(()));
        let bad_tenant = RecurringAgentStatusReport {
            tenant_id: "acme".to_string(),
            ..report.clone()
        };
        assert!(matches!(
            bad_tenant.validate(),
            Err(RecurringAgentError::BadUuid {
                field: "tenant_id",
                ..
            })
        ));
        let bad_time = RecurringAgentStatusReport {
            reported_at: "now".to_string(),
            ..report
        };
        assert!(matches!(
            bad_time.validate(),
            Err(RecurringAgentError::BadTimestamp {
                field: "reported_at",
                ..
            })
        ));
    }

    #[test]
    fn a_device_refusing_managed_tasks_reports_every_agent_as_refused() {
        let refused = RecurringAgentStatus::new(
            "a",
            RecurringAgentState::Unapplied,
            Some(UNAPPLIED_MANAGED_TASKS_REFUSED.to_string()),
            None,
        )
        .unwrap();
        let report = RecurringAgentStatusReport {
            tenant_id: TENANT.to_string(),
            reported_at: "2026-11-01T00:00:00Z".to_string(),
            accepts_managed: false,
            agents: vec![refused],
        };
        assert_eq!(report.validate(), Ok(()));
        for other in [
            RecurringAgentStatus::new("b", RecurringAgentState::Applied, None, None).unwrap(),
            RecurringAgentStatus::new(
                "b",
                RecurringAgentState::Unapplied,
                Some(UNAPPLIED_WORKSPACE_ROOT_UNSET.to_string()),
                None,
            )
            .unwrap(),
        ] {
            let mut bad = report.clone();
            bad.agents.push(other);
            assert_eq!(
                bad.validate(),
                Err(RecurringAgentError::Agent {
                    name: "b".to_string(),
                    error: Box::new(RecurringAgentError::NotRefusedWhileManagedOff),
                })
            );
        }
    }

    #[test]
    fn agent_error_display_prints_the_cause_once() {
        let err = RecurringAgentError::Agent {
            name: "a".to_string(),
            error: Box::new(RecurringAgentError::MissingOverride),
        };
        assert_eq!(
            err.to_string(),
            "agent \"a\": state overridden requires an override record"
        );
        assert!(std::error::Error::source(&err).is_none());
    }

    #[test]
    fn validate_stored_refuses_a_task_spec_on_a_catalog_row() {
        let served = RecurringAgentSpec {
            catalog_ref: Some("cleanup-steward".to_string()),
            ..custom_spec()
        };
        // Served shape: a catalog agent carries its rendered template.
        assert_eq!(served.validate(), Ok(()));
        assert_eq!(
            served.validate_stored(),
            Err(RecurringAgentError::CatalogWithTaskSpec)
        );
        let stored = RecurringAgentSpec {
            task_spec: None,
            ..served
        };
        assert_eq!(stored.validate_stored(), Ok(()));
        assert_eq!(custom_spec().validate_stored(), Ok(()));
    }

    #[test]
    fn sha256_hex_check() {
        assert!(is_sha256_hex(DIGEST));
        assert!(!is_sha256_hex(&DIGEST[1..]));
        assert!(!is_sha256_hex(&DIGEST.to_uppercase()));
        assert!(!is_sha256_hex(&format!("{}g", &DIGEST[1..])));
    }
}
