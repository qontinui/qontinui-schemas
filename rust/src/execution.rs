//! Execution-reporting DTO types.
//!
//! Wire-format types for the unified execution reporting API: lifecycle enums,
//! runner/workflow metadata, LLM metrics, and the request/response envelopes
//! used to create runs, report action executions, upload screenshots, report
//! issues, and complete runs.
//!
//! Ported from `qontinui-schemas/ts/src/execution/_api.ts` (tiers 1 + 2). Rust
//! is the source of truth; JSON Schema emitted from these types drives the TS
//! and Python bindings. The TS file's tier-3 types — real-time display state
//! (`ExecutionStatus`, `RoutingStatus`, `RetryStatus`, `CompressionStatus`,
//! `HookStatus`, `SubStepStatusDisplay`) and raw backend status events
//! (`Raw*Event`, `Raw*Payload`) — are hand-authored UI concerns that do not
//! belong in the shared wire-contract layer and are intentionally not ported.
//!
//! ## Conventions
//!
//! - Optional TS fields (`?`) map to `Option<T>` with
//!   `#[serde(default, skip_serializing_if = "Option::is_none")]`.
//! - Enum wire form is snake_case (e.g., `"qa_test"`, `"completed"`).
//! - Timestamps are ISO 8601 `String`s (see crate docs).
//! - Counts and indices are `u32`. Millisecond durations are `u64`. Second
//!   durations are `i64`. Floating-point values (coverage percentage, cost,
//!   confidence, average duration) use `f64`.
//! - `Record<string, unknown>` → `HashMap<String, serde_json::Value>`.
//! - `Record<string, number>` → `HashMap<String, u32>`.

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;

// ============================================================================
// Enums
// ============================================================================

/// Kind of execution run being reported.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum RunType {
    /// QA test run.
    QaTest,
    /// Integration test run.
    IntegrationTest,
    /// Live automation run in a production-like environment.
    LiveAutomation,
    /// Recording session used to capture workflow steps.
    Recording,
    /// Debug run used for troubleshooting.
    Debug,
}

/// Lifecycle status of an execution run.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum RunStatus {
    /// The run has been created but has not started yet.
    Pending,
    /// The run is currently executing.
    Running,
    /// The run finished successfully.
    Completed,
    /// The run finished with an error.
    Failed,
    /// The run exceeded its configured time limit.
    Timeout,
    /// The run was cancelled before natural completion.
    Cancelled,
    /// The run is paused and can be resumed.
    Paused,
}

/// Outcome of a single action within a run.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ActionStatus {
    /// The action completed successfully.
    Success,
    /// The action failed (e.g., assertion failure, element not found).
    Failed,
    /// The action exceeded its configured time limit.
    Timeout,
    /// The action was skipped (e.g., by a conditional).
    Skipped,
    /// The action errored at a system level (e.g., crash, network error).
    Error,
    /// The action has not been executed yet.
    Pending,
}

/// Type of action executed.
///
/// Covers vision, input, state-machine, control-flow, utility, AI, and custom
/// action kinds. Variants are serialized as snake_case strings.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
// Rename-on-schema only; Rust path stays `execution::ActionType`. The shared
// schema registry already has a `tree_events::ActionType` with a different
// meaning, so disambiguate at the JSON-Schema / generated-TS / generated-
// Python layer.
#[schemars(title = "ExecutionActionType")]
pub enum ActionType {
    // Vision actions
    /// Locate a single pattern in the current scene.
    Find,
    /// Locate all instances of a pattern in the current scene.
    FindAll,
    /// Wait for a pattern to appear.
    WaitFor,
    /// Wait for a pattern to disappear.
    WaitUntilGone,

    // Input actions
    /// Single mouse click.
    Click,
    /// Double mouse click.
    DoubleClick,
    /// Right mouse click.
    RightClick,
    /// Type text.
    Type,
    /// Press a single key.
    PressKey,
    /// Press a hotkey combination.
    Hotkey,
    /// Scroll the current view.
    Scroll,
    /// Drag from one point to another.
    Drag,

