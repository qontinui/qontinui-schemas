//! Verification-plan DTO types.
//!
//! Wire-format types for the orchestrator's planning → execution → verification
//! pipeline: success criteria, verification plans, worker instances and their
//! signals, findings, iteration-level verification results, criterion overrides,
//! and the task-completion envelope.
//!
//! These types are a port of the shape-bearing portion of
//! `qontinui-runner/src-tauri/src/orchestrator/types.rs`. Behavior
//! (constructors that stamp timestamps, parsers that extract signals from AI
//! output text, override-application logic, feedback-string builders) stays in
//! the runner — this crate is data-only. The runner exposes behavior via
//! extension traits in `orchestrator::types`.
//!
//! Wire-format notes:
//! - Dates/times are ISO 8601 `String`s (see crate-level docs).
//! - UUIDs are `String`s (wire-format), not `uuid::Uuid`.
//! - `SuccessCriterion.criterion_type` serializes as `"type"` on the wire (the
//!   field is renamed via `#[serde(rename = "type")]`).
//! - `WorkerCoordinationMessage` uses the `{ "type": ..., "data": ... }`
//!   externally-tagged envelope.
//! - `WorkerSignal` uses `{ "signal": ..., "data": ... }` as its discriminator
//!   pair — `signal` rather than `type` to avoid collision with other tagged
//!   unions and to match the pre-extraction wire.
//! - `TaskCompletionResult` uses `{ "status": ..., ... }` with the variant
//!   fields inlined (internally tagged).

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

// ============================================================================
// Serde default helpers
// ============================================================================

fn default_true() -> bool {
    true
}

fn default_worker_count() -> u32 {
    1
}

fn default_version() -> u32 {
    1
}

// ============================================================================
// Criterion & verification plan
// ============================================================================

/// A success criterion that must be met for task completion.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
#[schemars(deny_unknown_fields)]
pub struct SuccessCriterion {
    /// Unique identifier for this criterion.
    #[serde(alias = "id")]
    pub id: String,

    /// Human-readable description.
    #[serde(alias = "description")]
    pub description: String,

    /// Type of verification required. Serialized as `"type"` on the wire.
    #[serde(rename = "type", alias = "criterion_type")]
    pub criterion_type: CriterionType,

    /// For deterministic criteria: the verification method to use.
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        alias = "verification_method"
    )]
    pub verification_method: Option<VerificationMethod>,

    /// Configuration blob for the verification method (command args, log
    /// patterns, Playwright script path, etc.).
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        alias = "verification_config"
    )]
    pub verification_config: Option<serde_json::Value>,

    /// For AI-evaluated criteria: the evaluation prompt.
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        alias = "evaluation_prompt"
    )]
    pub evaluation_prompt: Option<String>,

    /// Whether this criterion must pass for task completion.
    #[serde(default = "default_true", alias = "required")]
    pub required: bool,

    /// Whether failure of this criterion blocks task completion.
    /// - `true` (default): failure blocks completion, worker must fix.
    /// - `false`: failure is informational, doesn't block completion.
    #[serde(default = "default_true", alias = "is_critical")]
    pub is_critical: bool,

    /// Optional weight for partial success scoring.
    #[serde(default, skip_serializing_if = "Option::is_none", alias = "weight")]
    pub weight: Option<f64>,

    /// Domain this criterion belongs to (for multi-worker verification).
    #[serde(default, skip_serializing_if = "Option::is_none", alias = "domain")]
    pub domain: Option<String>,
}

/// Type of verification for a criterion.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum CriterionType {
    /// Can be verified programmatically without AI.
    Deterministic,
    /// Requires AI evaluation (e.g., screenshot review).
    AiEvaluated,
}

/// Methods for deterministic verification.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum VerificationMethod {
    /// Build must succeed.
    BuildSuccess,
    /// Unit tests must pass.
    UnitTest,
    /// Integration tests must pass.
    IntegrationTest,
    /// Playwright script must pass.
    Playwright,
    /// Log pattern must match (or not match).
    LogPattern,
    /// GUI automation workflow must succeed.
    GuiAutomation,
    /// Type check must pass.
    TypeCheck,
    /// Lint check must pass.
    LintCheck,
    /// Custom command must succeed.
    CustomCommand,
}

/// The verification plan created by the planning agent.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
#[schemars(deny_unknown_fields)]
pub struct VerificationPlan {
    /// Summary of the goal.
    #[serde(alias = "goal_summary")]
    pub goal_summary: String,

