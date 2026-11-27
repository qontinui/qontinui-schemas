// SPDX-License-Identifier: MIT
//! The fold: many three-state verdicts into one.
//!
//! One rule for every instrument that rolls verdicts up. vision-audit folds
//! through it here; the runner, the SDK and Spec CI move onto the same rule in
//! the later phases of plan
//! `2026-11-06-no-verification-instrument-reads-an-absence-as-a-pass` and its
//! sibling plans:
//!
//! 1. any [`AssertionOutcome::Failed`] → `failed`;
//! 2. else any [`AssertionOutcome::Unknown`] → `unknown`;
//! 3. else (at least one verdict, all [`AssertionOutcome::Passed`]) → `passed`;
//! 4. an EMPTY input → `unknown` with [`UnknownCode::NoResults`].
//!
//! Rule 4 is the reason this module exists. "Nothing failed" over nothing at
//! all is not a pass: a suite that checked nothing has told nobody anything,
//! and reading its silence as green is the defect plan
//! `2026-11-06-no-verification-instrument-reads-an-absence-as-a-pass` closes.
//!
//! **This is one of two Rust copies.** `qontinui_types::outcome` carries the
//! same types and the same rule; this crate is MIT and takes no dependency on
//! the AGPL `qontinui-types`, so the definitions are twinned rather than
//! shared. The golden vectors in `tests/vectors/outcome-fold/` are run by BOTH
//! crates (the SDK's TS twin is to vendor them too, plan Phase 2), so a
//! behavioural drift reddens a test, and `qontinui-types`'
//! `tests/outcome_fold_vectors.rs` requires the
//! region between the `twin:` markers to be byte-identical in both copies, so
//! a serde or schema drift does too. Collapse the twin into one definition
//! if the licence split or qontinui-types' publication ever changes.

use schemars::JsonSchema;
use serde::de::{self, Deserializer};
use serde::{Deserialize, Serialize};

use super::{AssertionOutcome, AssertionResult};
use crate::observation::UnknownCode;

// --- twin: begin (kept byte-identical with the other copy; tests/outcome_fold_vectors.rs checks) ---
/// One verdict as the fold reads it: an outcome, and — iff that outcome is
/// [`AssertionOutcome::Unknown`] — the typed reason.
///
/// The invariant is enforced by construction (the fields are private and the
/// constructors are the only way in) and on the wire (deserialization refuses
/// an `unknown` without a `code`, and a `code` on any other outcome), so the
/// fold can always report WHY an unknown is unknown.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
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

/// The schema states the same invariant the deserializer enforces, so a
/// binding generated from it (TS, Python) refuses what Rust refuses: a
/// `passed`/`failed` verdict with no `code`, or an `unknown` one with a
/// required `code`, and no other fields.
impl JsonSchema for Verdict {
    fn schema_name() -> std::borrow::Cow<'static, str> {
        "Verdict".into()
    }

    fn json_schema(generator: &mut schemars::SchemaGenerator) -> schemars::Schema {
        let code = generator.subschema_for::<UnknownCode>();
        schemars::json_schema!({
            "description": "One verdict: an outcome, and a typed `code` iff that outcome is `unknown`.",
            "oneOf": [
                {
                    "type": "object",
                    "properties": {
                        "outcome": { "type": "string", "enum": ["passed", "failed"] },
                        "code": { "type": "null" }
                    },
                    "required": ["outcome"],
                    "additionalProperties": false
                },
                {
                    "type": "object",
                    "properties": {
                        "outcome": { "type": "string", "const": "unknown" },
                        "code": code
                    },
                    "required": ["outcome", "code"],
                    "additionalProperties": false
                }
            ]
        })
    }
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
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
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct OutcomeCounts {
    pub passed: usize,
    pub failed: usize,
    pub unknown: usize,
}

impl OutcomeCounts {
    /// Every verdict counted. Saturates at `usize::MAX` instead of
    /// overflowing: an `OutcomeCounts` can be deserialized on its own, outside
    /// the checks a [`FoldedOutcome`] runs.
    pub const fn total(&self) -> usize {
        self.passed
            .saturating_add(self.failed)
            .saturating_add(self.unknown)
    }
}

/// The fold's answer. Serialized camelCase (`outcomeCounts`, `unknownCodes`),
/// the names the ui-bridge SDK already ships.
///
/// Only [`fold_outcomes`] builds one, and deserialization re-checks what the
/// fold guarantees: the outcome is the one the counts imply, an empty fold is
/// `unknown` with exactly `[no_results]`, and codes are present iff something
/// was unknown. So a `FoldedOutcome` that Rust deserializes off the wire
/// cannot say `passed` over zero results, or `passed` beside a failure. The
/// generated TS and Python bindings carry the shape only, not these checks.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
#[schemars(deny_unknown_fields)]
pub struct FoldedOutcome {
    outcome: AssertionOutcome,
    outcome_counts: OutcomeCounts,
    /// The distinct codes of the `unknown` inputs, in first-seen order —
    /// `[no_results]` for an empty input, `[]` when nothing was unknown. Kept
    /// on a `failed` fold too: the failure decides the outcome, and the
    /// unknowns beside it are still work someone owes.
    unknown_codes: Vec<UnknownCode>,
}

