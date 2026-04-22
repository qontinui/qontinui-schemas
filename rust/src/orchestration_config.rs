//! Orchestration-loop DTO types.
//!
//! Wire-format types for the runner's orchestration loop — the engine that
//! iteratively runs a workflow, optionally reflects/fixes/rebuilds between
//! iterations, and reports per-iteration results. Covers both:
//!
//! - **Saved-config CRUD** shapes (`OlConfig`, `CreateOlConfigRequest`,
//!   `UpdateOlConfigRequest`) — what the frontend sends/receives when managing
//!   stored orchestration presets. These are sourced from
//!   `qontinui-runner/src-tauri/src/orchestration_loop_configs/types.rs`.
//! - **Loop-engine configuration + runtime status** (`OrchestrationLoopConfig`,
//!   `OrchestrationLoopStatus`, sub-configs, enums, iteration/multi-loop
//!   shapes). These are sourced from
//!   `qontinui-runner/src-tauri/src/orchestration_loop/types.rs`.
//!
//! Runtime-only state (e.g. in-memory `LoopMetadata` maps that carry no
//! `Serialize`/`Deserialize`) stays in the runner. Behavior previously provided
//! by inherent `impl` blocks on `OrchestrationLoopConfig` (`iter_cap`,
//! `iter_cap_display`) is exposed runner-side through an extension trait — see
//! the runner's `orchestration_loop::types` shim for the pattern, matching
//! `scheduler.rs` / `unified_workflows.rs`.
//!
//! The short name `OlConfig` is preserved (not expanded to
//! `OrchestrationLoopConfig`) because it is a distinct type — it's the
//! persisted *wrapper* around an `OrchestrationLoopConfig` JSON blob, not an
//! alias for it.
//!
//! Wire-format notes:
//! - Dates/times are ISO 8601 strings (see crate-level docs).
//! - `OlConfig.config_json` is deliberately `serde_json::Value`: the runner
//!   persists the full loop config as JSON so older saved presets continue to
//!   deserialize after schema additions.
//! - Enum string values use `snake_case` to match the pre-extraction wire.

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

// ============================================================================
// Saved orchestration-loop configs (CRUD surface)
// ============================================================================

/// A saved orchestration-loop configuration preset.
///
/// Persisted in the runner's `orchestration_loop_configs` PostgreSQL table and
/// surfaced to the frontend through the `ol_list_configs` / `ol_get_config`
/// Tauri commands. The inner `config_json` is an `OrchestrationLoopConfig`
/// serialized as a JSON blob — stored as `Value` here to preserve forward
/// compatibility with older presets if the config schema grows new fields.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
#[schemars(deny_unknown_fields)]
pub struct OlConfig {
    /// UUID v4 identifier.
    #[serde(alias = "id")]
    pub id: String,
    /// Human-readable name (e.g., "Nightly regression sweep").
    #[serde(alias = "name")]
    pub name: String,
    /// Optional free-form description.
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        alias = "description"
    )]
    pub description: Option<String>,
    /// Whether this preset is pinned as a favorite in the UI.
    #[serde(alias = "is_favorite")]
    pub is_favorite: bool,
    /// The full [`OrchestrationLoopConfig`] as a JSON blob.
    #[serde(alias = "config_json")]
    pub config_json: serde_json::Value,
    /// ISO 8601 creation timestamp.
    #[serde(alias = "created_at")]
    pub created_at: String,
    /// ISO 8601 last-modified timestamp.
    #[serde(alias = "updated_at")]
    pub updated_at: String,
}

/// Request payload for creating a new saved orchestration-loop config.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
#[schemars(deny_unknown_fields)]
pub struct CreateOlConfigRequest {
    /// Human-readable name for the new preset.
    #[serde(alias = "name")]
    pub name: String,
    /// Optional description.
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        alias = "description"
    )]
    pub description: Option<String>,
    /// The full loop config as a JSON blob (should match the
    /// [`OrchestrationLoopConfig`] shape).
    #[serde(alias = "config_json")]
    pub config_json: serde_json::Value,
}