    /// All success criteria that must be verified.
    #[serde(alias = "success_criteria")]
    pub success_criteria: Vec<SuccessCriterion>,

    /// Execution steps to run before verification (GUI automation / setup).
    /// Stored as raw JSON values because the step discriminated union spans
    /// several types that are outside this module's scope.
    #[serde(
        default,
        skip_serializing_if = "Vec::is_empty",
        alias = "execution_steps"
    )]
    pub execution_steps: Vec<serde_json::Value>,

    /// Suggested number of worker agents.
    #[serde(default = "default_worker_count", alias = "suggested_worker_count")]
    pub suggested_worker_count: u32,

    /// Domain assignments for multiple workers.
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        alias = "worker_domains"
    )]
    pub worker_domains: Option<Vec<WorkerDomain>>,

    /// Plan version (incremented on replan).
    #[serde(default = "default_version", alias = "version")]
    pub version: u32,
}

/// Domain assignment for a worker agent (lightweight — used inside a
/// [`VerificationPlan`]'s `worker_domains` list).
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
#[schemars(deny_unknown_fields)]
pub struct WorkerDomain {
    /// Worker identifier.
    #[serde(alias = "worker_id")]
    pub worker_id: String,

    /// Optional specialization label (e.g., "frontend", "tests").
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        alias = "specialization"
    )]
    pub specialization: Option<String>,

    /// Files / paths this worker owns (glob patterns).
    #[serde(
        default,
        skip_serializing_if = "Vec::is_empty",
        alias = "file_patterns"
    )]
    pub file_patterns: Vec<String>,

    /// Additional system-prompt text to inject for this worker.
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        alias = "system_prompt_additions"
    )]
    pub system_prompt_additions: Option<String>,
}

// ============================================================================
// Multi-worker support
// ============================================================================

/// Full configuration for a domain that workers can be assigned to.
///
/// Domains represent logical areas of responsibility within a project
/// (e.g., "frontend", "backend", "database", "api"). Workers are assigned to
/// zero or more domains, and criteria can be scoped to a single domain for
/// multi-worker verification.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
#[schemars(deny_unknown_fields)]
pub struct DomainAssignment {
    /// Unique identifier for this domain.
    #[serde(alias = "domain_id")]
    pub domain_id: String,

    /// Human-readable name for the domain.
    #[serde(alias = "name")]
    pub name: String,

    /// Description of what this domain covers.
    #[serde(alias = "description")]
    pub description: String,

    /// File patterns that belong to this domain
    /// (e.g., `"src/frontend/**/*.ts"`).
    #[serde(
        default,
        skip_serializing_if = "Vec::is_empty",
        alias = "file_patterns"
    )]
    pub file_patterns: Vec<String>,

    /// Keywords that help identify this domain.
    #[serde(default, skip_serializing_if = "Vec::is_empty", alias = "keywords")]
    pub keywords: Vec<String>,

    /// Workers currently assigned to this domain.
    #[serde(
        default,
        skip_serializing_if = "Vec::is_empty",
        alias = "assigned_workers"
    )]
    pub assigned_workers: Vec<String>,

    /// Success-criterion IDs that are specific to this domain.
    #[serde(
        default,
        skip_serializing_if = "Vec::is_empty",
        alias = "domain_criteria"
    )]
    pub domain_criteria: Vec<String>,

    /// Additional system-prompt context for workers in this domain.
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        alias = "system_prompt_context"
    )]
    pub system_prompt_context: Option<String>,
}

/// Current state of an individual worker.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum WorkerStatus {
    /// Worker is idle, waiting for assignment.
    Idle,
    /// Worker is actively executing.
    Active,
    /// Worker signaled work complete, awaiting verification.
    AwaitingVerification,
    /// Worker is paused.
    Paused,
    /// Worker has completed its work.
    Completed,
    /// Worker encountered an error.
    Error,
}

/// Instance tracking for an individual worker.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
#[schemars(deny_unknown_fields)]
pub struct WorkerInstance {
    /// Unique identifier for this worker.
    #[serde(alias = "worker_id")]
    pub worker_id: String,

    /// Human-readable name.
    #[serde(alias = "name")]
    pub name: String,

    /// Current status.
    #[serde(alias = "status")]
    pub status: WorkerStatus,

    /// Domain this worker is assigned to (if any).
    #[serde(default, skip_serializing_if = "Option::is_none", alias = "domain")]
    pub domain: Option<String>,

