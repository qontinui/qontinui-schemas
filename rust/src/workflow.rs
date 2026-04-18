//! Workflow frame DTO types.
//!
//! These types mirror the workflow frame defined in
//! `qontinui-runner/src-tauri/src/unified_workflows.rs` and the TypeScript
//! counterpart in `qontinui-schemas/ts/src/workflow/index.ts`. Rust is the
//! source of truth; JSON Schema emitted from these types drives the TS and
//! Python bindings.
//!
//! This module is wire-format only: no business logic, no tests, and no
//! `impl` blocks beyond `Default` and the `deserialize_null_as_empty_vec`
//! helper that preserves Python-null tolerance on the wire.
//!
//! ## Step arrays are `Vec<serde_json::Value>`
//!
//! `UnifiedWorkflow` and `WorkflowStage` carry polymorphic step arrays
//! (`setup_steps`, `verification_steps`, `agentic_steps`, `completion_steps`).
//! Typed step discriminated unions are a future migration (Wave 4); until
//! then these fields pass through as opaque `serde_json::Value` payloads.
//!
//! Dates, times, and UUIDs are `String`s for wire-format stability — see
//! the crate-level docs.

use schemars::JsonSchema;
use serde::{Deserialize, Deserializer, Serialize};
use serde_json::Value;
use std::collections::HashMap;

use crate::workflow_step::UnifiedStep;

// ============================================================================
// Helpers
// ============================================================================

/// Deserialize a `Vec` field that might be `null` in JSON (e.g., from Python's
/// `None`). Returns an empty `Vec` for null, or the actual `Vec` for a valid
/// array.
///
/// Ported verbatim from the runner; keeping it here matters for round-trips
/// of Python-generated workflow payloads where list fields are often
/// serialized as `null` instead of `[]`.
fn deserialize_null_as_empty_vec<'de, D, T>(deserializer: D) -> Result<Vec<T>, D::Error>
where
    D: Deserializer<'de>,
    T: Deserialize<'de>,
{
    let opt: Option<Vec<T>> = Option::deserialize(deserializer)?;
    Ok(opt.unwrap_or_default())
}

// ============================================================================
// Default-value helpers
// ============================================================================

/// Default expected HTTP status code for a health check (`200`).
fn default_expected_status() -> u16 {
    200
}

/// Default timeout for a health check in seconds (`30`). Health checks need a
/// reasonable timeout to avoid hanging on unresponsive services.
fn default_health_timeout() -> u64 {
    30
}

/// Default "is critical" flag for a health check (`true`). Critical failures
/// stop the workflow.
fn default_is_critical() -> bool {
    true
}

/// Default retry delay in milliseconds (`2000`).
fn default_retry_delay_ms() -> u64 {
    2000
}

/// Default `true` for boolean fields that default to on.
fn default_true() -> bool {
    true
}

/// Default workflow category (`"general"`).
fn default_category() -> String {
    "general".to_string()
}

/// Default auto-include contexts flag (`true`).
fn default_auto_include_contexts() -> bool {
    true
}

/// Default log watch enabled flag (`true`).
fn default_log_watch_enabled() -> bool {
    true
}

/// Default health check enabled flag (`true`).
fn default_health_check_enabled() -> bool {
    true
}

/// Default pre-flight environment check enabled flag (`true`).
fn default_preflight_check_enabled() -> bool {
    true
}

/// Default maximum sweep iterations (`5`).
fn default_max_sweep_iterations() -> u32 {
    5
}

/// Default maximum consecutive non-improving fix attempts (`3`).
fn default_max_fix_attempts() -> u32 {
    3
}

/// Default maximum number of CI-triggered auto-resumes (`10`).
fn default_max_ci_auto_resumes() -> u32 {
    10
}

/// Default reflection mode flag (`true`).
fn default_reflection_mode() -> bool {
    true
}

/// Default "AI review ran" flag (`true`).
fn default_ai_reviewed() -> bool {
    true
}

/// Default multi-agent fixer mode flag (`true`).
fn default_multi_agent_mode() -> bool {
    true
}