    // State machine actions
    /// Navigate to a named state.
    GoToState,
    /// Execute a state transition.
    Transition,
    /// Verify the current state matches expectations.
    VerifyState,

    // Control flow
    /// Conditional branch.
    Conditional,
    /// Loop construct.
    Loop,
    /// Parallel execution block.
    Parallel,
    /// Sequential block.
    Sequence,

    // Utility
    /// Wait for a fixed duration.
    Wait,
    /// Capture a screenshot.
    Screenshot,
    /// Log a message.
    Log,
    /// Assert a condition.
    Assert,

    // AI actions
    /// Run an AI prompt.
    AiPrompt,
    /// Run a sequence of AI prompts.
    RunPromptSequence,

    // Custom/plugin
    /// Custom plugin-defined action.
    Custom,
}

/// Category of error that caused an action to fail.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ErrorType {
    /// The target element could not be located.
    ElementNotFound,
    /// The action exceeded its configured time limit.
    Timeout,
    /// An assertion on the result failed.
    AssertionFailed,
    /// The runtime crashed.
    Crash,
    /// A network request failed.
    NetworkError,
    /// Input validation failed.
    ValidationError,
    /// An uncategorized error.
    Other,
}

/// Severity of an issue reported against a run.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum IssueSeverity {
    /// Critical severity — blocks release or execution.
    Critical,
    /// High severity.
    High,
    /// Medium severity.
    Medium,
    /// Low severity.
    Low,
    /// Informational only.
    Informational,
}

/// Kind of screenshot captured.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ScreenshotType {
    /// Captured on error.
    Error,
    /// Captured on success.
    Success,
    /// Manually requested.
    Manual,
    /// Captured on a periodic schedule.
    Periodic,
    /// Captured immediately after an action.
    ActionResult,
    /// Captured as part of state verification.
    StateVerification,
}

// ============================================================================
// Metadata Types
// ============================================================================

/// Information about the runner environment that produced the run.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct RunnerMetadata {
    /// Semantic version of the runner binary.
    pub runner_version: String,
    /// Operating system identifier (e.g., `"windows"`, `"macos"`, `"linux"`).
    pub os: String,
    /// Host machine name.
    pub hostname: String,
    /// Screen resolution as a free-form string (e.g., `"1920x1080"`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub screen_resolution: Option<String>,
    /// CPU description.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cpu_info: Option<String>,
    /// Installed system memory in megabytes.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub memory_mb: Option<u32>,
    /// Arbitrary additional runner context.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub extra: Option<HashMap<String, Value>>,
}

/// Information about the workflow being executed in the run.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct WorkflowMetadata {
    /// Workflow identifier.
    pub workflow_id: String,
    /// Human-readable workflow name.
    pub workflow_name: String,
    /// Workflow version, if tracked.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub workflow_version: Option<String>,
    /// Number of states declared by the workflow.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub total_states: Option<u32>,
    /// Number of transitions declared by the workflow.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub total_transitions: Option<u32>,
    /// Free-form tags attached to the workflow.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tags: Option<Vec<String>>,
    /// Workflow description.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// IDs of the states that are active when the workflow starts.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub initial_state_ids: Option<Vec<String>>,
}

/// Aggregate execution statistics for a completed run.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct ExecutionStats {
    /// Total number of actions executed.
    pub total_actions: u32,
    /// Number of actions with [`ActionStatus::Success`].
    pub successful_actions: u32,
    /// Number of actions with [`ActionStatus::Failed`].
    pub failed_actions: u32,
    /// Number of actions with [`ActionStatus::Timeout`].
    pub timeout_actions: u32,
    /// Number of actions with [`ActionStatus::Skipped`].
    pub skipped_actions: u32,
    /// Sum of all action durations, in milliseconds.
    pub total_duration_ms: u64,
    /// Mean per-action duration, in milliseconds.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub avg_action_duration_ms: Option<f64>,
    /// Aggregate input tokens across all LLM actions.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub total_tokens_input: Option<u64>,
    /// Aggregate output tokens across all LLM actions.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub total_tokens_output: Option<u64>,
    /// Aggregate estimated cost in USD across all LLM actions.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub total_cost_usd: Option<f64>,
    /// Number of actions that used an LLM.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub llm_action_count: Option<u32>,
}

