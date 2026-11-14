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
    let runner_entry: fn(&Assertion, &EvalContext<'_>) -> AssertionResult =
        qontinui_vision_core::evaluate_assertion;
    let via_alias = runner_entry(&assertion(), &ctx);
    let direct = assertions::evaluate(&assertion(), &ctx);
    assert_eq!(via_alias.outcome, direct.outcome);
    direct
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
