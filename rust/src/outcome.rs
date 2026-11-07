//! The shared verdict vocabulary and its fold.
//!
//! `passed | failed | unknown` ([`AssertionOutcome`]) plus a typed reason for
//! every `unknown` ([`UnknownCode`]), and the one rule that rolls many verdicts
//! into one ([`fold_outcomes`]):
//!
//! 1. any `failed` → `failed`;
//! 2. else any `unknown` → `unknown`;
//! 3. else (at least one verdict, all `passed`) → `passed`;
//! 4. an EMPTY input → `unknown` with [`UnknownCode::NoResults`] — "nothing
//!    failed" over nothing at all is not a pass.
//!
//! **This is one of two Rust copies.** `qontinui-vision-core` (MIT) carries
//! the same types and rule in `observation.rs` / `assertions/fold.rs`; it
//! takes no dependency on this AGPL crate (`decision_record/ui-bridge-is-mit-licensed`),
//! and this crate cannot depend on an unpublished one (it is itself published
//! to crates.io from `publish-rust.yml`), so the definitions are twinned. Both
//! copies run the golden vectors in `rust-vision-core/tests/vectors/outcome-fold/`
//! (`tests/outcome_fold_vectors.rs` here), both pin [`UnknownCode::ALL`] to the
//! `UB-OBS-*` registry codes of [`crate::ui_bridge_diagnostics`], and the
//! runner's schema export compares their JSON Schemas. Collapse the twin into
//! one definition when either the licence split or the publication changes.
//! Plan `2026-11-06-no-verification-instrument-reads-an-absence-as-a-pass` D2.

use std::fmt;

use schemars::JsonSchema;
use serde::de::{self, Deserializer};
use serde::{Deserialize, Serialize};

// ---------------------------------------------------------------------------
// UnknownCode
// ---------------------------------------------------------------------------

/// WHY a producer could not answer. A closed vocabulary: an agent branches on
/// the code, and the human-readable [`UnknownInfo::detail`] is never the only
/// carrier of the reason.
///
/// Adding a member is a wire change for every consumer (runner, ui-bridge,
/// ui-bridge-mcp); the diagnostic-code registry name of each is
/// [`Self::diagnostic_code`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum UnknownCode {
    /// The bridge/app could not be reached.
    AppUnreachable,
    /// The running build does not serve this capability (e.g. an SDK route
    /// that needs a runner).
    CapabilityUnavailableInBuild,
    /// The producer ran and errored (transport, HTTP status, timeout).
    ProducerFailed,
    /// The producer did not run (e.g. zero registered components — nothing
    /// to look at yet).
    ProducerNotRun,
    /// A required input was absent (no `elements` key, no bbox, no frame).
    InputMissing,
    /// Observations existed but all fell under the caller's confidence floor.
    BelowConfidenceFloor,
    /// A model reply could not be parsed or validated.
    ModelReplyUnparseable,
    /// The question needs more than one captured frame, and only one (or
    /// none) was supplied.
    NeedsMultiFrameInput,
    /// The input is known to be stale.
    StaleInput,
    /// A negative journey answer ("not reachable", "no way out") was asked
    /// over a part of the graph with open frontier entries — affordances
    /// seen and never activated — so it cannot be settled yet.
    FrontierOpen,
    /// The store the answer is read from does not exist in this database
    /// (e.g. an embedded DB that predates the journey tables).
    LedgerSchemaAbsent,
    /// The store exists but writes to it are failing, so it stopped growing
    /// and an answer over it would describe a stale graph as a finished one.
    LedgerWriteFailing,
    /// The store holds no observation the question could be answered from
    /// (an empty ledger, or a node it has never seen).
    NoObservations,
    /// An action's destination was never observed: a second action arrived
    /// before any snapshot closed the first.
    ToNodeUnobserved,
    /// The page's runtime does not support the `PerformanceObserver` entry
    /// type the measurement needs.
    PerformanceObserverUnsupported,
    /// The awaited event did not occur before the timeout, so whether it
    /// would have occurred is unknown.
    NoEventBeforeTimeout,
    /// Nothing to judge: zero results or assertions were folded, so the
    /// verdict is unknown — never passed.
    NoResults,
    /// No usable expectation to judge against (assertion criteria that do not
    /// parse, an empty `activateStates`).
    NoExpectation,
    /// Deliberately not attempted, because performing the check would be a
    /// destructive action.
    NotAttemptedDestructive,
    /// Deliberately not attempted, because performing the check would be a
    /// write.
    NotAttemptedWrite,
    /// A policy withheld the attempt (e.g. `stopOnFailure` after an earlier
    /// failure, an effects allow-list).
    NotAttemptedByPolicy,
    /// The producer ran but reported no result for this item.
    NotReported,
    /// The hit-test looked and cannot tell whether a pointer reaches the
    /// target (a null hit, a shadow host on the target's composed path, a
    /// cross-origin frame, a target computing `pointer-events: none`).
    HitTestUnmeasurable,
}

