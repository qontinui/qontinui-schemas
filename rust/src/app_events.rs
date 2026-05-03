//! Application event envelope types.
//!
//! These are the Tauri-to-frontend event types. The runner emits [`AppEvent`]
//! instances over Tauri's event channel and WebSocket; the React frontend
//! dispatches on the `event_type` discriminator and reads the `data` payload.
//!
//! [`FlowEvent`] is a sub-envelope for flow-execution lifecycle events and is
//! wrapped by `AppEvent::FlowEvent`.

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;

// ============================================================================
// FlowEvent — internally tagged on "type"
// ============================================================================

/// Events emitted during flow execution for UI updates.
///
/// Internally tagged with `"type"` and snake_case variant names.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum FlowEvent {
    /// Flow execution started.
    FlowStarted {
        instance_id: String,
        flow_id: String,
        flow_name: String,
    },
    /// A step is about to execute.
    StepStarted {
        instance_id: String,
        step_id: String,
        step_name: String,
        step_type: String,
    },
    /// A step completed.
    StepCompleted {
        instance_id: String,
        step_id: String,
        success: bool,
        outputs: HashMap<String, Value>,
        #[serde(skip_serializing_if = "Option::is_none")]
        error: Option<String>,
        duration_ms: u64,
    },
    /// Flow execution completed.
    FlowCompleted {
        instance_id: String,
        flow_id: String,
        success: bool,
        #[serde(skip_serializing_if = "Option::is_none")]
        error: Option<String>,
        total_steps: usize,
        duration_ms: u64,
    },
    /// Waiting for human input.
    WaitingForInput {
        instance_id: String,
        step_id: String,
        prompt: String,
        options: Vec<String>,
    },
    /// Progress update during parallel execution.
    ParallelProgress {
        instance_id: String,
        step_id: String,
        completed: usize,
        total: usize,
    },
    /// Flow execution was paused via `pause_flow_execution`.
    ///
    /// Emitted on the `flow-event` Tauri channel by
    /// `qontinui-runner/src-tauri/src/commands/flow.rs::pause_flow_execution`
    /// and consumed by `useFlowExecutionData.ts`. `step_id` is intentionally
    /// emitted as `null` when no current step (rather than skipped) to
    /// preserve the legacy `serde_json::json!` wire shape.
    FlowPaused {
        instance_id: String,
        flow_id: String,
        /// Step the flow was paused at (`null` if no current step).
        step_id: Option<String>,
    },
    /// Flow execution was resumed via `resume_flow_execution` after a pause.
    ///
    /// Emitted on the `flow-event` Tauri channel by
    /// `qontinui-runner/src-tauri/src/commands/flow.rs::resume_flow_execution`.
    /// `step_id` is intentionally emitted as `null` when no current step.
    FlowResumed {
        instance_id: String,
        flow_id: String,
        /// Step execution will resume from (`null` if no current step).
        step_id: Option<String>,
    },
}

// ============================================================================
// AppEvent — adjacently tagged on "event_type" + "data"
// ============================================================================

/// Unified application events for frontend communication.
///
/// Adjacently tagged: each variant serializes as
/// `{"event_type": "<VariantName>", "data": {..}}`.
/// The React frontend dispatches on `event_type` and reads `data`.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "event_type", content = "data")]
pub enum AppEvent {
    // ── Executor Events ──
    /// Standard executor event with data payload.
    ExecutorEvent {
        event: String,
        timestamp: i64,
        sequence: u32,
        data: Value,
    },

    /// Tree event from workflow execution.
    ExecutorTreeEvent {
        event_type: String,
        node: Value,
        path: Vec<String>,
        timestamp: i64,
        sequence: u32,
    },

    /// Executor error event.
    ExecutorError {
        message: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        details: Option<String>,
    },

    /// Executor response for command completion.
    ExecutorResponse {
        id: String,
        success: bool,
        #[serde(skip_serializing_if = "Option::is_none")]
        data: Option<Value>,
        #[serde(skip_serializing_if = "Option::is_none")]
        error: Option<String>,
    },

    /// Image recognition result event.
    ImageRecognition { data: Value },

    // ── Extraction Events ──
    /// Web extraction event with data payload.
    ExtractionEvent {
        event: String,
        timestamp: i64,
        sequence: u32,
        data: Value,
    },

    /// Web extraction error event.
    ExtractionError {
        message: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        details: Option<String>,
    },

    /// Web extraction response.
    ExtractionResponse {
        id: String,
        success: bool,
        #[serde(skip_serializing_if = "Option::is_none")]
        data: Option<Value>,
        #[serde(skip_serializing_if = "Option::is_none")]
        error: Option<String>,
    },