/// Coverage data computed for a workflow run.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct CoverageData {
    /// Overall coverage as a percentage in the range `[0.0, 100.0]`.
    pub coverage_percentage: f64,
    /// Number of states visited at least once.
    pub states_covered: u32,
    /// Total number of states in the workflow.
    pub total_states: u32,
    /// Number of transitions executed at least once.
    pub transitions_covered: u32,
    /// Total number of transitions in the workflow.
    pub total_transitions: u32,
    /// IDs of states that were not visited.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub uncovered_states: Option<Vec<String>>,
    /// IDs of transitions that were not executed.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub uncovered_transitions: Option<Vec<String>>,
    /// Per-state visit counts, keyed by state ID.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub state_visit_counts: Option<HashMap<String, u32>>,
    /// Per-transition execution counts, keyed by transition ID.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub transition_execution_counts: Option<HashMap<String, u32>>,
}

// ============================================================================
// LLM Metrics
// ============================================================================

/// Token usage and cost metrics for an LLM-powered action.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct LLMMetrics {
    /// LLM model identifier.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub model: Option<String>,
    /// Provider name (e.g., `"anthropic"`, `"openai"`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub provider: Option<String>,
    /// Input/prompt token count.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tokens_input: Option<u64>,
    /// Completion token count.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tokens_output: Option<u64>,
    /// Computed total token count.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tokens_total: Option<u64>,
    /// Estimated cost in USD.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cost_usd: Option<f64>,
    /// Generation parameters (temperature, max_tokens, etc.).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub generation_params: Option<HashMap<String, Value>>,
}

// ============================================================================
// Request / Response Types — Run Lifecycle
// ============================================================================

/// Request payload for creating a new execution run.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct ExecutionRunCreate {
    /// Owning project ID.
    pub project_id: String,
    /// Kind of run being created.
    pub run_type: RunType,
    /// Human-readable run name.
    pub run_name: String,
    /// Optional free-form description.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// Runner environment metadata.
    pub runner_metadata: RunnerMetadata,
    /// Workflow metadata, if the run executes a workflow.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub workflow_metadata: Option<WorkflowMetadata>,
    /// Run configuration bag (opaque to this layer).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub configuration: Option<HashMap<String, Value>>,
}

/// Response envelope returned when a run is created or fetched.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct ExecutionRunResponse {
    /// Assigned run identifier.
    pub run_id: String,
    /// Owning project ID.
    pub project_id: String,
    /// Kind of run.
    pub run_type: RunType,
    /// Human-readable run name.
    pub run_name: String,
    /// Current lifecycle status.
    pub status: RunStatus,
    /// ISO 8601 timestamp when the run started.
    pub started_at: String,
    /// ISO 8601 timestamp when the run ended, if it has ended.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ended_at: Option<String>,
    /// Total duration in seconds, if the run has ended.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub duration_seconds: Option<f64>,
}

// ============================================================================
// Request / Response Types — Action Executions
// ============================================================================

/// Pixel coordinates and optional size of a matched pattern.
///
/// Inline object on [`ActionExecutionCreate::match_location`]. Lifted to a
/// named struct so it round-trips through JSON Schema.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
// Rename-on-schema only; see note on `ActionType` above — `tree_events` has
// a structurally different `MatchLocation` on the shared schema registry.
#[schemars(title = "ExecutionMatchLocation")]
pub struct MatchLocation {
    /// X coordinate in pixels.
    pub x: i32,
    /// Y coordinate in pixels.
    pub y: i32,
    /// Match width in pixels.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub width: Option<u32>,
    /// Match height in pixels.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub height: Option<u32>,
}

