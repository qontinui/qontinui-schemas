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

fn vec_is_empty<T>(v: &Vec<T>) -> bool {
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
pub struct ApiVariableExtraction {
    /// Variable name to bind the extracted value to.
    pub variable_name: String,
    /// JSONPath expression used to extract the value.
    pub json_path: String,
    /// Default value if the path does not resolve.
    #[serde(default, skip_serializing_if = "Option::is_none")]
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
pub struct ApiAssertion {
    /// Kind of assertion.
    #[serde(rename = "type")]
    pub assertion_type: ApiAssertionType,
    /// Expected value. The TS source allows either a string or number, so
    /// this field stays as `serde_json::Value` on the wire.
    pub expected: serde_json::Value,
    /// JSONPath for `json_path` assertions.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub json_path: Option<String>,
    /// Header name for `header` assertions.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub header_name: Option<String>,
    /// Comparison operator; defaults to `equals` on the consumer side.
    #[serde(default, skip_serializing_if = "Option::is_none")]
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
pub struct RetrySpec {
    /// Number of retry attempts (`0` = no retries).
    pub count: u32,
    /// Delay between retries in milliseconds.
    pub delay_ms: u64,
}

// ============================================================================
// BaseStepFields — shared across all four variants via #[serde(flatten)]
// ============================================================================

/// Shared fields common to every canonical step variant.
///
/// Flattened into each step struct via `#[serde(flatten)]` so the wire shape
/// stays flat (no nested `"base": { … }` envelope).
#[derive(Debug, Clone, Default, Serialize, Deserialize, JsonSchema)]
pub struct BaseStepFields {
    /// Unique identifier for the step.
    pub id: String,
    /// Display name for the step.
    pub name: String,
    /// If `Some(true)`, a console-error signal from the UI fails this step.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub fail_on_console_errors: Option<bool>,
    /// Named input bindings evaluated at step entry.
    #[serde(default, skip_serializing_if = "hashmap_is_empty")]
    pub inputs: HashMap<String, String>,
    /// Extractions published to subsequent steps.
    #[serde(default, skip_serializing_if = "hashmap_is_empty")]
    pub extract: HashMap<String, String>,
    /// IDs of other steps that must complete first.
    #[serde(default, skip_serializing_if = "vec_is_empty")]
    pub depends_on: Vec<String>,
    /// Whether this step is required (default: `true` on consumer side).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub required: Option<bool>,
    /// Per-step retry configuration.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub retry: Option<RetrySpec>,
    /// Acceptance criterion IDs verified by this step.
    #[serde(default, skip_serializing_if = "vec_is_empty")]
    pub criterion_ids: Vec<String>,
    /// Verification depth category.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub verification_category: Option<VerificationCategoryKind>,
    /// Provenance of this step when generated from a skill template.
    ///
    /// Typed as `serde_json::Value` here to avoid pulling the `skill`
    /// dependency chain into this module; the TS side re-imports the typed
    /// `SkillOrigin` after regeneration.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub skill_origin: Option<serde_json::Value>,
}

// ============================================================================
// Phase enums
// ============================================================================

/// Phases in which a [`CommandStep`] may appear.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum CommandStepPhase {
    Setup,
    Verification,
    Completion,
}

impl Default for CommandStepPhase {
    fn default() -> Self {
        Self::Setup
    }
}

/// Phases in which a [`PromptStep`] may appear.
///
/// Prompt steps are the only variant that may appear in the agentic phase.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum PromptStepPhase {
    Setup,
    Verification,
    Agentic,
    Completion,
}

impl Default for PromptStepPhase {
    fn default() -> Self {
        Self::Setup
    }
}

/// Phases in which a [`UiBridgeStep`] may appear.
///
/// UI-bridge interactions only run in deterministic phases — never inside
/// the agentic loop (where the AI drives steps directly via prompts).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum UiBridgeStepPhase {
    Setup,
    Verification,
    Completion,
}

impl Default for UiBridgeStepPhase {
    fn default() -> Self {
        Self::Setup
    }
}

/// Phases in which a [`WorkflowStep`] may appear.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum WorkflowStepPhase {
    Setup,
    Verification,
    Completion,
}

