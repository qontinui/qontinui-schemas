//! A degenerate snapshot must not audit clean.
//!
//! These tests pin the defect this crate's analyzer verdict exists to close:
//! an analyzer whose input carried nothing it could measure returned an
//! empty `Vec<Finding>`, and an empty `Vec<Finding>` was also what a healthy
//! page returned. The two were byte-identical, so every consumer that read
//! finding-emptiness as a verdict — the `vision-audit` exit code, the
//! runner's `/vision/analyze` response, an agent reading a report — was
//! given a clean bill of health for a page nobody had checked.
//!
//! Written against the crate's PUBLIC API only (`qontinui_vision_core::*`),
//! deliberately: the runner consumes this surface from another repo, and a
//! test that reached into module internals would not prove the surface it
//! actually gets is usable.
//!
//! The two degenerate shapes below are named rather than parameterised. They
//! are different defects with different producers, and a failure message
//! that says which one broke is worth more than a table.

use qontinui_vision_core::{
    analyzers, AnalyzeInput, Analyzer, AnalyzerVerdict, Element, ElementSnapshot, Region,
    SnapshotCoverage,
};

/// A fully-projected element: geometry, text, interactivity.
fn healthy_el(id: &str, x: i32) -> Element {
    Element {
        id: id.to_string(),
        bbox: Some(Region {
            x,
            y: 0,
            w: 100,
            h: 40,
        }),
        text: Some(format!("label {id}")),
        interactable: true,
        ..Default::default()
    }
}

/// The §1-case-2 shape: text and interactivity survived the projection,
/// geometry did not.
fn partially_projected_el(id: &str) -> Element {
    Element {
        id: id.to_string(),
        bbox: None,
        text: Some(format!("label {id}")),
        interactable: true,
        ..Default::default()
    }
}

fn snapshot(elements: Vec<Element>) -> ElementSnapshot {
    ElementSnapshot {
        elements,
        ..Default::default()
    }
}

fn run(analyzer: Analyzer, snap: &ElementSnapshot) -> analyzers::AnalyzerResult {
    analyzers::run(
        analyzer,
        &AnalyzeInput {
            frame: None,
            snapshot: Some(snap),
            prior_frame: None,
        },
    )
}

// ===========================================================================
// Shape 1 — zero geometry
// ===========================================================================

#[test]
fn a_zero_geometry_snapshot_blocks_the_layout_analyzer() {
    // Elements exist — this is NOT the empty-snapshot case `elements`
    // already refused. Every one of them simply lost its bbox.
    let snap = snapshot(vec![
        Element {
            id: "a".into(),
            ..Default::default()
        },
        Element {
            id: "b".into(),
            ..Default::default()
        },
        Element {
            id: "c".into(),
            ..Default::default()
        },
    ]);

    let cov = SnapshotCoverage::of(&snap);
    assert_eq!(cov.elements, 3);
    assert_eq!(cov.with_geometry, 0);

    let result = run(Analyzer::Layout, &snap);

    assert!(
        result.verdict.is_blocked(),
        "zero geometry must block layout, got {:?}",
        result.verdict
    );
    assert!(
        !result.conclusive,
        "the gate bit must be derived from the verdict, not from the finding list"
    );
    assert!(
        result.verdict.reason().unwrap_or_default().contains("bbox"),
        "the reason must name the missing dimension: {:?}",
        result.verdict
    );
    // The finding list IS empty here — that is precisely why it must never be
    // presented as a result. The verdict is the only thing standing between
    // this and a clean bill of health.
    assert!(result.findings.is_empty(), "{:?}", result.findings);
    assert_eq!(result.coverage.as_ref(), Some(&cov));
}

// ===========================================================================
// Shape 2 — the partial projection
// ===========================================================================

