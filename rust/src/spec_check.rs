//! Spec-Check v1 wire types.
//!
//! DTOs for the Spec-Check evaluation pipeline (Plan 01 foundation,
//! Spec-Check v1 redesign). These types cross HTTP, persist as JSONB, and
//! generate TS + Python bindings via the standard schemars pipeline. They
//! are the public API surface consumed by:
//!
//! - The matcher crate (`qontinui-runner/crates/spec-check/`, Plan 02) —
//!   `evaluate(snapshot, spec) -> SpecCheckResult`.
//! - HTTP / MCP / workflow-step adapters (Plan 03) — accept
//!   `SpecCheckStepConfig` / `SpecCheckPolicy`, return `SpecCheckResult`
//!   and `PolicyEvaluation`.
//! - Distinctness validator (Plan 04) — surfaces `SpecValidation` on
//!   `GET /spec/list`.
//! - Flywheel auto-promotion gate (Plan 05) — compares
//!   `SpecCheckResult.spec_content_hash` between landed and pending IRs.
//! - Observability (Plan 06) — persists results / policy evaluations in
//!   `workflow_verification_phase_results.details` JSONB with indexable
//!   expressions on `result_schema_version`, `match_outcome`, etc.
//!
//! ## Wire-format notes
//!
//! - All structs serialize as `camelCase` via `#[serde(rename_all = "camelCase")]`.
//! - `#[schemars(deny_unknown_fields)]` is set on every struct so callers
//!   cannot silently slip unrecognized fields past the schema.
//! - Unit enums (`MatchOutcome`, `Confidence`, `MissReason`, `PolicyStatus`)
//!   serialize as `snake_case` strings.
//! - Internally-tagged enums (`AssertionOutcome`, `ConjunctRule`) carry
//!   their discriminator inline; field names use `snake_case` to match
//!   the design context's policy DSL.
//!
//! ## `Confidence` registry naming
//!
//! `Confidence` collides with `qontinui_types::verification::Confidence` in
//! the cross-repo schema registry. Per `schema_export.rs` convention (see
//! the `ExecutionMatchLocation` precedent), the spec-check variant is
//! registered as `SpecCheckConfidence` while the verification variant
//! keeps the bare `Confidence` name. Downstream consumers (Plan 02
//! matcher, Plan 03 adapters) reference the type as
//! `qontinui_types::spec_check::Confidence` directly in Rust; the TS /
//! Python bindings emit `SpecCheckConfidence`.

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

// ============================================================================
// Serde default helpers
// ============================================================================

fn default_result_schema_version() -> u32 {
    0
}

// ============================================================================
// Step 1 — Result + fingerprint + validation types
// ============================================================================

/// Result of evaluating one or more page specs against a bridge snapshot.
///
/// `result_schema_version` is the FIRST field per §5.15 — read-time
/// forward migration uses it to route legacy JSONB through
/// `result_migration`. Pre-versioned rows (written before v1 shipped)
/// deserialize as `0`.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
#[schemars(deny_unknown_fields)]
pub struct SpecCheckResult {
    /// v1 = 1. Missing on the wire (pre-versioned persisted rows) → 0.
    #[serde(default = "default_result_schema_version")]
    pub result_schema_version: u32,

    /// Caller-minted snapshot ID. Format: `"scs_" + ULID()`. See §5.8.
    pub snapshot_id: String,

    /// JCS-canonicalized SHA-256 of the raw snapshot payload, as
    /// `"sha256-<hex>"`. Available on adapter calls that went through
    /// `wrap_snapshot` (HTTP / MCP fresh-fetch path); `None` for in-process
    /// evaluator calls (validator, distinctness check, supplied-snapshot
    /// path that doesn't precompute it) which either don't have raw bytes
    /// to hash or chose to skip the JCS round-trip.
    ///
    /// Consumers that join spec-check results back to a stored snapshot
    /// blob should prefer this over `bridge_fingerprint.snapshot_timestamp`
    /// (timestamps collide; content hashes don't).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub snapshot_sha256: Option<String>,

