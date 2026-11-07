// SPDX-License-Identifier: MIT
//! The fold: many three-state verdicts into one.
//!
//! One rule, shared by every instrument that rolls verdicts up (vision-audit,
//! the runner's assert route, the SDK, Spec CI):
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
//! crates (and vendored by the TS twins), and the runner's schema-export test
//! compares the two copies' JSON Schemas, so a drift reddens a test. Collapse
//! the twin into one definition if the licence split or qontinui-types'
//! publication ever changes.

use schemars::JsonSchema;
use serde::de::{self, Deserializer};
use serde::{Deserialize, Serialize};

use super::{AssertionOutcome, AssertionResult};
use crate::observation::UnknownCode;

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

impl From<&AssertionResult> for Verdict {
    /// An [`AssertionResult`] keeps the same invariant (`code` iff unknown,
    /// refused on the wire otherwise). Should a hand-built result ever break
    /// it, an unknown without a code is folded as `input_missing` rather than
    /// dropped — it stays unknown either way.
    fn from(r: &AssertionResult) -> Self {
        match r.outcome {
            AssertionOutcome::Passed => Self::passed(),
            AssertionOutcome::Failed => Self::failed(),
            AssertionOutcome::Unknown => {
                Self::unknown(r.code.unwrap_or(UnknownCode::InputMissing))
            }
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

/// [`fold_outcomes`] over assertion results directly.
pub fn fold_results(results: &[AssertionResult]) -> FoldedOutcome {
    let verdicts: Vec<Verdict> = results.iter().map(Verdict::from).collect();
    fold_outcomes(&verdicts)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

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