#[test]
fn a_partial_projection_blocks_layout_but_not_typography() {
    // Elements carrying `text` and `interactable: true` but NO `bbox`. This
    // is the shape that no analyzer marked: `elements` is content with it
    // (total > 0, interactive > 0, text > 0 — no finding), so on a FULL
    // analyzer run it produced output byte-identical to a healthy page's.
    let degenerate = snapshot(vec![
        partially_projected_el("save"),
        partially_projected_el("cancel"),
        partially_projected_el("title"),
    ]);

    let cov = SnapshotCoverage::of(&degenerate);
    assert_eq!(
        cov,
        SnapshotCoverage {
            elements: 3,
            with_geometry: 0,
            with_stacking: 0,
            with_text: 3,
            interactable: 3,
        },
        "the fixture must be the partial-projection shape, not something else"
    );

    // Layout: refuses. Every geometric check had an empty working set.
    let layout = run(Analyzer::Layout, &degenerate);
    assert!(
        layout.verdict.is_blocked(),
        "layout must refuse a geometry-less snapshot, got {:?}",
        layout.verdict
    );
    assert!(!layout.conclusive);

    // Typography: answers. It needs text, and the text survived the
    // projection — so blocking it here would be as wrong as passing layout.
    // The verdict is per-analyzer for exactly this reason.
    let typography = run(Analyzer::Typography, &degenerate);
    assert_eq!(
        typography.verdict,
        AnalyzerVerdict::Checked,
        "typography has its whole input and must stay checkable"
    );
    assert!(typography.conclusive);

    // `elements` stays meaningful too — pure structural, no geometry needed
    // — but records that target size went unmeasured.
    let elements = run(Analyzer::Elements, &degenerate);
    assert!(
        elements.conclusive,
        "the structural analyzer is never blocked on coverage: {:?}",
        elements.verdict
    );
    assert!(
        elements.verdict.is_degraded(),
        "…but `tiny_target` could not be evaluated, and that must be said: {:?}",
        elements.verdict
    );
}

#[test]
fn the_partial_projection_is_indistinguishable_from_a_healthy_page_by_findings_alone() {
    // The measurement that motivated the whole change, pinned as an
    // assertion: the two runs agree finding-for-finding and disagree only on
    // the verdict. If this ever starts failing because the finding lists
    // diverged, the verdict is no longer the ONLY thing separating them —
    // which would be good news, but it must be noticed rather than silently
    // making the verdict look redundant.
    let healthy = snapshot(vec![healthy_el("save", 0), healthy_el("cancel", 200)]);
    let degenerate = snapshot(vec![
        partially_projected_el("save"),
        partially_projected_el("cancel"),
    ]);

    let healthy_layout = run(Analyzer::Layout, &healthy);
    let degenerate_layout = run(Analyzer::Layout, &degenerate);

    assert_eq!(
        serde_json::to_string(&healthy_layout.findings).unwrap(),
        serde_json::to_string(&degenerate_layout.findings).unwrap(),
        "the premise of this test: findings alone cannot tell these apart"
    );
    assert_ne!(
        healthy_layout.conclusive, degenerate_layout.conclusive,
        "…and the verdict must"
    );
    assert_eq!(healthy_layout.verdict, AnalyzerVerdict::Checked);
    assert!(degenerate_layout.verdict.is_blocked());
}

// ===========================================================================
// The healthy page, and the gate bit
// ===========================================================================

#[test]
fn a_healthy_snapshot_is_checked() {
    let healthy = snapshot(vec![
        healthy_el("save", 0),
        healthy_el("cancel", 200),
        healthy_el("title", 400),
    ]);

    for analyzer in [Analyzer::Layout, Analyzer::Typography, Analyzer::Elements] {
        let result = run(analyzer, &healthy);
        assert_eq!(
            result.verdict,
            AnalyzerVerdict::Checked,
            "{} must reach a conclusion on a fully-projected snapshot",
            analyzer.name()
        );
        assert!(result.conclusive);
        assert!(
            result.findings.is_empty(),
            "{} on a clean page: {:?}",
            analyzer.name(),
            result.findings
        );
    }
}

#[test]
fn a_degraded_verdict_does_not_move_the_gate_bit() {
    // Two intersecting elements with no stacking order: occlusion is
    // genuinely UNKNOWN, which is a real gap and is reported as one — but it
    // must not fail a gate. A healthy projected snapshot routinely carries
    // no `z_index` at all (the projector emits it only for a computed
    // `zIndex` that parses as an integer, and `auto` does not), so a
    // gate-failing Degraded would fire on essentially every real capture and
    // be muted within a week, taking the Blocked signal with it.
    let mut a = healthy_el("a", 0);
    let mut b = healthy_el("b", 50);
    a.z_index = None;
    b.z_index = None;
    let snap = snapshot(vec![a, b]);

    let result = run(Analyzer::Layout, &snap);
    assert!(
        result.verdict.is_degraded(),
        "expected the occlusion-unknown degradation, got {:?}",
        result.verdict
    );
    assert!(
        result.conclusive,
        "Degraded is GREEN by design — see AnalyzerVerdict's type doc"
    );
    // The finding it was promoted from is retained, not replaced.
    assert!(
        result
            .findings
            .iter()
            .any(|f| f.kind == "occlusion_unknown"),
        "{:?}",
        result.findings
    );
}

