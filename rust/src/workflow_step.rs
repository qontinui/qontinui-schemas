//! Canonical step DTO types — the four core step variants.
//!
//! This module ports the hand-authored step interfaces from
//! `qontinui-schemas/ts/src/workflow/index.ts` (lines 100–289 in the
//! pre-migration source). Rust is the source of truth; TypeScript and Python
//! bindings are generated from the JSON Schemas emitted by `schemars` on these
//! types.
//!
//! ## The four canonical step types
//!
//! All workflow automation is organized around four phases
//! (setup / verification / agentic / completion). Each phase may carry any of
//! four canonical step variants:
//!
//! - [`CommandStep`] — shell commands, checks, check groups, tests
//! - [`PromptStep`] — AI task instructions
//! - [`UiBridgeStep`] — UI Bridge SDK interactions (navigate, execute, assert, snapshot)
//! - [`WorkflowStep`] — run a saved workflow inline (composition)
//!
//! [`UnifiedStep`] is the internally-tagged discriminated union over those
//! four variants. Its wire format is a flat object with a `"type"`
//! discriminator — not a nested `{ "type": "command", "data": {...} }`
//! envelope. Each variant flattens its [`BaseStepFields`] block so shared
//! fields (`id`, `name`, `inputs`, `extract`, …) sit at the top level.
//!
//! ## `UnifiedWorkflow` step arrays are NOT narrowed
//!
//! The runner dispatches more step types than the canonical four (`gate`,
//! `screenshot`, `playwright`, `state`, `action`, `log_watch`, …). The
//! workflow frame keeps its step arrays as `Vec<serde_json::Value>` so those
//! non-canonical variants continue to round-trip. `UnifiedStep` is available
//! to consumers that want typed access to the canonical variants, but it is
//! not forced into the frame.
//!
//! ## Conventions
//!
//! - Every field optional on the wire uses
//!   `#[serde(default, skip_serializing_if = "Option::is_none")]`.
//! - Empty collections are omitted via `skip_serializing_if = "…_is_empty"`.
//! - Phase enums are narrow per step type (e.g. command steps cannot appear
//!   in the agentic phase).
//! - Enum variants use `#[serde(rename_all = "snake_case")]` by default;
//!   [`HttpMethod`] overrides to `UPPERCASE` to match HTTP convention.
//! - [`BaseStepFields::skill_origin`] is typed as [`serde_json::Value`] to
//!   keep this module self-contained from the `skill` dependency chain.
//! - [`UiBridgeStep::action_plan`] is typed as [`serde_json::Value`] for the
//!   same reason (defined in `./action-plan` on the TS side).

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// ============================================================================
// Helper predicates
// ============================================================================

fn hashmap_is_empty<K, V>(m: &HashMap<K, V>) -> bool {
    m.is_empty()
}

fn vec_is_empty<T>(v: &[T]) -> bool {
    v.is_empty()
}

// ============================================================================
// HTTP / API helper unions
// ============================================================================

/// HTTP methods accepted by API-request command steps.
///
/// Serialized uppercase (`GET`, `POST`, …) to match HTTP convention.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "UPPERCASE")]
pub enum HttpMethod {
    Get,
    Post,
    Put,
    Patch,
    Delete,
}

/// Content-Type values for an API request body.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub enum ApiContentType {
    #[serde(rename = "application/json")]
    ApplicationJson,
    #[serde(rename = "application/x-www-form-urlencoded")]
    ApplicationFormUrlEncoded,
    #[serde(rename = "text/plain")]
    TextPlain,
    #[serde(rename = "none")]
    None,
}

/// Extract a named variable from an API response via JSONPath.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
#[schemars(deny_unknown_fields)]
pub struct ApiVariableExtraction {
    /// Variable name to bind the extracted value to.
    #[serde(alias = "variable_name")]
    pub variable_name: String,
    /// JSONPath expression used to extract the value.
    #[serde(alias = "json_path")]
    pub json_path: String,
    /// Default value if the path does not resolve.
    #[serde(default, skip_serializing_if = "Option::is_none", alias = "default_value")]
    pub default_value: Option<String>,
}

/// Assertion kinds supported on API responses.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ApiAssertionType {
    StatusCode,
    JsonPath,
    Header,
    BodyContains,
    ResponseTime,
}

/// Comparison operators supported by API assertions.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ApiAssertionOperator {
    Equals,
    Contains,
    Matches,
    GreaterThan,
    LessThan,
}

/// A single assertion evaluated against an API response.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
#[schemars(deny_unknown_fields)]
pub struct ApiAssertion {
    /// Kind of assertion.
    #[serde(rename = "type", alias = "assertion_type")]
    pub assertion_type: ApiAssertionType,
    /// Expected value. The TS source allows either a string or number, so
    /// this field stays as `serde_json::Value` on the wire.
    #[serde(alias = "expected")]
    pub expected: serde_json::Value,
    /// JSONPath for `json_path` assertions.
    #[serde(default, skip_serializing_if = "Option::is_none", alias = "json_path")]
    pub json_path: Option<String>,
    /// Header name for `header` assertions.
    #[serde(default, skip_serializing_if = "Option::is_none", alias = "header_name")]
    pub header_name: Option<String>,
    /// Comparison operator; defaults to `equals` on the consumer side.
    #[serde(default, skip_serializing_if = "Option::is_none", alias = "operator")]
    pub operator: Option<ApiAssertionOperator>,
}

// ============================================================================
// Test / check / playwright enums
// ============================================================================

/// Test-runner kinds surfaced by `CommandStep` when `mode = "test"`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum TestType {
    Playwright,
    QontinuiVision,
    Python,
    Repository,
    CustomCommand,
}

/// Whether a Playwright test executes independently or as part of a chain.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum PlaywrightExecutionMode {
    Independent,
    Chained,
}

/// Kinds of deterministic check surfaced by `CommandStep` when
/// `mode = "check"` or `mode = "check_group"`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum CheckType {
    Lint,
    Format,
    Typecheck,
    Analyze,
    Security,
    CustomCommand,
    HttpStatus,
    AiReview,
    CiCd,
}

// ============================================================================
// Verification category
// ============================================================================

/// Verification-depth category for a step.
///
/// Mirrors the TS `VerificationCategory` literal union. Kept local to this
/// module because it is only referenced from [`BaseStepFields`].
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum VerificationCategoryKind {
    Existence,
    Uniqueness,
    ReferentialIntegrity,
    SemanticCorrectness,
    RuntimeBehavior,
}

// ============================================================================
// Retry spec
// ============================================================================

/// Per-step retry configuration carried by [`BaseStepFields::retry`].
///
/// Distinct from the workflow frame's `RetryPolicy` — that one also carries a
/// `backoff` flag, this per-step form is the older, simpler shape that step
/// DTOs inherited from the TS `BaseStep` interface.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
#[schemars(deny_unknown_fields)]
pub struct RetrySpec {
    /// Number of retry attempts (`0` = no retries).
    #[serde(alias = "count")]
    pub count: u32,
    /// Delay between retries in milliseconds.
    #[serde(alias = "delay_ms")]
    pub delay_ms: u64,
}

// ============================================================================
// BaseStepFields — shared across all four variants via #[serde(flatten)]
// ============================================================================