    /// Hash of the spec IR document at evaluation time. `"sha256-<hex>"`.
    pub spec_content_hash: String,

    /// IR document `version` field (currently `"1.0"`).
    pub spec_version: String,

    /// Page being evaluated.
    pub page_id: String,

    /// One result per `IrState` in the spec.
    pub state_results: Vec<StateMatchResult>,

    /// Aggregate summary across all evaluated states.
    pub summary: SpecCheckSummary,

    /// App / route / bridge-version fingerprint at snapshot time. §5.8 G3.
    pub bridge_fingerprint: BridgeFingerprint,

    /// ISO-8601 UTC. When the spec was hashed and the snapshot indexed.
    pub evaluated_at: String,

    /// Soft signals — currently used for `Stale` from `SnapshotFetchError`.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub warnings: Vec<String>,
}

/// Aggregate summary across all evaluated states.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
#[schemars(deny_unknown_fields)]
pub struct SpecCheckSummary {
    /// Top-level pass / partial / no-match verdict.
    pub match_outcome: MatchOutcome,

    /// Mean of per-state match rates, weighted equally.
    pub overall_match_rate: f32,

    /// Counts of assertion failures bucketed by severity.
    pub severity_counts: AssertionSeverityCounts,

    /// The single state (if any) the matcher recommends the caller treat
    /// as "current". `None` when the matcher has no preference.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub recommended_state: Option<RecommendedState>,

    /// Diagnostic explaining why `recommended_state` is `None`. Set when
    /// the matcher *deliberately* withholds a recommendation (e.g. the
    /// spec failed distinctness validation per §5.12); absent otherwise
    /// (e.g. when every state simply scored below the confidence floor).
    /// Free-form for forward-compat; current values:
    /// `"spec_validation_failed"`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub recommendation_reason: Option<String>,
}

/// The single state the matcher recommends the caller treat as "current".
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
#[schemars(deny_unknown_fields)]
pub struct RecommendedState {
    /// The recommended state's identifier.
    pub state_id: String,

    /// Matcher confidence in the recommendation.
    pub confidence: Confidence,

    /// Free-form explanation (e.g. "highest match rate among all states").
    pub reason: String,
}

/// Top-level pass / partial / no-match verdict.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, JsonSchema, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
pub enum MatchOutcome {
    /// All critical assertions passed across all states.
    FullMatch,
    /// Some critical assertions failed; some passed.
    PartialMatch,
    /// No state's critical assertions passed.
    NoMatch,
}

/// Confidence level — high / medium / low.
///
/// Collides with `qontinui_types::verification::Confidence` in the
/// cross-repo schema registry; registered as `SpecCheckConfidence` in
/// `schema_export.rs`. See module docs.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, JsonSchema, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
#[schemars(title = "SpecCheckConfidence")]
pub enum Confidence {
    /// High confidence — strong evidence.
    High,
    /// Medium confidence — reasonable but not certain.
    Medium,
    /// Low confidence — speculative.
    Low,
}

/// Per-state evaluation result.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
#[schemars(deny_unknown_fields)]
pub struct StateMatchResult {
    /// Identifier of the IR state.
    pub state_id: String,

    /// Human-readable state name (snapshot of `IrState.name`).
    pub state_name: String,

    /// Fraction of assertions that passed (0.0..=1.0).
    pub match_rate: f32,

    /// One entry per IR assertion in the state.
    pub assertions: Vec<AssertionResult>,
}

/// Per-assertion evaluation result wrapper.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
#[schemars(deny_unknown_fields)]
pub struct AssertionResult {
    /// Identifier of the IR assertion.
    pub assertion_id: String,

    /// Snapshot of `IrAssertion.description`.
    pub description: String,