#[test]
fn zero_interactable_degrades_rather_than_blocks_when_geometry_is_present() {
    // Corrects a threshold that reading `layout.rs`'s first working set
    // alone suggests. Only the pairwise-overlap pass filters on
    // `interactable`; the directed-occlusion, zero-area and alignment passes
    // build their working sets from `bbox` alone. The crate's own
    // `007-widget-occludes-label` corpus fixture has FOUR elements, every
    // one `interactable: false`, and yields the Critical `occlusion` finding
    // that pins the Terminal-page regression — so blocking on
    // `interactable == 0` would stamp "no finding list from this run is a
    // statement about the page" on the analyzer's most valuable output.
    let covered = Element {
        id: "session-name".into(),
        bbox: Some(Region {
            x: 0,
            y: 0,
            w: 100,
            h: 20,
        }),
        text: Some("prod-deploy-7".into()),
        interactable: false,
        occluded_by: Some("toast".into()),
        ..Default::default()
    };
    let snap = snapshot(vec![covered]);
    assert_eq!(SnapshotCoverage::of(&snap).interactable, 0);

    let result = run(Analyzer::Layout, &snap);
    assert!(
        result.conclusive,
        "a real Critical finding must not be reported under a blocked verdict: {:?}",
        result.verdict
    );
    assert!(result.verdict.is_degraded(), "{:?}", result.verdict);
    assert!(
        result.findings.iter().any(|f| f.kind == "occlusion"),
        "{:?}",
        result.findings
    );
}

// ===========================================================================
// Wire shape — the runner reads this over HTTP
// ===========================================================================

#[test]
fn the_gate_bit_cannot_be_forged_on_the_wire() {
    // `conclusive` is DERIVED, so a payload asserting otherwise is asserting
    // something the type does not permit. A hand-written `"conclusive": true`
    // beside a blocked verdict is the vacuous pass in its purest form, and
    // deserialization recomputes rather than trusts it.
    let forged = r#"{
        "conclusive": true,
        "verdict": { "state": "blocked", "reason": "no element carries a bbox (0/3)" },
        "findings": []
    }"#;
    let result: analyzers::AnalyzerResult = serde_json::from_str(forged).unwrap();
    assert!(!result.conclusive);
    assert!(result.verdict.is_blocked());
}

#[test]
fn the_verdict_and_coverage_serialize_in_the_documented_shape() {
    let snap = snapshot(vec![partially_projected_el("save")]);
    let wire = serde_json::to_value(run(Analyzer::Layout, &snap)).unwrap();

    assert_eq!(wire["conclusive"], serde_json::json!(false));
    assert_eq!(wire["verdict"]["state"], serde_json::json!("blocked"));
    assert!(wire["verdict"]["reason"].is_string());
    assert_eq!(wire["coverage"]["withGeometry"], serde_json::json!(0));
    assert_eq!(wire["coverage"]["withText"], serde_json::json!(1));
    assert!(wire["findings"].is_array());
}

#[test]
fn dynamic_carries_no_coverage_and_a_missing_frame_is_blocked() {
    // `dynamic` takes two frames and no snapshot, so it has no element list
    // to measure — `coverage: None` is "not applicable", never "all zeroes".
    // Its precondition is frame availability, which the dispatcher checks,
    // and a call missing a frame keeps the `skipped` finding it has always
    // emitted while now also carrying the verdict that finding always meant.
    let snap = snapshot(vec![healthy_el("a", 0)]);
    let result = analyzers::run(
        Analyzer::Dynamic,
        &AnalyzeInput {
            frame: None,
            snapshot: Some(&snap),
            prior_frame: None,
        },
    );
    assert!(result.verdict.is_blocked(), "{:?}", result.verdict);
    assert!(!result.conclusive);
    assert!(result.coverage.is_none());
    assert!(
        result.findings.iter().any(|f| f.kind == "skipped"),
        "the long-standing `skipped` finding must survive: {:?}",
        result.findings
    );
}
