// SPDX-License-Identifier: MIT
//! The assertion evaluator's conformance fixture,
//! `tests/fixtures/vision_assertions_conformance.json`.
//!
//! Each case is one `ElementSnapshot` and one assertion, both in the wire form
//! this crate deserializes, with the outcome and code [`evaluate`] must reach.
//! This crate OWNS the fixture. The UI Bridge SDK's in-tab evaluator is a
//! TypeScript port of the snapshot-only kinds (plan
//! `2026-11-09-ui-bridge-relay-evaluates-vision-assertions-in-the-tab`): it
//! vendors this file with a drift check and must reproduce every case, so the
//! runner's `POST /vision/assert` and the SDK relay give the same answer for
//! the same element. A rule change therefore lands here first, with the cases
//! that pin it, and the port follows.
//!
//! The format is closed, because a second implementation reads it:
//!
//! ```text
//! { "version": 1, "generated_from": "qontinui-vision-core",
//!   "cases": [ { "id", "kind", "snapshot", "assertion",
//!                "expected": { "outcome", "code" } } ] }
//! ```
//!
//! - `kind` is the assertion's wire `type`, and is one of [`KINDS`].
//! - `code` is the result's machine-readable [`UnknownCode`] (snake_case), set
//!   iff `outcome` is `unknown`, and `null` (never absent) otherwise.
//! - Snapshots carry only the fields their kind reads, and every key they
//!   spell must survive this crate's own round trip unchanged
//!   ([`wire_subset_errors`]). serde IGNORES an unknown or misspelt key, so
//!   without that check a case could silently test something other than
//!   what it says.
//!
//! Every key of the envelope, of a case and of `expected` is required, and no
//! other key is accepted at those three levels, so a change to the shape is a
//! `version` bump the port is told about, never a silent drift.
//!
//! **Numbers, for a port.** `font_size_px` and `font_weight` are `f32` here,
//! and so is every comparison on them (the large-text thresholds) and the
//! vertical estimate `ceil(font_size_px * 1.5)`. A port whose numbers are
//! doubles must round to `f32` on both sides of each comparison and after the
//! multiplication (in JavaScript, `Math.fround`), or it disagrees within about
//! 1e-6 of a boundary: `10.666667 * 1.5` ceils to 16 in `f32` and to 17 in
//! `f64`. The cases stay clear of such values; the rule does not.
//!
//! **Geometry, for a port.** Rounding is scoped by kind (see `Element::bbox`).
//!
//! - `text_fits_container`: the comparison box is the border box rounded UP,
//!   the ceiling of the fractional `getBoundingClientRect()` width and height,
//!   and it is what a case's `bbox` `w` and `h` hold. `scroll_width_px` /
//!   `scroll_height_px` are the DOM's integer `scrollWidth` / `scrollHeight`,
//!   compared with it strictly, with no slack: content exactly as large as the
//!   box fits, and one pixel larger does not. The `*-ceiled-box-*` cases pin
//!   both sides of that boundary.
//! - The geometry kinds (`no_overlap`, `element_above`, `no_clipping`): `bbox`
//!   is the producer's best integer rounding of the fractional rect, and
//!   nothing requires a ceiling there. Rounding every box up would make three
//!   33.333 px flex items overlap; the `no-overlap-adjacent-flex-thirds-*` case
//!   is that row, rounded to the nearest pixel.

use std::collections::{BTreeMap, BTreeSet, HashSet};

use qontinui_vision_core::assertions::{
    evaluate, Assertion, AssertionOutcome, AssertionResult, EvalContext,
};
use qontinui_vision_core::{ElementSnapshot, UnknownCode};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

const FIXTURE: &str = include_str!("fixtures/vision_assertions_conformance.json");

/// The kinds the fixture covers: the five that read nothing but the
/// snapshot, which is what the SDK relay evaluates in the tab.
const KINDS: [&str; 5] = [
    "contrast_meets_wcag",
    "text_fits_container",
    "no_overlap",
    "element_above",
    "no_clipping",
];

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct Fixture {
    version: u32,
    generated_from: String,
    cases: Vec<Case>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct Case {
    id: String,
    kind: String,
    snapshot: Value,
    assertion: Value,
    expected: Expected,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct Expected {
    outcome: AssertionOutcome,
    /// Required, and nullable. An explicit `deserialize_with` turns off
    /// serde's implicit "absent `Option` is `None`", so a case that leaves
    /// the key out is refused rather than read as `null`.
    #[serde(deserialize_with = "Option::deserialize")]
    code: Option<UnknownCode>,
}

fn fixture() -> Fixture {
    serde_json::from_str(FIXTURE).expect("the conformance fixture parses in its closed format")
}

fn outcome_str(o: AssertionOutcome) -> &'static str {
    match o {
        AssertionOutcome::Passed => "passed",
        AssertionOutcome::Failed => "failed",
        AssertionOutcome::Unknown => "unknown",
    }
}

fn code_str(c: Option<UnknownCode>) -> &'static str {
    c.map(UnknownCode::as_str).unwrap_or("null")
}

