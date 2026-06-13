//! Golden-fixture regression anchor for the functional-spec-contract keystone
//! (`2026-06-13-functional-spec-contract.md`, Phase 3).
//!
//! Hand-authored Functional Spec + expected Completeness Verdict for one known page
//! (the `invoices` page — the plan's canonical example). Every downstream plan
//! (#1 app-gen, #2 backend-gen, #3 comprehension) keeps these green. The two fixtures
//! are deliberately coupled: the verdict's `provenance_mix` MUST equal an independent
//! tally of the spec's nodes, so a change to either side that breaks the rubric's
//! accounting split fails here.

use qontinui_types::completeness_verdict::{CompletenessVerdict, ProvenanceMix};
use qontinui_types::functional_spec::{FunctionalSpec, SpecProvenance};

const SPEC_JSON: &str = include_str!("fixtures/functional_spec/invoices.functional_spec.json");
const VERDICT_JSON: &str =
    include_str!("fixtures/functional_spec/invoices.completeness_verdict.json");

/// Independently tally the spec's nodes by provenance, applying the documented v0
/// rule that each `ui_states` / `navigation` node counts as `Observed` (they carry
/// `IrProvenance`, not `SpecProvenance`).
fn tally(spec: &FunctionalSpec) -> ProvenanceMix {
    let mut mix = ProvenanceMix::default();
    for e in &spec.entities {
        mix.add(e.confidence);
        for f in &e.fields {
            mix.add(f.confidence);
        }
        for r in &e.relationships {
            mix.add(r.confidence);
        }
    }
    for op in &spec.operations {
        mix.add(op.confidence);
        if let Some(eff) = &op.effect {
            mix.add(eff.confidence);
        }
        for inp in &op.inputs {
            if let Some(v) = &inp.validation {
                mix.add(v.confidence);
            }
        }
    }
    if let Some(auth) = &spec.auth {
        mix.add(auth.confidence);
        for role in &auth.roles {
            mix.add(role.confidence);
        }
    }
    for _ in &spec.ui_states {
        mix.add(SpecProvenance::Observed);
    }
    for _ in &spec.navigation {
        mix.add(SpecProvenance::Observed);
    }
    mix
}

#[test]
fn spec_fixture_parses_and_reuses_ir_types() {
    let spec: FunctionalSpec = serde_json::from_str(SPEC_JSON).expect("spec fixture parses");
    assert_eq!(spec.spec_version, "0");
    // ui_states / navigation deserialized as the IR types (compile-time guaranteed
    // by the field types); assert the page's shape.
    assert_eq!(spec.ui_states.len(), 2);
    assert_eq!(spec.navigation.len(), 1);
    assert_eq!(spec.ui_states[0].id, "invoice-list");
    assert_eq!(spec.navigation[0].activate_states, vec!["invoice-detail"]);
    // Round-trips back to camelCase without loss.
    let round: FunctionalSpec =
        serde_json::from_str(&serde_json::to_string(&spec).unwrap()).unwrap();
    assert_eq!(round, spec);
}

#[test]
fn verdict_fixture_parses_and_is_internally_consistent() {
    let verdict: CompletenessVerdict =
        serde_json::from_str(VERDICT_JSON).expect("verdict fixture parses");
    assert_eq!(verdict.spec_version, "0");
    assert!(
        verdict.coverage_is_consistent(),
        "verdict.coverage ({}) must equal coverage_from(mix, counted_gaps)",
        verdict.coverage
    );
    assert_eq!(verdict.counted_gaps(), 2);
    // Every gap is an Observed/Inferred node (Assumed nodes never gap).
    for g in &verdict.gaps {
        assert_ne!(g.node_provenance, SpecProvenance::Assumed);
    }
}

#[test]
fn verdict_provenance_mix_matches_spec_tally() {
    let spec: FunctionalSpec = serde_json::from_str(SPEC_JSON).unwrap();
    let verdict: CompletenessVerdict = serde_json::from_str(VERDICT_JSON).unwrap();
    assert_eq!(
        tally(&spec),
        verdict.provenance_mix,
        "the verdict's provenance mix must equal an independent tally of the spec"
    );
    // The plan's worked example: 7 observed + 4 inferred + 1 assumed.
    assert_eq!(verdict.provenance_mix.observed, 7);
    assert_eq!(verdict.provenance_mix.inferred, 4);
    assert_eq!(verdict.provenance_mix.assumed, 1);
    assert_eq!(verdict.provenance_mix.denominator(), 11);
}

#[test]
fn section_mixes_sum_to_overall() {
    let verdict: CompletenessVerdict = serde_json::from_str(VERDICT_JSON).unwrap();
    let mut summed = ProvenanceMix::default();
    for s in &verdict.sections {
        summed.observed += s.provenance_mix.observed;
        summed.inferred += s.provenance_mix.inferred;
        summed.assumed += s.provenance_mix.assumed;
    }
    assert_eq!(summed, verdict.provenance_mix, "per-section mixes must sum to the overall mix");
}

#[test]
fn assumed_fill_reported_separately_from_coverage() {
    let verdict: CompletenessVerdict = serde_json::from_str(VERDICT_JSON).unwrap();
    // The single assumed node (createInvoice.effect) was filled.
    assert!((verdict.assumed_fill_rate - 1.0).abs() < 1e-9);
    // ...and it is NOT folded into the headline coverage (9/11, not 10/12).
    assert!((verdict.coverage - 9.0 / 11.0).abs() < 1e-9);
}