/// Shared fields common to every canonical step variant.
///
/// Flattened into each step struct via `#[serde(flatten)]` so the wire shape
/// stays flat (no nested `"base": { … }` envelope).
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
#[schemars(deny_unknown_fields)]
pub struct BaseStepFields {
    /// Unique identifier for the step.
    #[serde(alias = "id")]
    pub id: String,
    /// Display name for the step.
    #[serde(alias = "name")]
    pub name: String,
    /// If `Some(true)`, a console-error signal from the UI fails this step.
    #[serde(default, skip_serializing_if = "Option::is_none", alias = "fail_on_console_errors")]
    pub fail_on_console_errors: Option<bool>,
    /// Named input bindings evaluated at step entry.
    #[serde(default, skip_serializing_if = "hashmap_is_empty", alias = "inputs")]
    pub inputs: HashMap<String, String>,
    /// Extractions published to subsequent steps.
    #[serde(default, skip_serializing_if = "hashmap_is_empty", alias = "extract")]
    pub extract: HashMap<String, String>,
    /// IDs of other steps that must complete first.
    #[serde(default, skip_serializing_if = "vec_is_empty", alias = "depends_on")]
    pub depends_on: Vec<String>,
    /// Whether this step is required (default: `true` on consumer side).
    #[serde(default, skip_serializing_if = "Option::is_none", alias = "required")]
    pub required: Option<bool>,
    /// Per-step retry configuration.
    #[serde(default, skip_serializing_if = "Option::is_none", alias = "retry")]
    pub retry: Option<RetrySpec>,
    /// Acceptance criterion IDs verified by this step.
    #[serde(default, skip_serializing_if = "vec_is_empty", alias = "criterion_ids")]
    pub criterion_ids: Vec<String>,
    /// Verification depth category.
    #[serde(default, skip_serializing_if = "Option::is_none", alias = "verification_category")]
    pub verification_category: Option<VerificationCategoryKind>,
    /// Provenance of this step when generated from a skill template.
    ///
    /// Typed as `serde_json::Value` here to avoid pulling the `skill`
    /// dependency chain into this module; the TS side re-imports the typed
    /// `SkillOrigin` after regeneration.
    #[serde(default, skip_serializing_if = "Option::is_none", alias = "skill_origin")]
    pub skill_origin: Option<serde_json::Value>,
}

// ============================================================================
// Phase enums
// ============================================================================

/// Phases in which a [`CommandStep`] may appear.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum CommandStepPhase {
    #[default]
    Setup,
    Verification,
    Completion,
}

/// Phases in which a [`PromptStep`] may appear.
///
/// Prompt steps are the only variant that may appear in the agentic phase.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum PromptStepPhase {
    #[default]
    Setup,
    Verification,
    Agentic,
    Completion,
}

/// Phases in which a [`UiBridgeStep`] may appear.
///
/// UI-bridge interactions only run in deterministic phases — never inside
/// the agentic loop (where the AI drives steps directly via prompts).
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum UiBridgeStepPhase {
    #[default]
    Setup,
    Verification,
    Completion,
}

/// Phases in which a [`WorkflowStep`] may appear.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum WorkflowStepPhase {
    #[default]
    Setup,
    Verification,
    Completion,
}

// ============================================================================
// CommandStep
// ============================================================================

/// Execution mode for a [`CommandStep`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum CommandMode {
    Shell,
    Check,
    CheckGroup,
    Test,
}

/// Shell commands, checks, check groups, and tests.
///
/// A single variant covers all command-like steps; the specific sub-kind is
/// carried by [`CommandMode`] and the matching `*_id` / `*_type` fields.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct CommandStep {
    #[serde(flatten)]
    pub base: BaseStepFields,
    /// Phase in which the step appears.
    #[serde(alias = "phase")]
    pub phase: CommandStepPhase,
    /// Execution mode — which sub-kind of command step this is.
    #[serde(default, skip_serializing_if = "Option::is_none", alias = "mode")]
    pub mode: Option<CommandMode>,
    /// Shell command line (for `shell` mode).
    #[serde(default, skip_serializing_if = "Option::is_none", alias = "command")]
    pub command: Option<String>,
    /// Working directory for the command.
    #[serde(default, skip_serializing_if = "Option::is_none", alias = "working_directory")]
    pub working_directory: Option<String>,
    /// Timeout in seconds.
    #[serde(default, skip_serializing_if = "Option::is_none", alias = "timeout_seconds")]
    pub timeout_seconds: Option<u64>,
    /// Whether non-zero exit status fails the step.
    #[serde(default, skip_serializing_if = "Option::is_none", alias = "fail_on_error")]
    pub fail_on_error: Option<bool>,
    /// Re-run this step on every verification-agentic iteration.
    #[serde(default, skip_serializing_if = "Option::is_none", alias = "run_on_subsequent_iterations")]
    pub run_on_subsequent_iterations: Option<bool>,
    /// Saved shell command template ID.
    #[serde(default, skip_serializing_if = "Option::is_none", alias = "shell_command_id")]
    pub shell_command_id: Option<String>,
    /// Kind of deterministic check (for `check` / `check_group` modes).
    #[serde(default, skip_serializing_if = "Option::is_none", alias = "check_type")]
    pub check_type: Option<CheckType>,
    /// Tool identifier (e.g., `eslint`, `ruff`).
    #[serde(default, skip_serializing_if = "Option::is_none", alias = "tool")]
    pub tool: Option<String>,
    /// Saved check definition ID.
    #[serde(default, skip_serializing_if = "Option::is_none", alias = "check_id")]
    pub check_id: Option<String>,
    /// Path to the check's config file.
    #[serde(default, skip_serializing_if = "Option::is_none", alias = "config_path")]
    pub config_path: Option<String>,
    /// Whether to auto-fix during the check.
    #[serde(default, skip_serializing_if = "Option::is_none", alias = "auto_fix")]
    pub auto_fix: Option<bool>,
    /// Fail the step on warnings in addition to errors.
    #[serde(default, skip_serializing_if = "Option::is_none", alias = "fail_on_warning")]
    pub fail_on_warning: Option<bool>,
    /// Repository selector for repository-targeted steps.
    #[serde(default, skip_serializing_if = "Option::is_none", alias = "repository")]
    pub repository: Option<String>,
    /// Name of a workflow to invoke.
    #[serde(default, skip_serializing_if = "Option::is_none", alias = "workflow_name")]
    pub workflow_name: Option<String>,
    /// Branch selector for repository-targeted steps.
    #[serde(default, skip_serializing_if = "Option::is_none", alias = "branch")]
    pub branch: Option<String>,
    /// Whether the caller waits for the workflow to complete.
    #[serde(default, skip_serializing_if = "Option::is_none", alias = "wait_for_completion")]
    pub wait_for_completion: Option<bool>,
    /// Saved check-group ID.
    #[serde(default, skip_serializing_if = "Option::is_none", alias = "check_group_id")]
    pub check_group_id: Option<String>,
    /// Test runner kind.
    #[serde(default, skip_serializing_if = "Option::is_none", alias = "test_type")]
    pub test_type: Option<TestType>,
    /// Saved test ID.
    #[serde(default, skip_serializing_if = "Option::is_none", alias = "test_id")]
    pub test_id: Option<String>,
    /// Inline code body (e.g., Python snippet).
    #[serde(default, skip_serializing_if = "Option::is_none", alias = "code")]
    pub code: Option<String>,
    /// Saved script ID.
    #[serde(default, skip_serializing_if = "Option::is_none", alias = "script_id")]
    pub script_id: Option<String>,
    /// Inline script contents.
    #[serde(default, skip_serializing_if = "Option::is_none", alias = "script_content")]
    pub script_content: Option<String>,
    /// Target URL for navigation-style tests.
    #[serde(default, skip_serializing_if = "Option::is_none", alias = "target_url")]
    pub target_url: Option<String>,
    /// Saved fused-script ID.
    #[serde(default, skip_serializing_if = "Option::is_none", alias = "fused_script_id")]
    pub fused_script_id: Option<String>,
    /// Execution mode for Playwright tests.
    #[serde(default, skip_serializing_if = "Option::is_none", alias = "execution_mode")]
    pub execution_mode: Option<PlaywrightExecutionMode>,
}

// ============================================================================
// PromptStep
// ============================================================================