/// Evaluates a case through the crate's public entry point, exactly as a
/// caller would: wire JSON in, [`evaluate`] over a snapshot-only context.
fn run(case: &Case) -> Result<AssertionResult, String> {
    let snapshot: ElementSnapshot = serde_json::from_value(case.snapshot.clone())
        .map_err(|e| format!("snapshot does not deserialize: {e}"))?;
    let assertion: Assertion = serde_json::from_value(case.assertion.clone())
        .map_err(|e| format!("assertion does not deserialize: {e}"))?;
    Ok(evaluate(
        &assertion,
        &EvalContext {
            snapshot: Some(&snapshot),
            ..Default::default()
        },
    ))
}

/// What this crate writes on the wire for `v`, read back as JSON.
///
/// Through the TEXT form on purpose: `serde_json::to_value` widens an `f32`
/// to the `f64` nearest it (`18.66` becomes `18.65999984741211`), while the
/// wire, and so every port reading this fixture, carries the `f32`'s shortest
/// form (`18.66`).
fn wire_of<T: Serialize>(v: &T) -> Value {
    let text = serde_json::to_string(v).expect("the value re-serializes");
    serde_json::from_str(&text).expect("the wire form parses")
}

/// Every key and value the fixture spells, checked against what this crate
/// re-serializes from it. Keys the fixture omits (a defaulted field) are
/// allowed; a key it adds, or a value the crate reads differently, is not.
fn wire_subset_errors(path: &str, fixture: &Value, roundtrip: &Value, out: &mut Vec<String>) {
    match (fixture, roundtrip) {
        (Value::Object(f), Value::Object(r)) => {
            for (k, fv) in f {
                match r.get(k) {
                    Some(rv) => wire_subset_errors(&format!("{path}.{k}"), fv, rv, out),
                    None => out.push(format!(
                        "{path}.{k} is not a key vision-core reads (it was dropped on the round trip)"
                    )),
                }
            }
        }
        (Value::Array(f), Value::Array(r)) if f.len() == r.len() => {
            for (i, (fv, rv)) in f.iter().zip(r).enumerate() {
                wire_subset_errors(&format!("{path}[{i}]"), fv, rv, out);
            }
        }
        // `24` and the `24.0` an f32 re-serializes to are the same value.
        (Value::Number(f), Value::Number(r)) if f.as_f64() == r.as_f64() => {}
        (f, r) if f == r => {}
        (f, r) => out.push(format!(
            "{path} is {f} in the fixture but {r} as vision-core reads it"
        )),
    }
}

fn is_kebab(id: &str) -> bool {
    !id.is_empty()
        && !id.starts_with('-')
        && !id.ends_with('-')
        && !id.contains("--")
        && id
            .chars()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-')
}

/// The whole contract: every case evaluates to its recorded outcome and code.
/// All mismatches are collected, so a red run names every broken case.
#[test]
fn vision_core_reproduces_every_conformance_case() {
    let fx = fixture();
    assert!(!fx.cases.is_empty(), "the fixture has no cases");
    let mut errors = Vec::new();
    for case in &fx.cases {
        let result = match run(case) {
            Ok(r) => r,
            Err(e) => {
                errors.push(format!("{}: {e}", case.id));
                continue;
            }
        };
        if result.outcome != case.expected.outcome || result.code != case.expected.code {
            errors.push(format!(
                "{}: expected {} (code {}), vision-core says {} (code {}): {}",
                case.id,
                outcome_str(case.expected.outcome),
                code_str(case.expected.code),
                outcome_str(result.outcome),
                code_str(result.code),
                result.detail.as_deref().unwrap_or("no detail"),
            ));
        } else if result.passed != (case.expected.outcome == AssertionOutcome::Passed) {
            // The gate bit a caller reads must agree with the outcome too.
            errors.push(format!("{}: `passed` disagrees with the outcome", case.id));
        }
    }
    assert!(
        errors.is_empty(),
        "{} of {} conformance case(s) disagree with vision-core:\n  {}",
        errors.len(),
        fx.cases.len(),
        errors.join("\n  ")
    );
}

