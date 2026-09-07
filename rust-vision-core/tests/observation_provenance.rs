//! Every observation carries its provenance, and says whether it is an
//! estimate or a deduction.
//!
//! These tests pin the two halves of that promise, and one property that is
//! easy to lose by accident:
//!
//! 1. **Attribution.** Every `Finding` leaving `analyzers::run` names the
//!    analyzer that produced it — including the `skipped` findings of a
//!    `Blocked` run, because a refusal to answer is still an observation and
//!    is the one most likely to be read out of context.
//! 2. **Qualification.** `confidence` is `Some` exactly where the input was
//!    estimated, and `None` everywhere else — where `None` is a POSITIVE
//!    statement ("this is a deduction") rather than a gap.
//! 3. **That the statement survives serialization.** A `None` confidence has
//!    to be distinguishable ON THE WIRE from a confidence nobody mentioned.
//!    A `skip_serializing_if` on that field would silently collapse the two
//!    and leave the doc comment claiming a distinction the bytes do not
//!    carry — so the wire shape is asserted directly, not inferred from a
//!    round trip (which passes either way).
//!
//! Written against the crate's PUBLIC API only, deliberately: the runner
//! consumes this surface from another repo, and it is the projection at that
//! boundary — `vision_routes.rs` — that dropped `captured_at` on the floor
//! and motivated this work.

use qontinui_vision_core::{
    analyzers, assertions, evaluate_assertion, AnalyzeInput, Analyzer, AnalyzerVerdict, Assertion,
    AssertionOutcome, AssertionResult, Element, ElementSnapshot, EvalContext, Finding, Frame,
    FrameSource, OcrBlockRef, Region, Rgb, Severity, TextMatchKind, TextTarget, WcagLevel,
};

// ---------------------------------------------------------------- fixtures

fn region(x: i32, y: i32, w: u32, h: u32) -> Region {
    Region { x, y, w, h }
}

fn solid_frame(w: u32, h: u32, color: [u8; 4]) -> Frame {
    Frame::from_rgba(
        image::RgbaImage::from_pixel(w, h, image::Rgba(color)),
        FrameSource::synthetic_now(),
    )
}

