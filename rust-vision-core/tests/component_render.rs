// SPDX-License-Identifier: MIT
//! `matches_isolated_render` through the crate's PUBLIC surface, the way the
//! runner's `vision/assert` reaches it: the assertion and both captures are
//! parsed from their wire JSON, and the verdict comes from
//! the public dispatcher (`qontinui_vision_core::assertions::evaluate`,
//! re-exported as `evaluate_assertion`). The unit tests in
//! `assertions/component.rs` pin the evaluator's semantics; this file pins
//! that the variant, the capture types and the `EvalContext` fields are all
//! reachable from outside the crate.

use std::collections::HashMap;

use qontinui_vision_core::{
    assertions, Assertion, AssertionOutcome, AssertionResult, ComponentBaseline, ComponentCapture,
    ComponentDiffReport, ComponentFindingKind, EvalContext,
};
use serde_json::json;

const BASELINE: &str = "storybook/DateRangePicker--default@abc123";
const COMPONENT: &str = "date-range-picker";

fn capture(width: f64, parent_width: f64) -> ComponentCapture {
    serde_json::from_value(json!({
        "schemaVersion": 1,
        "target": { "kind": "component", "id": COMPONENT },
        "viewport": { "width": 1280, "height": 800 },
        "dpr": 1,
        "root": {
            "bbox": { "x": 0, "y": 0, "width": width, "height": 40 },
            "parentContentBox": { "x": 0, "y": 0, "width": parent_width, "height": 40 }
        },
        "nodes": [
            { "path": "", "tag": "div", "componentName": "DateRangePicker",
              "bbox": { "x": 0, "y": 0, "width": width, "height": 40 },
              "styles": { "display": "block" } }
        ],
        "clipAncestors": [],
        "unknownChecks": []
    }))
    .expect("the contract's wire form parses as a ComponentCapture")
}

fn assertion() -> Assertion {
    serde_json::from_value(json!({
        "type": "matches_isolated_render",
        "component": COMPONENT,
        "baseline": BASELINE,
    }))
    .expect("the assertion's wire form parses")
}