    /// Current iteration for this worker.
    #[serde(alias = "iteration")]
    pub iteration: u32,

    /// Maximum iterations allowed for this worker.
    #[serde(alias = "max_iterations")]
    pub max_iterations: u32,

    /// Last signal received from this worker.
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        alias = "last_signal"
    )]
    pub last_signal: Option<WorkerSignal>,

    /// Findings recorded by this worker.
    #[serde(default, skip_serializing_if = "Vec::is_empty", alias = "findings")]
    pub findings: Vec<Finding>,

    /// Files this worker has touched.
    #[serde(
        default,
        skip_serializing_if = "Vec::is_empty",
        alias = "touched_files"
    )]
    pub touched_files: Vec<String>,

    /// ISO 8601 timestamp when the worker started.
    #[serde(default, skip_serializing_if = "Option::is_none", alias = "started_at")]
    pub started_at: Option<String>,

    /// ISO 8601 timestamp when the worker completed.
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        alias = "completed_at"
    )]
    pub completed_at: Option<String>,

    /// Error message if the worker is in error state.
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        alias = "error_message"
    )]
    pub error_message: Option<String>,
}

/// Coordination message between workers.
///
/// Externally tagged as `{ "type": ..., "data": { ... } }` to match the
/// pre-extraction wire.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "type", content = "data")]
pub enum WorkerCoordinationMessage {
    /// Worker completed work on a set of files.
    #[serde(rename = "files_modified")]
    FilesModified {
        /// ID of the worker that modified the files.
        worker_id: String,
        /// Paths of the modified files.
        files: Vec<String>,
    },

    /// Worker found an issue that may affect other workers.
    #[serde(rename = "shared_finding")]
    SharedFinding {
        /// ID of the worker that made the finding.
        worker_id: String,
        /// The finding payload.
        finding: Finding,
    },

    /// Worker is blocked waiting for another worker.
    #[serde(rename = "blocked")]
    Blocked {
        /// ID of the blocked worker.
        worker_id: String,
        /// ID of the worker being waited on.
        waiting_for: String,
        /// Human-readable reason for the block.
        reason: String,
    },

    /// Worker is ready for verification.
    #[serde(rename = "ready_for_verification")]
    ReadyForVerification {
        /// ID of the worker that is ready.
        worker_id: String,
        /// Optional domain scope for the verification.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        domain: Option<String>,
    },

    /// Coordination signal to synchronize multiple workers.
    #[serde(rename = "sync_point")]
    SyncPoint {
        /// IDs of the workers that should synchronize.
        worker_ids: Vec<String>,
        /// Human-readable reason for the sync point.
        reason: String,
    },
}

/// Result of domain-scoped verification.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
#[schemars(deny_unknown_fields)]
pub struct DomainVerificationResult {
    /// Domain that was verified.
    #[serde(alias = "domain_id")]
    pub domain_id: String,

    /// Workers that contributed to this domain.
    #[serde(alias = "worker_ids")]
    pub worker_ids: Vec<String>,

    /// Verification results for domain-specific criteria.
    #[serde(alias = "results")]
    pub results: Vec<VerificationResult>,

    /// Whether all domain criteria passed.
    #[serde(alias = "all_passed")]
    pub all_passed: bool,

    /// Summary of any failures.
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        alias = "failure_summary"
    )]
    pub failure_summary: Option<String>,
}

// ============================================================================
// Worker signals & findings
// ============================================================================

/// Signal emitted by a worker agent.
///
/// Tagged as `{ "signal": ..., "data": ... }` rather than the more common
/// `type`/`data` pair; this matches the pre-extraction wire.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "signal", content = "data")]
pub enum WorkerSignal {
    /// Worker believes the work is complete, ready for verification.
    #[serde(rename = "work_complete")]
    WorkComplete {
        /// Optional reason provided by the worker.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        reason: Option<String>,
    },

    /// Worker needs the plan to be revised.
    #[serde(rename = "need_replan")]
    NeedReplan {
        /// Reason for the replan request.
        reason: String,
    },

    /// Worker is continuing work (default — typically emitted only when the
    /// caller wants to assert the worker is still alive).
    #[serde(rename = "continue")]
    Continue,

    /// Worker has recorded a finding.
    #[serde(rename = "finding")]
    Finding(Finding),
}

/// A finding recorded by a worker agent.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
#[schemars(deny_unknown_fields)]
pub struct Finding {
    /// Unique identifier.
    #[serde(alias = "id")]
    pub id: String,

