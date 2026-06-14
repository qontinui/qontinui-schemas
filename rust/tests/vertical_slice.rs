//! Website→Mobile regeneration — **stubbed vertical slice** (anti-drift gate).
//! Plan: `2026-06-14-website-mobile-vertical-slice.md`.
//!
//! Proves the regeneration seam end-to-end against the real, frozen v0 contract
//! types: `FunctionalSpec → stub generators → evaluate_completeness →
//! CompletenessVerdict`. Two layers:
//!
//! - **Phase 1 (oracle):** `evaluate_completeness` reproduces the hand-authored
//!   `invoices` golden verdict from a constructed coverage evidence set. If the
//!   mechanical walk cannot reconstruct the frozen golden, the rubric model is wrong.
//! - **Phase 2 (slice):** a hand-authored `connect-runner` spec (one real Qontinui
//!   page) round-trips through stub app+backend generators to a 1.0 verdict, and a
//!   deliberately-dropped node is provably caught as a `CoverageGap`.
//!
//! Every downstream plan (#1 app-gen, #2 backend-gen, #3 comprehension) keeps the
//! Phase-2 tests green as its integration anchor.

use std::collections::{BTreeMap, BTreeSet};

use qontinui_types::completeness_eval::{
    enumerate_nodes, evaluate_completeness, CoverageEvidence, GapEvidence,
};
use qontinui_types::completeness_verdict::{CompletenessVerdict, GapReason, SpecSection};
use qontinui_types::functional_spec::FunctionalSpec;
use qontinui_types::priorities_profile::Profile;

const INVOICES_SPEC: &str = include_str!("fixtures/functional_spec/invoices.functional_spec.json");
const INVOICES_VERDICT: &str =
    include_str!("fixtures/functional_spec/invoices.completeness_verdict.json");
const CONNECT_SPEC: &str =
    include_str!("fixtures/functional_spec/connect-runner.functional_spec.json");
const CONNECT_PROFILE: &str = include_str!("fixtures/functional_spec/connect-runner.profile.json");

fn approx(a: f64, b: f64) -> bool {
    (a - b).abs() < 1e-9
}

// ===========================================================================
// Phase 1 — the oracle: reproduce the frozen invoices golden verdict
// ===========================================================================

#[test]
fn evaluate_completeness_reproduces_invoices_golden_verdict() {
    let spec: FunctionalSpec = serde_json::from_str(INVOICES_SPEC).expect("spec parses");
    let golden: CompletenessVerdict =
        serde_json::from_str(INVOICES_VERDICT).expect("golden verdict parses");

    // The verifier's evidence: 9 of 11 Observed/Inferred nodes covered; 2 gaps
    // (status not generated; admin-role behavior mismatch); the 1 assumed node filled.
    let covered: BTreeSet<String> = [
        "entities.Invoice",
        "entities.Invoice.fields.amount",
        "entities.Invoice.relationships.Customer",
        "operations.createInvoice",
        "operations.createInvoice.inputs.amount.validation",
        "uiStates.invoice-list",
        "uiStates.invoice-detail",
        "navigation.open-invoice-detail",
        "auth",
    ]
    .iter()
    .map(|s| s.to_string())
    .collect();

    let mut gaps = BTreeMap::new();
    gaps.insert(
        "entities.Invoice.fields.status".to_string(),
        GapEvidence {
            reason: GapReason::NotGenerated,
            detail: Some("generated model has no status enum column".to_string()),
        },
    );
    gaps.insert(
        "auth.roles.admin".to_string(),
        GapEvidence {
            reason: GapReason::BehaviorMismatch,
            detail: Some("admin-only route renders for all roles in the generated app".to_string()),
        },
    );

    let filled_assumed: BTreeSet<String> = ["operations.createInvoice.effect"]
        .iter()
        .map(|s| s.to_string())
        .collect();

    let evidence = CoverageEvidence {
        covered,
        gaps,
        filled_assumed,
    };
    let got = evaluate_completeness(&spec, &evidence, "2026-06-13T00:00:00Z");

    // Exact structure.
    assert_eq!(got.spec_version, golden.spec_version);
    assert_eq!(got.provenance_mix, golden.provenance_mix);
    assert_eq!(
        got.gaps, golden.gaps,
        "gap work-list (refs/sections/reasons/details) must match the golden"
    );
    assert_eq!(got.evaluated_at, golden.evaluated_at);
    assert!(got.staleness_seconds.is_none());
    assert!(got.ui_states_spec_check.is_none());

    // Floats within epsilon (accumulation order differs from the literal).
    assert!(
        approx(got.coverage, golden.coverage),
        "coverage {} vs golden {}",
        got.coverage,
        golden.coverage
    );
    assert!(approx(got.assumed_fill_rate, golden.assumed_fill_rate));
    assert!(
        approx(got.credibility, golden.credibility),
        "credibility {} vs golden {}",
        got.credibility,
        golden.credibility
    );

    // Per-section breakdown, in canonical order.
    assert_eq!(got.sections.len(), golden.sections.len());
    for (g, e) in got.sections.iter().zip(golden.sections.iter()) {
        assert_eq!(g.section, e.section);
        assert_eq!(
            g.provenance_mix, e.provenance_mix,
            "section {:?} mix",
            g.section
        );
        assert!(approx(g.coverage, e.coverage), "section {:?} coverage", g.section);
        assert!(approx(g.assumed_fill_rate, e.assumed_fill_rate));
        assert!(
            approx(g.credibility, e.credibility),
            "section {:?} credibility {} vs {}",
            g.section,
            g.credibility,
            e.credibility
        );
    }

    assert!(got.coverage_is_consistent());
}