impl UnknownCode {
    /// Every member, in declaration order. The closed set, so a test can
    /// pin it against the wire contract rather than against itself.
    pub const ALL: [UnknownCode; 23] = [
        Self::AppUnreachable,
        Self::CapabilityUnavailableInBuild,
        Self::ProducerFailed,
        Self::ProducerNotRun,
        Self::InputMissing,
        Self::BelowConfidenceFloor,
        Self::ModelReplyUnparseable,
        Self::NeedsMultiFrameInput,
        Self::StaleInput,
        Self::FrontierOpen,
        Self::LedgerSchemaAbsent,
        Self::LedgerWriteFailing,
        Self::NoObservations,
        Self::ToNodeUnobserved,
        Self::PerformanceObserverUnsupported,
        Self::NoEventBeforeTimeout,
        Self::NoResults,
        Self::NoExpectation,
        Self::NotAttemptedDestructive,
        Self::NotAttemptedWrite,
        Self::NotAttemptedByPolicy,
        Self::NotReported,
        Self::HitTestUnmeasurable,
    ];

    /// The snake_case wire value.
    pub fn as_str(self) -> &'static str {
        match self {
            Self::AppUnreachable => "app_unreachable",
            Self::CapabilityUnavailableInBuild => "capability_unavailable_in_build",
            Self::ProducerFailed => "producer_failed",
            Self::ProducerNotRun => "producer_not_run",
            Self::InputMissing => "input_missing",
            Self::BelowConfidenceFloor => "below_confidence_floor",
            Self::ModelReplyUnparseable => "model_reply_unparseable",
            Self::NeedsMultiFrameInput => "needs_multi_frame_input",
            Self::StaleInput => "stale_input",
            Self::FrontierOpen => "frontier_open",
            Self::LedgerSchemaAbsent => "ledger_schema_absent",
            Self::LedgerWriteFailing => "ledger_write_failing",
            Self::NoObservations => "no_observations",
            Self::ToNodeUnobserved => "to_node_unobserved",
            Self::PerformanceObserverUnsupported => "performance_observer_unsupported",
            Self::NoEventBeforeTimeout => "no_event_before_timeout",
            Self::NoResults => "no_results",
            Self::NoExpectation => "no_expectation",
            Self::NotAttemptedDestructive => "not_attempted_destructive",
            Self::NotAttemptedWrite => "not_attempted_write",
            Self::NotAttemptedByPolicy => "not_attempted_by_policy",
            Self::NotReported => "not_reported",
            Self::HitTestUnmeasurable => "hit_test_unmeasurable",
        }
    }

    /// The ui-bridge diagnostic-registry name: `UB-OBS-<CODE-IN-UPPER-KEBAB>`,
    /// e.g. `UB-OBS-INPUT-MISSING`.
    pub fn diagnostic_code(self) -> String {
        format!(
            "UB-OBS-{}",
            self.as_str().to_ascii_uppercase().replace('_', "-")
        )
    }
}

impl fmt::Display for UnknownCode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

// ---------------------------------------------------------------------------
// AssertionOutcome
// ---------------------------------------------------------------------------