/// The envelope and every case are in the closed format a second
/// implementation reads, and say what they claim to.
#[test]
fn every_case_is_well_formed() {
    let fx = fixture();
    assert_eq!(fx.version, 1, "a format change bumps `version`");
    assert_eq!(fx.generated_from, "qontinui-vision-core");

    let mut seen = HashSet::new();
    let mut errors = Vec::new();
    for case in &fx.cases {
        let id = &case.id;
        if !is_kebab(id) {
            errors.push(format!("{id}: the id is not kebab-case"));
        }
        if !seen.insert(id.as_str()) {
            errors.push(format!("{id}: the id is not unique"));
        }
        if !KINDS.contains(&case.kind.as_str()) {
            errors.push(format!(
                "{id}: kind {:?} is not one of {KINDS:?}",
                case.kind
            ));
        }
        if case.assertion.get("type").and_then(Value::as_str) != Some(case.kind.as_str()) {
            errors.push(format!(
                "{id}: kind {:?} is not the assertion's wire type {}",
                case.kind, case.assertion["type"]
            ));
        }
        let unknown = case.expected.outcome == AssertionOutcome::Unknown;
        if unknown != case.expected.code.is_some() {
            errors.push(format!(
                "{id}: `code` must be set iff the outcome is unknown (outcome {}, code {})",
                outcome_str(case.expected.outcome),
                code_str(case.expected.code)
            ));
        }

        match serde_json::from_value::<ElementSnapshot>(case.snapshot.clone()) {
            Ok(s) => wire_subset_errors(
                &format!("{id}: snapshot"),
                &case.snapshot,
                &wire_of(&s),
                &mut errors,
            ),
            Err(e) => errors.push(format!("{id}: snapshot does not deserialize: {e}")),
        }
        match serde_json::from_value::<Assertion>(case.assertion.clone()) {
            Ok(a) => wire_subset_errors(
                &format!("{id}: assertion"),
                &case.assertion,
                &wire_of(&a),
                &mut errors,
            ),
            Err(e) => errors.push(format!("{id}: assertion does not deserialize: {e}")),
        }
    }
    assert!(
        errors.is_empty(),
        "malformed case(s):\n  {}",
        errors.join("\n  ")
    );
}

/// Each kind is pinned in all three directions. A kind with only passing
/// cases would let a port that never fails, or never says unknown, conform.
#[test]
fn every_kind_has_a_passed_a_failed_and_an_unknown_case() {
    let fx = fixture();
    let mut by_kind: BTreeMap<&str, BTreeSet<&str>> = BTreeMap::new();
    for case in &fx.cases {
        by_kind
            .entry(case.kind.as_str())
            .or_default()
            .insert(outcome_str(case.expected.outcome));
    }
    for kind in KINDS {
        let outcomes = by_kind.get(kind).cloned().unwrap_or_default();
        for want in ["passed", "failed", "unknown"] {
            assert!(
                outcomes.contains(want),
                "{kind} has no {want} case (it has {outcomes:?})"
            );
        }
    }
}

/// The format is closed in fact, not only in the doc: an extra key at either
/// level, or a case that leaves `code` out, does not parse.
#[test]
fn the_fixture_format_refuses_what_it_does_not_define() {
    let good = json!({
        "version": 1,
        "generated_from": "qontinui-vision-core",
        "cases": [{
            "id": "x",
            "kind": "no_clipping",
            "snapshot": {"elements": []},
            "assertion": {"type": "no_clipping"},
            "expected": {"outcome": "unknown", "code": "input_missing"}
        }]
    });
    serde_json::from_value::<Fixture>(good.clone()).expect("a minimal well-formed fixture parses");

    let mut no_code = good.clone();
    no_code["cases"][0]["expected"]
        .as_object_mut()
        .expect("expected is an object")
        .remove("code");
    let mut case_key = good.clone();
    case_key["cases"][0]["note"] = json!("x");
    let mut expected_key = good.clone();
    expected_key["cases"][0]["expected"]["detail"] = json!("x");
    let mut envelope_key = good;
    envelope_key["extra"] = json!(1);

    for (what, value) in [
        ("`code` left out instead of null", no_code),
        ("an undefined key on a case", case_key),
        ("an undefined key on `expected`", expected_key),
        ("an undefined key on the envelope", envelope_key),
    ] {
        assert!(
            serde_json::from_value::<Fixture>(value).is_err(),
            "{what} must be refused"
        );
    }
}