impl Default for WorkflowStepPhase {
    fn default() -> Self {
        Self::Setup
    }
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
#[derive(Debug, Clone, Default, Serialize, Deserialize, JsonSchema)]
pub struct CommandStep {
    #[serde(flatten)]
    pub base: BaseStepFields,
    /// Phase in which the step appears.
    pub phase: CommandStepPhase,
    /// Execution mode — which sub-kind of command step this is.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub mode: Option<CommandMode>,
    /// Shell command line (for `shell` mode).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub command: Option<String>,
    /// Working directory for the command.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub working_directory: Option<String>,
    /// Timeout in seconds.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub timeout_seconds: Option<u64>,
    /// Whether non-zero exit status fails the step.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub fail_on_error: Option<bool>,
    /// Re-run this step on every verification-agentic iteration.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub run_on_subsequent_iterations: Option<bool>,
    /// Saved shell command template ID.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub shell_command_id: Option<String>,
    /// Kind of deterministic check (for `check` / `check_group` modes).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub check_type: Option<CheckType>,
    /// Tool identifier (e.g., `eslint`, `ruff`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tool: Option<String>,
    /// Saved check definition ID.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub check_id: Option<String>,
    /// Path to the check's config file.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub config_path: Option<String>,
    /// Whether to auto-fix during the check.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub auto_fix: Option<bool>,
    /// Fail the step on warnings in addition to errors.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub fail_on_warning: Option<bool>,
    /// Repository selector for repository-targeted steps.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub repository: Option<String>,
    /// Name of a workflow to invoke.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub workflow_name: Option<String>,
    /// Branch selector for repository-targeted steps.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub branch: Option<String>,
    /// Whether the caller waits for the workflow to complete.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub wait_for_completion: Option<bool>,
    /// Saved check-group ID.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub check_group_id: Option<String>,
    /// Test runner kind.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub test_type: Option<TestType>,
    /// Saved test ID.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub test_id: Option<String>,
    /// Inline code body (e.g., Python snippet).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub code: Option<String>,
    /// Saved script ID.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub script_id: Option<String>,
    /// Inline script contents.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub script_content: Option<String>,
    /// Target URL for navigation-style tests.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub target_url: Option<String>,
    /// Saved fused-script ID.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub fused_script_id: Option<String>,
    /// Execution mode for Playwright tests.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub execution_mode: Option<PlaywrightExecutionMode>,
}

// ============================================================================
// PromptStep
// ============================================================================

/// AI task instructions (prompt).
#[derive(Debug, Clone, Default, Serialize, Deserialize, JsonSchema)]
pub struct PromptStep {
    #[serde(flatten)]
    pub base: BaseStepFields,
    /// Phase in which the step appears.
    pub phase: PromptStepPhase,
    /// Prompt body.
    pub content: String,
    /// Saved prompt ID (when the body is a reference).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub prompt_id: Option<String>,
    /// AI provider override.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub provider: Option<String>,
    /// Model override.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub model: Option<String>,
    /// Marks this prompt as the summary step at the end of completion.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub is_summary_step: Option<bool>,
}

// ============================================================================
// UiBridgeStep
// ============================================================================

/// UI Bridge action kind.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum UiBridgeAction {
    Navigate,
    Execute,
    Assert,
    Snapshot,
    Compare,
    SnapshotAssert,
    ActionPlan,
}

impl Default for UiBridgeAction {
    fn default() -> Self {
        Self::Navigate
    }
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
#[derive(Debug, Clone, Default, Serialize, Deserialize, JsonSchema)]
pub struct UiBridgeStep {
    #[serde(flatten)]
    pub base: BaseStepFields,
    /// Phase in which the step appears.
    pub phase: UiBridgeStepPhase,
    /// Action kind.
    pub action: UiBridgeAction,
    /// Navigation URL (for `navigate`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
    /// Free-form instruction text (for `execute`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub instruction: Option<String>,
    /// Target selector or element ID.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub target: Option<String>,
    /// Assertion kind (for `assert`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub assert_type: Option<UiBridgeAssertType>,
    /// Expected value for assertions.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub expected: Option<String>,
    /// Timeout in milliseconds.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub timeout_ms: Option<u64>,
    /// Comparison mode (for `compare` / `snapshot_assert`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub comparison_mode: Option<UiBridgeComparisonMode>,
    /// Reference snapshot ID (for `compare` / `snapshot_assert`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reference_snapshot_id: Option<String>,
    /// Severity threshold (for `compare` / `snapshot_assert`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub severity_threshold: Option<UiBridgeSeverity>,
    /// Snapshot target — `"control"`, `"sdk"`, or `"proxy:PORT"`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ui_bridge_snapshot_target: Option<String>,
    /// Structured action plan (for `action_plan`).
    ///
    /// Typed as `serde_json::Value` here to avoid pulling the `action-plan`
    /// module into this crate; the TS side re-imports the typed `ActionPlan`
    /// after regeneration.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub action_plan: Option<serde_json::Value>,
}

// ============================================================================
// WorkflowStep
// ============================================================================

/// Run a saved workflow inline (composition).
#[derive(Debug, Clone, Default, Serialize, Deserialize, JsonSchema)]
pub struct WorkflowStep {
    #[serde(flatten)]
    pub base: BaseStepFields,
    /// Phase in which the step appears.
    pub phase: WorkflowStepPhase,
    /// ID of the saved workflow to run.
    pub workflow_id: String,
    /// Display name of the saved workflow (denormalized for UI).
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
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "type", rename_all = "snake_case")]
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
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(untagged)]
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