impl FoldedOutcome {
    pub const fn outcome(&self) -> AssertionOutcome {
        self.outcome
    }

    pub const fn outcome_counts(&self) -> OutcomeCounts {
        self.outcome_counts
    }

    pub fn unknown_codes(&self) -> &[UnknownCode] {
        &self.unknown_codes
    }

    /// The gate bit: only a `passed` fold is green.
    pub fn passed(&self) -> bool {
        self.outcome.passed()
    }

    /// Why `self` is not a value [`fold_outcomes`] could have returned, if it
    /// is not.
    fn inconsistency(&self) -> Option<String> {
        let c = self.outcome_counts;
        // Counts come off the wire, so their sum is checked rather than
        // trusted: a wrapped total must not pass for an empty fold.
        let Some(total) = c
            .passed
            .checked_add(c.failed)
            .and_then(|t| t.checked_add(c.unknown))
        else {
            return Some("`outcomeCounts` overflows when summed".into());
        };
        if total == 0 {
            return (self.outcome != AssertionOutcome::Unknown
                || self.unknown_codes != [UnknownCode::NoResults])
            .then(|| "an empty fold is `unknown` with exactly `[no_results]`".to_string());
        }
        let implied = if c.failed > 0 {
            AssertionOutcome::Failed
        } else if c.unknown > 0 {
            AssertionOutcome::Unknown
        } else {
            AssertionOutcome::Passed
        };
        if self.outcome != implied {
            return Some(format!(
                "outcome {:?} contradicts the counts {c:?}, which fold to {implied:?}",
                self.outcome
            ));
        }
        if (c.unknown > 0) == self.unknown_codes().is_empty() {
            return Some("`unknownCodes` must be non-empty iff `outcomeCounts.unknown` > 0".into());
        }
        if self.unknown_codes.len() > c.unknown {
            return Some("more distinct `unknownCodes` than unknown verdicts".into());
        }
        let mut seen = Vec::with_capacity(self.unknown_codes.len());
        for code in &self.unknown_codes {
            if seen.contains(code) {
                return Some(format!("`unknownCodes` repeats {code}"));
            }
            seen.push(*code);
        }
        None
    }
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct FoldedOutcomeWire {
    outcome: AssertionOutcome,
    outcome_counts: OutcomeCounts,
    unknown_codes: Vec<UnknownCode>,
}

impl<'de> Deserialize<'de> for FoldedOutcome {
    fn deserialize<D: Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        let w = FoldedOutcomeWire::deserialize(d)?;
        let f = Self {
            outcome: w.outcome,
            outcome_counts: w.outcome_counts,
            unknown_codes: w.unknown_codes,
        };
        match f.inconsistency() {
            None => Ok(f),
            Some(why) => Err(de::Error::custom(format!("not a fold result: {why}"))),
        }
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
// --- twin: end ---

impl From<&AssertionResult> for Verdict {
    /// An [`AssertionResult`] keeps the same invariant (`code` iff unknown,
    /// refused on the wire otherwise). Should a hand-built result ever break
    /// it, an unknown without a code is folded as `not_reported` (the producer
    /// gave no reason) rather than dropped — it stays unknown either way.
    fn from(r: &AssertionResult) -> Self {
        match r.outcome {
            AssertionOutcome::Passed => Self::passed(),
            AssertionOutcome::Failed => Self::failed(),
            AssertionOutcome::Unknown => Self::unknown(r.code.unwrap_or(UnknownCode::NotReported)),
        }
    }
}

/// [`fold_outcomes`] over assertion results directly.
pub fn fold_results(results: &[AssertionResult]) -> FoldedOutcome {
    let verdicts: Vec<Verdict> = results.iter().map(Verdict::from).collect();
    fold_outcomes(&verdicts)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    /// `total()` saturates rather than overflowing: an `OutcomeCounts`
    /// deserializes on its own, outside `FoldedOutcome`'s checked sum, so a
    /// wire value can carry counts whose plain sum would overflow (a debug
    /// build panics on that, a release build wraps).
    #[test]
    fn outcome_counts_total_saturates_instead_of_overflowing() {
        let counts = OutcomeCounts {
            passed: usize::MAX,
            failed: 1,
            unknown: 0,
        };
        assert_eq!(counts.total(), usize::MAX);
        let counts = OutcomeCounts {
            passed: 1,
            failed: usize::MAX,
            unknown: usize::MAX,
        };
        assert_eq!(counts.total(), usize::MAX);
    }

    #[test]
    fn empty_is_unknown_no_results_never_passed() {
        let f = fold_outcomes(&[]);
        assert_eq!(f.outcome(), AssertionOutcome::Unknown);
        assert_eq!(f.unknown_codes(), vec![UnknownCode::NoResults]);
        assert_eq!(f.outcome_counts().total(), 0);
        assert!(!f.passed());
    }

    #[test]
    fn failed_beats_unknown_beats_passed() {
        let f = fold_outcomes(&[
            Verdict::unknown(UnknownCode::InputMissing),
            Verdict::failed(),
            Verdict::passed(),
        ]);
        assert_eq!(f.outcome(), AssertionOutcome::Failed);
        assert_eq!(f.unknown_codes(), vec![UnknownCode::InputMissing]);
        let f = fold_outcomes(&[Verdict::passed(), Verdict::unknown(UnknownCode::StaleInput)]);
        assert_eq!(f.outcome(), AssertionOutcome::Unknown);
        let f = fold_outcomes(&[Verdict::passed(), Verdict::passed()]);
        assert_eq!(f.outcome(), AssertionOutcome::Passed);
        assert!(f.unknown_codes().is_empty());
    }

    #[test]
    fn unknown_codes_are_distinct_in_first_seen_order() {
        let f = fold_outcomes(&[
            Verdict::unknown(UnknownCode::StaleInput),
            Verdict::unknown(UnknownCode::InputMissing),
            Verdict::unknown(UnknownCode::StaleInput),
        ]);
        assert_eq!(
            f.unknown_codes(),
            vec![UnknownCode::StaleInput, UnknownCode::InputMissing]
        );
        assert_eq!(f.outcome_counts().unknown, 3);
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
        assert_eq!(
            serde_json::to_value(Verdict::passed()).unwrap(),
            json!({"outcome": "passed"})
        );
    }

    #[test]
    fn folded_outcome_wire_refuses_a_pass_the_fold_could_not_give() {
        let read = |v| serde_json::from_value::<FoldedOutcome>(v);
        // Passed over nothing: the absence this plan exists to stop reading as green.
        assert!(read(json!({"outcome": "passed",
            "outcomeCounts": {"passed": 0, "failed": 0, "unknown": 0}, "unknownCodes": []}))
        .is_err());
        // Passed beside a failure.
        assert!(read(json!({"outcome": "passed",
            "outcomeCounts": {"passed": 1, "failed": 3, "unknown": 0}, "unknownCodes": []}))
        .is_err());
        // Unknown with no reason.
        assert!(read(json!({"outcome": "unknown",
            "outcomeCounts": {"passed": 0, "failed": 0, "unknown": 1}, "unknownCodes": []}))
        .is_err());
        // Counts whose sum overflows are refused, not wrapped to an empty fold.
        assert!(read(json!({"outcome": "unknown",
            "outcomeCounts": {"passed": usize::MAX, "failed": 1, "unknown": 0},
            "unknownCodes": ["no_results"]}))
        .is_err());
        // An unknown field in the counts is refused.
        assert!(read(json!({"outcome": "passed",
            "outcomeCounts": {"passed": 1, "failed": 0, "unknown": 0, "skipped": 5},
            "unknownCodes": []}))
        .is_err());
        // Every fold result round-trips.
        for f in [
            fold_outcomes(&[]),
            fold_outcomes(&[Verdict::passed()]),
            fold_outcomes(&[Verdict::failed(), Verdict::unknown(UnknownCode::StaleInput)]),
            fold_outcomes(&[
                Verdict::unknown(UnknownCode::NotReported),
                Verdict::passed(),
            ]),
        ] {
            assert_eq!(read(serde_json::to_value(&f).unwrap()).unwrap(), f);
        }
    }

    #[test]
    fn verdict_schema_states_the_code_iff_unknown_invariant() {
        let schema = serde_json::to_value(schemars::schema_for!(Verdict)).unwrap();
        let arms = schema["oneOf"].as_array().expect("oneOf");
        assert_eq!(arms.len(), 2);
        assert_eq!(arms[0]["required"], json!(["outcome"]));
        assert_eq!(arms[1]["required"], json!(["outcome", "code"]));
        assert!(arms
            .iter()
            .all(|a| a["additionalProperties"] == json!(false)));
        assert!(
            arms[1]["properties"]["code"]["$ref"]
                .as_str()
                .is_some_and(|r| r.ends_with("/UnknownCode")),
            "the unknown arm's code must reference UnknownCode: {}",
            arms[1]
        );
        let folded = serde_json::to_value(schemars::schema_for!(FoldedOutcome)).unwrap();
        assert_eq!(folded["additionalProperties"], json!(false));
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