/// Request payload for partially updating an existing saved preset.
///
/// All fields are optional — only those set are applied.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
#[schemars(deny_unknown_fields)]
pub struct UpdateOlConfigRequest {
    /// Rename the preset.
    #[serde(default, skip_serializing_if = "Option::is_none", alias = "name")]
    pub name: Option<String>,
    /// Replace the description.
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        alias = "description"
    )]
    pub description: Option<String>,
    /// Toggle favorite status.
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        alias = "is_favorite"
    )]
    pub is_favorite: Option<bool>,
    /// Replace the stored config JSON blob.
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        alias = "config_json"
    )]
    pub config_json: Option<serde_json::Value>,
}

// ============================================================================
// Loop identifiers
// ============================================================================

/// Unique identifier for a loop instance within the multi-loop manager.
pub type LoopId = String;

// ============================================================================
// Loop-engine sub-configs
// ============================================================================

/// Configuration for the stall-detection subsystem.
///
/// Controls how the loop engine decides a run is stuck in a repeated-action /
/// oscillation / runaway-step pattern and forces an exit.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
#[schemars(deny_unknown_fields)]
pub struct StallDetectorConfig {
    /// Maximum times the same action may repeat before stall is declared.
    #[serde(alias = "max_repeated_actions")]
    pub max_repeated_actions: u32,
    /// Absolute ceiling on total steps across all iterations.
    #[serde(alias = "max_total_steps")]
    pub max_total_steps: u32,
    /// Wall-clock seconds without progress before stall is declared.
    #[serde(alias = "stall_timeout_secs")]
    pub stall_timeout_secs: u64,
    /// Window (in actions) used to detect oscillation between two states.
    #[serde(alias = "oscillation_window")]
    pub oscillation_window: u32,
}

impl Default for StallDetectorConfig {
    fn default() -> Self {
        Self {
            max_repeated_actions: 5,
            max_total_steps: 100,
            stall_timeout_secs: 300,
            oscillation_window: 10,
        }
    }
}

/// Configuration for the context-summarization subsystem.
///
/// When the loop's rolling context approaches the token budget, older
/// iterations are compressed into a summary to keep the prompt small.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
#[schemars(deny_unknown_fields)]
pub struct SummarizationConfig {
    /// Whether summarization is active.
    #[serde(alias = "enabled")]
    pub enabled: bool,
    /// Fraction of the token budget (0.0–1.0) at which summarization triggers.
    #[serde(alias = "token_threshold_pct")]
    pub token_threshold_pct: f32,
    /// Target maximum tokens for the full loop context.
    #[serde(alias = "max_tokens_budget")]
    pub max_tokens_budget: usize,
    /// Number of most-recent iterations to keep verbatim (never summarized).
    #[serde(alias = "preserve_last_n_iterations")]
    pub preserve_last_n_iterations: u32,
    /// Cap on tokens emitted by a single summarization pass.
    #[serde(alias = "summary_max_tokens")]
    pub summary_max_tokens: usize,
}

impl Default for SummarizationConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            token_threshold_pct: 0.75,
            max_tokens_budget: 80000,
            preserve_last_n_iterations: 2,
            summary_max_tokens: 2000,
        }
    }
}

/// Configuration for the task-decomposition subsystem.
///
/// When enabled, a single high-level task may be split into a plan of
/// sub-tasks executed in sequence within the loop.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
#[schemars(deny_unknown_fields)]
pub struct DecomposerConfig {
    /// Whether decomposition is active.
    #[serde(alias = "enabled")]
    pub enabled: bool,
    /// Lower bound on plan length.
    #[serde(alias = "min_subtasks")]
    pub min_subtasks: u32,
    /// Upper bound on plan length.
    #[serde(alias = "max_subtasks")]
    pub max_subtasks: u32,
    /// Override the default AI model used for planning.
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        alias = "model_override"
    )]
    pub model_override: Option<String>,
}

impl Default for DecomposerConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            min_subtasks: 3,
            max_subtasks: 7,
            model_override: None,
        }
    }
}

// ============================================================================
// Serde default helpers (preserved from the runner so wire defaults match)
// ============================================================================

fn default_supervisor_port() -> u16 {
    9875
}

fn default_wait_for_fixer() -> bool {
    true
}

fn default_capture_snapshot() -> bool {
    true
}

fn default_snapshot_max_chars() -> usize {
    8000
}

// ============================================================================
// OrchestrationLoopConfig
// ============================================================================

