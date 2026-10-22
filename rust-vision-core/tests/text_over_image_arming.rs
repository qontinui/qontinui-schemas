//! The arming test for `text_over_image_contrast` (plan
//! 2026-10-13-image-generation-and-visual-judgment, Phase 16): the assertion
//! reached the way its population reaches it — a wire assertion
//! (`{"type": "text_over_image_contrast", ...}`, as the CLI's assertions file
//! and the runner's `vision/assert` body carry it) handed to the evaluator.
//!
//! Written against the API the crate had BEFORE the phase, deliberately: it
//! names no type, field or variant the phase added, so with the change
//! reverted it still compiles and goes RED at run time (the wire type is
//! unknown), and with the change it is GREEN. The measurement itself is
//! covered by `tests/text_over_image.rs`, which needs the new
//! `EvalContext::text_hidden_frame` and so cannot serve as the arming test.

use qontinui_vision_core::assertions::{Assertion, AssertionOutcome, EvalContext};
use qontinui_vision_core::{Element, ElementMedia, ElementSnapshot, Frame, FrameSource};
use qontinui_vision_core::{MediaKind, ObjectFit, Region, Rgb};

#[test]
fn a_wire_text_over_image_contrast_is_evaluated_and_needs_the_text_hidden_frame() {
    let assertion: Assertion = serde_json::from_value(serde_json::json!({
        "type": "text_over_image_contrast",
        "min_ratio": 4.5,
    }))
    .expect("the wire assertion parses");
    let snapshot = ElementSnapshot {
        elements: vec![
            Element {
                id: "hero".into(),
                bbox: Some(Region {
                    x: 0,
                    y: 0,
                    w: 200,
                    h: 100,
                }),
                media: Some(ElementMedia {
                    object_fit: Some(ObjectFit::Fill),
                    ..ElementMedia::new(MediaKind::Img)
                }),
                ..Default::default()
            },
            Element {
                id: "heading".into(),
                bbox: Some(Region {
                    x: 20,
                    y: 30,
                    w: 120,
                    h: 30,
                }),
                text: Some("Summer sale".into()),
                fg_color: Some(Rgb::new(255, 255, 255)),
                ..Default::default()
            },
        ],
        ..Default::default()
    };
    let frame = Frame::from_rgba(
        image::RgbaImage::from_pixel(200, 100, image::Rgba([40, 44, 52, 255])),
        FrameSource::synthetic_now(),
    );
    let r = qontinui_vision_core::assertions::evaluate(
        &assertion,
        &EvalContext {
            snapshot: Some(&snapshot),
            frame: Some(&frame),
            ..Default::default()
        },
    );
    // Not a pass: without the text-hidden capture nothing can be measured,
    // and the typed code says which input to supply.
    assert_eq!(r.outcome, AssertionOutcome::Unknown, "{:?}", r.detail);
    let wire = serde_json::to_value(&r).unwrap();
    assert_eq!(wire["code"], "needs_multi_frame_input", "{wire}");
    assert_eq!(wire["assertion"]["type"], "text_over_image_contrast");
}