/// AI task instructions (prompt).
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct PromptStep {
    #[serde(flatten)]
    pub base: BaseStepFields,
    /// Phase in which the step appears.
    #[serde(alias = "phase")]
    pub phase: PromptStepPhase,
    /// Prompt body.
    #[serde(alias = "content")]
    pub content: String,
    /// Saved prompt ID (when the body is a reference).
    #[serde(default, skip_serializing_if = "Option::is_none", alias = "prompt_id")]
    pub prompt_id: Option<String>,
    /// AI provider override.
    #[serde(default, skip_serializing_if = "Option::is_none", alias = "provider")]
    pub provider: Option<String>,
    /// Model override.
    #[serde(default, skip_serializing_if = "Option::is_none", alias = "model")]
    pub model: Option<String>,
    /// Marks this prompt as the summary step at the end of completion.
    #[serde(default, skip_serializing_if = "Option::is_none", alias = "is_summary_step")]
    pub is_summary_step: Option<bool>,
}

// ============================================================================
// UiBridgeStep
// ============================================================================

/// UI Bridge action kind.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum UiBridgeAction {
    #[default]
    Navigate,
    Execute,
    Assert,
    Snapshot,
    Compare,
    SnapshotAssert,
    ActionPlan,
}

/// Kinds of assertion supported by `assert` actions.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum UiBridgeAssertType {
    Exists,
    TextEquals,
    Contains,
    Visible,
    Enabled,
}

/// Comparison mode for `compare` / `snapshot_assert` actions.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum UiBridgeComparisonMode {
    Structural,
    Visual,
    Both,
}

/// Severity threshold for `compare` / `snapshot_assert` actions.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum UiBridgeSeverity {
    Critical,
    Major,
    Minor,
    Info,
}

/// UI Bridge SDK interaction — navigate, execute, assert, snapshot, compare.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct UiBridgeStep {
    #[serde(flatten)]
    pub base: BaseStepFields,
    /// Phase in which the step appears.
    #[serde(alias = "phase")]
    pub phase: UiBridgeStepPhase,
    /// Action kind.
    #[serde(alias = "action")]
    pub action: UiBridgeAction,
    /// Navigation URL (for `navigate`).
    #[serde(default, skip_serializing_if = "Option::is_none", alias = "url")]
    pub url: Option<String>,
    /// Free-form instruction text (for `execute`).
    #[serde(default, skip_serializing_if = "Option::is_none", alias = "instruction")]
    pub instruction: Option<String>,
    /// Target selector or element ID.
    #[serde(default, skip_serializing_if = "Option::is_none", alias = "target")]
    pub target: Option<String>,
    /// Assertion kind (for `assert`).
    #[serde(default, skip_serializing_if = "Option::is_none", alias = "assert_type")]
    pub assert_type: Option<UiBridgeAssertType>,
    /// Expected value for assertions.
    #[serde(default, skip_serializing_if = "Option::is_none", alias = "expected")]
    pub expected: Option<String>,
    /// Timeout in milliseconds.
    #[serde(default, skip_serializing_if = "Option::is_none", alias = "timeout_ms")]
    pub timeout_ms: Option<u64>,
    /// Comparison mode (for `compare` / `snapshot_assert`).
    #[serde(default, skip_serializing_if = "Option::is_none", alias = "comparison_mode")]
    pub comparison_mode: Option<UiBridgeComparisonMode>,
    /// Reference snapshot ID (for `compare` / `snapshot_assert`).
    #[serde(default, skip_serializing_if = "Option::is_none", alias = "reference_snapshot_id")]
    pub reference_snapshot_id: Option<String>,
    /// Severity threshold (for `compare` / `snapshot_assert`).
    #[serde(default, skip_serializing_if = "Option::is_none", alias = "severity_threshold")]
    pub severity_threshold: Option<UiBridgeSeverity>,
    /// Snapshot target — `"control"`, `"sdk"`, or `"proxy:PORT"`.
    #[serde(default, skip_serializing_if = "Option::is_none", alias = "ui_bridge_snapshot_target")]
    pub ui_bridge_snapshot_target: Option<String>,
    /// Structured action plan (for `action_plan`).
    ///
    /// Typed as `serde_json::Value` here to avoid pulling the `action-plan`
    /// module into this crate; the TS side re-imports the typed `ActionPlan`
    /// after regeneration.
    #[serde(default, skip_serializing_if = "Option::is_none", alias = "action_plan")]
    pub action_plan: Option<serde_json::Value>,
}

// ============================================================================
// WorkflowStep
// ============================================================================

/// Run a saved workflow inline (composition).
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct WorkflowStep {
    #[serde(flatten)]
    pub base: BaseStepFields,
    /// Phase in which the step appears.
    #[serde(alias = "phase")]
    pub phase: WorkflowStepPhase,
    /// ID of the saved workflow to run.
    #[serde(alias = "workflow_id")]
    pub workflow_id: String,
    /// Display name of the saved workflow (denormalized for UI).
    #[serde(alias = "workflow_name")]
    pub workflow_name: String,
}

// ============================================================================
// CanonicalStep — the four canonical step variants, internally tagged
// ============================================================================

/// Discriminated union over the four canonical step variants.
///
/// Wire format is a flat object with a `"type"` discriminator — serde's
/// internal tagging merges the inner struct's fields (including the flattened
/// [`BaseStepFields`]) up into the top-level object. Example:
///
/// ```text
/// {"type":"command","id":"s1","name":"build","phase":"setup","mode":"shell","command":"cargo build"}
/// ```
///
/// Consumers that want a strict 4-variant typed view should use
/// [`CanonicalStep`]. Consumers that need to tolerate runner-specific step
/// types (e.g. `gate`, `screenshot`, `playwright`, `state`, `action`,
/// `log_watch`, and others dispatched by the runner but absent from the
/// wire-contract surface) should use [`UnifiedStep`], which preserves
/// unknown payloads verbatim as `serde_json::Value`.
///
/// Variant sizes are similar (~200–672 bytes each); the asymmetry reflects
/// real differences in step-field cardinality and doesn't warrant boxing.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "type", rename_all = "snake_case")]
#[allow(clippy::large_enum_variant)]
pub enum CanonicalStep {
    Command(CommandStep),
    Prompt(PromptStep),
    UiBridge(UiBridgeStep),
    Workflow(WorkflowStep),
}

// ============================================================================
// UnifiedStep — canonical-first with opaque fallback
// ============================================================================

/// A workflow step, preferring typed canonical variants and falling back to an
/// opaque [`serde_json::Value`] for runner-specific types not yet part of the
/// wire contract.
///
/// Serialization is transparent: a [`CanonicalStep`] serializes with its flat
/// `"type"`-tagged shape; [`UnifiedStep::Other`] serializes the wrapped value
/// as-is. Deserialization tries the canonical shape first; any payload that
/// does not match (unknown `"type"`, missing fields, or missing discriminator)
/// is preserved as [`UnifiedStep::Other`].
///
/// This catch-all is what makes the type *robust* on the wire: a runner can
/// emit a `{"type":"gate", ...}` step and a consumer using [`UnifiedStep`]
/// will round-trip it losslessly even though `gate` is not in the canonical
/// set.
///
/// ## Layout note (`#[allow(large_enum_variant)]`)
///
/// `Canonical` carries a [`CanonicalStep`] (~672 bytes) while `Other` carries
/// a [`serde_json::Value`] (~32 bytes). The size asymmetry is intentional and
/// the enum is not held in bulk by any hot path today — `UnifiedWorkflow.*_steps`
/// remains `Vec<serde_json::Value>`. Boxing `Canonical` would save stack space
/// in hypothetical dense `Vec<UnifiedStep>` consumers at the cost of an extra
/// heap allocation per step everywhere else and noisier pattern-matching.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(untagged)]
#[allow(clippy::large_enum_variant)]
pub enum UnifiedStep {
    /// One of the four canonical step variants (command / prompt / ui_bridge / workflow).
    Canonical(CanonicalStep),
    /// Any other step-shaped JSON value, preserved verbatim.
    Other(serde_json::Value),
}