/// Full configuration for an orchestration loop run.
///
/// Covers the simple mode (fixed `workflow_id`, `max_iterations`, exit
/// strategy) as well as pipeline mode (build / diagnose / reflect / fix
/// phases) when `pipeline` is populated.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
#[schemars(deny_unknown_fields)]
pub struct OrchestrationLoopConfig {
    /// Target runner port to execute workflows on.
    /// If `None`, targets self (this runner's own port).
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        alias = "target_runner_port"
    )]
    pub target_runner_port: Option<u16>,

    /// Target runner ID (for supervisor restart calls). If `None`, uses
    /// `"primary"`.
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        alias = "target_runner_id"
    )]
    pub target_runner_id: Option<String>,

    /// Supervisor port for restart/build operations.
    #[serde(default = "default_supervisor_port", alias = "supervisor_port")]
    pub supervisor_port: u16,

    /// Workflow ID to execute each iteration.
    /// Required for simple mode; optional in pipeline mode if the build phase
    /// is configured.
    #[serde(default, alias = "workflow_id")]
    pub workflow_id: String,

    /// Maximum number of iterations.
    /// `None` (omitted) means no cap — loop until success or explicit stop.
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        alias = "max_iterations"
    )]
    pub max_iterations: Option<u32>,

    /// How to decide when to stop.
    #[serde(default, alias = "exit_strategy")]
    pub exit_strategy: ExitStrategy,

    /// What to do between iterations.
    #[serde(default, alias = "between_iterations")]
    pub between_iterations: BetweenIterations,

    /// When `true`, a failed workflow iteration doesn't count as a terminal
    /// failure. Instead, the loop waits for the fixer workflow to complete,
    /// then re-runs. The loop only exits on success or `max_iterations`.
    #[serde(default, alias = "retry_on_failure")]
    pub retry_on_failure: bool,

    /// Whether to wait for the fixer workflow before starting the next
    /// iteration. Only applies when `retry_on_failure` is `true`.
    #[serde(default = "default_wait_for_fixer", alias = "wait_for_fixer")]
    pub wait_for_fixer: bool,

    /// Pipeline-mode configuration. When present, enables build / reflect /
    /// fix phases.
    #[serde(default, skip_serializing_if = "Option::is_none", alias = "pipeline")]
    pub pipeline: Option<PipelineConfig>,

    /// Stall-detection sub-config (omit to disable).
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        alias = "stall_detection"
    )]
    pub stall_detection: Option<StallDetectorConfig>,

    /// Context-summarization sub-config (omit to disable).
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        alias = "summarization"
    )]
    pub summarization: Option<SummarizationConfig>,

    /// Task-decomposition sub-config (omit to disable).
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        alias = "decomposition"
    )]
    pub decomposition: Option<DecomposerConfig>,
}

// ============================================================================
// Pipeline mode
// ============================================================================

/// Pipeline-mode configuration for the build → execute → diagnose → reflect →
/// fix cycle.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
#[schemars(deny_unknown_fields)]
pub struct PipelineConfig {
    /// Generate the workflow from a description (optional — if absent, the
    /// top-level `workflow_id` is used).
    #[serde(default, skip_serializing_if = "Option::is_none", alias = "build")]
    pub build: Option<BuildPhaseConfig>,
    /// Implement fixes via Claude CLI after reflection finds issues.
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        alias = "implement_fixes"
    )]
    pub implement_fixes: Option<ImplementFixesConfig>,
    /// Diagnostic-evaluation phase — runs after Execute, before Reflect.
    /// Captures UI state via UI Bridge and classifies failure root causes.
    #[serde(default, skip_serializing_if = "Option::is_none", alias = "diagnose")]
    pub diagnose: Option<DiagnosePhaseConfig>,
}

/// Configuration for the build (workflow-generation) phase.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
#[schemars(deny_unknown_fields)]
pub struct BuildPhaseConfig {
    /// Human description of the desired workflow.
    #[serde(alias = "description")]
    pub description: String,
    /// Additional free-form context to pass to the generator.
    #[serde(default, skip_serializing_if = "Option::is_none", alias = "context")]
    pub context: Option<String>,
    /// IDs of stored `Context` records to include.
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        alias = "context_ids"
    )]
    pub context_ids: Option<Vec<String>>,
}