/// Whether the given log-source selection is the default (`LogSourceMode::Default`).
/// Used by the `skip_serializing_if` hook on `UnifiedWorkflow::log_source_selection`.
fn is_default_log_source(selection: &LogSourceSelection) -> bool {
    matches!(selection, LogSourceSelection::Mode(LogSourceMode::Default))
}

// ============================================================================
// Routing & Model Overrides
// ============================================================================

/// A conditional routing rule that selects model/provider based on runtime
/// context.
///
/// Rules are evaluated in order; the first matching rule wins. Condition
/// syntax: `"<variable> <op> <value>"` where:
/// - Variables: `verification_failures`, `iteration`, `stage_index`
/// - Operators: `>=`, `>`, `<=`, `<`, `==`, `!=`
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
#[schemars(deny_unknown_fields)]
pub struct RoutingRule {
    /// Condition expression, e.g. `"verification_failures >= 2"`.
    #[serde(alias = "condition")]
    pub condition: String,
    /// Model to use when this rule matches.
    #[serde(default, skip_serializing_if = "Option::is_none", alias = "model")]
    pub model: Option<String>,
    /// Provider to use when this rule matches.
    #[serde(default, skip_serializing_if = "Option::is_none", alias = "provider")]
    pub provider: Option<String>,
    /// Temperature override when this rule matches.
    #[serde(default, skip_serializing_if = "Option::is_none", alias = "temperature")]
    pub temperature: Option<f32>,
    /// Max tokens override when this rule matches.
    #[serde(default, skip_serializing_if = "Option::is_none", alias = "max_tokens")]
    pub max_tokens: Option<u32>,
}

/// Per-phase model override configuration.
///
/// Each phase can independently specify a provider and/or model, along with
/// optional temperature, max_tokens, fallback config, and conditional routing
/// rules.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
#[schemars(deny_unknown_fields)]
pub struct ModelOverrideConfig {
    /// Provider override for this phase.
    #[serde(default, skip_serializing_if = "Option::is_none", alias = "provider")]
    pub provider: Option<String>,
    /// Model override for this phase.
    #[serde(default, skip_serializing_if = "Option::is_none", alias = "model")]
    pub model: Option<String>,
    /// Temperature override for this phase (`0.0`–`1.0`).
    #[serde(default, skip_serializing_if = "Option::is_none", alias = "temperature")]
    pub temperature: Option<f32>,
    /// Max output tokens override for this phase.
    #[serde(default, skip_serializing_if = "Option::is_none", alias = "max_tokens")]
    pub max_tokens: Option<u32>,
    /// Fallback provider if the primary fails with a retryable error.
    #[serde(default, skip_serializing_if = "Option::is_none", alias = "fallback_provider")]
    pub fallback_provider: Option<String>,
    /// Fallback model if the primary fails with a retryable error.
    #[serde(default, skip_serializing_if = "Option::is_none", alias = "fallback_model")]
    pub fallback_model: Option<String>,
    /// Conditional routing rules evaluated at runtime. First matching rule
    /// wins; unmatched falls back to this config's static fields.
    #[serde(default, skip_serializing_if = "Option::is_none", alias = "routing_rules")]
    pub routing_rules: Option<Vec<RoutingRule>>,
}

/// Map of phase name → model override config.
///
/// Valid keys: `"setup"`, `"agentic"`, `"completion"`, `"verification"`,
/// `"investigation"`, `"summary"`, `"generation"`.
pub type ModelOverrides = HashMap<String, ModelOverrideConfig>;

// ============================================================================
// Log Source Selection
// ============================================================================

/// Simple log-source mode.
///
/// Serialized as a bare lowercase string: `"default"`, `"ai"`, or `"all"`.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "lowercase")]
pub enum LogSourceMode {
    /// Use the global default profile (from Settings → Log Sources).
    #[default]
    Default,
    /// Let AI automatically select relevant sources based on context.
    Ai,
    /// Use all enabled log sources.
    All,
}

/// Log source selection for a workflow.
///
/// - `"default"` / `"ai"` / `"all"`: Use the corresponding [`LogSourceMode`].
/// - `{ "profile_id": "..." }`: Use a specific profile.
///
/// Serialized as an untagged union: a bare string for the mode variants, or an
/// object with a `profile_id` key for the profile variant.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(untagged)]
pub enum LogSourceSelection {
    /// One of the built-in modes: `default`, `ai`, `all`.
    Mode(LogSourceMode),
    /// Specific profile selection.
    Profile {
        /// ID of the log-source profile to use.
        profile_id: String,
    },
}