    /// Type of finding.
    ///
    /// Valid types: `"bug"`, `"root_cause"`, `"observation"`, `"hypothesis"`,
    /// `"solution"`, `"environment"`. Note: `"environment"` findings (PATH
    /// issues, disk space, tools not installed) require user intervention and
    /// should NOT trigger automatic retries.
    #[serde(alias = "finding_type")]
    pub finding_type: String,

    /// Description of the finding.
    #[serde(alias = "description")]
    pub description: String,

    /// Supporting evidence (file paths, log excerpts, etc.).
    #[serde(default, skip_serializing_if = "Option::is_none", alias = "evidence")]
    pub evidence: Option<String>,

    /// Confidence level.
    #[serde(alias = "confidence")]
    pub confidence: Confidence,

    /// Related file paths.
    #[serde(
        default,
        skip_serializing_if = "Vec::is_empty",
        alias = "related_files"
    )]
    pub related_files: Vec<String>,
}

/// Confidence level for findings and verification results.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "lowercase")]
pub enum Confidence {
    /// High confidence — deterministic evidence or clear observation.
    High,
    /// Medium confidence — reasonable but not certain.
    Medium,
    /// Low confidence — speculative or weakly supported.
    Low,
}

// ============================================================================
// Verification results
// ============================================================================

/// Result of a single verification check.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
#[schemars(deny_unknown_fields)]
pub struct VerificationResult {
    /// The criterion that was checked.
    #[serde(alias = "criterion_id")]
    pub criterion_id: String,

    /// Whether the check passed.
    #[serde(alias = "passed")]
    pub passed: bool,

    /// Type of verification performed.
    #[serde(alias = "criterion_type")]
    pub criterion_type: CriterionType,

    /// Confidence level (for AI verification).
    #[serde(default, skip_serializing_if = "Option::is_none", alias = "confidence")]
    pub confidence: Option<Confidence>,

    /// What was observed.
    #[serde(default, skip_serializing_if = "Vec::is_empty", alias = "observations")]
    pub observations: Vec<String>,

    /// Issues found (if failed).
    #[serde(default, skip_serializing_if = "Vec::is_empty", alias = "issues")]
    pub issues: Vec<String>,

    /// Suggestions for fixing (if failed).
    #[serde(default, skip_serializing_if = "Vec::is_empty", alias = "suggestions")]
    pub suggestions: Vec<String>,

    /// Raw output / details, e.g., captured command output.
    #[serde(default, skip_serializing_if = "Option::is_none", alias = "raw_output")]
    pub raw_output: Option<String>,
}

/// Aggregated verification results for a single iteration.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
#[schemars(deny_unknown_fields)]
pub struct IterationVerificationResults {
    /// Iteration number.
    #[serde(alias = "iteration")]
    pub iteration: u32,

    /// Deterministic verification results.
    #[serde(alias = "deterministic_results")]
    pub deterministic_results: Vec<VerificationResult>,

    /// AI verification results (empty if skipped).
    #[serde(alias = "ai_results")]
    pub ai_results: Vec<VerificationResult>,

    /// Whether all required deterministic checks passed.
    #[serde(alias = "deterministic_passed")]
    pub deterministic_passed: bool,

    /// Whether all required AI checks passed (true if no AI criteria).
    #[serde(alias = "ai_passed")]
    pub ai_passed: bool,

    /// Overall pass/fail.
    #[serde(alias = "all_passed")]
    pub all_passed: bool,

    /// Human-readable summary of failures.
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        alias = "failure_summary"
    )]
    pub failure_summary: Option<String>,

    /// Criterion overrides applied in this iteration.
    #[serde(
        default,
        skip_serializing_if = "Vec::is_empty",
        alias = "applied_overrides"
    )]
    pub applied_overrides: Vec<CriterionOverride>,

    /// Criteria that failed but were accepted due to overrides.
    #[serde(
        default,
        skip_serializing_if = "Vec::is_empty",
        alias = "overridden_criteria"
    )]
    pub overridden_criteria: Vec<String>,
}

/// Context passed to the verification agent (intentionally limited — does not
/// include work history, to avoid biasing AI evaluation).
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
#[schemars(deny_unknown_fields)]
pub struct VerificationAgentContext {
    /// The screenshot to evaluate, base64-encoded.
    #[serde(alias = "screenshot_base64")]
    pub screenshot_base64: String,