impl UnifiedStep {
    /// Construct a canonical command step.
    pub fn command(step: CommandStep) -> Self {
        Self::Canonical(CanonicalStep::Command(step))
    }
    /// Construct a canonical prompt step.
    pub fn prompt(step: PromptStep) -> Self {
        Self::Canonical(CanonicalStep::Prompt(step))
    }
    /// Construct a canonical UI Bridge step.
    pub fn ui_bridge(step: UiBridgeStep) -> Self {
        Self::Canonical(CanonicalStep::UiBridge(step))
    }
    /// Construct a canonical workflow (composition) step.
    pub fn workflow(step: WorkflowStep) -> Self {
        Self::Canonical(CanonicalStep::Workflow(step))
    }

    /// Try to view this step as one of the canonical variants.
    /// Returns `None` if the step is an [`UnifiedStep::Other`] payload.
    pub fn as_canonical(&self) -> Option<&CanonicalStep> {
        match self {
            Self::Canonical(c) => Some(c),
            Self::Other(_) => None,
        }
    }

    /// Return the step's `"type"` discriminator, if present in the payload.
    ///
    /// For canonical variants the discriminator is always present and derived
    /// from the variant. For [`UnifiedStep::Other`], the `"type"` field of the
    /// wrapped JSON object is returned if any; otherwise `None`.
    pub fn step_type(&self) -> Option<&str> {
        match self {
            Self::Canonical(CanonicalStep::Command(_)) => Some("command"),
            Self::Canonical(CanonicalStep::Prompt(_)) => Some("prompt"),
            Self::Canonical(CanonicalStep::UiBridge(_)) => Some("ui_bridge"),
            Self::Canonical(CanonicalStep::Workflow(_)) => Some("workflow"),
            Self::Other(v) => v.get("type").and_then(|t| t.as_str()),
        }
    }
}

// ============================================================================
// Additional runner-specific step variant structs
// (not in the canonical 4, but registered in HandlerRegistry)
// ============================================================================

// ── CodeExecutionStep ────────────────────────────────────────────────────────

/// Execute inline Python code or a Python file in an optional sandbox.
///
/// Wire tag: `"code_execution"`.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct CodeExecutionStep {
    #[serde(flatten)]
    pub base: BaseStepFields,
    /// Inline Python source code to execute.
    #[serde(default, skip_serializing_if = "Option::is_none", alias = "code")]
    pub code: Option<String>,
    /// Path to a Python file to execute.
    #[serde(default, skip_serializing_if = "Option::is_none", alias = "code_file")]
    pub code_file: Option<String>,
    /// Sandbox mode: `"enforce"` (default) or `"warn"`.
    #[serde(default, skip_serializing_if = "Option::is_none", alias = "sandbox_mode")]
    pub sandbox_mode: Option<String>,
    /// Timeout in seconds.
    #[serde(default, skip_serializing_if = "Option::is_none", alias = "timeout_seconds")]
    pub timeout_seconds: Option<u64>,
}

// ── ExecutePlaybookStep ──────────────────────────────────────────────────────

/// Drive a state machine through recorded transitions from a playbook.
///
/// Wire tag: `"execute_playbook"`.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct ExecutePlaybookStep {
    #[serde(flatten)]
    pub base: BaseStepFields,
    /// Playbook markdown content (inline).
    #[serde(default, skip_serializing_if = "Option::is_none", alias = "content")]
    pub content: Option<String>,
    /// Path to a playbook file.
    #[serde(default, skip_serializing_if = "Option::is_none", alias = "playbook_path")]
    pub playbook_path: Option<String>,
    /// Timeout in seconds.
    #[serde(default, skip_serializing_if = "Option::is_none", alias = "timeout_seconds")]
    pub timeout_seconds: Option<u64>,
}

// ── NativeAccessibilityStep ──────────────────────────────────────────────────

/// Accessibility action kinds.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum A11yAction {
    #[default]
    Capture,
    Click,
    Type,
    Focus,
    Query,
    AiContext,
}

/// Interact with native UI elements via the accessibility layer (UIA/AT-SPI/AX).
///
/// Wire tag: `"native_accessibility"`.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct NativeAccessibilityStep {
    #[serde(flatten)]
    pub base: BaseStepFields,
    /// Action to perform.
    #[serde(default, alias = "action")]
    pub action: A11yAction,
    /// Connection target: `"Desktop"`, window title, or `"pid:1234"`.
    #[serde(default, skip_serializing_if = "Option::is_none", alias = "target")]
    pub target: Option<String>,
    /// Element ref ID for click/type/focus (e.g. `"@e3"`).
    #[serde(default, skip_serializing_if = "Option::is_none", alias = "ref_id")]
    pub ref_id: Option<String>,
    /// Text to type (for `type` action).
    #[serde(default, skip_serializing_if = "Option::is_none", alias = "text")]
    pub text: Option<String>,
    /// Whether to clear existing text before typing.
    #[serde(default, skip_serializing_if = "Option::is_none", alias = "clear_first")]
    pub clear_first: Option<bool>,
    /// Role filter for `query` action (e.g. `"button"`, `"textbox"`).
    #[serde(default, skip_serializing_if = "Option::is_none", alias = "query_role")]
    pub query_role: Option<String>,
    /// Label filter for `query` action.
    #[serde(default, skip_serializing_if = "Option::is_none", alias = "query_label")]
    pub query_label: Option<String>,
    /// Only include interactive elements (for `query` and `ai_context`).
    #[serde(default, skip_serializing_if = "Option::is_none", alias = "interactive_only")]
    pub interactive_only: Option<bool>,
    /// Maximum elements for `ai_context` action (default: 50).
    #[serde(default, skip_serializing_if = "Option::is_none", alias = "max_elements")]
    pub max_elements: Option<u32>,
    /// Include hidden elements in `capture`.
    #[serde(default, skip_serializing_if = "Option::is_none", alias = "include_hidden")]
    pub include_hidden: Option<bool>,
    /// Maximum tree depth for `capture`.
    #[serde(default, skip_serializing_if = "Option::is_none", alias = "max_depth")]
    pub max_depth: Option<u32>,
    /// Timeout in seconds.
    #[serde(default, skip_serializing_if = "Option::is_none", alias = "timeout_seconds")]
    pub timeout_seconds: Option<u64>,
}

// ── RestartProcessStep ───────────────────────────────────────────────────────

/// Restart a managed process by ID or name.
///
/// Wire tag: `"restart_process"`.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct RestartProcessStep {
    #[serde(flatten)]
    pub base: BaseStepFields,
    /// Process config ID to restart.
    #[serde(default, skip_serializing_if = "Option::is_none", alias = "restart_process_id")]
    pub restart_process_id: Option<String>,
    /// Process name to restart (resolved to ID at runtime).
    #[serde(default, skip_serializing_if = "Option::is_none", alias = "restart_process_name")]
    pub restart_process_name: Option<String>,
    /// Whether to wait for health port after restart (default: `true`).
    #[serde(default, skip_serializing_if = "Option::is_none", alias = "restart_wait_for_health")]
    pub restart_wait_for_health: Option<bool>,
    /// Timeout in seconds.
    #[serde(default, skip_serializing_if = "Option::is_none", alias = "timeout_seconds")]
    pub timeout_seconds: Option<u64>,
}

// ── SaveWorkflowArtifactStep ─────────────────────────────────────────────────

/// Read a generated workflow JSON file and save it to the database.
///
/// Wire tag: `"save_workflow_artifact"`.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct SaveWorkflowArtifactStep {
    #[serde(flatten)]
    pub base: BaseStepFields,
    /// Path to the workflow JSON file to save.
    #[serde(default, skip_serializing_if = "Option::is_none", alias = "artifact_input_path")]
    pub artifact_input_path: Option<String>,
    /// When `true`, also creates a `PipelineArtifact` from the artifact directory.
    #[serde(default, skip_serializing_if = "Option::is_none", alias = "artifact_capture_prompts")]
    pub artifact_capture_prompts: Option<bool>,
}