    /// Free-form severity string — `"critical"` / `"error"` / `"warning"`
    /// / `"info"`. Free-form so policy adapters can introduce new tiers
    /// without bumping the schema.
    pub severity: String,

    /// Free-form category — `"layout"`, `"a11y"`, etc.
    pub category: String,

    /// Pass / fail discriminator + payload.
    pub outcome: AssertionOutcome,
}

/// Internally-tagged on `status` — `{ "status": "pass", "matched": {...} }`
/// or `{ "status": "fail", "miss": {...} }`. Distinct from the
/// worker-output `{ "type": ..., "data": ... }` envelope used elsewhere
/// in the codebase.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "status", rename_all = "snake_case")]
pub enum AssertionOutcome {
    /// Assertion passed against the named matched element.
    Pass {
        /// The element that satisfied the assertion.
        matched: MatchedElement,
    },
    /// Assertion failed; the miss carries reason + near-match candidates.
    Fail {
        /// Diagnostic detail.
        miss: AssertionMiss,
    },
}

/// Identity of the bridge element that satisfied an assertion.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
#[schemars(deny_unknown_fields)]
pub struct MatchedElement {
    /// UI Bridge element identifier (if exposed).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub element_id: Option<String>,

    /// ARIA role of the matched element.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub role: Option<String>,

    /// Visible text of the matched element.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub text: Option<String>,

    /// CSS selector or DOM path identifying the element.
    pub path: String,
}

/// Top-level miss diagnostic for a failed assertion.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
#[schemars(deny_unknown_fields)]
pub struct AssertionMiss {
    /// Coarse-grained classification of why the match failed.
    pub reason: MissReason,

    /// Top-N near-match candidates, capped externally. Ordered by score
    /// descending.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub candidates: Vec<CandidateMiss>,
}

/// A near-match candidate — an element that scored above the cut-off but
/// didn't fully match the assertion criteria.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
#[schemars(deny_unknown_fields)]
pub struct CandidateMiss {
    /// UI Bridge element identifier (if exposed).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub element_id: Option<String>,

    /// ARIA role of the candidate.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub role: Option<String>,

    /// Visible text of the candidate.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub text: Option<String>,

    /// CSS selector or DOM path identifying the candidate.
    pub path: String,

    /// Per-field diffs explaining why the candidate didn't fully match.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub field_diffs: Vec<FieldDiff>,

    /// Aggregate match score (0.0..=1.0).
    pub score: f32,
}

/// Per-field diff between an assertion's expected criteria and a
/// candidate element's actual value.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
#[schemars(deny_unknown_fields)]
pub struct FieldDiff {
    /// Field name (`role`, `text`, `textContains`, etc.).
    pub field: String,

    /// Expected value from the IR assertion.
    pub expected: serde_json::Value,

    /// Actual value from the bridge element.
    pub actual: serde_json::Value,

    /// Field-level similarity score (0.0..=1.0).
    pub similarity: f32,
}

/// Coarse-grained classification of an assertion miss.
///
/// Per §5.3 — locked as a unit enum. Structured detail (per-field diffs,
/// near-match candidates) lives on `AssertionMiss` / `CandidateMiss`.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, JsonSchema, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
pub enum MissReason {
    /// No bridge element scored above the match threshold.
    NoCandidates,
    /// Best candidate's ARIA role differed from the expected role.
    RoleMismatch,
    /// Best candidate's text differed from the expected text.
    TextMismatch,
    /// Best candidate was found but not visible.
    VisibilityMismatch,
    /// Best candidate had a differing attribute (data-*, aria-*, etc.).
    AttributeMismatch,
    /// Multiple elements matched equally well; assertion expected a
    /// unique target.
    MultipleMatches,
}

/// Counts of assertion failures bucketed by severity.
#[derive(Debug, Clone, Default, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
#[schemars(deny_unknown_fields)]
pub struct AssertionSeverityCounts {
    /// Number of `critical` failures.
    #[serde(default)]
    pub critical: u32,