/// Configuration for the implement-fixes phase (Claude CLI).
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
#[schemars(deny_unknown_fields)]
pub struct ImplementFixesConfig {
    /// Model to use (e.g., `"claude-opus-4-6"`). Defaults to `claude-opus-4-6`
    /// when unset.
    #[serde(default, skip_serializing_if = "Option::is_none", alias = "model")]
    pub model: Option<String>,
    /// Timeout in seconds for the fix agent. Defaults to 600 when unset.
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        alias = "timeout_secs"
    )]
    pub timeout_secs: Option<u64>,
    /// Additional context to include in the fix prompt.
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        alias = "additional_context"
    )]
    pub additional_context: Option<String>,
}

/// Configuration for the diagnostic-evaluation phase.
///
/// Captures UI state after workflow execution and classifies failure root
/// causes.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
#[schemars(deny_unknown_fields)]
pub struct DiagnosePhaseConfig {
    /// Assertions to run against the UI after workflow execution.
    /// Each assertion is a JSON object passed to
    /// `POST /ui-bridge/control/assert`.
    #[serde(default, skip_serializing_if = "Vec::is_empty", alias = "assertions")]
    pub assertions: Vec<serde_json::Value>,
    /// Whether to capture a full DOM snapshot for AI triage context.
    #[serde(default = "default_capture_snapshot", alias = "capture_snapshot")]
    pub capture_snapshot: bool,
    /// Maximum characters to include from the snapshot in the AI triage
    /// prompt.
    #[serde(default = "default_snapshot_max_chars", alias = "snapshot_max_chars")]
    pub snapshot_max_chars: usize,
    /// Model override for the triage AI call. If `None`, uses default routing.
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        alias = "model_override"
    )]
    pub model_override: Option<String>,
}

// ============================================================================
// Diagnostic results
// ============================================================================

/// Root-cause classification for a diagnostic failure.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum RootCauseCategory {
    /// The UI itself rendered incorrectly (wrong components, broken layout).
    BadUiRendering,
    /// UI Bridge misread the page (element not found, wrong selector).
    BadUiBridgeEvaluation,
    /// The assertions/verification are wrong (testing the wrong thing).
    BadVerificationSteps,
    /// The workflow-generation prompt was unclear or incomplete.
    BadGenerationPrompt,
    /// The generated state machine has logical errors (wrong transitions,
    /// missing states).
    BadStateMachineLogic,
    /// Network error, timeout, crashed app, or other infrastructure failure.
    InfrastructureIssue,
    /// Cannot determine.
    Unknown,
}

/// Result of a single diagnostic evaluation.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
#[schemars(deny_unknown_fields)]
pub struct DiagnosticResult {
    /// Whether all assertions passed and the page is healthy.
    #[serde(alias = "passed")]
    pub passed: bool,
    /// Page-health status blob from UI Bridge.
    #[serde(alias = "page_health")]
    pub page_health: serde_json::Value,
    /// Assertion results (each with pass/fail and details).
    #[serde(alias = "assertion_results")]
    pub assertion_results: Vec<serde_json::Value>,
    /// Classified root cause (only meaningful when `passed == false`).
    #[serde(default, skip_serializing_if = "Option::is_none", alias = "root_cause")]
    pub root_cause: Option<RootCauseCategory>,
    /// AI-generated explanation of the failure.
    #[serde(default, skip_serializing_if = "Option::is_none", alias = "diagnosis")]
    pub diagnosis: Option<String>,
    /// AI-generated recommendation for the next iteration's prompt.
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        alias = "prompt_rewrite_suggestion"
    )]
    pub prompt_rewrite_suggestion: Option<String>,
}

// ============================================================================
// Exit strategy + between-iterations policy
// ============================================================================

/// How to evaluate whether the loop should exit.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, Default, PartialEq, Eq)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ExitStrategy {
    /// Exit when reflection finds 0 new fixes.
    #[default]
    Reflection,
    /// Exit when workflow verification passes on the first iteration.
    WorkflowVerification,
    /// Always run `max_iterations` times.
    FixedIterations,
    /// Exit when diagnostic evaluation passes (all assertions pass + healthy
    /// page).
    DiagnosticEvaluation,
}