impl Default for LogSourceSelection {
    fn default() -> Self {
        LogSourceSelection::Mode(LogSourceMode::Default)
    }
}

// ============================================================================
// Health Check Configuration
// ============================================================================

/// Configuration for a single health check URL.
///
/// A workflow can have zero or more of these; they run before verification to
/// confirm required services are up.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
#[schemars(deny_unknown_fields)]
pub struct HealthCheckUrl {
    /// Display name for the health check (e.g., `"Backend Server"`).
    #[serde(alias = "name")]
    pub name: String,
    /// URL to check (e.g., `"http://localhost:8000/health"`).
    #[serde(alias = "url")]
    pub url: String,
    /// Expected HTTP status code (default: `200`).
    #[serde(default = "default_expected_status", alias = "expected_status")]
    pub expected_status: u16,
    /// Timeout in seconds (default: `30`).
    #[serde(default = "default_health_timeout", alias = "timeout_seconds")]
    pub timeout_seconds: u64,
    /// Whether failure should stop the workflow (default: `true`).
    #[serde(default = "default_is_critical", alias = "is_critical")]
    pub is_critical: bool,
}

// ============================================================================
// Workflow Architecture
// ============================================================================

/// The workflow execution architecture to use.
///
/// This is a first-class workflow architecture option, allowing direct
/// comparison between traditional deterministic verification and agentic
/// verification approaches. Mirrors the runner-side enum in
/// `crate::agentic_verification`.
#[derive(
    Debug, Clone, PartialEq, Eq, Hash, Default, Serialize, Deserialize, JsonSchema,
)]
#[serde(rename_all = "snake_case")]
pub enum WorkflowArchitecture {
    /// Traditional: Setup → \[Deterministic Verification ↔ Agentic Fix\]* →
    /// Completion. Pre-defined verification steps run deterministically; the
    /// agentic phase fixes failures.
    #[default]
    Traditional,
    /// Agentic Verification: \[Verification Agent → Worker Agent\]* loop.
    /// No pre-defined verification steps — a verification agent reasons about
    /// success.
    AgenticVerification,
    /// Multi-Agent Pipeline: specialized agents in a DAG-structured pipeline.
    MultiAgentPipeline,
}

impl std::fmt::Display for WorkflowArchitecture {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Traditional => write!(f, "traditional"),
            Self::AgenticVerification => write!(f, "agentic_verification"),
            Self::MultiAgentPipeline => write!(f, "multi_agent_pipeline"),
        }
    }
}

// ============================================================================
// Stage Support Types
// ============================================================================

/// Condition for conditional stage execution.
///
/// When attached to a [`WorkflowStage`], the stage is skipped if the condition
/// evaluates to "should skip". All condition fields are optional and combine
/// with AND semantics — all specified conditions must be met for the stage to
/// run.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
#[schemars(deny_unknown_fields)]
pub struct StageCondition {
    /// Run this stage only if the previous stage had this outcome.
    ///
    /// - `"passed"`: run only if previous stage verification passed
    /// - `"failed"`: run only if previous stage verification failed
    /// - `"any"`: always run regardless of previous outcome (default behavior)
    #[serde(default, skip_serializing_if = "Option::is_none", alias = "if_previous")]
    pub if_previous: Option<String>,

    /// Run this stage only after this many loop iterations have occurred
    /// (across all stages). Useful for "escalation" stages that only kick in
    /// after initial attempts have failed.
    #[serde(default, skip_serializing_if = "Option::is_none", alias = "min_iteration")]
    pub min_iteration: Option<u32>,

    /// Skip this stage if the total number of failed stages so far is below
    /// this threshold. Useful for "recovery" stages that only run when
    /// multiple prior stages have failed.
    #[serde(default, skip_serializing_if = "Option::is_none", alias = "min_failures")]
    pub min_failures: Option<u32>,
}