fn result_of(baseline: Option<ComponentCapture>, in_situ: ComponentCapture) -> AssertionResult {
    let baselines: HashMap<String, ComponentBaseline> = baseline
        .into_iter()
        .map(|capture| {
            let b: ComponentBaseline = serde_json::from_value(json!({
                "name": BASELINE,
                "storyId": "DateRangePicker--default",
                "capture": capture,
            }))
            .expect("the baseline's wire form parses");
            (BASELINE.to_string(), b)
        })
        .collect();
    let captures = HashMap::from([(COMPONENT.to_string(), in_situ)]);
    let ctx = EvalContext {
        component_baselines: Some(&baselines),
        component_captures: Some(&captures),
        ..Default::default()
    };
    // `assertions::evaluate` is the dispatcher the crate re-exports as
    // `evaluate_assertion`; the runner calls it under that name.
    // Pin the re-exported name the runner calls: renaming or dropping it
    // breaks this file, not first the runner.
    let _runner_entry: fn(&Assertion, &EvalContext<'_>) -> AssertionResult =
        qontinui_vision_core::evaluate_assertion;
    assertions::evaluate(&assertion(), &ctx)
}

/// A 400x40 root at page (100, 300) with one child node at root-relative
/// `child`, inside a 400x40 parent, clipped (or not) by `clip`.
fn clipped_capture(child: serde_json::Value, clip: Option<serde_json::Value>) -> ComponentCapture {
    serde_json::from_value(json!({
        "schemaVersion": 1,
        "target": { "kind": "component", "id": COMPONENT },
        "viewport": { "width": 1280, "height": 800 },
        "dpr": 1,
        "root": {
            "bbox": { "x": 100, "y": 300, "width": 400, "height": 40 },
            "parentContentBox": { "x": 100, "y": 300, "width": 400, "height": 40 }
        },
        "nodes": [
            { "path": "", "tag": "div", "componentName": "DateRangePicker",
              "bbox": { "x": 0, "y": 0, "width": 400, "height": 40 } },
            { "path": "0", "tag": "ul", "componentName": "DateRangePicker", "bbox": child }
        ],
        "clipAncestors": clip.into_iter().collect::<Vec<_>>(),
    }))
    .expect("the contract's wire form parses as a ComponentCapture")
}

/// A clip ancestor exactly covering the root, clipping on both axes.
fn clip_hidden() -> serde_json::Value {
    json!({ "descriptor": "div.frame", "overflowX": "hidden", "overflowY": "clip",
            "rect": { "x": 100, "y": 300, "width": 400, "height": 40 } })
}

fn clipped_paths(r: &AssertionResult) -> Vec<String> {
    let report: ComponentDiffReport = serde_json::from_str(
        r.detail
            .as_deref()
            .expect("a compared result carries detail"),
    )
    .expect("detail is a ComponentDiffReport");
    report
        .findings
        .iter()
        .filter(|f| f.kind == ComponentFindingKind::Clipped)
        .map(|f| f.path.clone())
        .collect()
}

#[test]
fn a_stretched_component_fails_through_the_public_dispatcher() {
    // 200 of a 400-wide parent in isolation, 1000 of a 1000-wide parent in situ.
    let r = result_of(Some(capture(200.0, 400.0)), capture(1000.0, 1000.0));
    assert_eq!(r.outcome, AssertionOutcome::Failed);
    let report: ComponentDiffReport =
        serde_json::from_str(r.detail.as_deref().expect("a failure carries detail"))
            .expect("detail is a ComponentDiffReport");
    assert!(
        !report.findings.is_empty()
            && report
                .findings
                .iter()
                .all(|f| f.kind == ComponentFindingKind::Size),
        "the only divergence is size: {report:?}"
    );
}

#[test]
fn an_identical_render_passes_through_the_public_dispatcher() {
    assert_eq!(
        result_of(Some(capture(200.0, 400.0)), capture(200.0, 400.0)).outcome,
        AssertionOutcome::Passed
    );
}

#[test]
fn an_unregistered_baseline_is_unknown_through_the_public_dispatcher() {
    assert_eq!(
        result_of(None, capture(200.0, 400.0)).outcome,
        AssertionOutcome::Unknown
    );
}

#[test]
fn a_child_overflowing_the_bottom_is_clipped_on_the_y_axis() {
    // A dropdown list 200px tall below a 40px clipping frame in situ; in
    // isolation there is no frame.
    let child = json!({ "x": 0, "y": 20, "width": 400, "height": 200 });
    let r = result_of(
        Some(clipped_capture(child.clone(), None)),
        clipped_capture(child, Some(clip_hidden())),
    );
    assert_eq!(r.outcome, AssertionOutcome::Failed, "{:?}", r.detail);
    assert_eq!(clipped_paths(&r), vec!["0".to_string()]);
}

#[test]
fn a_child_past_the_top_or_left_edge_is_clipped() {
    for child in [
        json!({ "x": 0, "y": -30, "width": 100, "height": 40 }),
        json!({ "x": -50, "y": 0, "width": 100, "height": 40 }),
    ] {
        let r = result_of(
            Some(clipped_capture(child.clone(), None)),
            clipped_capture(child.clone(), Some(clip_hidden())),
        );
        assert_eq!(
            r.outcome,
            AssertionOutcome::Failed,
            "{child}: {:?}",
            r.detail
        );
        assert_eq!(clipped_paths(&r), vec!["0".to_string()], "{child}");
    }
}

#[test]
fn a_child_inside_the_clip_rect_is_not_clipped() {
    let child = json!({ "x": 0, "y": 0, "width": 400, "height": 40 });
    let r = result_of(
        Some(clipped_capture(child.clone(), None)),
        clipped_capture(child, Some(clip_hidden())),
    );
    assert_eq!(r.outcome, AssertionOutcome::Passed, "{:?}", r.detail);
}

/// The size tolerance is exclusive: a difference of exactly 5% (absolute and
/// fill ratio alike) passes, and anything beyond it fails.
#[test]
fn exactly_the_default_size_tolerance_passes_and_just_beyond_fails() {
    // 200 -> 210 in an unchanged 1000-wide parent: +5.0% absolute, +5.0% fill.
    let r = result_of(Some(capture(200.0, 1000.0)), capture(210.0, 1000.0));
    assert_eq!(r.outcome, AssertionOutcome::Passed, "{:?}", r.detail);

    // 200 -> 211: +5.5%.
    let r = result_of(Some(capture(200.0, 1000.0)), capture(211.0, 1000.0));
    assert_eq!(r.outcome, AssertionOutcome::Failed, "{:?}", r.detail);
}