/// Request payload for reporting a single action execution.
///
/// Usually submitted as part of a batch; see [`ActionExecutionResponse`].
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct ActionExecutionCreate {
    /// Zero-based sequence number of the action within the run.
    pub sequence_number: u32,
    /// Kind of action.
    pub action_type: ActionType,
    /// Human-readable action name.
    pub action_name: String,
    /// Outcome of the action.
    pub status: ActionStatus,
    /// ISO 8601 timestamp when the action started.
    pub started_at: String,
    /// ISO 8601 timestamp when the action completed.
    pub completed_at: String,
    /// Action duration in milliseconds.
    pub duration_ms: u64,
    /// Source state ID for state-machine actions.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub from_state: Option<String>,
    /// Destination state ID for state-machine actions.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub to_state: Option<String>,
    /// IDs of states active when the action ran.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub active_states: Option<Vec<String>>,
    /// ID of the pattern the action targeted.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub pattern_id: Option<String>,
    /// Human-readable pattern name.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub pattern_name: Option<String>,
    /// Confidence score of the match in the range `[0.0, 1.0]`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub confidence_score: Option<f64>,
    /// Pixel location of the match.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub match_location: Option<MatchLocation>,
    /// Error message if the action failed.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub error_message: Option<String>,
    /// Category of error, if any.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub error_type: Option<ErrorType>,
    /// Captured stack trace, if any.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub error_stack: Option<String>,
    /// ID of a screenshot associated with the action.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub screenshot_id: Option<String>,
    /// ID of a parent action, if this is a sub-action.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub parent_action_id: Option<String>,
    /// Opaque input data captured for the action.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub input_data: Option<HashMap<String, Value>>,
    /// Opaque output data produced by the action.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub output_data: Option<HashMap<String, Value>>,
    /// Opaque additional metadata.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub metadata: Option<HashMap<String, Value>>,
    /// LLM token and cost metrics, if the action used an LLM.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub llm_metrics: Option<LLMMetrics>,
    /// Span type for tracing (e.g., `"llm"`, `"tool"`, `"agent"`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub span_type: Option<String>,
    /// Trace ID correlating related actions.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub trace_id: Option<String>,
    /// Parent action ID for child actions within a sequence.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub parent_id: Option<String>,
}

/// Response envelope returned after reporting action executions.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct ActionExecutionResponse {
    /// Number of actions recorded.
    pub recorded: u32,
    /// Associated run ID.
    pub run_id: String,
    /// Assigned action IDs, in the same order as the submitted batch.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub action_ids: Option<Vec<String>>,
}

// ============================================================================
// Request / Response Types — Screenshots
// ============================================================================

/// Kind of shape overlaid on a screenshot annotation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ScreenshotAnnotationShape {
    /// A rectangle.
    Box,
    /// A circle.
    Circle,
    /// An arrow.
    Arrow,
    /// A text label.
    Text,
}

/// Annotation overlaid on a screenshot (box, circle, arrow, or text).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct ScreenshotAnnotation {
    /// Shape of the annotation.
    #[serde(rename = "type")]
    pub shape: ScreenshotAnnotationShape,
    /// X coordinate in pixels.
    pub x: i32,
    /// Y coordinate in pixels.
    pub y: i32,
    /// Annotation width in pixels.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub width: Option<u32>,
    /// Annotation height in pixels.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub height: Option<u32>,
    /// Free-form label for the annotation.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub label: Option<String>,
    /// CSS-style color string (e.g., `"#FF0000"`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub color: Option<String>,
}

