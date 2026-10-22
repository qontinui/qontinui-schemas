//! Every `Blocked` and `Degraded` verdict carries a typed [`UnknownCode`].
//!
//! PER ANALYZER, not aggregate (plan `489ed69e` lesson 6): an aggregate
//! "every non-Checked verdict in this run has a code" is satisfied by a
//! fixture that drives one analyzer into one arm and leaves the rest
//! unexercised. Each test below drives ONE analyzer into EACH of the
//! blocked/degraded arms it can emit — its own arms and the dispatcher's
//! missing-input arm — asserts it actually landed in that arm (so a fixture
//! that drifts into `Checked` fails loudly rather than passing vacuously),
//! and asserts the code, in memory and on the wire.

use qontinui_vision_core::{
    analyzers, AnalyzeInput, Analyzer, AnalyzerResult, AnalyzerVerdict, Element, ElementMedia,
    ElementSnapshot, Frame, FrameSource, MediaKind, Region, Rgb, UnknownCode,
};
use serde_json::json;

// ---------------------------------------------------------------- fixtures

fn region(x: i32, y: i32, w: u32, h: u32) -> Region {
    Region { x, y, w, h }
}

fn frame(w: u32, h: u32) -> Frame {
    Frame::from_rgba(
        image::RgbaImage::from_pixel(w, h, image::Rgba([0x80, 0x80, 0x80, 0xff])),
        FrameSource::synthetic_now(),
    )
}

fn snapshot(elements: Vec<Element>) -> ElementSnapshot {
    ElementSnapshot {
        elements,
        ..Default::default()
    }
}

fn bare(id: &str) -> Element {
    Element {
        id: id.to_string(),
        ..Default::default()
    }
}

