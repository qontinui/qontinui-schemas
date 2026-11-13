// SPDX-License-Identifier: MIT
//! `matches_isolated_render` through the crate's PUBLIC surface, the way the
//! runner's `vision/assert` reaches it: the assertion and both captures are
//! parsed from their wire JSON, and the verdict comes from
//! `qontinui_vision_core::evaluate_assertion`. The unit tests in
//! `assertions/component.rs` pin the evaluator's semantics; this file pins
//! that the variant, the capture types and the `EvalContext` fields are all
//! reachable from outside the crate.

use std::collections::HashMap;

use qontinui_vision_core::{
    evaluate_assertion, Assertion, AssertionOutcome, ComponentBaseline, ComponentCapture,
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

fn evaluate(baseline: Option<ComponentCapture>, in_situ: ComponentCapture) -> AssertionOutcome {
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
    let r = evaluate_assertion(&assertion(), &ctx);
    if r.outcome == AssertionOutcome::Failed {
        let report: ComponentDiffReport =
            serde_json::from_str(r.detail.as_deref().expect("a failure carries detail"))
                .expect("detail is a ComponentDiffReport");
        assert!(
            report
                .findings
                .iter()
                .any(|f| f.kind == ComponentFindingKind::Size),
            "the only divergence is size: {report:?}"
        );
    }
    r.outcome
}

#[test]
fn a_stretched_component_fails_through_the_public_dispatcher() {
    // 200 of a 400-wide parent in isolation, 1000 of a 1000-wide parent in situ.
    assert_eq!(
        evaluate(Some(capture(200.0, 400.0)), capture(1000.0, 1000.0)),
        AssertionOutcome::Failed
    );
}

#[test]
fn an_identical_render_passes_through_the_public_dispatcher() {
    assert_eq!(
        evaluate(Some(capture(200.0, 400.0)), capture(200.0, 400.0)),
        AssertionOutcome::Passed
    );
}

#[test]
fn an_unregistered_baseline_is_unknown_through_the_public_dispatcher() {
    assert_eq!(
        evaluate(None, capture(200.0, 400.0)),
        AssertionOutcome::Unknown
    );
}