/// Which of the THREE things happened to an assertion.
///
/// `passed: bool` alone cannot say them: it collapses "evaluated, and the
/// expectation was violated" together with "could not be evaluated at all".
/// Those are different facts and call for different action — the first is a
/// bug in the page, the second is a gap in the snapshot (or in the caller's
/// inputs), and only the first should send anyone looking at the UI.
///
/// The bit is kept alongside as the GATE, and every consumer that reads it
/// keeps its current behaviour: [`AssertionOutcome::Unknown`] is
/// `passed = false`. That is deliberate and is the safe direction — a
/// vacuous pass over an un-evaluated assertion is a silent false negative
/// for exactly the bug the assertion was written to catch.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum AssertionOutcome {
    /// The assertion was evaluated against real inputs and HELD.
    Passed,
    /// The assertion was evaluated against real inputs and was VIOLATED.
    /// A verdict was reached; the page is wrong.
    Failed,
    /// The assertion could NOT be evaluated — an input it needs was absent
    /// (no snapshot, an operand missing from the snapshot, a measurement
    /// field the producer does not populate, no registered baseline, no OCR
    /// blocks). Nothing is known about whether the expectation holds.
    ///
    /// This is NOT a weaker failure. Reading it as one sends a reader
    /// looking for a layout bug that no evidence points at; reading it as a
    /// pass hides one that no evidence would have ruled out.
    Unknown,
}

impl AssertionOutcome {
    /// The gate bit. Only [`Self::Passed`] is green — see the type doc for
    /// why `Unknown` deliberately is not.
    pub fn passed(self) -> bool {
        matches!(self, Self::Passed)
    }
    /// True when no verdict was reached. Callers that want to report
    /// coverage ("3 passed, 1 failed, 2 not evaluated") read this rather
    /// than subtracting counts.
    pub fn is_unknown(self) -> bool {
        matches!(self, Self::Unknown)
    }
}

// ---------------------------------------------------------------------------
// The fold
// ---------------------------------------------------------------------------

/// One verdict as the fold reads it: an outcome, and — iff that outcome is
/// [`AssertionOutcome::Unknown`] — the typed reason.
///
/// The invariant is enforced by construction (the fields are private and the
/// constructors are the only way in) and on the wire (deserialization refuses
/// an `unknown` without a `code`, and a `code` on any other outcome), so the
/// fold can always report WHY an unknown is unknown.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct Verdict {
    outcome: AssertionOutcome,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    code: Option<UnknownCode>,
}

impl Verdict {
    pub const fn passed() -> Self {
        Self {
            outcome: AssertionOutcome::Passed,
            code: None,
        }
    }

    pub const fn failed() -> Self {
        Self {
            outcome: AssertionOutcome::Failed,
            code: None,
        }
    }

    pub const fn unknown(code: UnknownCode) -> Self {
        Self {
            outcome: AssertionOutcome::Unknown,
            code: Some(code),
        }
    }

    pub const fn outcome(&self) -> AssertionOutcome {
        self.outcome
    }

    /// The reason, present iff [`Self::outcome`] is `Unknown`.
    pub const fn code(&self) -> Option<UnknownCode> {
        self.code
    }
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct VerdictWire {
    outcome: AssertionOutcome,
    #[serde(default)]
    code: Option<UnknownCode>,
}

impl<'de> Deserialize<'de> for Verdict {
    fn deserialize<D: Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        let w = VerdictWire::deserialize(d)?;
        match (w.outcome, w.code) {
            (AssertionOutcome::Unknown, Some(code)) => Ok(Self::unknown(code)),
            (AssertionOutcome::Unknown, None) => Err(de::Error::custom(
                "an `unknown` verdict must carry a `code` saying why",
            )),
            (AssertionOutcome::Passed, None) => Ok(Self::passed()),
            (AssertionOutcome::Failed, None) => Ok(Self::failed()),
            (outcome, Some(code)) => Err(de::Error::custom(format!(
                "`code` ({code}) is only carried by an `unknown` verdict, not `{}`",
                serde_json::to_value(outcome)
                    .ok()
                    .and_then(|v| v.as_str().map(str::to_string))
                    .unwrap_or_default()
            ))),
        }
    }
}

/// How many of the folded verdicts landed in each state.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct OutcomeCounts {
    pub passed: usize,
    pub failed: usize,
    pub unknown: usize,
}

impl OutcomeCounts {
    pub const fn total(&self) -> usize {
        self.passed + self.failed + self.unknown
    }
}