/// Retry policy for a step or stage.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
#[schemars(deny_unknown_fields)]
pub struct RetryPolicy {
    /// Number of retry attempts (`0` = no retries).
    #[serde(default, alias = "count")]
    pub count: u32,
    /// Delay between retries in milliseconds.
    #[serde(default = "default_retry_delay_ms", alias = "delay_ms")]
    pub delay_ms: u64,
    /// Whether to use exponential backoff.
    #[serde(default, alias = "backoff")]
    pub backoff: bool,
}

/// An output declared by a stage, available to subsequent stages.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
#[schemars(deny_unknown_fields)]
pub struct StageOutput {
    /// Unique key for this output (e.g. `"api_url"`, `"auth_token"`).
    #[serde(alias = "key")]
    pub key: String,
    /// Human-readable description.
    #[serde(default, alias = "description")]
    pub description: String,
}

/// An input required by a stage, referencing a prior stage's output.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
#[schemars(deny_unknown_fields)]
pub struct StageInput {
    /// The key to bind this input to (matches a [`StageOutput::key`] from a
    /// prior stage).
    #[serde(alias = "key")]
    pub key: String,
    /// Which stage provides this input (stage id). If omitted, searches all
    /// prior stages.
    #[serde(default, skip_serializing_if = "Option::is_none", alias = "from_stage")]
    pub from_stage: Option<String>,
    /// Whether this input is required (default: `true`). Missing required
    /// inputs are Critical findings.
    #[serde(default = "default_true", alias = "required")]
    pub required: bool,
}

/// A workflow stage — a self-contained unit of execution with its own
/// setup / verification / agentic / completion steps and verification-agentic
/// loop.
///
/// Multi-stage workflows execute stages sequentially. Each stage gets its own
/// verification-agentic loop, and later stages see full output from all prior
/// stages. Step arrays are opaque `serde_json::Value` payloads pending the
/// Wave 4 typed-step migration.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
#[schemars(deny_unknown_fields)]
pub struct WorkflowStage {
    /// Unique identifier (UUID v4).
    #[serde(default, alias = "id")]
    pub id: String,
    /// Display name for this stage.
    #[serde(alias = "name")]
    pub name: String,
    /// Description of what this stage does.
    #[serde(default, alias = "description")]
    pub description: String,
    /// Setup phase steps for this stage (polymorphic; see module docs).
    #[serde(default, alias = "setup_steps")]
    #[schemars(with = "Vec<UnifiedStep>")]
    pub setup_steps: Vec<Value>,
    /// Verification phase steps for this stage.
    #[serde(default, alias = "verification_steps")]
    #[schemars(with = "Vec<UnifiedStep>")]
    pub verification_steps: Vec<Value>,
    /// Agentic phase steps for this stage.
    #[serde(default, alias = "agentic_steps")]
    #[schemars(with = "Vec<UnifiedStep>")]
    pub agentic_steps: Vec<Value>,
    /// Completion phase steps for this stage.
    #[serde(default, alias = "completion_steps")]
    #[schemars(with = "Vec<UnifiedStep>")]
    pub completion_steps: Vec<Value>,
    /// Maximum iterations for this stage's verification-agentic loop.
    ///
    /// `None` (omitted in JSON) means no iteration cap — the loop terminates
    /// on success, explicit stop, or fix-attempt exhaustion.
    #[serde(default, skip_serializing_if = "Option::is_none", alias = "max_iterations")]
    pub max_iterations: Option<u32>,
    /// Optional inactivity timeout in seconds for this stage's AI sessions.
    #[serde(default, skip_serializing_if = "Option::is_none", alias = "timeout_seconds")]
    pub timeout_seconds: Option<u64>,
    /// AI provider override for this stage.
    #[serde(default, skip_serializing_if = "Option::is_none", alias = "provider")]
    pub provider: Option<String>,
    /// Model override for this stage.
    #[serde(default, skip_serializing_if = "Option::is_none", alias = "model")]
    pub model: Option<String>,
    /// Per-phase model overrides for this stage.
    #[serde(default, skip_serializing_if = "HashMap::is_empty", alias = "model_overrides")]
    pub model_overrides: ModelOverrides,
    /// Whether to pause for human approval after each agentic phase.
    #[serde(default, alias = "approval_gate")]
    pub approval_gate: bool,
    /// Optional condition for stage execution. When set, the stage is
    /// evaluated against this condition before running. If the condition is
    /// not met, the stage is skipped.
    #[serde(default, skip_serializing_if = "Option::is_none", alias = "condition")]
    pub condition: Option<StageCondition>,
    /// When true, run completion prompt steps BEFORE automation steps.
    /// Default (`false`) runs automation first, then prompts.
    #[serde(default, alias = "completion_prompts_first")]
    pub completion_prompts_first: bool,
    /// Retry policy for this stage (overrides per-step defaults).
    #[serde(default, skip_serializing_if = "Option::is_none", alias = "retry_policy")]
    pub retry_policy: Option<RetryPolicy>,
    /// Declared outputs that this stage produces for downstream stages.
    #[serde(default, skip_serializing_if = "Option::is_none", alias = "outputs")]
    pub outputs: Option<Vec<StageOutput>>,
    /// Inputs required from prior stages.
    #[serde(default, skip_serializing_if = "Option::is_none", alias = "inputs")]
    pub inputs: Option<Vec<StageInput>>,
}