/// Request payload describing a screenshot being uploaded.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct ExecutionScreenshotCreate {
    /// Client-generated screenshot identifier.
    pub screenshot_id: String,
    /// Sequence number of the screenshot within the run.
    pub sequence_number: u32,
    /// Kind of screenshot.
    pub screenshot_type: ScreenshotType,
    /// ISO 8601 timestamp when the screenshot was taken.
    pub timestamp: String,
    /// Image width in pixels.
    pub width: u32,
    /// Image height in pixels.
    pub height: u32,
    /// Sequence number of the associated action, if any.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub action_sequence_number: Option<u32>,
    /// State ID active when the screenshot was taken.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub state: Option<String>,
    /// IDs of states active when the screenshot was taken.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub active_states: Option<Vec<String>>,
    /// Overlaid annotations.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub annotations: Option<Vec<ScreenshotAnnotation>>,
    /// Opaque additional metadata.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub metadata: Option<HashMap<String, Value>>,
}

/// Response envelope returned after uploading a screenshot.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct ExecutionScreenshotResponse {
    /// Assigned screenshot identifier.
    pub screenshot_id: String,
    /// Associated run ID.
    pub run_id: String,
    /// URL to the full-size image.
    pub image_url: String,
    /// URL to a thumbnail image, if generated.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub thumbnail_url: Option<String>,
    /// ISO 8601 timestamp when the image was uploaded.
    pub uploaded_at: String,
    /// File size in bytes.
    pub file_size_bytes: u64,
}

// ============================================================================
// Request / Response Types — Issues
// ============================================================================

/// Request payload for reporting an issue observed during a run.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct ExecutionIssueCreate {
    /// Short human-readable title.
    pub title: String,
    /// Full issue description.
    pub description: String,
    /// Severity.
    pub severity: IssueSeverity,
    /// Free-form issue type label (e.g., `"visual_regression"`, `"flaky"`).
    pub issue_type: String,
    /// Sequence number of the associated action, if any.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub action_sequence_number: Option<u32>,
    /// State ID active when the issue was observed.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub state: Option<String>,
    /// IDs of screenshots illustrating the issue.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub screenshot_ids: Option<Vec<String>>,
    /// Steps to reproduce the issue.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reproduction_steps: Option<Vec<String>>,
    /// Expected behavior.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub expected_behavior: Option<String>,
    /// Actual observed behavior.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub actual_behavior: Option<String>,
    /// Opaque additional metadata.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub metadata: Option<HashMap<String, Value>>,
}

/// Response envelope returned after reporting issues.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct ExecutionIssueResponse {
    /// Number of issues recorded.
    pub recorded: u32,
    /// Associated run ID.
    pub run_id: String,
    /// Assigned issue IDs, in the same order as the submitted batch.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub issue_ids: Option<Vec<String>>,
}

// ============================================================================
// Request / Response Types — Run Completion
// ============================================================================

/// Request payload for marking a run complete.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct ExecutionRunComplete {
    /// Final lifecycle status.
    pub status: RunStatus,
    /// ISO 8601 timestamp when the run ended.
    pub ended_at: String,
    /// Aggregate execution statistics.
    pub stats: ExecutionStats,
    /// Coverage data, if the run executed a workflow.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub coverage: Option<CoverageData>,
    /// Free-form run summary.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub summary: Option<String>,
    /// Error message if the run failed.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub error_message: Option<String>,
}

/// Response envelope returned after completing a run.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct ExecutionRunCompleteResponse {
    /// Associated run ID.
    pub run_id: String,
    /// Final lifecycle status.
    pub status: RunStatus,
    /// ISO 8601 timestamp when the run started.
    pub started_at: String,
    /// ISO 8601 timestamp when the run ended.
    pub ended_at: String,
    /// Total duration in seconds.
    pub duration_seconds: f64,
    /// Aggregate execution statistics.
    pub stats: ExecutionStats,
    /// Coverage data, if the run executed a workflow.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub coverage: Option<CoverageData>,
}