// ── WorkflowFixupStep ────────────────────────────────────────────────────────

/// Fixup mode for [`WorkflowFixupStep`].
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum WorkflowFixupMode {
    #[default]
    Autofix,
    Harden,
    ValidateCriteria,
}

/// Run deterministic Rust fixups on a workflow JSON file.
///
/// Wire tag: `"workflow_fixup"`.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct WorkflowFixupStep {
    #[serde(flatten)]
    pub base: BaseStepFields,
    /// Path to the workflow JSON file to fix (supports `{{artifact_dir}}`).
    #[serde(default, skip_serializing_if = "Option::is_none", alias = "fixup_input_path")]
    pub fixup_input_path: Option<String>,
    /// Fixup mode.
    #[serde(default, skip_serializing_if = "Option::is_none", alias = "fixup_mode")]
    pub fixup_mode: Option<WorkflowFixupMode>,
    /// Path to criteria JSON file (for `validate_criteria` mode).
    #[serde(default, skip_serializing_if = "Option::is_none", alias = "fixup_criteria_path")]
    pub fixup_criteria_path: Option<String>,
}

// ── UiBridgeDesignAuditStep ──────────────────────────────────────────────────

/// Run a UI Bridge design-audit check (contrast, accessibility, select visibility).
///
/// Wire tag: `"ui_bridge_design_audit"`.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct UiBridgeDesignAuditStep {
    #[serde(flatten)]
    pub base: BaseStepFields,
    /// Timeout in seconds.
    #[serde(default, skip_serializing_if = "Option::is_none", alias = "timeout_seconds")]
    pub timeout_seconds: Option<u64>,
}

// ── VgaAutomateStep ──────────────────────────────────────────────────────────

/// Kinds of action supported by a [`VgaAutomateStep`] `action_sequence` entry.
///
/// Mirrors the TS `VgaActionKind` literal union. Each variant carries its own
/// set of fields (see [`VgaAction`]).
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum VgaActionKind {
    #[default]
    Click,
    Type,
    WaitFor,
}

/// A single action inside a [`VgaAutomateStep`] action sequence.
///
/// Wire shape is internally tagged with `"kind"`:
/// - `{ "kind": "click", "elementId": "<uuid>", "timeoutMs": 10000 }`
/// - `{ "kind": "type", "text": "hello", "elementId": "<uuid>", "timeoutMs": 10000 }`
///   (`elementId` omitted/null = type into the currently focused element)
/// - `{ "kind": "wait_for", "elementId": "<uuid>", "timeoutMs": 30000 }`
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum VgaAction {
    /// Click the element identified by `element_id` (UUID referencing an
    /// element inside the VGA state machine).
    Click {
        /// UUID of the target element in the VGA state machine's element set.
        #[serde(rename = "elementId", alias = "element_id")]
        element_id: String,
        /// Per-action timeout in milliseconds. Defaults to `10000` on the
        /// consumer side.
        #[serde(
            default,
            skip_serializing_if = "Option::is_none",
            rename = "timeoutMs",
            alias = "timeout_ms"
        )]
        timeout_ms: Option<u64>,
    },
    /// Type `text` into `element_id`, or into the currently focused element
    /// when `element_id` is absent/`null`.
    Type {
        /// Literal text to type.
        #[serde(alias = "text")]
        text: String,
        /// UUID of the target element. Absent / `null` means "type into
        /// whichever element currently has focus".
        #[serde(
            default,
            skip_serializing_if = "Option::is_none",
            rename = "elementId",
            alias = "element_id"
        )]
        element_id: Option<String>,
        /// Per-action timeout in milliseconds. Defaults to `10000` on the
        /// consumer side.
        #[serde(
            default,
            skip_serializing_if = "Option::is_none",
            rename = "timeoutMs",
            alias = "timeout_ms"
        )]
        timeout_ms: Option<u64>,
    },
    /// Wait for `element_id` to become visible / actionable.
    WaitFor {
        /// UUID of the target element.
        #[serde(rename = "elementId", alias = "element_id")]
        element_id: String,
        /// Per-action timeout in milliseconds. Defaults to `30000` on the
        /// consumer side.
        #[serde(
            default,
            skip_serializing_if = "Option::is_none",
            rename = "timeoutMs",
            alias = "timeout_ms"
        )]
        timeout_ms: Option<u64>,
    },
}

/// Run a Visual GUI Automation (VGA) action sequence against a target process.
///
/// Wire tag: `"vga_automate"`.
///
/// Delegates to the Python `qontinui.vga.worker` via the `python-bridge` IPC.
/// The worker loads the referenced state machine (`state_machine_id` →
/// `runner.vga_state_machines`), focuses `target_process`, and executes each
/// entry in `action_sequence` by grounding the element prompt against a fresh
/// screenshot and dispatching the HAL click/type/wait primitive.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct VgaAutomateStep {
    #[serde(flatten)]
    pub base: BaseStepFields,
    /// UUID referencing `runner.vga_state_machines.id` — the persisted state
    /// machine that defines the elements this step may click / type into /
    /// wait for.
    #[serde(alias = "state_machine_id")]
    pub state_machine_id: String,
    /// Target process / window — e.g. `"notepad++.exe"`. Used by the HAL to
    /// focus the correct top-level window before each action.
    #[serde(alias = "target_process")]
    pub target_process: String,
    /// Ordered sequence of VGA actions to execute.
    #[serde(default, skip_serializing_if = "vec_is_empty", alias = "action_sequence")]
    pub action_sequence: Vec<VgaAction>,
    /// Overall step timeout in milliseconds. Defaults to `300000` (5 minutes)
    /// on the consumer side; bounds `[1000, 3600000]`.
    #[serde(default, skip_serializing_if = "Option::is_none", alias = "timeout_ms")]
    pub timeout_ms: Option<u64>,
    /// Reserved for future async mode. Currently must be `false` (or omitted)
    /// — the handler rejects `true` until async mode is implemented.
    #[serde(default, skip_serializing_if = "Option::is_none", alias = "async")]
    pub r#async: Option<bool>,
}

// ── UiBridgeVisualAssertionStep ──────────────────────────────────────────────

/// Visual assertion type.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum VisualAssertionType {
    #[default]
    Text,
    Screenshot,
    Highlight,
}

/// Assert visual properties of UI elements via the UI Bridge.
///
/// Wire tag: `"ui_bridge_visual_assertion"`.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct UiBridgeVisualAssertionStep {
    #[serde(flatten)]
    pub base: BaseStepFields,
    /// Assertion type: `"text"`, `"screenshot"`, or `"highlight"`.
    #[serde(default, skip_serializing_if = "Option::is_none", alias = "visual_assertion_type")]
    pub visual_assertion_type: Option<VisualAssertionType>,
    /// Element query JSON for text assertions.
    #[serde(default, skip_serializing_if = "Option::is_none", alias = "visual_assertion_query")]
    pub visual_assertion_query: Option<serde_json::Value>,
    /// Expected text (for text assertion) or element ID (for screenshot/highlight).
    #[serde(default, skip_serializing_if = "Option::is_none", alias = "visual_assertion_expected")]
    pub visual_assertion_expected: Option<String>,
    /// Options JSON for the assertion.
    #[serde(default, skip_serializing_if = "Option::is_none", alias = "visual_assertion_options")]
    pub visual_assertion_options: Option<serde_json::Value>,
    /// Timeout in seconds.
    #[serde(default, skip_serializing_if = "Option::is_none", alias = "timeout_seconds")]
    pub timeout_seconds: Option<u64>,
}

// ── WorkflowRefStep ──────────────────────────────────────────────────────────