// ============================================================================
// UnifiedWorkflow
// ============================================================================

/// A unified workflow with steps organized by phase.
///
/// The "frame" carries all non-step metadata — iteration caps, provider/model
/// selection, log-source routing, health checks, stage list, generator
/// outputs (dependency graph, cost annotations, quality report, acceptance
/// criteria), and timestamps. Step arrays remain opaque until the Wave 4
/// typed-step migration lands.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
#[schemars(deny_unknown_fields)]
pub struct UnifiedWorkflow {
    /// Unique identifier (UUID v4).
    #[serde(default, alias = "id")]
    pub id: String,
    /// Display name.
    #[serde(alias = "name")]
    pub name: String,
    /// Description of what this workflow does.
    #[serde(default, alias = "description")]
    pub description: String,
    /// Category for organization.
    #[serde(default = "default_category", alias = "category")]
    pub category: String,
    /// Tags for filtering.
    #[serde(default, alias = "tags")]
    pub tags: Vec<String>,

    /// Setup phase steps (polymorphic JSON array; see module docs).
    #[serde(default, alias = "setup_steps")]
    #[schemars(with = "Vec<UnifiedStep>")]
    pub setup_steps: Vec<Value>,
    /// Verification phase steps (polymorphic JSON array).
    #[serde(default, alias = "verification_steps")]
    #[schemars(with = "Vec<UnifiedStep>")]
    pub verification_steps: Vec<Value>,
    /// Agentic phase steps (polymorphic JSON array).
    #[serde(default, alias = "agentic_steps")]
    #[schemars(with = "Vec<UnifiedStep>")]
    pub agentic_steps: Vec<Value>,
    /// Completion phase steps (polymorphic JSON array) — runs once after the
    /// verification loop exits.
    #[serde(default, alias = "completion_steps")]
    #[schemars(with = "Vec<UnifiedStep>")]
    pub completion_steps: Vec<Value>,

    /// Maximum iterations for the agentic phase.
    ///
    /// `None` means no iteration cap — the loop terminates on success,
    /// explicit stop, or fix-attempt exhaustion.
    #[serde(default, skip_serializing_if = "Option::is_none", alias = "max_iterations")]
    pub max_iterations: Option<u32>,

    /// Maximum consecutive non-improving fix attempts before escalating.
    ///
    /// When the verification check count does not improve across this many
    /// iterations, the loop exits with `fix_attempts_exhausted`. `0` =
    /// disabled. Default: `3`.
    #[serde(default = "default_max_fix_attempts", alias = "max_fix_attempts")]
    pub max_fix_attempts: u32,

    /// Maximum number of CI-triggered auto-resumes before requiring human
    /// intervention. Used by the PR watcher integration. `0` = disabled.
    /// Default: `10`.
    #[serde(default = "default_max_ci_auto_resumes", alias = "max_ci_auto_resumes")]
    pub max_ci_auto_resumes: u32,

