//! A DPR-2 frame, entered where every caller enters the analyzers: the public
//! dispatcher. The colour analyzer must read the pixels a snapshot box was
//! actually rendered on — the box's CSS coordinates times the frame's scale
//! factor — and not the box read as if CSS pixels were buffer pixels.
//!
//! The buffer below is built so the two readings disagree: where the caption
//! really rendered (buffer 80..120) it is a low-contrast pair, near 1.4:1;
//! at the unscaled spot (buffer 40..60) it is black text on white, 21:1. Only
//! a sampler that scales the box finds the low contrast.
//!
//! Plan 2026-10-13-image-generation-and-visual-judgment, Phase 9.

use qontinui_vision_core::analyzers::{self, AnalyzeInput, Analyzer, Severity};
use qontinui_vision_core::element_snapshot::{Element, ElementSnapshot};
use qontinui_vision_core::frame::{Frame, FrameSource, Region};

#[test]
fn a_dpr2_caption_is_sampled_where_it_rendered() {
    // A 100x100 CSS page captured at DPR 2: a 200x200 white buffer.
    let mut img = image::RgbaImage::from_pixel(200, 200, image::Rgba([0xff, 0xff, 0xff, 0xff]));
    // The unscaled spot: a black band on white.
    for y in 40..45u32 {
        for x in 40..60u32 {
            img.put_pixel(x, y, image::Rgba([0, 0, 0, 0xff]));
        }
    }
    // Where the caption rendered at DPR 2: #888 with a #777 band.
    for y in 80..120u32 {
        for x in 80..120u32 {
            let v = if y < 90 { 0x77 } else { 0x88 };
            img.put_pixel(x, y, image::Rgba([v, v, v, 0xff]));
        }
    }
    let mut source = FrameSource::synthetic_now();
    source.scale_factor = 2.0;
    let frame = Frame::from_rgba(img, source);

    // The caption's box in CSS pixels, with no declared colours, so its
    // contrast can only come from the frame.
    let snap = ElementSnapshot {
        elements: vec![Element {
            id: "hero-caption".to_string(),
            bbox: Some(Region {
                x: 40,
                y: 40,
                w: 20,
                h: 20,
            }),
            text: Some("caption".to_string()),
            ..Default::default()
        }],
        ..Default::default()
    };

    let input = AnalyzeInput {
        frame: Some(&frame),
        snapshot: Some(&snap),
        prior_frame: None,
        text_hidden_frame: None,
        text_hidden_ids: None,
    };
    let result = analyzers::run(Analyzer::Color, &input);

    let low: Vec<_> = result
        .findings
        .iter()
        .filter(|f| f.kind == "low_contrast")
        .collect();
    assert_eq!(
        low.len(),
        1,
        "the low-contrast pair where the caption rendered must be found: {:?}",
        result.findings
    );
    assert_eq!(low[0].severity, Severity::Info);
    assert_eq!(low[0].elements, vec!["hero-caption".to_string()]);
    assert!(
        low[0].detail.contains("ratio≈1.4"),
        "sampled #888/#777, not black on white: {}",
        low[0].detail
    );
}