/// The fold's answer. Serialized camelCase (`outcomeCounts`, `unknownCodes`),
/// the names the ui-bridge SDK already ships.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct FoldedOutcome {
    pub outcome: AssertionOutcome,
    pub outcome_counts: OutcomeCounts,
    /// The distinct codes of the `unknown` inputs, in first-seen order —
    /// `[no_results]` for an empty input, `[]` when nothing was unknown. Kept
    /// on a `failed` fold too: the failure decides the outcome, and the
    /// unknowns beside it are still work someone owes.
    pub unknown_codes: Vec<UnknownCode>,
}

impl FoldedOutcome {
    /// The gate bit: only a `passed` fold is green.
    pub fn passed(&self) -> bool {
        self.outcome.passed()
    }
}

/// Fold many verdicts into one — see the module doc for the rule.
pub fn fold_outcomes(verdicts: &[Verdict]) -> FoldedOutcome {
    if verdicts.is_empty() {
        return FoldedOutcome {
            outcome: AssertionOutcome::Unknown,
            outcome_counts: OutcomeCounts::default(),
            unknown_codes: vec![UnknownCode::NoResults],
        };
    }
    let mut counts = OutcomeCounts::default();
    let mut unknown_codes: Vec<UnknownCode> = Vec::new();
    for v in verdicts {
        match v.outcome {
            AssertionOutcome::Passed => counts.passed += 1,
            AssertionOutcome::Failed => counts.failed += 1,
            AssertionOutcome::Unknown => {
                counts.unknown += 1;
                if let Some(code) = v.code {
                    if !unknown_codes.contains(&code) {
                        unknown_codes.push(code);
                    }
                }
            }
        }
    }
    let outcome = if counts.failed > 0 {
        AssertionOutcome::Failed
    } else if counts.unknown > 0 {
        AssertionOutcome::Unknown
    } else {
        AssertionOutcome::Passed
    };
    FoldedOutcome {
        outcome,
        outcome_counts: counts,
        unknown_codes,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use std::collections::BTreeSet;

    #[test]
    fn unknown_code_is_exactly_the_contract_set() {
        let wire: Vec<Value> = UnknownCode::ALL
            .iter()
            .map(|c| serde_json::to_value(c).unwrap())
            .collect();
        assert_eq!(
            wire,
            vec![
                json!("app_unreachable"),
                json!("capability_unavailable_in_build"),
                json!("producer_failed"),
                json!("producer_not_run"),
                json!("input_missing"),
                json!("below_confidence_floor"),
                json!("model_reply_unparseable"),
                json!("needs_multi_frame_input"),
                json!("stale_input"),
                json!("frontier_open"),
                json!("ledger_schema_absent"),
                json!("ledger_write_failing"),
                json!("no_observations"),
                json!("to_node_unobserved"),
                json!("performance_observer_unsupported"),
                json!("no_event_before_timeout"),
                json!("no_results"),
                json!("no_expectation"),
                json!("not_attempted_destructive"),
                json!("not_attempted_write"),
                json!("not_attempted_by_policy"),
                json!("not_reported"),
                json!("hit_test_unmeasurable"),
            ]
        );
        for c in UnknownCode::ALL {
            // `as_str` and serde agree, and the value round-trips.
            assert_eq!(serde_json::to_value(c).unwrap(), json!(c.as_str()));
            let back: UnknownCode = serde_json::from_value(json!(c.as_str())).unwrap();
            assert_eq!(back, c);
        }
        assert!(serde_json::from_value::<UnknownCode>(json!("healthy")).is_err());
        assert_eq!(
            UnknownCode::CapabilityUnavailableInBuild.diagnostic_code(),
            "UB-OBS-CAPABILITY-UNAVAILABLE-IN-BUILD"
        );
        assert_eq!(
            UnknownCode::InputMissing.diagnostic_code(),
            "UB-OBS-INPUT-MISSING"
        );
        assert_eq!(
            UnknownCode::PerformanceObserverUnsupported.diagnostic_code(),
            "UB-OBS-PERFORMANCE-OBSERVER-UNSUPPORTED"
        );
    }

    /// [`UnknownCode::ALL`] and the `UB-OBS-*` members of the generated
    /// registry mirror name the same set, in both directions — as
    /// `rust-vision-core/tests/unknown_code_registry.rs` pins vision-core's
    /// copy. Read from the mirror's source text because the generated enum
    /// carries no member list.
    #[test]
    fn every_unknown_code_is_a_registry_code_and_vice_versa() {
        let text = include_str!("ui_bridge_diagnostics.rs");
        let registry: BTreeSet<String> = text
            .lines()
            .filter_map(|line| {
                let (_, rhs) = line.split_once("=> \"UB-OBS-")?;
                Some(format!("UB-OBS-{}", rhs.split('"').next()?))
            })
            .collect();
        assert!(registry.len() >= 9, "parsed only {} UB-OBS codes", registry.len());
        let envelope: BTreeSet<String> =
            UnknownCode::ALL.iter().map(|c| c.diagnostic_code()).collect();
        let only_envelope: Vec<_> = envelope.difference(&registry).collect();
        let only_registry: Vec<_> = registry.difference(&envelope).collect();
        assert!(
            only_envelope.is_empty() && only_registry.is_empty(),
            "UnknownCode without a registry entry: {only_envelope:?}; \
             UB-OBS registry entry without an UnknownCode: {only_registry:?}"
        );
        for c in UnknownCode::ALL {
            let parsed: crate::ui_bridge_diagnostics::UiBridgeErrorCode =
                c.diagnostic_code().parse().expect("registry parses its own code");
            assert_eq!(parsed.as_str(), c.diagnostic_code());
        }
    }

    #[test]
    fn empty_is_unknown_no_results_never_passed() {
        let f = fold_outcomes(&[]);
        assert_eq!(f.outcome, AssertionOutcome::Unknown);
        assert_eq!(f.unknown_codes, vec![UnknownCode::NoResults]);
        assert_eq!(f.outcome_counts.total(), 0);
        assert!(!f.passed());
    }

    #[test]
    fn failed_beats_unknown_beats_passed() {
        let f = fold_outcomes(&[
            Verdict::unknown(UnknownCode::InputMissing),
            Verdict::failed(),
            Verdict::passed(),
        ]);
        assert_eq!(f.outcome, AssertionOutcome::Failed);
        assert_eq!(f.unknown_codes, vec![UnknownCode::InputMissing]);
        let f = fold_outcomes(&[Verdict::passed(), Verdict::unknown(UnknownCode::StaleInput)]);
        assert_eq!(f.outcome, AssertionOutcome::Unknown);
        let f = fold_outcomes(&[Verdict::passed(), Verdict::passed()]);
        assert_eq!(f.outcome, AssertionOutcome::Passed);
        assert!(f.unknown_codes.is_empty());
    }

    #[test]
    fn unknown_codes_are_distinct_in_first_seen_order() {
        let f = fold_outcomes(&[
            Verdict::unknown(UnknownCode::StaleInput),
            Verdict::unknown(UnknownCode::InputMissing),
            Verdict::unknown(UnknownCode::StaleInput),
        ]);
        assert_eq!(
            f.unknown_codes,
            vec![UnknownCode::StaleInput, UnknownCode::InputMissing]
        );
        assert_eq!(f.outcome_counts.unknown, 3);
    }

    #[test]
    fn verdict_wire_refuses_a_broken_invariant() {
        assert!(serde_json::from_value::<Verdict>(json!({"outcome": "unknown"})).is_err());
        assert!(serde_json::from_value::<Verdict>(
            json!({"outcome": "passed", "code": "input_missing"})
        )
        .is_err());
        let v: Verdict =
            serde_json::from_value(json!({"outcome": "unknown", "code": "no_expectation"}))
                .unwrap();
        assert_eq!(v, Verdict::unknown(UnknownCode::NoExpectation));
        assert_eq!(serde_json::to_value(Verdict::passed()).unwrap(), json!({"outcome": "passed"}));
    }

    #[test]
    fn folded_outcome_wire_is_camel_case() {
        let v = serde_json::to_value(fold_outcomes(&[])).unwrap();
        assert_eq!(
            v,
            json!({
                "outcome": "unknown",
                "outcomeCounts": {"passed": 0, "failed": 0, "unknown": 0},
                "unknownCodes": ["no_results"]
            })
        );
    }
}
