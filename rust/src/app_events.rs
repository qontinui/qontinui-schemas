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
// UI Bridge IPC envelopes — request/response over the `ui-bridge-request` /
// `ui-bridge-response` Tauri channels
// ============================================================================

/// Wire envelope for `ui-bridge-request` events emitted by
/// `mcp::ui_bridge::request::ui_bridge_request_sync` (and the long-timeout
/// variant in `mcp::ui_bridge::screenshots::ui_bridge_capture_element_images_handler`).
///
/// The runner emits an envelope whose flat shape is
/// `{ requestId, type, ...payloadFields }` — the previous implementation built
/// it via `serde_json::json!({ "requestId": ..., "type": ... })` plus a manual
/// merge of an `additional_payload: Value::Object`. This struct preserves that
/// wire shape via `#[serde(flatten)]` on `data` so byte output is identical
/// when callers pass `Value::Object` payloads (the only shape they use today).
///
/// The discriminator field is **`type`** on the wire; the Rust field is named
/// `request_type` for ergonomics and renamed via `#[serde(rename = "type")]`.
/// Keep this struct in sync with the TS `UIBridgeRequestPayload` envelope head
/// in `qontinui-runner/src/hooks/ui-bridge-events/types.ts`.
///
/// `request_type` is a free-form `String` — the set of valid discriminators is
/// the 171-variant `UIBridgeRequestType` TS union, but per-payload typing is
/// deferred indefinitely. Dispatch exhaustiveness is enforced on the React
/// side via `AssertEqual<AllHandledTypes, UIBridgeRequestType>` plus per-hook
/// `never` checks.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct UiBridgeRequestEnvelope {
    /// UUID generated by the runner; round-trips back inside the matching
    /// [`UiBridgeResponseEnvelope`] so the Rust dispatcher can pair responses
    /// with their pending oneshot senders.
    pub request_id: String,
    /// Wire discriminator — the runner-side request kind (e.g. `"get_elements"`,
    /// `"capture_element_images"`). Field is named `request_type` in Rust for
    /// ergonomics; serializes as `type`.
    #[serde(rename = "type")]
    pub request_type: String,
    /// Per-`type` extra payload fields. Flattened so the wire shape is
    /// `{ requestId, type, ...data }`. Always a `Value::Object` at the call
    /// sites today; see the round-trip test for the byte-identity guarantee.
    #[serde(flatten)]
    pub data: Value,
}

/// Wire envelope for `ui-bridge-response` events emitted by the React
/// frontend in response to a [`UiBridgeRequestEnvelope`].
///
/// The runner-side dispatcher in
/// `mcp::ui_bridge::request::handle_ui_bridge_response` matches `request_id`
/// to a pending oneshot sender and forwards the inner `data` to the awaiting
/// caller; `success`, `error`, `hint`, and `timestamp` are surfaced via
/// `wrap_ipc_result`'s F2 two-tier envelope flattening.
///
/// `data`, `error`, and `hint` are all optional. The hint sibling carries
/// closest-match / recovery hints (set by frontend handlers like
/// `useControlEvents` for typo recovery on element-not-found and
/// action-not-allowed) and stays a sibling of `error` — the success/error
/// envelope shape is unchanged.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct UiBridgeResponseEnvelope {
    /// Echoes the request's [`UiBridgeRequestEnvelope::request_id`].
    pub request_id: String,
    /// Echoes the request's [`UiBridgeRequestEnvelope::request_type`].
    /// Serializes as `type` to match the wire.
    #[serde(rename = "type")]
    pub request_type: String,
    /// Whether the frontend handler succeeded.
    pub success: bool,
    /// Inner data payload on success. Frontend handlers set this to whatever
    /// shape the operation returns (e.g. an `elements` array for
    /// `get_elements`, a discovery report for `discover`, etc.).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub data: Option<Value>,
    /// Error message on failure. Mirrors the inner-failure path of
    /// `wrap_ipc_result`'s F2 two-tier envelope flattening.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
    /// Optional closest-match / recovery hint payload. Sibling field to
    /// `error` (does NOT replace the success/error envelope shape). Used by
    /// `useControlEvents` for element-not-found / action-not-allowed typo
    /// recovery and by Rust `page.rs` for eval-rejected workaround guidance.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub hint: Option<Value>,
    /// Frontend-side timestamp (ms since Unix epoch) of when the response was
    /// produced.
    pub timestamp: i64,
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

// ============================================================================
// UI Bridge HTTP envelopes — the Next.js `/api/ui-bridge/*` HTTP surface
// ============================================================================
//
// These model the JSON bodies the *HTTP* UI-Bridge surface returns, as
// distinct from [`UiBridgeResponseEnvelope`] above (which is the runner's
// WS/relay IPC envelope and always carries `requestId` / `type` / `timestamp`).
// The HTTP envelopes below lack those fields, so they need their own canonical
// types — `conforms_to: "UiBridgeResponseEnvelope"` correctly rejects them.
//
// They are the source-of-truth for the Spec-CI `conforms_to` C1 contract
// checks on `frontend/specs/pages/ui-bridge-states/state-machine.derived.json`
// in qontinui-web, which validate the live `/api/ui-bridge/*` responses
// against these schemas via ajv.