/// Run a saved workflow inline with input variable substitution.
///
/// Wire tag: `"workflow_ref"`.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct WorkflowRefStep {
    #[serde(flatten)]
    pub base: BaseStepFields,
    /// ID of the saved workflow to run.
    #[serde(default, skip_serializing_if = "Option::is_none", alias = "workflow_id")]
    pub workflow_id: Option<String>,
    /// Display name of the referenced workflow (denormalized for UI).
    #[serde(default, skip_serializing_if = "Option::is_none", alias = "ref_workflow_name")]
    pub ref_workflow_name: Option<String>,
    /// Input variables substituted into the child workflow's prompt.
    #[serde(default, skip_serializing_if = "Option::is_none", alias = "ref_workflow_inputs")]
    pub ref_workflow_inputs: Option<HashMap<String, String>>,
    /// Whether to inherit model overrides from the parent context (default: `true`).
    #[serde(default, skip_serializing_if = "Option::is_none", alias = "ref_inherit_model_overrides")]
    pub ref_inherit_model_overrides: Option<bool>,
    /// Timeout in seconds.
    #[serde(default, skip_serializing_if = "Option::is_none", alias = "timeout_seconds")]
    pub timeout_seconds: Option<u64>,
}

// ── DagCancelStep ────────────────────────────────────────────────────────────

/// DAG cancel node — terminates the workflow run with a cancellation reason.
///
/// Wire tag: `"dag_cancel"`.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct DagCancelStep {
    #[serde(flatten)]
    pub base: BaseStepFields,
    /// Cancellation reason message.
    #[serde(default, skip_serializing_if = "Option::is_none", alias = "cancel_reason")]
    pub cancel_reason: Option<String>,
}

// ── DagApprovalStep ──────────────────────────────────────────────────────────

/// DAG approval gate — pauses and waits for human approval before continuing.
///
/// Wire tag: `"dag_approval"`.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct DagApprovalStep {
    #[serde(flatten)]
    pub base: BaseStepFields,
    /// Prompt shown to the approver.
    #[serde(default, skip_serializing_if = "Option::is_none", alias = "approval_prompt")]
    pub approval_prompt: Option<String>,
    /// Timeout in seconds before auto-approving (to prevent permanent block).
    #[serde(default, skip_serializing_if = "Option::is_none", alias = "timeout_seconds")]
    pub timeout_seconds: Option<u64>,
}

// ── DagLoopStep ──────────────────────────────────────────────────────────────

/// DAG loop node — repeats a set of steps up to a maximum iteration count.
///
/// Wire tag: `"dag_loop"`.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct DagLoopStep {
    #[serde(flatten)]
    pub base: BaseStepFields,
    /// Maximum number of loop iterations.
    #[serde(default, skip_serializing_if = "Option::is_none", alias = "max_iterations")]
    pub max_iterations: Option<u32>,
    /// Condition to evaluate each iteration (JSON expression or step ID).
    #[serde(default, skip_serializing_if = "Option::is_none", alias = "loop_condition")]
    pub loop_condition: Option<String>,
}

// ============================================================================
// FullRunnerStep — all 16 handler-registered step variants, internally tagged
// ============================================================================

/// Fully typed discriminated union over **all** step variants registered in
/// the runner's `HandlerRegistry`.
///
/// ## Wire format
///
/// Internally tagged with `"type"`, matching existing JSON on the wire:
/// ```json
/// {"type": "command", "mode": "shell", "command": "cargo build", ...}
/// {"type": "prompt", "phase": "agentic", "content": "..."}
/// {"type": "ui_bridge", "action": "navigate", "url": "..."}
/// {"type": "code_execution", "code": "print('hello')"}
/// ```
///
/// ## Step arrays remain `Vec<serde_json::Value>`
///
/// `UnifiedWorkflow.setup_steps` / `.verification_steps` / `.agentic_steps` /
/// `.completion_steps` stay as `Vec<serde_json::Value>` until the Session 2
/// migration lands. `FullRunnerStep` is available for typed access but is not
/// yet threaded into the workflow frame fields.
///
/// ## Variant coverage
///
/// | Variant | Wire tag | Handler |
/// |---------|----------|---------|
/// | `Command` | `"command"` | `CommandHandler` (sub-modes: shell/check/check_group/test) |
/// | `Prompt` | `"prompt"` | `PromptStepHandler` |
/// | `UiBridge` | `"ui_bridge"` | `UiBridgeHandler` (actions: navigate/execute/assert/snapshot/compare/snapshot_assert/action_plan) |
/// | `Workflow` | `"workflow"` | `WorkflowStepHandler` |
/// | `CodeExecution` | `"code_execution"` | `CodeExecutionHandler` |
/// | `ExecutePlaybook` | `"execute_playbook"` | `ExecutePlaybookHandler` |
/// | `NativeAccessibility` | `"native_accessibility"` | `NativeAccessibilityHandler` |
/// | `RestartProcess` | `"restart_process"` | `RestartProcessHandler` |
/// | `SaveWorkflowArtifact` | `"save_workflow_artifact"` | `SaveWorkflowArtifactHandler` |
/// | `WorkflowFixup` | `"workflow_fixup"` | `WorkflowFixupHandler` |
/// | `UiBridgeDesignAudit` | `"ui_bridge_design_audit"` | `UiBridgeDesignAuditHandler` |
/// | `UiBridgeVisualAssertion` | `"ui_bridge_visual_assertion"` | `UiBridgeVisualAssertionHandler` |
/// | `VgaAutomate` | `"vga_automate"` | `VgaAutomateHandler` |
/// | `WorkflowRef` | `"workflow_ref"` | `WorkflowRefHandler` |
/// | `DagCancel` | `"dag_cancel"` | `dag_nodes::DagCancelHandler` |
/// | `DagApproval` | `"dag_approval"` | `dag_nodes::DagApprovalHandler` |
/// | `DagLoop` | `"dag_loop"` | `dag_nodes::DagLoopHandler` |
///
/// Variant sizes range ~200–672 bytes depending on each step struct's field
/// cardinality. `#[allow(large_enum_variant)]` because the sizes reflect real
/// step shapes; boxing would add heap indirection on every deserialize and
/// dispatch without meaningful savings — no hot path holds dense
/// `Vec<FullRunnerStep>` in memory.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "type", rename_all = "snake_case")]
#[allow(clippy::large_enum_variant)]
pub enum FullRunnerStep {
    // ── 4 canonical variants (field structs reused from CanonicalStep) ──
    Command(CommandStep),
    Prompt(PromptStep),
    UiBridge(UiBridgeStep),
    Workflow(WorkflowStep),
    // ── 12 runner-specific variants ──
    CodeExecution(CodeExecutionStep),
    ExecutePlaybook(ExecutePlaybookStep),
    NativeAccessibility(NativeAccessibilityStep),
    RestartProcess(RestartProcessStep),
    SaveWorkflowArtifact(SaveWorkflowArtifactStep),
    WorkflowFixup(WorkflowFixupStep),
    UiBridgeDesignAudit(UiBridgeDesignAuditStep),
    UiBridgeVisualAssertion(UiBridgeVisualAssertionStep),
    VgaAutomate(VgaAutomateStep),
    WorkflowRef(WorkflowRefStep),
    DagCancel(DagCancelStep),
    DagApproval(DagApprovalStep),
    DagLoop(DagLoopStep),
}