/// A fully-projected element: geometry, text, interactivity.
fn el(id: &str, x: i32) -> Element {
    Element {
        id: id.to_string(),
        bbox: Some(region(x, 0, 100, 40)),
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

const ALL: [Analyzer; 5] = [
    Analyzer::Layout,
    Analyzer::Typography,
    Analyzer::Color,
    Analyzer::Dynamic,
    Analyzer::Elements,
];

// ------------------------------------------------- 1. attribution (phase 3)

/// Every finding from a real run names its analyzer. The stamp lives in the
/// dispatcher, so this is one assertion over all five rather than five
/// assertions each analyzer could individually have forgotten.
#[test]
fn every_finding_from_a_real_run_names_its_analyzer() {
    // The fixture has to make ALL FIVE speak, or the per-analyzer guard at
    // the end fails loudly rather than letting a silent zero pass for
    // coverage. Overlapping bboxes feed `layout`; five distinct font
    // families trip `typography`'s >3 drift heuristic; undeclared colours
    // over a flat frame feed `color`'s sampled arm; the two frames differ
    // wholesale for `dynamic`; and a sub-24px interactive target trips
    // `elements`' WCAG 2.5.8 `tiny_target` check, which is the only one of
    // its four that a healthy-looking snapshot reaches.
    let families = ["Inter", "Georgia", "Menlo", "Comic Sans MS", "Helvetica"];
    let mut elements: Vec<Element> = families
        .iter()
        .enumerate()
        .map(|(i, fam)| {
            let mut e = el(&format!("e{i}"), (i as i32) * 20);
            e.font_family = Some((*fam).to_string());
            e
        })
        .collect();
    let mut tiny = el("tiny", 150);
    tiny.bbox = Some(region(150, 0, 16, 16));
    elements.push(tiny);
    let snap = snapshot(elements);
    let frame = solid_frame(200, 60, [0xC8, 0xC8, 0xC8, 0xff]);
    let prior = solid_frame(200, 60, [0x10, 0x10, 0x10, 0xff]);

    let mut per: Vec<(Analyzer, usize)> = Vec::new();
    for a in ALL {
        let input = AnalyzeInput {
            frame: Some(&frame),
            snapshot: Some(&snap),
            prior_frame: Some(&prior),
        };
        let result = analyzers::run(a, &input);
        for f in &result.findings {
            assert_eq!(
                f.analyzer,
                Some(a),
                "{a:?} produced an unattributed finding: {:?}",
                f.kind
            );
        }
        per.push((a, result.findings.len()));
    }

    // Per-analyzer, not aggregate. An aggregate `total > 0` is satisfied by
    // ONE analyzer, and an analyzer that produced nothing asserted nothing —
    // its arm of the loop above ran zero times. `AnalyzerResult::blocked`
    // does not synthesize a finding either (`color::run`'s `with_text == 0`
    // path passes an empty vec), so silent zero coverage is reachable, not
    // hypothetical.
    for (a, n) in &per {
        assert!(
            *n > 0,
            "{a:?} produced no findings on this fixture, so its attribution \
             was never actually checked — fix the fixture rather than the \
             assertion"
        );
    }
}

/// A refusal to answer is still an observation, and it needs its provenance
/// MORE than a clean result does — a `skipped` finding merged into a
/// five-analyzer report is unreadable without it.
#[test]
fn a_blocked_verdict_still_carries_its_provenance() {
    for a in ALL {
        // Every required input withheld: the canonical Blocked case.
        let input = AnalyzeInput {
            frame: None,
            snapshot: None,
            prior_frame: None,
        };
        let result = analyzers::run(a, &input);

        assert!(
            matches!(result.verdict, AnalyzerVerdict::Blocked { .. }),
            "{a:?} was expected to block on empty input, got {:?}",
            result.verdict
        );
        assert!(!result.conclusive, "{a:?} blocked but reported conclusive");
        assert!(
            !result.findings.is_empty(),
            "{a:?} blocked without saying why"
        );
        for f in &result.findings {
            assert_eq!(
                f.analyzer,
                Some(a),
                "{a:?} blocked and lost its attribution on a {:?} finding",
                f.kind
            );
        }
    }
}

/// The stamp is the dispatcher's job, so a finding built any other way is
/// honestly unattributed rather than falsely labelled.
#[test]
fn a_hand_built_finding_is_unattributed_rather_than_mislabelled() {
    let f = Finding::new("overlap", Severity::Warning, "two boxes intersect");
    assert_eq!(f.analyzer, None);
}

// ------------------------------------------------ 2. qualification (phase 3)

/// The pixel-sampled contrast arm is the one estimated analyzer input in the
/// crate, and it is the one that carries a number.
#[test]
fn a_pixel_sampled_contrast_finding_carries_a_confidence() {
    // No declared colours: the analyzer must sample the frame. A flat frame
    // collapses both modes onto one colour, so the ratio lands at ~1.0 and
    // the Info finding is emitted.
    let mut e = el("sampled", 0);
    e.fg_color = None;
    e.bg_color = None;
    let snap = snapshot(vec![e]);
    let frame = solid_frame(200, 60, [0x80, 0x80, 0x80, 0xff]);

    let result = analyzers::run(
        Analyzer::Color,
        &AnalyzeInput {
            frame: Some(&frame),
            snapshot: Some(&snap),
            prior_frame: None,
        },
    );

    let f = result
        .findings
        .iter()
        .find(|f| f.kind == "low_contrast")
        .expect("sampled arm should have emitted a low_contrast finding");
    assert_eq!(
        f.severity,
        Severity::Info,
        "sampled findings stay informational"
    );
    let c = f
        .confidence
        .expect("a sampled contrast reading is an ESTIMATE and must say so");
    assert!(
        (0.0..=1.0).contains(&c),
        "confidence {c} outside the documented 0.0..=1.0"
    );
    // A single flat colour is described perfectly by the two-colour model.
    assert!(
        (c - 1.0).abs() < 1e-9,
        "flat region should score 1.0, got {c}"
    );
}

/// Declared colours are exact arithmetic. `None` here is the statement that
/// nothing was estimated — not a measurement somebody forgot to take.
#[test]
fn a_declared_contrast_finding_states_that_it_is_a_deduction() {
    let mut e = el("declared", 0);
    e.fg_color = Some(Rgb::new(200, 200, 200));
    e.bg_color = Some(Rgb::new(255, 255, 255));
    let snap = snapshot(vec![e]);
    let frame = solid_frame(200, 60, [0xff, 0xff, 0xff, 0xff]);

    let result = analyzers::run(
        Analyzer::Color,
        &AnalyzeInput {
            frame: Some(&frame),
            snapshot: Some(&snap),
            prior_frame: None,
        },
    );

    let f = result
        .findings
        .iter()
        .find(|f| f.kind == "low_contrast")
        .expect("declared low-contrast pair should have been flagged");
    assert_ne!(
        f.severity,
        Severity::Info,
        "declared findings are gated, not informational"
    );
    assert_eq!(
        f.confidence, None,
        "contrast computed from DECLARED colours is a deduction, not an estimate"
    );
}

/// Geometry findings are deductions over exact bboxes.
#[test]
fn geometry_findings_state_that_they_are_deductions() {
    let snap = snapshot(vec![el("a", 0), el("b", 10)]);
    let result = analyzers::run(
        Analyzer::Layout,
        &AnalyzeInput {
            frame: None,
            snapshot: Some(&snap),
            prior_frame: None,
        },
    );
    assert!(
        !result.findings.is_empty(),
        "overlapping boxes should produce findings"
    );
    for f in &result.findings {
        assert_eq!(
            f.confidence, None,
            "{:?} is derived from bounding boxes and must not claim a confidence",
            f.kind
        );
    }
}

/// The OCR fallback is the one estimated input in the assertion evaluator.
/// It reports the LOWEST contributing block's confidence: the aggregate
/// string is only as good as its worst-read fragment.
#[test]
fn the_ocr_fallback_reports_the_lowest_contributing_confidence() {
    let target = region(0, 0, 100, 40);
    let blocks = [
        OcrBlockRef {
            bbox: region(0, 0, 40, 40),
            text: "Save",
            confidence: 0.94,
        },
        OcrBlockRef {
            bbox: region(40, 0, 40, 40),
            text: "changes",
            confidence: 0.61,
        },
        // Outside the target: must not drag the reported confidence down.
        OcrBlockRef {
            bbox: region(500, 500, 10, 10),
            text: "junk",
            confidence: 0.02,
        },
    ];
    let ctx = EvalContext {
        ocr_blocks: Some(&blocks),
        ..Default::default()
    };

    let r = evaluate_assertion(
        &Assertion::ContainsText {
            target: TextTarget::Region(assertions::RegionTextTarget { region: target }),
            text: "Save changes".to_string(),
            kind: TextMatchKind::Contains,
        },
        &ctx,
    );

    assert_eq!(
        r.outcome,
        AssertionOutcome::Passed,
        "detail: {:?}",
        r.detail
    );
    assert_eq!(
        r.confidence,
        Some(0.61),
        "an OCR-sourced verdict must report the worst block that fed it"
    );
}

/// A failing OCR verdict is just as estimated as a passing one.
#[test]
fn a_failing_ocr_verdict_is_qualified_too() {
    let blocks = [OcrBlockRef {
        bbox: region(0, 0, 40, 40),
        text: "Saue",
        confidence: 0.33,
    }];
    let ctx = EvalContext {
        ocr_blocks: Some(&blocks),
        ..Default::default()
    };
    let r = evaluate_assertion(
        &Assertion::ContainsText {
            target: TextTarget::Region(assertions::RegionTextTarget {
                region: region(0, 0, 100, 40),
            }),
            text: "Save".to_string(),
            kind: TextMatchKind::Contains,
        },
        &ctx,
    );
    assert_eq!(r.outcome, AssertionOutcome::Failed);
    assert_eq!(r.confidence, Some(0.33));
}

/// The case both doc sites promise and neither would have caught: OCR
/// blocks were supplied but NONE overlapped the target, so nothing was read
/// and nothing estimated reached the verdict. `None` here is accurate, and
/// a fold that yielded a value on an empty contributor set would be a
/// fabricated confidence about text that was never looked at.
#[test]
fn an_ocr_run_with_no_overlapping_block_states_no_estimate() {
    let blocks = [OcrBlockRef {
        bbox: region(500, 500, 10, 10),
        text: "junk",
        confidence: 0.02,
    }];
    let ctx = EvalContext {
        ocr_blocks: Some(&blocks),
        ..Default::default()
    };
    let r = evaluate_assertion(
        &Assertion::ContainsText {
            target: TextTarget::Region(assertions::RegionTextTarget {
                region: region(0, 0, 100, 40),
            }),
            text: "Save".to_string(),
            kind: TextMatchKind::Contains,
        },
        &ctx,
    );
    assert_eq!(r.outcome, AssertionOutcome::Failed);
    assert_eq!(
        r.confidence, None,
        "no block was read, so nothing was estimated"
    );
}

/// A block DID contribute and carried no usable confidence. `None` would be
/// a false claim of exactness about a verdict OCR decided — and passing the
/// `NaN` through would be the same false claim, because `serde_json` writes
/// a non-finite float as `null`, byte-identical to the deduction statement.
/// The floor is the honest answer: estimated, trusted not at all.
#[test]
fn an_unusable_ocr_confidence_floors_rather_than_reading_as_a_deduction() {
    let blocks = [
        OcrBlockRef {
            bbox: region(0, 0, 40, 40),
            text: "Save",
            confidence: f64::NAN,
        },
        OcrBlockRef {
            bbox: region(40, 0, 40, 40),
            text: "changes",
            confidence: f64::INFINITY,
        },
    ];
    let ctx = EvalContext {
        ocr_blocks: Some(&blocks),
        ..Default::default()
    };
    let r = evaluate_assertion(
        &Assertion::ContainsText {
            target: TextTarget::Region(assertions::RegionTextTarget {
                region: region(0, 0, 100, 40),
            }),
            text: "Save changes".to_string(),
            kind: TextMatchKind::Contains,
        },
        &ctx,
    );
    assert_eq!(
        r.outcome,
        AssertionOutcome::Passed,
        "detail: {:?}",
        r.detail
    );
    assert_eq!(r.confidence, Some(0.0));

    // And it survives the wire as a number, not as the `null` a NaN would
    // have become — which is the whole reason the arm exists.
    let v = serde_json::to_value(&r).unwrap();
    assert_eq!(v["confidence"], 0.0);
    assert!(!v["confidence"].is_null());
}

/// One usable block among unusable ones still decides the answer: a single
/// malformed block must not erase a reading the rest agree on.
#[test]
fn one_unusable_block_does_not_erase_the_readable_ones() {
    let blocks = [
        OcrBlockRef {
            bbox: region(0, 0, 40, 40),
            text: "Save",
            confidence: f64::NAN,
        },
        OcrBlockRef {
            bbox: region(40, 0, 40, 40),
            text: "changes",
            confidence: 0.55,
        },
    ];
    let ctx = EvalContext {
        ocr_blocks: Some(&blocks),
        ..Default::default()
    };
    let r = evaluate_assertion(
        &Assertion::ContainsText {
            target: TextTarget::Region(assertions::RegionTextTarget {
                region: region(0, 0, 100, 40),
            }),
            text: "Save changes".to_string(),
            kind: TextMatchKind::Contains,
        },
        &ctx,
    );
    assert_eq!(r.confidence, Some(0.55));
}

/// `contrast_meets_wcag` reads DECLARED colours only and never samples, so
/// its verdicts are exact — the `None` is accurate, not an oversight.
#[test]
fn an_exact_assertion_states_that_it_is_a_deduction() {
    let mut e = el("btn", 0);
    e.fg_color = Some(Rgb::new(0, 0, 0));
    e.bg_color = Some(Rgb::new(255, 255, 255));
    let snap = snapshot(vec![e]);
    let ctx = EvalContext {
        snapshot: Some(&snap),
        ..Default::default()
    };

    let r = evaluate_assertion(
        &Assertion::ContrastMeetsWcag {
            element: "btn".to_string(),
            level: WcagLevel::Aa,
        },
        &ctx,
    );
    assert_eq!(
        r.outcome,
        AssertionOutcome::Passed,
        "detail: {:?}",
        r.detail
    );
    assert_eq!(r.confidence, None);

    // And a geometry assertion, for the same reason.
    let snap2 = snapshot(vec![el("a", 0), el("b", 10)]);
    let ctx2 = EvalContext {
        snapshot: Some(&snap2),
        ..Default::default()
    };
    let r2 = evaluate_assertion(
        &Assertion::NoOverlap {
            elements: ["a".into(), "b".into()],
            tolerance_px: None,
        },
        &ctx2,
    );
    assert_eq!(r2.confidence, None);
}

// -------------------------------------------------- 3. the wire (phase 4)

/// The load-bearing one. `None` says "this was a deduction, not an estimate",
/// and a statement that is omitted from the wire is not a statement.
#[test]
fn a_none_confidence_is_on_the_wire_and_an_absent_one_is_a_different_document() {
    let f = Finding::new("overlap", Severity::Warning, "two boxes intersect");
    let v = serde_json::to_value(&f).unwrap();
    let obj = v.as_object().expect("Finding serializes as an object");

    assert!(
        obj.contains_key("confidence"),
        "a None confidence was SKIPPED on the wire, collapsing \
         'this is a deduction' into 'nobody said' — the exact distinction \
         the field exists to make. Do not add skip_serializing_if here."
    );
    assert!(obj["confidence"].is_null());

    // The asymmetry a consumer actually faces, asserted as an asymmetry:
    // the two documents are DISTINGUISHABLE as JSON (a reader can tell a
    // stated deduction from an unstated confidence) and INDISTINGUISHABLE
    // as Rust values (both parse to `None`, because there is no third state
    // in the type). Both halves matter — the first is the property the doc
    // comment claims, the second is the caveat it must not pretend away.
    let mut absent_obj = obj.clone();
    absent_obj.remove("confidence");
    let absent = serde_json::Value::Object(absent_obj);
    assert_ne!(
        absent, v,
        "stated-null and absent must not serialize identically"
    );
    let from_null: Finding = serde_json::from_value(v.clone()).unwrap();
    let from_absent: Finding = serde_json::from_value(absent).unwrap();
    assert_eq!(from_null.confidence, None);
    assert_eq!(from_absent.confidence, None);

    // The asymmetry with `analyzer` is deliberate: ITS None is a genuine
    // UNKNOWN (produced outside the dispatcher), so it is omitted rather
    // than asserted as null.
    assert!(
        !obj.contains_key("analyzer"),
        "an unattributed finding must not claim `analyzer: null` as a statement"
    );
}

#[test]
fn finding_round_trips_its_analyzer_and_confidence() {
    let f = Finding::new("low_contrast", Severity::Info, "sampled")
        .with_region(region(1, 2, 3, 4))
        .with_elements(vec!["a".to_string()])
        .with_confidence(0.62);
    let stamped = analyzers::run(
        Analyzer::Color,
        &AnalyzeInput {
            frame: None,
            snapshot: None,
            prior_frame: None,
        },
    );
    // Sanity: the dispatcher's own blocked finding is attributed, and that
    // is the shape we round-trip below.
    assert_eq!(stamped.findings[0].analyzer, Some(Analyzer::Color));

    let json = serde_json::to_string(&f).unwrap();
    let back: Finding = serde_json::from_str(&json).unwrap();
    assert_eq!(back.confidence, Some(0.62));
    assert_eq!(back.analyzer, None);
    assert_eq!(back.kind, "low_contrast");
    assert_eq!(back.region, Some(region(1, 2, 3, 4)));

    let attributed = serde_json::to_string(&stamped.findings[0]).unwrap();
    assert!(
        attributed.contains("\"analyzer\":\"color\""),
        "attribution must reach the wire in the analyzer's snake_case wire name: {attributed}"
    );
    let back2: Finding = serde_json::from_str(&attributed).unwrap();
    assert_eq!(back2.analyzer, Some(Analyzer::Color));
    assert_eq!(back2.confidence, None);
}

/// The whole `AnalyzerResult` round-trips with attribution intact — this is
/// the value the runner's `/vision/analyze` actually serializes.
#[test]
fn an_analyzer_result_round_trips_with_its_findings_attributed() {
    let snap = snapshot(vec![el("a", 0), el("b", 10)]);
    let result = analyzers::run(
        Analyzer::Layout,
        &AnalyzeInput {
            frame: None,
            snapshot: Some(&snap),
            prior_frame: None,
        },
    );
    let json = serde_json::to_string(&result).unwrap();
    let back: analyzers::AnalyzerResult = serde_json::from_str(&json).unwrap();

    assert_eq!(back.findings.len(), result.findings.len());
    assert!(
        !back.findings.is_empty(),
        "the fixture stopped producing findings, so the loop below asserts nothing"
    );
    for f in &back.findings {
        assert_eq!(f.analyzer, Some(Analyzer::Layout));
        assert_eq!(f.confidence, None);
    }
}

#[test]
fn assertion_result_states_a_none_confidence_on_the_wire() {
    let snap = snapshot(vec![el("a", 0), el("b", 500)]);
    let ctx = EvalContext {
        snapshot: Some(&snap),
        ..Default::default()
    };
    let r = evaluate_assertion(
        &Assertion::NoOverlap {
            elements: ["a".into(), "b".into()],
            tolerance_px: None,
        },
        &ctx,
    );

    let v = serde_json::to_value(&r).unwrap();
    let obj = v.as_object().unwrap();
    assert!(
        obj.contains_key("confidence"),
        "AssertionResult skipped its None confidence — see the Finding test"
    );
    assert!(obj["confidence"].is_null());
}

#[test]
fn assertion_result_round_trips_its_confidence_through_the_wire_shim() {
    let blocks = [OcrBlockRef {
        bbox: region(0, 0, 40, 40),
        text: "Save",
        confidence: 0.77,
    }];
    let ctx = EvalContext {
        ocr_blocks: Some(&blocks),
        ..Default::default()
    };
    let r = evaluate_assertion(
        &Assertion::ContainsText {
            target: TextTarget::Region(assertions::RegionTextTarget {
                region: region(0, 0, 100, 40),
            }),
            text: "Save".to_string(),
            kind: TextMatchKind::Contains,
        },
        &ctx,
    );
    assert_eq!(r.confidence, Some(0.77));

    let json = serde_json::to_string(&r).unwrap();
    // `AssertionResult` deserializes through `AssertionResultWire`, so this
    // is the path that would silently drop the field if the shim forgot it.
    let back: AssertionResult = serde_json::from_str(&json).unwrap();
    assert_eq!(back.confidence, Some(0.77));
    assert_eq!(back.outcome, AssertionOutcome::Passed);
    assert!(back.passed);
}

/// Append-only: a payload written before the field existed still parses, and
/// reads as "nobody stated a confidence".
#[test]
fn a_payload_predating_the_field_still_parses() {
    let legacy = r#"{
        "passed": true,
        "outcome": "passed",
        "assertion": {"type": "no_overlap", "elements": ["a", "b"]}
    }"#;
    let back: AssertionResult = serde_json::from_str(legacy).unwrap();
    assert_eq!(back.outcome, AssertionOutcome::Passed);
    assert_eq!(back.confidence, None);

    let legacy_finding = r#"{
        "kind": "overlap",
        "severity": "warning",
        "detail": "two boxes intersect"
    }"#;
    let f: Finding = serde_json::from_str(legacy_finding).unwrap();
    assert_eq!(f.analyzer, None);
    assert_eq!(f.confidence, None);
}