    /// The evaluation prompt from the criterion.
    #[serde(alias = "evaluation_prompt")]
    pub evaluation_prompt: String,

    /// Brief goal context (NOT work history).
    #[serde(alias = "goal_context")]
    pub goal_context: String,
}

/// Request to extend iterations after the max has been reached.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
#[schemars(deny_unknown_fields)]
pub struct ExtendIterationsRequest {
    /// Additional iterations to add.
    #[serde(alias = "additional_iterations")]
    pub additional_iterations: u32,

    /// Optional guidance for the worker.
    #[serde(default, skip_serializing_if = "Option::is_none", alias = "guidance")]
    pub guidance: Option<String>,
}

// ============================================================================
// Criterion overrides
// ============================================================================

/// An override for a verification criterion.
///
/// Workers can emit these to indicate that a failing criterion should be
/// accepted as-is with justification, rather than requiring a fix.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
#[schemars(deny_unknown_fields)]
pub struct CriterionOverride {
    /// The criterion ID being overridden.
    #[serde(alias = "criterion_id")]
    pub criterion_id: String,

    /// What specifically is being overridden (e.g., class name, file path).
    #[serde(alias = "item")]
    pub item: String,

    /// Justification for why this override is acceptable.
    #[serde(alias = "justification")]
    pub justification: String,

    /// Iteration when this override was recorded.
    #[serde(alias = "iteration")]
    pub iteration: u32,

    /// Worker ID that provided the override (if multi-worker).
    #[serde(default, skip_serializing_if = "Option::is_none", alias = "worker_id")]
    pub worker_id: Option<String>,

    /// ISO 8601 timestamp when the override was recorded.
    #[serde(alias = "recorded_at")]
    pub recorded_at: String,
}

/// Collection of overrides for a task run.
#[derive(Debug, Clone, Default, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
#[schemars(deny_unknown_fields)]
pub struct OverrideCollection {
    /// All recorded overrides.
    #[serde(default, skip_serializing_if = "Vec::is_empty", alias = "overrides")]
    pub overrides: Vec<CriterionOverride>,
}

// ============================================================================
// Task completion & stage tracking
// ============================================================================

/// Task-completion result.
///
/// Internally tagged by `status`: the variant fields are inlined alongside the
/// discriminator rather than nested under a `data` key.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "status")]
pub enum TaskCompletionResult {
    /// Task completed successfully.
    #[serde(rename = "success")]
    Success {
        /// Number of iterations used.
        iterations: u32,
        /// Findings accumulated during the run.
        findings: Vec<Finding>,
        /// Final verification results.
        verification_results: IterationVerificationResults,
    },

    /// Task failed (max iterations or unrecoverable error).
    #[serde(rename = "failed")]
    Failed {
        /// Human-readable reason for the failure.
        reason: String,
        /// Number of iterations attempted.
        iterations: u32,
        /// Last verification results (if any).
        #[serde(default, skip_serializing_if = "Option::is_none")]
        last_results: Option<IterationVerificationResults>,
        /// Findings accumulated during the run.
        findings: Vec<Finding>,
    },

    /// Task stopped by user or system.
    #[serde(rename = "stopped")]
    Stopped {
        /// Iteration at which the task was stopped.
        at_iteration: u32,
        /// Findings accumulated up to the stop.
        findings: Vec<Finding>,
        /// Whether the task can be resumed from this point.
        can_resume: bool,
    },

    /// Task paused at max iterations, awaiting user decision.
    #[serde(rename = "paused")]
    Paused {
        /// Iteration at which the task paused.
        at_iteration: u32,
        /// The max-iteration limit that triggered the pause.
        max_iterations: u32,
        /// Last verification results (if any).
        #[serde(default, skip_serializing_if = "Option::is_none")]
        last_results: Option<IterationVerificationResults>,
        /// Findings accumulated up to the pause.
        findings: Vec<Finding>,
    },
}

/// A record of a stage transition during task execution.
///
/// Used for building the stage-based timeline on the recap page.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
#[schemars(deny_unknown_fields)]
pub struct StageTransition {
    /// Previous stage.
    #[serde(alias = "from")]
    pub from: String,
    /// New stage.
    #[serde(alias = "to")]
    pub to: String,
    /// When the transition occurred (ISO 8601).
    #[serde(alias = "timestamp")]
    pub timestamp: String,
    /// Iteration number at the time of the transition.
    #[serde(alias = "iteration")]
    pub iteration: u32,
}