    // ── RAG Events ──
    /// RAG processing progress update.
    RagProgress {
        project_id: String,
        status: String,
        message: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        percent: Option<i32>,
        #[serde(skip_serializing_if = "Option::is_none")]
        elements_processed: Option<i32>,
        #[serde(skip_serializing_if = "Option::is_none")]
        total_elements: Option<i32>,
        #[serde(skip_serializing_if = "Option::is_none")]
        error: Option<String>,
    },

    /// RAG processing completion event.
    RagCompletion {
        project_id: String,
        success: bool,
        total_processed: i32,
        successful: i32,
        failed: i32,
        web_sync_success: bool,
        #[serde(skip_serializing_if = "Option::is_none")]
        web_sync_error: Option<String>,
    },

    // ── Flow Events ──
    /// Flow execution event (wraps [`FlowEvent`]).
    FlowEvent(FlowEvent),

    // ── AI Output Events ──
    /// AI session output event.
    AiOutput {
        session_id: String,
        content: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        content_type: Option<String>,
    },

    // ── Findings Events ──
    /// Finding detected during AI analysis.
    FindingDetected { finding: Value },

    /// Finding resolved.
    FindingResolved { finding: Value },

    // ── Navigation Events ──
    /// Test navigation event for UI.
    TestNavigation { data: Value },

    /// UI bridge request event.
    UiBridgeRequest { data: Value },

    // ── Orchestrator State Events ──
    /// Orchestrator state change event for real-time UI updates.
    OrchestratorStateChange {
        /// Task run ID.
        task_run_id: String,
        /// Current workflow stage name.
        workflow_stage: String,
        /// Current iteration number.
        iteration: u32,
        /// Current phase (setup, verification, agentic, completion).
        phase: String,
        /// Optional additional state data.
        #[serde(skip_serializing_if = "Option::is_none")]
        state_data: Option<Value>,
    },

    /// Step progress event for tracking individual steps.
    StepProgress {
        /// Task run ID.
        task_run_id: String,
        /// Step index (0-based).
        step_index: usize,
        /// Step name/description.
        step_name: String,
        /// Status: "started", "running", "completed", "failed", "skipped".
        status: String,
        /// Optional details about the step.
        #[serde(skip_serializing_if = "Option::is_none")]
        details: Option<Value>,
        /// Timestamp in milliseconds.
        timestamp: i64,
    },

    /// Task run update event for status changes.
    TaskRunUpdate {
        /// Task run ID.
        task_run_id: String,
        /// Status: "running", "completed", "failed", "stopped", "paused".
        status: String,
        /// Current iteration (if applicable).
        #[serde(skip_serializing_if = "Option::is_none")]
        iteration: Option<u32>,
        /// Optional additional details.
        #[serde(skip_serializing_if = "Option::is_none")]
        details: Option<Value>,
        /// Timestamp in milliseconds.
        timestamp: i64,
    },

    // ── Approval Events ──
    /// Approval required -- workflow is paused waiting for human review.
    ApprovalRequired {
        /// Task run ID.
        task_run_id: String,
        /// Approval request ID.
        approval_id: String,
        /// Current iteration.
        iteration: u32,
        /// Prompt shown to the reviewer.
        prompt: String,
    },

    /// Approval resolved -- human responded to approval request.
    ApprovalResolved {
        /// Task run ID.
        task_run_id: String,
        /// Approval request ID.
        approval_id: String,
        /// Whether approved.
        approved: bool,
        /// Action taken.
        action: String,
    },

    // ── Deferred Feedback Events ──
    /// A deferred question was created during autonomous execution.
    DeferredQuestionCreated {
        /// Task run ID.
        task_run_id: String,
        /// Deferred question ID.
        question_id: String,
        /// Iteration when the question was raised.
        iteration: u32,
        /// The question text.
        question: String,
        /// Confidence score (0.0-1.0).
        confidence: f64,
        /// Risk level.
        risk_level: String,
    },

    /// A deferred question was reviewed by a human.
    DeferredQuestionReviewed {
        /// Task run ID.
        task_run_id: String,
        /// Deferred question ID.
        question_id: String,
        /// Review status: "approved" or "rejected".
        status: String,
        /// Whether rework was triggered.
        rework_triggered: bool,
    },

    // ── Canvas Events ──
    /// Canvas panel created, updated, or removed.
    ///
    /// Wraps a [`CanvasPanel`] when `action` is `"create"` / `"update"`;
    /// `panel` is `None` for `"delete"` and `"clear"`. `panel.data` remains
    /// a free-form `serde_json::Value` because the inner shape varies per
    /// component type (Markdown, CodeDiff, Table, …) — the canonical map
    /// is in `qontinui-schemas/ts/src/canvas/index.ts`.
    CanvasUpdate {
        action: String,
        panel_id: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        panel: Option<CanvasPanel>,
        #[serde(skip_serializing_if = "Option::is_none")]
        task_run_id: Option<String>,
    },