/// Action to take between iterations.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, Default, PartialEq, Eq)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum BetweenIterations {
    /// Restart the target runner between iterations.
    RestartRunner {
        /// Whether to also rebuild the target before restarting.
        #[serde(default)]
        rebuild: bool,
    },
    /// Only restart if the workflow signaled a restart is needed.
    RestartOnSignal {
        /// Whether to also rebuild when a signal triggers the restart.
        #[serde(default)]
        rebuild: bool,
    },
    /// Wait for the target runner to be healthy (no restart).
    WaitHealthy,
    /// No action between iterations.
    #[default]
    None,
}

// ============================================================================
// Loop runtime state
// ============================================================================

/// Current phase of the orchestration loop.
#[derive(Debug, Clone, Default, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum LoopPhase {
    /// No loop is running.
    #[default]
    Idle,
    /// Pipeline mode: generating the workflow.
    BuildingWorkflow,
    /// Executing the workflow on the target runner.
    RunningWorkflow,
    /// Diagnostic-evaluation phase in progress.
    Diagnosing,
    /// Reflection phase in progress.
    Reflecting,
    /// Implementing AI-suggested fixes.
    ImplementingFixes,
    /// Checking whether the exit condition is met.
    EvaluatingExit,
    /// Waiting for the fixer workflow to complete.
    WaitingForFixer,
    /// Between-iterations action is running (e.g., restart).
    BetweenIterations,
    /// Waiting for the target runner to become healthy.
    WaitingForRunner,
    /// Stall-detection check in progress.
    StallDetecting,
    /// Planning / decomposition in progress.
    Planning,
    /// Loop finished successfully.
    Complete,
    /// Loop stopped by the user.
    Stopped,
    /// Loop terminated with an error.
    Error,
}

/// Runtime state of an orchestration loop.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
#[schemars(deny_unknown_fields)]
pub struct OrchestrationLoopStatus {
    /// Whether the loop is currently running.
    #[serde(alias = "running")]
    pub running: bool,
    /// Current phase.
    #[serde(alias = "phase")]
    pub phase: LoopPhase,
    /// Iteration index (1-based) currently executing or just completed.
    #[serde(alias = "current_iteration")]
    pub current_iteration: u32,
    /// Iteration cap for this run. `None` renders as `"∞"`/unlimited in the UI.
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        alias = "max_iterations"
    )]
    pub max_iterations: Option<u32>,
    /// The workflow ID being executed.
    #[serde(alias = "workflow_id")]
    pub workflow_id: String,
    /// Target runner port.
    #[serde(alias = "target_runner_port")]
    pub target_runner_port: u16,
    /// Target runner ID.
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        alias = "target_runner_id"
    )]
    pub target_runner_id: Option<String>,
    /// Whether this loop is in pipeline mode.
    #[serde(alias = "is_pipeline")]
    pub is_pipeline: bool,
    /// ISO 8601 start timestamp.
    #[serde(default, skip_serializing_if = "Option::is_none", alias = "started_at")]
    pub started_at: Option<String>,
    /// Terminal error message (only set in the `Error` phase).
    #[serde(default, skip_serializing_if = "Option::is_none", alias = "error")]
    pub error: Option<String>,
    /// Per-iteration results accumulated so far.
    #[serde(
        default,
        skip_serializing_if = "Vec::is_empty",
        alias = "iteration_results"
    )]
    pub iteration_results: Vec<IterationResult>,
}