    /// Optional inactivity timeout in seconds for AI sessions.
    ///
    /// - `None` (default): no timeout, runs until completion or manual stop.
    /// - `Some(N)`: kill AI session after `N` seconds of no output.
    ///
    /// Takes precedence over the global AI settings timeout.
    #[serde(default, skip_serializing_if = "Option::is_none", alias = "timeout_seconds")]
    pub timeout_seconds: Option<u64>,

    /// AI provider override.
    #[serde(default, skip_serializing_if = "Option::is_none", alias = "provider")]
    pub provider: Option<String>,
    /// Model override.
    #[serde(default, skip_serializing_if = "Option::is_none", alias = "model")]
    pub model: Option<String>,

    /// Per-phase model overrides.
    #[serde(default, skip_serializing_if = "HashMap::is_empty", alias = "model_overrides")]
    pub model_overrides: ModelOverrides,

    /// Skip AI summary generation at the end (default: `false`, meaning the
    /// AI summary is generated).
    #[serde(default, alias = "skip_ai_summary")]
    pub skip_ai_summary: bool,

    /// Error IDs targeted by this workflow (for auto-resolution on success).
    ///
    /// When the workflow completes successfully, these errors will be marked
    /// as resolved. Used by error-fix workflows generated from the Error
    /// Monitor.
    #[serde(default, skip_serializing_if = "Vec::is_empty", alias = "targeted_error_ids")]
    pub targeted_error_ids: Vec<i64>,

    /// Log source selection for this workflow.
    ///
    /// - `"default"`: use the global default profile (from Settings → Log Sources)
    /// - `"ai"`: let AI automatically select relevant sources based on context
    /// - `"all"`: use all enabled log sources
    /// - `{ "profile_id": "..." }`: use a specific profile
    #[serde(default, skip_serializing_if = "is_default_log_source", alias = "log_source_selection")]
    pub log_source_selection: LogSourceSelection,

    /// Manually added context IDs.
    #[serde(default, skip_serializing_if = "Vec::is_empty", alias = "context_ids")]
    pub context_ids: Vec<String>,

    /// Disabled context IDs (excluded from auto-include).
    #[serde(default, skip_serializing_if = "Vec::is_empty", alias = "disabled_context_ids")]
    pub disabled_context_ids: Vec<String>,

    /// Whether to auto-include contexts based on task mentions (default:
    /// `true`).
    #[serde(default = "default_auto_include_contexts", alias = "auto_include_contexts")]
    pub auto_include_contexts: bool,

    /// Custom developer prompt template for this workflow.
    ///
    /// When set, this template is used instead of the global default when
    /// running the workflow. Supports variables: `{{SESSION_ID}}`,
    /// `{{ITERATION}}`, `{{MAX_ITERATIONS}}`, `{{GOAL}}`,
    /// `{{EXECUTION_STEPS}}`, `{{WORKSPACE_ESCAPED}}`.
    #[serde(default, skip_serializing_if = "Option::is_none", alias = "prompt_template")]
    pub prompt_template: Option<String>,

    /// Whether to automatically include a `log_watch` step before verification.
    ///
    /// When enabled (default), a `log_watch` step is prepended to
    /// verification steps to detect runtime errors in backend/frontend logs.
    #[serde(default = "default_log_watch_enabled", alias = "log_watch_enabled")]
    pub log_watch_enabled: bool,

    /// Whether to automatically include health check steps before
    /// verification.
    ///
    /// When enabled and `health_check_urls` is non-empty, health check steps
    /// are prepended to verification steps to verify configured servers are
    /// running.
    #[serde(default = "default_health_check_enabled", alias = "health_check_enabled")]
    pub health_check_enabled: bool,

    /// URLs to health check before verification (user-configurable).
    ///
    /// Each entry specifies a URL to check, expected status, and timeout.
    /// If empty, no health checks are performed even if `health_check_enabled`
    /// is true.
    #[serde(default, skip_serializing_if = "Vec::is_empty", alias = "health_check_urls")]
    pub health_check_urls: Vec<HealthCheckUrl>,

    /// Whether to automatically include a pre-flight environment check at the
    /// start of setup.
    ///
    /// When enabled (default), a shell command step runs to verify disk
    /// space, Node.js/npm, Python/Poetry, Rust/Cargo, and Git availability.
    /// Uses the global setting from Settings if not explicitly set per
    /// workflow.
    #[serde(default = "default_preflight_check_enabled", alias = "preflight_check_enabled")]
    pub preflight_check_enabled: bool,