    /// Number of `error` failures.
    #[serde(default)]
    pub error: u32,

    /// Number of `warning` failures.
    #[serde(default)]
    pub warning: u32,

    /// Number of `info` failures.
    #[serde(default)]
    pub info: u32,
}

/// Fingerprint of the bridge snapshot's app/route/version context — §5.8 G3.
///
/// Apps may surface additional fingerprint fields in future; new fields
/// require a schema bump (`deny_unknown_fields` is set).
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
#[schemars(deny_unknown_fields)]
pub struct BridgeFingerprint {
    /// Stable application identifier (e.g. `"qontinui-web"`,
    /// `"qontinui-runner"`).
    pub app_id: String,

    /// Optional app version string (semver or git short-SHA).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub app_version: Option<String>,

    /// Optional route path / hash at snapshot time.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub route: Option<String>,

    /// Optional UI Bridge SDK version.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub bridge_version: Option<String>,

    /// ISO-8601 UTC timestamp when the snapshot was indexed.
    pub snapshot_timestamp: String,

    /// Number of elements visible in the snapshot.
    pub element_count: u32,
}

/// Per-page validation summary surfaced on `GET /spec/list`. §5.12 G2.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
#[schemars(deny_unknown_fields)]
pub struct SpecValidation {
    /// Page identifier.
    pub page_id: String,

    /// State IDs whose distinctness signal is empty (a state with no
    /// assertions that distinguish it from any other state).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub degenerate_state_ids: Vec<String>,

    /// Pairs of state IDs that share an identical normalized distinctness
    /// signal. Each inner array is sorted lexicographically so the same
    /// pair always serializes the same way regardless of detection order.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub indistinguishable_state_pairs: Vec<[String; 2]>,
}

// ============================================================================
// Step 2 — Policy + step-config types
// ============================================================================

/// Workflow-step variant payload — the persisted shape of a
/// `spec_check` step in a unified workflow.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
#[schemars(deny_unknown_fields)]
pub struct SpecCheckStepConfig {
    /// Page to evaluate.
    pub page_id: String,

    /// Policy that determines pass / fail.
    pub policy: SpecCheckPolicy,

    /// If `true`, the step fails when the target app isn't connected to
    /// the bridge.
    #[serde(default)]
    pub fail_when_no_app: bool,

    /// If `true`, the step fails when no spec exists for `page_id`.
    #[serde(default)]
    pub fail_when_no_spec: bool,

    /// Severities that should cause the step to fail when present in
    /// `severity_counts`. Empty means "all severities count as failure".
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub fail_on: Vec<String>,
}

/// AND-only policy — all conjuncts must hold for the policy to pass.
/// Per §5.7 there is no OR; compose multiple steps if you need
/// disjunction.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
#[schemars(deny_unknown_fields)]
pub struct SpecCheckPolicy {
    /// Conjuncts ANDed together.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub conjuncts: Vec<PolicyConjunct>,
}

/// One AND-conjunct: a named rule applied within a scope.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
#[schemars(deny_unknown_fields)]
pub struct PolicyConjunct {
    /// Human-readable name used in `ConjunctEvaluation.name`.
    pub name: String,

    /// Filter narrowing which assertions / states this conjunct applies
    /// to.
    pub scope: AssertionScope,

    /// The conjunct rule.
    pub rule: ConjunctRule,
}

/// Filter narrowing which assertions / states a policy conjunct applies
/// to. Empty fields (default) mean "everything"; `assertion_ids` uses an
/// explicit `Option` to distinguish "everything" (`None`) from "none"
/// (`Some([])`).
#[derive(Debug, Clone, Default, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
#[schemars(deny_unknown_fields)]
pub struct AssertionScope {
    /// Restrict to these state IDs. Empty = all states.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub states: Vec<String>,

    /// Restrict to these severity values. Empty = all severities.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub severities: Vec<String>,