// ===========================================================================
// Phase 2 — stub generators + connect-runner round-trip + anti-drift
// ===========================================================================

/// Stub **app** generator: "produces" the UI-states, navigation, and operation
/// nodes of the spec (the app-gen's territory), minus any `skip`ped ref. Returns
/// the spec refs it produced — the contract a real app-generator must satisfy.
fn stub_app_generate(spec: &FunctionalSpec, skip: &[&str]) -> Vec<String> {
    enumerate_nodes(spec)
        .into_iter()
        .filter(|n| {
            matches!(
                n.section,
                SpecSection::UiStates | SpecSection::Navigation | SpecSection::Operations
            )
        })
        .map(|n| n.r#ref)
        .filter(|r| !skip.contains(&r.as_str()))
        .collect()
}

/// Stub **backend** generator: "produces" the entity and auth nodes (the
/// backend-gen's territory), minus any `skip`ped ref.
fn stub_backend_generate(spec: &FunctionalSpec, skip: &[&str]) -> Vec<String> {
    enumerate_nodes(spec)
        .into_iter()
        .filter(|n| matches!(n.section, SpecSection::Entities | SpecSection::Auth))
        .map(|n| n.r#ref)
        .filter(|r| !skip.contains(&r.as_str()))
        .collect()
}

/// Union the two generators' produced refs into the [`CoverageEvidence`] the
/// verify-phase consumes — the seam a real generation+observation pass reproduces.
fn coverage_evidence_from(
    spec: &FunctionalSpec,
    app_refs: &[String],
    backend_refs: &[String],
) -> CoverageEvidence {
    let produced: BTreeSet<&str> = app_refs
        .iter()
        .chain(backend_refs.iter())
        .map(|s| s.as_str())
        .collect();
    let mut ev = CoverageEvidence::default();
    for node in enumerate_nodes(spec) {
        if produced.contains(node.r#ref.as_str()) {
            match node.provenance {
                qontinui_types::functional_spec::SpecProvenance::Assumed => {
                    ev.filled_assumed.insert(node.r#ref);
                }
                _ => {
                    ev.covered.insert(node.r#ref);
                }
            }
        }
        // Unproduced Observed/Inferred nodes fall through → default NotGenerated gap.
    }
    ev
}

#[test]
fn connect_runner_fixtures_parse_against_v0_types() {
    let spec: FunctionalSpec =
        serde_json::from_str(CONNECT_SPEC).expect("connect-runner spec parses against FunctionalSpec");
    assert_eq!(spec.spec_version, "0");
    let _profile: Profile =
        serde_json::from_str(CONNECT_PROFILE).expect("connect-runner profile parses against Profile");
    // The page must be minimal-but-complete: a real entity, an operation, ui states,
    // and a navigation edge, so the slice exercises every section.
    assert!(!spec.entities.is_empty(), "needs ≥1 entity");
    assert!(!spec.operations.is_empty(), "needs ≥1 operation");
    assert!(!spec.ui_states.is_empty(), "needs ≥1 ui state");
    assert!(!spec.navigation.is_empty(), "needs ≥1 navigation edge");
    assert!(spec.auth.is_some(), "needs an auth model");
}

#[test]
fn complete_stub_generation_round_trips_to_full_coverage() {
    let spec: FunctionalSpec = serde_json::from_str(CONNECT_SPEC).unwrap();

    let app = stub_app_generate(&spec, &[]);
    let backend = stub_backend_generate(&spec, &[]);
    let evidence = coverage_evidence_from(&spec, &app, &backend);

    let verdict = evaluate_completeness(&spec, &evidence, "2026-06-14T00:00:00Z");

    // The whole point of the round-trip: a complete generation covers every
    // Observed/Inferred node → coverage 1.0, no gaps, all assumed nodes filled.
    assert!(
        approx(verdict.coverage, 1.0),
        "complete stub gen must cover everything; got {}",
        verdict.coverage
    );
    assert!(verdict.gaps.is_empty(), "no gaps expected: {:?}", verdict.gaps);
    assert!(approx(verdict.assumed_fill_rate, 1.0));
    assert!(verdict.coverage_is_consistent());
    // Every section the spec populates appears in the breakdown.
    for sec in [
        SpecSection::Entities,
        SpecSection::Operations,
        SpecSection::UiStates,
        SpecSection::Navigation,
        SpecSection::Auth,
    ] {
        assert!(
            verdict.sections.iter().any(|s| s.section == sec),
            "section {:?} missing from breakdown",
            sec
        );
    }
}

#[test]
fn dropped_ui_state_is_caught_as_a_gap() {
    // The anti-drift proof: if the app generator silently fails to produce a node,
    // the completeness diff MUST surface it as a gap — not rubber-stamp 1.0.
    let spec: FunctionalSpec = serde_json::from_str(CONNECT_SPEC).unwrap();

    let dropped = format!("uiStates.{}", spec.ui_states[0].id);
    let app = stub_app_generate(&spec, &[dropped.as_str()]);
    let backend = stub_backend_generate(&spec, &[]);
    let evidence = coverage_evidence_from(&spec, &app, &backend);

    let verdict = evaluate_completeness(&spec, &evidence, "2026-06-14T00:00:00Z");

    assert!(
        verdict.coverage < 1.0,
        "dropping a ui state must drop coverage below 1.0; got {}",
        verdict.coverage
    );
    let gap = verdict
        .gaps
        .iter()
        .find(|g| g.r#ref == dropped)
        .expect("the dropped ui_state must appear in the gap work-list");
    assert_eq!(gap.section, SpecSection::UiStates);
    assert_eq!(gap.reason, GapReason::NotGenerated);
    assert_eq!(gap.node_provenance, qontinui_types::functional_spec::SpecProvenance::Observed);
    assert!(verdict.coverage_is_consistent());
}

#[test]
fn dropped_entity_field_is_caught_as_a_gap_in_entities() {
    // Same proof on the backend seam: a missing entity field surfaces as an
    // Entities-section gap.
    let spec: FunctionalSpec = serde_json::from_str(CONNECT_SPEC).unwrap();

    // Find the first entity field ref to drop.
    let entity = &spec.entities[0];
    let field = entity
        .fields
        .first()
        .expect("connect-runner entity must have ≥1 field");
    let dropped = format!("entities.{}.fields.{}", entity.name, field.name);

    let app = stub_app_generate(&spec, &[]);
    let backend = stub_backend_generate(&spec, &[dropped.as_str()]);
    let evidence = coverage_evidence_from(&spec, &app, &backend);

    let verdict = evaluate_completeness(&spec, &evidence, "2026-06-14T00:00:00Z");

    assert!(verdict.coverage < 1.0, "dropping a field must drop coverage");
    let gap = verdict
        .gaps
        .iter()
        .find(|g| g.r#ref == dropped)
        .expect("the dropped field must appear as a gap");
    assert_eq!(gap.section, SpecSection::Entities);
    assert_eq!(gap.reason, GapReason::NotGenerated);
    assert!(verdict.coverage_is_consistent());
}