    /// Whether to run a completion sweep after verification passes.
    ///
    /// The sweep reviews all completed work for gaps before proceeding to
    /// completion.
    #[serde(default, alias = "enable_sweep")]
    pub enable_sweep: bool,

    /// Maximum number of sweep iterations (default: `5`).
    #[serde(default = "default_max_sweep_iterations", alias = "max_sweep_iterations")]
    pub max_sweep_iterations: u32,

    /// Task run ID that generated this workflow (for meta-workflow tracking).
    #[serde(default, skip_serializing_if = "Option::is_none", alias = "generated_by_task_run_id")]
    pub generated_by_task_run_id: Option<String>,

    /// Optional stages for multi-stage workflows.
    ///
    /// When non-empty, the workflow executes stages sequentially instead of
    /// using top-level steps. Each stage has its own
    /// setup / verification / agentic / completion steps and loop.
    #[serde(
        default,
        skip_serializing_if = "Vec::is_empty",
        deserialize_with = "deserialize_null_as_empty_vec",
        alias = "stages"
    )]
    pub stages: Vec<WorkflowStage>,

    /// Whether to stop execution if a stage fails verification.
    ///
    /// Default: `false` (autonomous mode — continue to the next stage even
    /// if the previous failed).
    #[serde(default, alias = "stop_on_failure")]
    pub stop_on_failure: bool,

    /// Per-constraint overrides: map of `constraint_id` to enabled (`true`) /
    /// disabled (`false`).
    ///
    /// Applied to the constraint engine at execution time, after loading
    /// builtins and config.
    #[serde(default, skip_serializing_if = "HashMap::is_empty", alias = "constraint_overrides")]
    pub constraint_overrides: HashMap<String, bool>,

    /// Whether to pause for human approval after each agentic phase.
    #[serde(default, alias = "approval_gate")]
    pub approval_gate: bool,

    /// Whether to enable reflection mode during agentic iterations.
    ///
    /// When `true`, the AI investigates root causes before fixing failures.
    /// Default: `true` for user-created workflows.
    #[serde(default = "default_reflection_mode", alias = "reflection_mode")]
    pub reflection_mode: bool,

    /// When `true`, run completion prompt steps BEFORE automation steps.
    ///
    /// Used by meta-workflows so the AI hardener runs before
    /// `save_workflow_artifact`. Default (`false`) runs automation first,
    /// then prompts.
    #[serde(default, alias = "completion_prompts_first")]
    pub completion_prompts_first: bool,

    /// Whether this workflow is marked as a favorite for quick access.
    #[serde(default, alias = "is_favorite")]
    pub is_favorite: bool,

    /// Dependency graph computed during generation (opaque JSON blob).
    #[serde(default, skip_serializing_if = "Option::is_none", alias = "dependency_graph")]
    pub dependency_graph: Option<Value>,

    /// Cost annotations computed during generation (opaque JSON blob).
    #[serde(default, skip_serializing_if = "Option::is_none", alias = "cost_annotations")]
    pub cost_annotations: Option<Value>,

    /// Acceptance criteria from the specification agent (opaque JSON blob).
    ///
    /// Used by the canvas panel manager to show a live requirements tracker.
    #[serde(default, skip_serializing_if = "Option::is_none", alias = "acceptance_criteria")]
    pub acceptance_criteria: Option<Value>,

    /// Quality report from the revision phase (opaque JSON blob).
    #[serde(default, skip_serializing_if = "Option::is_none", alias = "quality_report")]
    pub quality_report: Option<Value>,

    /// Enable multi-agent fixer mode for the agentic phase.
    ///
    /// When `true`, verification failures are triaged and fixed by
    /// specialized agents (quick-fix for lint/compilation, feature-fix for
    /// missing functionality). Default: `true`.
    #[serde(default = "default_multi_agent_mode", alias = "multi_agent_mode")]
    pub multi_agent_mode: bool,

    /// Policy for automatic git rollback when the workflow fails.
    ///
    /// Values: `"none"` (default), `"last_good"`, `"clean"`.
    #[serde(default, skip_serializing_if = "Option::is_none", alias = "rollback_policy")]
    pub rollback_policy: Option<String>,

    /// When `true`, the pipeline will stop execution if accumulated token
    /// usage exceeds the token budget. Disabled by default — only logs
    /// warnings.
    #[serde(default, alias = "enforce_token_budget")]
    pub enforce_token_budget: bool,

    /// Restrict working directory resolution to the workspace boundary.
    ///
    /// When `true`, steps cannot resolve paths outside the workspace root.
    /// Default: `false` (permissive, current behavior).
    #[serde(default, alias = "strict_cwd")]
    pub strict_cwd: bool,

    /// Tags for per-execution tool whitelisting.
    ///
    /// When non-empty, only skills matching at least one tag are included in
    /// the AI prompt context, reducing prompt bloat.
    #[serde(default, skip_serializing_if = "Vec::is_empty", alias = "tool_tags")]
    pub tool_tags: Vec<String>,

    /// Per-workflow security profile override.
    ///
    /// When set, overrides the default security profile from settings for
    /// this workflow. Values: `"permissive"`, `"standard"`, `"strict"`,
    /// or `"custom"`. If `None`, uses the default from Settings > Security.
    #[serde(default, skip_serializing_if = "Option::is_none", alias = "security_profile")]
    pub security_profile: Option<String>,

    /// Run the workflow in an isolated git worktree.
    ///
    /// When `true`, a new branch and worktree are created before execution.
    /// Changes stay on the worktree branch and can be merged back after
    /// review. Default: `false`.
    #[serde(default, alias = "use_worktree")]
    pub use_worktree: bool,

    /// Workflow execution architecture override.
    ///
    /// When set, forces the workflow to use a specific execution architecture
    /// instead of the default Traditional loop. When `None`, the system
    /// infers the best architecture based on workflow complexity.
    #[serde(default, skip_serializing_if = "Option::is_none", alias = "workflow_architecture")]
    pub workflow_architecture: Option<WorkflowArchitecture>,

    /// Whether the AI semantic review actually ran successfully during
    /// generation.
    ///
    /// When `false`, the workflow passed through the pipeline without AI
    /// verification (e.g., all verification iterations failed at
    /// infrastructure level).
    #[serde(default = "default_ai_reviewed", alias = "ai_reviewed")]
    pub ai_reviewed: bool,

    /// Flow control configuration as a JSON string (e.g., concurrency limits,
    /// queue behavior).
    #[serde(default, skip_serializing_if = "Option::is_none", alias = "flow_control_json")]
    pub flow_control_json: Option<String>,

    /// Per-phase timeout configuration as a JSON string.
    #[serde(default, skip_serializing_if = "Option::is_none", alias = "phase_timeouts_json")]
    pub phase_timeouts_json: Option<String>,

    /// Whether HTN (Hierarchical Task Network) planning is enabled for this
    /// workflow.
    ///
    /// When `true`, the loop attempts structured plan-based fixes before
    /// falling back to AI agentic sessions.
    #[serde(default, alias = "htn_enabled")]
    pub htn_enabled: bool,

    /// UI Bridge URL for HTN planning (e.g., `"http://localhost:1420"`).
    ///
    /// When set, the HTN planner connects to UI Bridge for querying element
    /// state. If `None`, HTN runs in plan-only mode without GUI execution.
    #[serde(default, skip_serializing_if = "Option::is_none", alias = "htn_ui_bridge_url")]
    pub htn_ui_bridge_url: Option<String>,

    /// Path to a serialized state machine JSON file for HTN planning.
    ///
    /// When `None` and HTN is enabled, defaults to the bundled
    /// `data/runner_state_machine.json`.
    #[serde(default, skip_serializing_if = "Option::is_none", alias = "htn_state_machine_path")]
    pub htn_state_machine_path: Option<String>,

    /// ISO 8601 timestamp of creation.
    #[serde(default, alias = "created_at")]
    pub created_at: String,
    /// ISO 8601 timestamp of last modification (serialized as `"modified_at"`
    /// to match the frontend).
    #[serde(rename = "modified_at", default, alias = "updated_at")]
    pub updated_at: String,
}