    /// Restrict to these category values. Empty = all categories.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub categories: Vec<String>,

    /// Restrict to these group identifiers. Empty = all groups.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub groups: Vec<String>,

    /// Restrict to a specific set of assertion IDs. `None` = all
    /// assertions; `Some([])` = no assertions (a deliberately empty
    /// scope — useful for testing policies).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub assertion_ids: Option<Vec<String>>,
}

/// Internally-tagged on `kind`. AND-conjuncts only per §5.7 — no OR.
/// Unknown variants are rejected at the wire.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum ConjunctRule {
    /// All assertions in scope must pass.
    AllPass,
    /// At most `count` assertions in scope may fail.
    MaxFailures {
        /// Maximum allowed failures.
        count: u32,
    },
    /// Fraction of failures (failed / total in scope) must be strictly
    /// less than `rate`.
    FailureRateBelow {
        /// Threshold in [0.0, 1.0].
        rate: f32,
    },
    /// For every state in scope, its `match_rate` must be at least `rate`.
    StateMatchRateAtLeast {
        /// Threshold in [0.0, 1.0].
        rate: f32,
    },
    /// At least one state in scope must have `match_rate` at least `rate`.
    AnyStateMatchRateAtLeast {
        /// Threshold in [0.0, 1.0].
        rate: f32,
    },
    /// Top-level `match_outcome` must be at least the given outcome,
    /// ordered `NoMatch < PartialMatch < FullMatch`.
    MatchOutcomeAtLeast {
        /// Minimum acceptable outcome.
        outcome: MatchOutcome,
    },
}

/// Top-level wrapper for evaluating a `SpecCheckPolicy` against a
/// `SpecCheckResult`.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
#[schemars(deny_unknown_fields)]
pub struct PolicyEvaluation {
    /// Aggregate of all conjunct statuses — fails if any conjunct fails;
    /// indeterminate if any conjunct is indeterminate and none failed.
    pub overall_status: PolicyStatus,

    /// One result per conjunct in the policy, in policy declaration
    /// order.
    pub conjunct_results: Vec<ConjunctEvaluation>,
}

/// Per-conjunct evaluation result.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
#[schemars(deny_unknown_fields)]
pub struct ConjunctEvaluation {
    /// Snapshot of `PolicyConjunct.name`.
    pub name: String,

    /// Pass / fail / indeterminate.
    pub status: PolicyStatus,

    /// Free-form human-readable explanation (e.g.
    /// `"3 of 17 assertions failed (max 5)"`).
    pub evidence: String,
}

/// Pass / fail / indeterminate verdict for a policy or conjunct.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, JsonSchema, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
pub enum PolicyStatus {
    /// Policy / conjunct passed.
    Pass,
    /// Policy / conjunct failed.
    Fail,
    /// Could not be evaluated (e.g. empty scope, missing inputs).
    Indeterminate,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn conjunct_rule_round_trip_all_variants() {
        // Values chosen to be exactly representable in f32 so the
        // serialized JSON is byte-identical after a round trip (f32
        // widening to f64 in serde_json introduces rounding noise for
        // values like 0.1 that aren't dyadic fractions).
        let cases = vec![
            serde_json::json!({"kind": "all_pass"}),
            serde_json::json!({"kind": "max_failures", "count": 3}),
            serde_json::json!({"kind": "failure_rate_below", "rate": 0.125}),
            serde_json::json!({"kind": "state_match_rate_at_least", "rate": 0.5}),
            serde_json::json!({"kind": "any_state_match_rate_at_least", "rate": 0.75}),
            serde_json::json!({"kind": "match_outcome_at_least", "outcome": "partial_match"}),
        ];
        for input in cases {
            let rule: ConjunctRule = serde_json::from_value(input.clone()).expect("deserialize");
            let re_serialized = serde_json::to_value(&rule).expect("serialize");
            assert_eq!(re_serialized, input, "round-trip mismatch for {input}");
        }
    }
}