impl FullRunnerStep {
    /// Return the wire `"type"` discriminator for this step variant.
    pub fn step_type(&self) -> &'static str {
        match self {
            Self::Command(_) => "command",
            Self::Prompt(_) => "prompt",
            Self::UiBridge(_) => "ui_bridge",
            Self::Workflow(_) => "workflow",
            Self::CodeExecution(_) => "code_execution",
            Self::ExecutePlaybook(_) => "execute_playbook",
            Self::NativeAccessibility(_) => "native_accessibility",
            Self::RestartProcess(_) => "restart_process",
            Self::SaveWorkflowArtifact(_) => "save_workflow_artifact",
            Self::WorkflowFixup(_) => "workflow_fixup",
            Self::UiBridgeDesignAudit(_) => "ui_bridge_design_audit",
            Self::UiBridgeVisualAssertion(_) => "ui_bridge_visual_assertion",
            Self::VgaAutomate(_) => "vga_automate",
            Self::WorkflowRef(_) => "workflow_ref",
            Self::DagCancel(_) => "dag_cancel",
            Self::DagApproval(_) => "dag_approval",
            Self::DagLoop(_) => "dag_loop",
        }
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::{json, Value};

    // ─── helpers ─────────────────────────────────────────────────────────────

    fn base(id: &str, name: &str) -> BaseStepFields {
        BaseStepFields {
            id: id.to_string(),
            name: name.to_string(),
            ..Default::default()
        }
    }

    fn roundtrip<T>(value: &T) -> T
    where
        T: Serialize + for<'de> Deserialize<'de>,
    {
        let json = serde_json::to_string(value).expect("serialize");
        serde_json::from_str(&json).expect("deserialize")
    }

    fn assert_type_tag(json: &Value, expected: &str) {
        assert_eq!(
            json.get("type").and_then(|t| t.as_str()),
            Some(expected),
            "expected type tag {:?}, got: {}",
            expected,
            json
        );
    }

    // ─── FullRunnerStep round-trips ───────────────────────────────────────────

    #[test]
    fn command_step_round_trip() {
        let step = FullRunnerStep::Command(CommandStep {
            base: base("s1", "Build"),
            phase: CommandStepPhase::Setup,
            mode: Some(CommandMode::Shell),
            command: Some("cargo build".into()),
            ..Default::default()
        });
        let json = serde_json::to_value(&step).unwrap();
        assert_type_tag(&json, "command");
        assert_eq!(json["mode"], "shell");
        assert_eq!(json["command"], "cargo build");
        assert_eq!(roundtrip(&step), step);
    }

    #[test]
    fn prompt_step_round_trip() {
        let step = FullRunnerStep::Prompt(PromptStep {
            base: base("p1", "Agentic Task"),
            phase: PromptStepPhase::Agentic,
            content: "Fix the failing tests.".into(),
            ..Default::default()
        });
        let json = serde_json::to_value(&step).unwrap();
        assert_type_tag(&json, "prompt");
        assert_eq!(json["phase"], "agentic");
        assert_eq!(json["content"], "Fix the failing tests.");
        assert_eq!(roundtrip(&step), step);
    }

    #[test]
    fn ui_bridge_step_round_trip() {
        let step = FullRunnerStep::UiBridge(UiBridgeStep {
            base: base("u1", "Navigate"),
            phase: UiBridgeStepPhase::Setup,
            action: UiBridgeAction::Navigate,
            url: Some("http://localhost:3000".into()),
            ..Default::default()
        });
        let json = serde_json::to_value(&step).unwrap();
        assert_type_tag(&json, "ui_bridge");
        assert_eq!(json["action"], "navigate");
        assert_eq!(json["url"], "http://localhost:3000");
        assert_eq!(roundtrip(&step), step);
    }

    #[test]
    fn workflow_step_round_trip() {
        let step = FullRunnerStep::Workflow(WorkflowStep {
            base: base("w1", "Inner Workflow"),
            phase: WorkflowStepPhase::Setup,
            workflow_id: "wf-uuid-123".into(),
            workflow_name: "My Workflow".into(),
        });
        let json = serde_json::to_value(&step).unwrap();
        assert_type_tag(&json, "workflow");
        assert_eq!(json["workflowId"], "wf-uuid-123");
        assert_eq!(roundtrip(&step), step);
    }

    #[test]
    fn code_execution_step_round_trip() {
        let step = FullRunnerStep::CodeExecution(CodeExecutionStep {
            base: base("ce1", "Run Script"),
            code: Some("print('hello')".into()),
            sandbox_mode: Some("enforce".into()),
            ..Default::default()
        });
        let json = serde_json::to_value(&step).unwrap();
        assert_type_tag(&json, "code_execution");
        assert_eq!(json["code"], "print('hello')");
        assert_eq!(json["sandboxMode"], "enforce");
        assert_eq!(roundtrip(&step), step);
    }

    #[test]
    fn execute_playbook_step_round_trip() {
        let step = FullRunnerStep::ExecutePlaybook(ExecutePlaybookStep {
            base: base("ep1", "Drive State Machine"),
            playbook_path: Some("/artifacts/playbook.md".into()),
            ..Default::default()
        });
        let json = serde_json::to_value(&step).unwrap();
        assert_type_tag(&json, "execute_playbook");
        assert_eq!(json["playbookPath"], "/artifacts/playbook.md");
        assert_eq!(roundtrip(&step), step);
    }

    #[test]
    fn native_accessibility_step_round_trip() {
        let step = FullRunnerStep::NativeAccessibility(NativeAccessibilityStep {
            base: base("na1", "Click Button"),
            action: A11yAction::Click,
            target: Some("Desktop".into()),
            ref_id: Some("@e5".into()),
            ..Default::default()
        });
        let json = serde_json::to_value(&step).unwrap();
        assert_type_tag(&json, "native_accessibility");
        assert_eq!(json["action"], "click");
        assert_eq!(json["refId"], "@e5");
        assert_eq!(roundtrip(&step), step);
    }

    #[test]
    fn restart_process_step_round_trip() {
        let step = FullRunnerStep::RestartProcess(RestartProcessStep {
            base: base("rp1", "Restart Backend"),
            restart_process_name: Some("backend".into()),
            restart_wait_for_health: Some(true),
            ..Default::default()
        });
        let json = serde_json::to_value(&step).unwrap();
        assert_type_tag(&json, "restart_process");
        assert_eq!(json["restartProcessName"], "backend");
        assert_eq!(roundtrip(&step), step);
    }

    #[test]
    fn save_workflow_artifact_step_round_trip() {
        let step = FullRunnerStep::SaveWorkflowArtifact(SaveWorkflowArtifactStep {
            base: base("swa1", "Save Generated Workflow"),
            artifact_input_path: Some("{{artifact_dir}}/workflow.json".into()),
            artifact_capture_prompts: Some(true),
        });
        let json = serde_json::to_value(&step).unwrap();
        assert_type_tag(&json, "save_workflow_artifact");
        assert_eq!(json["artifactInputPath"], "{{artifact_dir}}/workflow.json");
        assert_eq!(roundtrip(&step), step);
    }

    #[test]
    fn workflow_fixup_step_round_trip() {
        let step = FullRunnerStep::WorkflowFixup(WorkflowFixupStep {
            base: base("wf1", "Auto-fix Workflow"),
            fixup_input_path: Some("{{artifact_dir}}/workflow.json".into()),
            fixup_mode: Some(WorkflowFixupMode::Harden),
            ..Default::default()
        });
        let json = serde_json::to_value(&step).unwrap();
        assert_type_tag(&json, "workflow_fixup");
        assert_eq!(json["fixupMode"], "harden");
        assert_eq!(roundtrip(&step), step);
    }

    #[test]
    fn ui_bridge_design_audit_step_round_trip() {
        let step = FullRunnerStep::UiBridgeDesignAudit(UiBridgeDesignAuditStep {
            base: base("da1", "Design Audit"),
            timeout_seconds: Some(30),
        });
        let json = serde_json::to_value(&step).unwrap();
        assert_type_tag(&json, "ui_bridge_design_audit");
        assert_eq!(json["timeoutSeconds"], 30);
        assert_eq!(roundtrip(&step), step);
    }