    // ── AI Output Streaming Events ──
    /// Real-time AI output chunk for live streaming to the frontend.
    AiOutputChunk {
        /// Task run ID this output belongs to.
        task_run_id: String,
        /// The text chunk received from the AI.
        chunk: String,
        /// Total accumulated output length so far.
        accumulated_length: usize,
    },

    // ── Convergence Tracking Events ──
    /// Per-iteration metrics for tracking convergence.
    IterationMetrics {
        /// Task run ID.
        task_run_id: String,
        /// Current iteration number (1-indexed).
        iteration: u32,
        /// Number of verification steps that failed.
        failed_step_count: u32,
        /// Number of verification steps that passed.
        passed_step_count: u32,
        /// Number of verification steps that were skipped.
        skipped_step_count: u32,
        /// Failures not present in the previous iteration.
        new_failures: u32,
        /// Failures that were also present in the previous iteration.
        repeated_failures: u32,
        /// True if failed_step_count has not decreased in 3 consecutive iterations.
        is_stalled: bool,
    },

    // ── Blame Attribution Events ──
    /// Blame attribution results from the blame engine.
    BlameAttribution {
        /// Task run ID.
        task_run_id: String,
        /// Current iteration number.
        iteration: u32,
        /// Number of failures with blame attributions.
        attributed_failures: u32,
        /// Number of files exhibiting oscillation.
        oscillating_files: u32,
        /// Number of files exhibiting revert patterns.
        revert_patterns: u32,
        /// Full blame report as JSON.
        blame_json: String,
    },

    // ── Constraint Engine Events ──
    /// Constraint evaluation results after an agentic phase.
    ConstraintResults {
        /// Task run ID.
        task_run_id: String,
        /// Current iteration number (1-indexed).
        iteration: u32,
        /// Human-readable summary of results.
        summary: String,
        /// Whether any blocking violations exist.
        has_blocking: bool,
        /// Serialized constraint results.
        results: Value,
    },

    // ── Queue Events ──
    /// Workflow has been added to the execution queue.
    WorkflowQueued {
        /// Task run ID for the queued workflow.
        task_run_id: String,
        /// Human-readable workflow name.
        workflow_name: String,
        /// Position in the queue (0-indexed).
        queue_position: usize,
    },

    /// Workflow has been dequeued and is starting execution.
    WorkflowDequeued {
        /// Task run ID for the dequeued workflow.
        task_run_id: String,
        /// Human-readable workflow name.
        workflow_name: String,
        /// Time spent waiting in the queue, in milliseconds.
        wait_time_ms: u64,
    },

    // ── Cost Management Events ──
    /// Real-time cost update after each AI call.
    CostUpdate {
        task_run_id: String,
        phase: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        iteration: Option<u32>,
        input_tokens: u64,
        output_tokens: u64,
        cache_creation_tokens: u64,
        cache_read_tokens: u64,
        cost_usd: f64,
        cumulative_cost_usd: f64,
        cache_hit_rate: f64,
        timestamp: i64,
    },

    /// Budget warning when consumption exceeds 80%.
    BudgetWarning {
        task_run_id: String,
        remaining_fraction: f64,
        total_cost_usd: f64,
        budget_limit_usd: f64,
        message: String,
        timestamp: i64,
    },

    /// Cost anomaly detected via statistical analysis.
    CostAnomaly {
        task_run_id: String,
        cost_usd: f64,
        mean_cost_usd: f64,
        std_dev: f64,
        z_score: f64,
        message: String,
        timestamp: i64,
    },

    // ── Generic Events ──
    /// Generic error event.
    Error {
        message: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        context: Option<String>,
    },
}

// ============================================================================
// CanvasPanel — wire-format mirror of runner StoredPanel
// ============================================================================

/// A canvas panel rendered in the dashboard widget.
///
/// Wire-format mirror of the runner's `StoredPanel` struct
/// (`qontinui-runner/src-tauri/src/mcp/canvas.rs`). The runner emits this
/// inside [`AppEvent::CanvasUpdate`] via the `canvas-update` Tauri channel.
/// Field names are snake_case to match the existing TS interface in
/// `qontinui-schemas/ts/src/canvas/index.ts`.
///
/// `data` stays `serde_json::Value` because each `component` type has a
/// different inner shape (Markdown, CodeDiff, Table, …); the per-component
/// data schemas live in the TS module above and are intentionally not
/// modeled as a Rust discriminated union (would balloon the schema for
/// little gain on the Rust side).
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct CanvasPanel {
    pub panel_id: String,
    pub component: String,
    pub title: String,
    pub data: Value,
    pub priority: i32,
    pub size: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub group: Option<String>,
    pub task_run_id: String,
    pub created_at: String,
    pub updated_at: String,
}