/// Canonical error envelope for the qontinui-web Next.js UI-Bridge proxy.
///
/// The catch-all route at
/// `qontinui-web/frontend/src/app/api/ui-bridge/[...path]/route.ts`
/// short-circuits requests the SDK would otherwise 404 on — and the
/// browser-required routes that can't respond without a live SDK client —
/// to a structured HTTP 503 with this body
/// (`route.ts::noBrowserResponse`):
///
/// ```json
/// { "success": false, "code": "NO_BROWSER_CONNECTED",
///   "message": "<path> requires a browser SDK client" }
/// ```
///
/// Modeled as a *generic* error envelope rather than a one-off
/// `NoBrowserConnected` type: the `{success:false, code, message}` shape is
/// the canonical structured-error contract for the whole proxy surface (not
/// just the no-browser path), so a single reusable type is the more
/// scalable and cleaner home. `code` stays a free-form `String` — the
/// discriminator set (currently just `NO_BROWSER_CONNECTED`) is expected to
/// grow, and pinning it to an enum here would force a schemas release on every
/// new proxy error code. `success` is always `false` on this envelope (the
/// success path uses [`UiBridgeHttpHealthEnvelope`] / the SDK's own envelopes).
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct UiBridgeHttpErrorEnvelope {
    /// Always `false` — this envelope is only emitted on the error path.
    pub success: bool,
    /// Machine-readable error discriminator (e.g. `"NO_BROWSER_CONNECTED"`).
    pub code: String,
    /// Human-readable explanation (e.g. `"<path> requires a browser SDK client"`).
    pub message: String,
}

/// Inner `data` payload of the UI-Bridge SDK HTTP health envelope.
///
/// Emitted by the SDK's relay-transport handler for `GET /health`
/// (`@qontinui/ui-bridge/server` `handleRelayRoute`, `nextjs.ts`): the
/// `responsive` + `lastHeartbeat` fields plus a spread of the relay's
/// `TransportDiagnostics`. Only `responsive` and `lastHeartbeat` are modeled
/// explicitly here — the diagnostics spread (pendingCommandCount, connectedTabs,
/// buildId, …) is intentionally left open (no `additionalProperties: false`,
/// which schemars omits by default) so the diagnostics surface can evolve
/// without a schemas release while the two stable health signals stay typed.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct UiBridgeHttpHealthData {
    /// True when at least one connected tab has a fresh heartbeat.
    pub responsive: bool,
    /// Max heartbeat timestamp (ms since epoch) across all tabs; `0` when none.
    pub last_heartbeat: i64,
}

/// UI-Bridge metadata block included in the health envelope for the app
/// discovery scanner. Spread of `config.appInfo` plus a fixed `capabilities`
/// list. Present only when the server is configured with `appInfo` (the
/// qontinui-web proxy always is — `route.ts` sets `appId: "qontinui-web"`).
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct UiBridgeHttpHealthAppInfo {
    /// Stable app identifier (e.g. `"qontinui-web"`).
    pub app_id: String,
    /// Human-readable app name.
    pub app_name: String,
    /// App surface type (e.g. `"web"`).
    pub app_type: String,
    /// UI framework (e.g. `"nextjs"`).
    pub framework: String,
    /// Advertised UI-Bridge capabilities (e.g. `["control","renderLog","debug"]`).
    pub capabilities: Vec<String>,
}

/// Canonical success envelope for `GET /api/ui-bridge/health`.
///
/// Distinct from [`UiBridgeResponseEnvelope`]: the HTTP health envelope has
/// no `requestId` / `type`, and carries an app-discovery `uiBridge` block.
/// Wire shape (`nextjs.ts::handleRelayRoute`):
///
/// ```json
/// { "success": true,
///   "data": { "responsive": false, "lastHeartbeat": 0, ...diagnostics },
///   "timestamp": 1713200000000,
///   "uiBridge": { "appId": "qontinui-web", ..., "capabilities": [...] } }
/// ```
///
/// `uiBridge` is optional (omitted when the server has no `appInfo`); the
/// other three fields are always present.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct UiBridgeHttpHealthEnvelope {
    /// Always `true` on the health path.
    pub success: bool,
    /// Transport-diagnostics payload (typed signals + open diagnostics spread).
    pub data: UiBridgeHttpHealthData,
    /// Server-side timestamp (ms since epoch) of when the response was produced.
    pub timestamp: i64,
    /// App-discovery metadata; present only when the server is configured
    /// with `appInfo`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ui_bridge: Option<UiBridgeHttpHealthAppInfo>,
}