    #[test]
    fn vga_automate_step_round_trip() {
        let step = FullRunnerStep::VgaAutomate(VgaAutomateStep {
            base: base("vga1", "Drive Notepad++"),
            state_machine_id: "11111111-1111-1111-1111-111111111111".into(),
            target_process: "notepad++.exe".into(),
            action_sequence: vec![
                VgaAction::WaitFor {
                    element_id: "22222222-2222-2222-2222-222222222222".into(),
                    timeout_ms: Some(30000),
                },
                VgaAction::Click {
                    element_id: "22222222-2222-2222-2222-222222222222".into(),
                    timeout_ms: None,
                },
                VgaAction::Type {
                    text: "hello world".into(),
                    element_id: None,
                    timeout_ms: Some(10000),
                },
            ],
            timeout_ms: Some(300000),
            r#async: Some(false),
        });
        let json = serde_json::to_value(&step).unwrap();
        assert_type_tag(&json, "vga_automate");
        assert_eq!(json["stateMachineId"], "11111111-1111-1111-1111-111111111111");
        assert_eq!(json["targetProcess"], "notepad++.exe");
        assert_eq!(json["actionSequence"][0]["kind"], "wait_for");
        assert_eq!(json["actionSequence"][1]["kind"], "click");
        assert_eq!(json["actionSequence"][2]["kind"], "type");
        assert_eq!(json["actionSequence"][2]["text"], "hello world");
        assert_eq!(json["timeoutMs"], 300000);
        assert_eq!(json["async"], false);
        assert_eq!(roundtrip(&step), step);
    }

    #[test]
    fn ui_bridge_visual_assertion_step_round_trip() {
        let step = FullRunnerStep::UiBridgeVisualAssertion(UiBridgeVisualAssertionStep {
            base: base("va1", "Assert Button Text"),
            visual_assertion_type: Some(VisualAssertionType::Text),
            visual_assertion_expected: Some("Submit".into()),
            ..Default::default()
        });
        let json = serde_json::to_value(&step).unwrap();
        assert_type_tag(&json, "ui_bridge_visual_assertion");
        assert_eq!(json["visualAssertionType"], "text");
        assert_eq!(roundtrip(&step), step);
    }

    #[test]
    fn workflow_ref_step_round_trip() {
        let step = FullRunnerStep::WorkflowRef(WorkflowRefStep {
            base: base("wr1", "Call Sub-Workflow"),
            workflow_id: Some("child-wf-uuid".into()),
            ref_inherit_model_overrides: Some(true),
            ..Default::default()
        });
        let json = serde_json::to_value(&step).unwrap();
        assert_type_tag(&json, "workflow_ref");
        assert_eq!(json["workflowId"], "child-wf-uuid");
        assert_eq!(roundtrip(&step), step);
    }

    #[test]
    fn dag_cancel_step_round_trip() {
        let step = FullRunnerStep::DagCancel(DagCancelStep {
            base: base("dc1", "Cancel on Failure"),
            cancel_reason: Some("Build failed".into()),
        });
        let json = serde_json::to_value(&step).unwrap();
        assert_type_tag(&json, "dag_cancel");
        assert_eq!(json["cancelReason"], "Build failed");
        assert_eq!(roundtrip(&step), step);
    }

    #[test]
    fn dag_approval_step_round_trip() {
        let step = FullRunnerStep::DagApproval(DagApprovalStep {
            base: base("da1", "Approval Gate"),
            approval_prompt: Some("Approve deployment?".into()),
            timeout_seconds: Some(3600),
        });
        let json = serde_json::to_value(&step).unwrap();
        assert_type_tag(&json, "dag_approval");
        assert_eq!(json["approvalPrompt"], "Approve deployment?");
        assert_eq!(roundtrip(&step), step);
    }

    #[test]
    fn dag_loop_step_round_trip() {
        let step = FullRunnerStep::DagLoop(DagLoopStep {
            base: base("dl1", "Retry Loop"),
            max_iterations: Some(5),
            loop_condition: Some("not_passing".into()),
        });
        let json = serde_json::to_value(&step).unwrap();
        assert_type_tag(&json, "dag_loop");
        assert_eq!(json["maxIterations"], 5);
        assert_eq!(roundtrip(&step), step);
    }

    // ─── Value-bridge tests: real-wire-format JSON → FullRunnerStep ───────────

    #[test]
    fn value_bridge_command_shell() {
        let raw = json!({
            "type": "command",
            "id": "abc",
            "name": "Build",
            "phase": "setup",
            "mode": "shell",
            "command": "npm install"
        });
        let step: FullRunnerStep = serde_json::from_value(raw.clone()).expect("deserialize");
        assert_eq!(step.step_type(), "command");
        let back = serde_json::to_value(&step).unwrap();
        assert_eq!(back["type"], "command");
        assert_eq!(back["command"], "npm install");
    }

    #[test]
    fn value_bridge_prompt_agentic() {
        let raw = json!({
            "type": "prompt",
            "id": "p1",
            "name": "Fix Tests",
            "phase": "agentic",
            "content": "Please fix the failing tests."
        });
        let step: FullRunnerStep = serde_json::from_value(raw).expect("deserialize");
        assert_eq!(step.step_type(), "prompt");
    }

    #[test]
    fn value_bridge_ui_bridge_navigate() {
        let raw = json!({
            "type": "ui_bridge",
            "id": "u1",
            "name": "Open Dashboard",
            "phase": "setup",
            "action": "navigate",
            "url": "http://localhost:3000/dashboard"
        });
        let step: FullRunnerStep = serde_json::from_value(raw).expect("deserialize");
        assert_eq!(step.step_type(), "ui_bridge");
    }

    #[test]
    fn value_bridge_code_execution() {
        let raw = json!({
            "type": "code_execution",
            "id": "ce1",
            "name": "Setup Data",
            "code": "import os; print(os.getcwd())"
        });
        let step: FullRunnerStep = serde_json::from_value(raw).expect("deserialize");
        assert_eq!(step.step_type(), "code_execution");
    }

    #[test]
    fn value_bridge_dag_approval() {
        let raw = json!({
            "type": "dag_approval",
            "id": "appr1",
            "name": "Deploy Approval",
            "approval_prompt": "Ready to deploy to production?"
        });
        let step: FullRunnerStep = serde_json::from_value(raw).expect("deserialize");
        assert_eq!(step.step_type(), "dag_approval");
    }

    // ─── Negative test: unknown type tag returns an error ─────────────────────

    #[test]
    fn unknown_type_tag_returns_error() {
        let raw = json!({
            "type": "totally_unknown_step_xyz",
            "id": "x1",
            "name": "Mystery Step"
        });
        let result: Result<FullRunnerStep, _> = serde_json::from_value(raw);
        assert!(
            result.is_err(),
            "Expected error for unknown type tag, got: {:?}",
            result
        );
    }

    // ─── CanonicalStep / UnifiedStep still work (non-regression) ─────────────

    #[test]
    fn canonical_step_non_regression() {
        let step = CanonicalStep::Command(CommandStep {
            base: base("c1", "Check"),
            phase: CommandStepPhase::Verification,
            mode: Some(CommandMode::Check),
            check_type: Some(CheckType::Lint),
            ..Default::default()
        });
        let json = serde_json::to_value(&step).unwrap();
        assert_type_tag(&json, "command");
        assert_eq!(json["mode"], "check");
    }

    #[test]
    fn unified_step_other_round_trips_verbatim() {
        let raw = json!({
            "type": "log_watch",
            "id": "lw1",
            "name": "Watch Logs",
            "custom_field": 42
        });
        let step: UnifiedStep = serde_json::from_value(raw.clone()).expect("deserialize");
        // Should fall through to Other
        assert!(matches!(&step, UnifiedStep::Other(_)));
        assert_eq!(step.step_type(), Some("log_watch"));
        let back = serde_json::to_value(&step).unwrap();
        assert_eq!(back, raw);
    }
}