fn run(
    a: Analyzer,
    frame: Option<&Frame>,
    snap: Option<&ElementSnapshot>,
    prior: Option<&Frame>,
) -> AnalyzerResult {
    analyzers::run(
        a,
        &AnalyzeInput {
            frame,
            snapshot: snap,
            prior_frame: prior,
            text_hidden_frame: None,
            text_hidden_ids: None,
        },
    )
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum Arm {
    Blocked,
    Degraded,
}

/// The one assertion every case makes: the result landed in `arm` (not
/// `Checked`, not the other arm), carries `code`, and says so on the wire.
fn assert_arm(label: &str, r: &AnalyzerResult, arm: Arm, code: UnknownCode) {
    let landed = match &r.verdict {
        AnalyzerVerdict::Blocked { .. } => Some(Arm::Blocked),
        AnalyzerVerdict::Degraded { .. } => Some(Arm::Degraded),
        AnalyzerVerdict::Checked => None,
    };
    assert_eq!(
        landed,
        Some(arm),
        "{label}: the fixture did not reach the {arm:?} arm (got {:?}) — fix the fixture, \
         not the assertion",
        r.verdict
    );
    assert_eq!(
        r.verdict.code(),
        Some(code),
        "{label}: wrong or missing code"
    );
    let wire = serde_json::to_value(&r.verdict).unwrap();
    assert_eq!(
        wire["code"],
        json!(code.as_str()),
        "{label}: code missing on the wire: {wire}"
    );
    assert!(
        wire["reason"].as_str().is_some_and(|s| !s.is_empty()),
        "{label}: the human reason must survive beside the code: {wire}"
    );
    // The derived gate bit is untouched by the code.
    assert_eq!(
        r.conclusive,
        arm == Arm::Degraded,
        "{label}: gate bit changed"
    );
}

// ---------------------------------------------------------------- per analyzer

#[test]
fn layout_every_non_checked_arm_carries_a_code() {
    // Dispatcher: no snapshot.
    let r = run(Analyzer::Layout, None, None, None);
    assert_arm(
        "layout/no-snapshot",
        &r,
        Arm::Blocked,
        UnknownCode::InputMissing,
    );

    // Own blocked arm: no element carries a bbox.
    let s = snapshot(vec![bare("a"), bare("b")]);
    let r = run(Analyzer::Layout, None, Some(&s), None);
    assert_arm(
        "layout/no-bbox",
        &r,
        Arm::Blocked,
        UnknownCode::InputMissing,
    );

    // Own degraded arm #1: geometry present, nothing interactable.
    let mut a = bare("a");
    a.bbox = Some(region(0, 0, 50, 50));
    let mut b = bare("b");
    b.bbox = Some(region(100, 0, 50, 50));
    let s = snapshot(vec![a, b]);
    let r = run(Analyzer::Layout, None, Some(&s), None);
    assert_arm(
        "layout/no-interactable",
        &r,
        Arm::Degraded,
        UnknownCode::InputMissing,
    );

    // Own degraded arm #2: interactable, intersecting, no stacking order.
    let mut a = bare("a");
    a.bbox = Some(region(0, 0, 50, 50));
    a.interactable = true;
    let mut b = bare("b");
    b.bbox = Some(region(25, 25, 50, 50));
    b.interactable = true;
    let s = snapshot(vec![a, b]);
    let r = run(Analyzer::Layout, None, Some(&s), None);
    assert_arm(
        "layout/no-stacking",
        &r,
        Arm::Degraded,
        UnknownCode::InputMissing,
    );
    assert!(
        r.verdict.reason().unwrap().contains("stacking"),
        "reached the degraded arm by the wrong route: {:?}",
        r.verdict
    );
}

#[test]
fn typography_every_non_checked_arm_carries_a_code() {
    let r = run(Analyzer::Typography, None, None, None);
    assert_arm(
        "typography/no-snapshot",
        &r,
        Arm::Blocked,
        UnknownCode::InputMissing,
    );

    // Own blocked arm: no element carries text.
    let s = snapshot(vec![bare("a")]);
    let r = run(Analyzer::Typography, None, Some(&s), None);
    assert_arm(
        "typography/no-text",
        &r,
        Arm::Blocked,
        UnknownCode::InputMissing,
    );
}

#[test]
fn color_every_non_checked_arm_carries_a_code() {
    let f = frame(200, 60);

    // Dispatcher: snapshot without a frame.
    let s = snapshot(vec![bare("a")]);
    let r = run(Analyzer::Color, None, Some(&s), None);
    assert_arm(
        "color/no-frame",
        &r,
        Arm::Blocked,
        UnknownCode::InputMissing,
    );

    // Own blocked arm #1: no element carries text.
    let r = run(Analyzer::Color, Some(&f), Some(&s), None);
    assert_arm("color/no-text", &r, Arm::Blocked, UnknownCode::InputMissing);

    // Own blocked arm #2: text, but neither declared colours nor a
    // sampleable bbox, so no ratio could be computed.
    let mut t = bare("t");
    t.text = Some("label".into());
    let s = snapshot(vec![t.clone()]);
    let r = run(Analyzer::Color, Some(&f), Some(&s), None);
    assert_arm(
        "color/no-ratio",
        &r,
        Arm::Blocked,
        UnknownCode::InputMissing,
    );

    // Own degraded arm: one text element measurable (declared colours), one
    // not.
    let mut declared = bare("d");
    declared.text = Some("ok".into());
    declared.fg_color = Some(Rgb::new(0, 0, 0));
    declared.bg_color = Some(Rgb::new(255, 255, 255));
    let s = snapshot(vec![declared, t]);
    let r = run(Analyzer::Color, Some(&f), Some(&s), None);
    assert_arm(
        "color/partial",
        &r,
        Arm::Degraded,
        UnknownCode::InputMissing,
    );
}

#[test]
fn dynamic_every_non_checked_arm_carries_a_code() {
    // Dispatcher: no prior frame.
    let cur = frame(10, 10);
    let r = run(Analyzer::Dynamic, Some(&cur), None, None);
    assert_arm(
        "dynamic/no-prior",
        &r,
        Arm::Blocked,
        UnknownCode::InputMissing,
    );

    // Own blocked arm: two zero-pixel frames.
    let empty_a = frame(0, 0);
    let empty_b = frame(0, 0);
    let r = run(Analyzer::Dynamic, Some(&empty_b), None, Some(&empty_a));
    assert_arm(
        "dynamic/zero-pixels",
        &r,
        Arm::Blocked,
        UnknownCode::InputMissing,
    );
}

#[test]
fn elements_every_non_checked_arm_carries_a_code() {
    let r = run(Analyzer::Elements, None, None, None);
    assert_arm(
        "elements/no-snapshot",
        &r,
        Arm::Blocked,
        UnknownCode::InputMissing,
    );

    // Own degraded arm: no element carries a bbox, so target size is
    // unmeasured.
    let mut a = bare("a");
    a.interactable = true;
    let s = snapshot(vec![a]);
    let r = run(Analyzer::Elements, None, Some(&s), None);
    assert_arm(
        "elements/no-bbox",
        &r,
        Arm::Degraded,
        UnknownCode::InputMissing,
    );
}

#[test]
fn image_every_non_checked_arm_carries_a_code() {
    let f = frame(200, 60);

    // Dispatcher: no frame — the image analyzer's ONLY blocked arm.
    let s = snapshot(vec![bare("a")]);
    let r = run(Analyzer::Image, None, Some(&s), None);
    assert_arm(
        "image/no-frame",
        &r,
        Arm::Blocked,
        UnknownCode::InputMissing,
    );

    // Own degraded arm #1: a frame and no snapshot at all.
    let r = run(Analyzer::Image, Some(&f), None, None);
    assert_arm(
        "image/no-snapshot",
        &r,
        Arm::Degraded,
        UnknownCode::InputMissing,
    );

    // Own degraded arm #2: no element carries media.
    let r = run(Analyzer::Image, Some(&f), Some(&s), None);
    assert_arm(
        "image/no-media",
        &r,
        Arm::Degraded,
        UnknownCode::InputMissing,
    );

    // Own degraded arm #3: a media element that cannot be located on the
    // frame (no bbox).
    let mut m = bare("img");
    m.media = Some(ElementMedia::new(MediaKind::Img));
    let s = snapshot(vec![m]);
    let r = run(Analyzer::Image, Some(&f), Some(&s), None);
    assert_arm(
        "image/unlocated",
        &r,
        Arm::Degraded,
        UnknownCode::InputMissing,
    );
    assert!(
        r.verdict.reason().unwrap().contains("could not be read"),
        "reached the degraded arm by the wrong route: {:?}",
        r.verdict
    );

    // Own degraded arm #4 (Phase 16): every image located, but a text-hidden
    // frame was supplied that cannot be paired with the frame (another size).
    let mut m = bare("img");
    m.bbox = Some(region(10, 10, 60, 30));
    m.media = Some(ElementMedia {
        object_fit: Some(qontinui_vision_core::ObjectFit::Fill),
        ..ElementMedia::new(MediaKind::Img)
    });
    let s = snapshot(vec![m]);
    let r = run(Analyzer::Image, Some(&f), Some(&s), None);
    assert_eq!(r.verdict, AnalyzerVerdict::Checked, "the fixture's control");
    let other = frame(100, 60);
    let r = analyzers::run(
        Analyzer::Image,
        &AnalyzeInput {
            frame: Some(&f),
            snapshot: Some(&s),
            text_hidden_frame: Some(&other),
            ..Default::default()
        },
    );
    assert_arm(
        "image/text-hidden-unpaired",
        &r,
        Arm::Degraded,
        UnknownCode::InputMissing,
    );
    assert!(
        r.verdict.reason().unwrap().contains("text-hidden frame"),
        "reached the degraded arm by the wrong route: {:?}",
        r.verdict
    );
}

/// The assertion half of the multi-frame code (Phase 16): without the
/// text-hidden twin, `text_over_image_contrast` answers
/// `needs_multi_frame_input`, in memory and on the wire, and the code is in
/// the closed vocabulary.
#[test]
fn text_over_image_contrast_without_its_twin_needs_multi_frame_input() {
    assert!(UnknownCode::ALL.contains(&UnknownCode::NeedsMultiFrameInput));
    let f = frame(20, 20);
    let s = snapshot(vec![bare("a")]);
    let r = qontinui_vision_core::evaluate_assertion(
        &qontinui_vision_core::Assertion::TextOverImageContrast {
            min_ratio: Some(4.5),
            percentile: None,
        },
        &qontinui_vision_core::EvalContext {
            snapshot: Some(&s),
            frame: Some(&f),
            ..Default::default()
        },
    );
    assert_eq!(r.code, Some(UnknownCode::NeedsMultiFrameInput));
    let wire = serde_json::to_value(&r).unwrap();
    assert_eq!(wire["outcome"], json!("unknown"));
    assert_eq!(wire["code"], json!("needs_multi_frame_input"));
    assert_eq!(wire["assertion"]["type"], json!("text_over_image_contrast"));
}

// ---------------------------------------------------------------- wire

/// The wire form keeps `state` as the tag and adds `code` beside `reason`.
#[test]
fn verdict_wire_shape_is_state_code_reason() {
    let r = run(Analyzer::Layout, None, None, None);
    let v = serde_json::to_value(&r.verdict).unwrap();
    assert_eq!(v["state"], json!("blocked"));
    assert_eq!(v["code"], json!("input_missing"));
    assert_eq!(
        serde_json::to_value(AnalyzerVerdict::Checked).unwrap(),
        json!({"state": "checked"})
    );
}

/// A `blocked` or `degraded` payload with no `code` does not say why in a
/// form an agent can branch on, and is refused.
#[test]
fn a_verdict_without_a_code_fails_to_deserialize() {
    for state in ["blocked", "degraded"] {
        let v = json!({"state": state, "reason": "no bbox"});
        assert!(
            serde_json::from_value::<AnalyzerVerdict>(v).is_err(),
            "a `{state}` verdict with no code parsed"
        );
    }
    // And the derived gate bit is still recomputed through the result shim.
    let r: AnalyzerResult = serde_json::from_value(json!({
        "conclusive": true,
        "verdict": {"state": "blocked", "code": "input_missing", "reason": "no bbox"},
        "findings": []
    }))
    .unwrap();
    assert!(
        !r.conclusive,
        "the wire's `conclusive: true` must not be trusted"
    );
}