/// Result of a single iteration.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
#[schemars(deny_unknown_fields)]
pub struct IterationResult {
    /// 1-based iteration index.
    #[serde(alias = "iteration")]
    pub iteration: u32,
    /// ISO 8601 start timestamp.
    #[serde(alias = "started_at")]
    pub started_at: String,
    /// ISO 8601 completion timestamp.
    #[serde(alias = "completed_at")]
    pub completed_at: String,
    /// Task-run ID produced by the workflow execution.
    #[serde(alias = "task_run_id")]
    pub task_run_id: String,
    /// Task-run ID produced by the reflection step (if any).
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        alias = "reflection_task_run_id"
    )]
    pub reflection_task_run_id: Option<String>,
    /// Number of fixes proposed during reflection.
    #[serde(default, skip_serializing_if = "Option::is_none", alias = "fix_count")]
    pub fix_count: Option<u32>,
    /// Exit-check outcome for this iteration.
    #[serde(alias = "exit_check")]
    pub exit_check: ExitCheckResult,
    /// Pipeline mode: ID of the workflow generated during the build phase.
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        alias = "generated_workflow_id"
    )]
    pub generated_workflow_id: Option<String>,
    /// Pipeline mode: whether fixes were implemented during this iteration.
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        alias = "fixes_implemented"
    )]
    pub fixes_implemented: Option<bool>,
    /// Pipeline mode: whether a rebuild was triggered for the next iteration.
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        alias = "rebuild_triggered"
    )]
    pub rebuild_triggered: Option<bool>,
    /// Stall-detection reason, if a stall was detected.
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        alias = "stall_detected"
    )]
    pub stall_detected: Option<String>,
    /// Whether the loop context was summarized during this iteration.
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        alias = "context_summarized"
    )]
    pub context_summarized: Option<bool>,
    /// Diagnostic-phase result (if the diagnose phase is configured).
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        alias = "diagnostic_result"
    )]
    pub diagnostic_result: Option<DiagnosticResult>,
}

/// Result of evaluating whether the loop should exit.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
#[schemars(deny_unknown_fields)]
pub struct ExitCheckResult {
    /// Whether the loop should stop after this iteration.
    #[serde(alias = "should_exit")]
    pub should_exit: bool,
    /// Human-readable reason for the decision.
    #[serde(alias = "reason")]
    pub reason: String,
}

// ============================================================================
// Multi-loop manager
// ============================================================================

/// Configuration for launching multiple loops at once.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
#[schemars(deny_unknown_fields)]
pub struct MultiLoopConfig {
    /// Individual loop configurations, each targeting a different runner.
    #[serde(alias = "loops")]
    pub loops: Vec<MultiLoopEntry>,
    /// Stop all loops if any single loop errors out.
    #[serde(default, alias = "stop_all_on_error")]
    pub stop_all_on_error: bool,
}

/// A single entry in a multi-loop configuration.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
#[schemars(deny_unknown_fields)]
pub struct MultiLoopEntry {
    /// Unique identifier for this loop instance.
    #[serde(alias = "loop_id")]
    pub loop_id: LoopId,
    /// Human label (e.g., `"Pages 1–10"` or an app section name).
    #[serde(default, skip_serializing_if = "Option::is_none", alias = "label")]
    pub label: Option<String>,
    /// The loop configuration.
    #[serde(alias = "config")]
    pub config: OrchestrationLoopConfig,
}

/// Aggregated status across all active loops.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
#[schemars(deny_unknown_fields)]
pub struct MultiLoopStatus {
    /// Per-loop status snapshots.
    #[serde(alias = "loops")]
    pub loops: Vec<LoopInstanceStatus>,
    /// Whether every loop has reached a terminal phase.
    #[serde(alias = "all_complete")]
    pub all_complete: bool,
    /// Whether any loop is in the `Error` phase.
    #[serde(alias = "any_error")]
    pub any_error: bool,
    /// Whether the multi-loop manager is configured to abort all loops on the
    /// first error.
    #[serde(alias = "stop_all_on_error")]
    pub stop_all_on_error: bool,
}

/// Status of a single loop instance within a multi-loop.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
#[schemars(deny_unknown_fields)]
pub struct LoopInstanceStatus {
    /// Unique identifier for this loop.
    #[serde(alias = "loop_id")]
    pub loop_id: LoopId,
    /// Human label.
    #[serde(default, skip_serializing_if = "Option::is_none", alias = "label")]
    pub label: Option<String>,
    /// The per-loop status snapshot.
    #[serde(alias = "status")]
    pub status: OrchestrationLoopStatus,
}

// ============================================================================
// Notes on types intentionally left in the runner
// ============================================================================
//
// `LoopMetadata` and `LoopMetadataMap` are deliberately **not** part of this
// module. They are purely in-memory runner bookkeeping (no `Serialize`/
// `Deserialize`) used by the multi-loop manager to attach per-loop labels and
// abort policy to the running state, and they never cross the wire. They stay
// in `qontinui-runner/src-tauri/src/orchestration_loop/types.rs`.
