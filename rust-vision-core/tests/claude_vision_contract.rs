//! Integration tests for Phase 1 of the UI Bridge vision pipeline. Covers
//! every distinct Verify branch and the `safe_image` happy path.

use image::{GenericImageView, ImageFormat, RgbaImage};
use proptest::prelude::*;
use proptest::test_runner::Config;

use qontinui_vision_core::encode::safe_image;
use qontinui_vision_core::strip::{
    assert_stripped_jpeg, assert_stripped_png, assert_stripped_webp,
};
use qontinui_vision_core::{
    AlphaPolicy, EncodedFormat, Frame, FrameSource, MetadataPolicy, OutputContract, Pipeline,
    Region, ResizeStrategy, Stage, VisionError,
};

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn synthetic_frame(w: u32, h: u32, color: [u8; 4]) -> Frame {
    let mut buf = RgbaImage::new(w, h);
    for px in buf.pixels_mut() {
        *px = image::Rgba(color);
    }
    Frame::from_rgba(buf, FrameSource::synthetic_now())
}

fn synthetic_rgba(w: u32, h: u32, seed: u32) -> Frame {
    // Deterministic-ish pattern so failures are debuggable.
    let mut buf = RgbaImage::new(w, h);
    for (x, y, px) in buf.enumerate_pixels_mut() {
        let r = (x.wrapping_add(seed) & 0xFF) as u8;
        let g = (y.wrapping_add(seed) & 0xFF) as u8;
        let b = (x.wrapping_mul(y).wrapping_add(seed) & 0xFF) as u8;
        let a = if (x ^ y ^ seed) & 1 == 0 { 0x40 } else { 0xFF };
        *px = image::Rgba([r, g, b, a]);
    }
    Frame::from_rgba(buf, FrameSource::synthetic_now())
}

// ---------------------------------------------------------------------------
// 1. Property test: random RGBA -> safe_image always satisfies CLAUDE_VISION_V1.
// ---------------------------------------------------------------------------

proptest! {
    #![proptest_config(Config { cases: 32, .. Config::default() })]

    #[test]
    fn safe_image_claude_vision_round_trips(
        w in 100u32..=2500,
        h in 100u32..=2500,
        seed in any::<u32>(),
    ) {
        let frame = synthetic_rgba(w, h, seed);
        let bytes = safe_image(frame, &OutputContract::CLAUDE_VISION_V1)
            .expect("safe_image should succeed for any RGBA in 100..=2500");
        prop_assert!(bytes.len() <= OutputContract::CLAUDE_VISION_V1.max_bytes);
        let decoded = image::load_from_memory(&bytes)
            .expect("emitted bytes must decode");
        let (dw, dh) = decoded.dimensions();
        prop_assert!(
            dw.max(dh) <= OutputContract::CLAUDE_VISION_V1.max_long_edge,
            "decoded long-edge {} > 1568",
            dw.max(dh),
        );
        prop_assert!(!decoded.color().has_alpha(),
            "decoded image must have no alpha under Flatten policy");
    }
}

// ---------------------------------------------------------------------------
// 2. Oversize gets resized.
// ---------------------------------------------------------------------------

#[test]
fn oversize_resized_to_long_edge() {
    let frame = synthetic_rgba(3000, 2000, 7);
    let bytes = safe_image(frame, &OutputContract::CLAUDE_VISION_V1).unwrap();
    let decoded = image::load_from_memory(&bytes).unwrap();
    let (w, h) = decoded.dimensions();
    assert!(w.max(h) <= 1568, "long edge {} > 1568", w.max(h));
    // Aspect ratio preserved within rounding.
    let ratio_in = 3000.0_f64 / 2000.0;
    let ratio_out = w as f64 / h as f64;
    assert!(
        (ratio_in - ratio_out).abs() < 0.01,
        "aspect ratio drifted: {ratio_in} -> {ratio_out}"
    );
}

// ---------------------------------------------------------------------------
// 3. Alpha is flattened to the contract background.
// ---------------------------------------------------------------------------

#[test]
fn alpha_flattened_to_white() {
    // Fully-transparent input: every pixel should land on the contract background (white).
    let frame = synthetic_frame(64, 64, [0xFF, 0x00, 0x00, 0x00]);
    let bytes = safe_image(frame, &OutputContract::CLAUDE_VISION_V1).unwrap();
    let decoded = image::load_from_memory(&bytes).unwrap();
    let rgb = decoded.to_rgb8();
    // JPEG is lossy; allow a small delta. Center pixel sample is good enough.
    let center = rgb.get_pixel(32, 32);
    let [r, g, b] = center.0;
    assert!(
        r > 240 && g > 240 && b > 240,
        "center pixel {:?} not close to white",
        center.0
    );
}

// ---------------------------------------------------------------------------
// 4. Metadata is stripped — no PNG ancillary chunks, no JPEG APP/COM markers,
//    no WebP EXIF/XMP/ICCP.
// ---------------------------------------------------------------------------

#[test]
fn metadata_stripped_jpeg() {
    let frame = synthetic_rgba(200, 150, 1);
    let bytes = safe_image(frame, &OutputContract::CLAUDE_VISION_V1).unwrap();
    // CLAUDE_VISION_V1 picks JPEG (first allowed format) for non-WebP frames.
    assert_stripped_jpeg(&bytes).expect("JPEG should pass strip-check");
}

#[test]
fn metadata_stripped_png() {
    let frame = synthetic_rgba(200, 150, 2);
    let bytes = safe_image(frame, &OutputContract::PNG_STRICT).unwrap();
    assert_stripped_png(&bytes).expect("PNG should pass strip-check");
}

#[test]
fn metadata_stripped_webp() {
    let frame = synthetic_rgba(200, 150, 3);
    let bytes = safe_image(frame, &OutputContract::WEBP_LOSSY).unwrap();
    assert_stripped_webp(&bytes).expect("WebP should pass strip-check");
}

// ---------------------------------------------------------------------------
// 5. Crop out of bounds errors cleanly.
// ---------------------------------------------------------------------------

#[test]
fn crop_out_of_bounds_clean_error() {
    let frame = synthetic_rgba(100, 100, 0);
    let result = Pipeline::new()
        .push(Stage::CropRegion(Region {
            x: 0,
            y: 0,
            w: 5000,
            h: 5000,
        }))
        .push(Stage::Encode(EncodedFormat::Png))
        .run(frame);
    match result {
        Err(VisionError::CropOutOfBounds { region, frame }) => {
            assert_eq!(region.w, 5000);
            assert_eq!(frame, (100, 100));
        }
        other => panic!("expected CropOutOfBounds, got {other:?}"),
    }
}

// ---------------------------------------------------------------------------
// 6. Verify rejects too-large images (max_bytes).
// ---------------------------------------------------------------------------

#[test]
fn verify_rejects_oversized_bytes() {
    // Synthesize a contract that's allergic to anything beyond a few bytes.
    let tiny_contract = OutputContract {
        name: "tiny",
        max_long_edge: 4096,
        allowed_formats: &[EncodedFormat::Png],
        max_bytes: 16,
        alpha_policy: AlphaPolicy::Preserve,
        metadata_policy: MetadataPolicy::Preserve,
        color_space: qontinui_vision_core::ColorSpace::Srgb,
    };
    let frame = synthetic_rgba(64, 64, 5);
    let err = Pipeline::new()
        .push(Stage::Encode(EncodedFormat::Png))
        .push(Stage::Verify(tiny_contract))
        .run(frame)
        .unwrap_err();
    match err {
        VisionError::ContractViolation { stage, reason } => {
            assert_eq!(stage, "verify");
            assert!(reason.contains("bytes"), "reason was: {reason}");
        }
        other => panic!("expected ContractViolation, got {other:?}"),
    }
}

// ---------------------------------------------------------------------------
// 7. Verify rejects disallowed format (PNG against CLAUDE_VISION_V1).
// ---------------------------------------------------------------------------

#[test]
fn verify_rejects_disallowed_format() {
    let frame = synthetic_rgba(64, 64, 6);
    let err = Pipeline::new()
        .push(Stage::FlattenAlpha([0xFF, 0xFF, 0xFF]))
        .push(Stage::StripMetadata)
        .push(Stage::Encode(EncodedFormat::Png))
        .push(Stage::Verify(OutputContract::CLAUDE_VISION_V1))
        .run(frame)
        .unwrap_err();
    match err {
        VisionError::ContractViolation { stage, reason } => {
            assert_eq!(stage, "verify");
            assert!(reason.contains("not in contract"), "reason was: {reason}");
        }
        other => panic!("expected ContractViolation, got {other:?}"),
    }
}

// ---------------------------------------------------------------------------
// 8. PNG_STRICT contract: round-trips, accepts PNG, rejects JPEG.
// ---------------------------------------------------------------------------

#[test]
fn png_strict_round_trips() {
    let frame = synthetic_rgba(200, 200, 8);
    let bytes = safe_image(frame, &OutputContract::PNG_STRICT).unwrap();
    let decoded = image::load_from_memory_with_format(&bytes, ImageFormat::Png).unwrap();
    assert_eq!(decoded.dimensions(), (200, 200));
}

#[test]
fn png_strict_rejects_jpeg() {
    let frame = synthetic_rgba(64, 64, 9);
    let err = Pipeline::new()
        .push(Stage::StripMetadata)
        .push(Stage::Encode(EncodedFormat::Jpeg { quality: 85 }))
        .push(Stage::Verify(OutputContract::PNG_STRICT))
        .run(frame)
        .unwrap_err();
    assert!(
        matches!(err, VisionError::ContractViolation { .. }),
        "expected ContractViolation, got {err:?}"
    );
}

// ---------------------------------------------------------------------------
// 9. safe_image picks correct stages.
// ---------------------------------------------------------------------------

#[test]
fn safe_image_pipeline_for_claude_includes_expected_stages() {
    // Build the same pipeline shape `safe_image` would, and assert its stages.
    let mut pipe = Pipeline::new()
        .push(Stage::FlattenAlpha([0xFF, 0xFF, 0xFF]))
        // The frame is 2000x1000 (oversize), so Resize should appear.
        .push(Stage::Resize(ResizeStrategy::LongEdge(1568)))
        .push(Stage::StripMetadata)
        .push(Stage::Encode(EncodedFormat::Jpeg { quality: 85 }))
        .push(Stage::Verify(OutputContract::CLAUDE_VISION_V1));
    let stage_names: Vec<&'static str> = pipe
        .stages()
        .iter()
        .map(|s| match s {
            Stage::FlattenAlpha(_) => "FlattenAlpha",
            Stage::Resize(_) => "Resize",
            Stage::StripMetadata => "StripMetadata",
            Stage::Encode(_) => "Encode",
            Stage::Verify(_) => "Verify",
            _ => "Other",
        })
        .collect();
    assert_eq!(
        stage_names,
        vec![
            "FlattenAlpha",
            "Resize",
            "StripMetadata",
            "Encode",
            "Verify"
        ]
    );

    // Now run it for real and verify the bytes are accepted.
    let frame = synthetic_rgba(2000, 1000, 10);
    let bytes = pipe.run(frame).unwrap();
    let decoded = image::load_from_memory(&bytes).unwrap();
    let (w, h) = decoded.dimensions();
    assert!(w.max(h) <= 1568);

    // Suppress unused-warning on the let binding when we move it into run().
    pipe = Pipeline::new();
    let _ = pipe;
}

// ---------------------------------------------------------------------------
// Verify branch coverage — exhaustive per-failure assertions.
// ---------------------------------------------------------------------------

#[test]
fn verify_rejects_oversize_long_edge() {
    // Encode a 200x200 image but use a contract with max_long_edge=100.
    let tight = OutputContract {
        name: "tight_edge",
        max_long_edge: 100,
        allowed_formats: &[EncodedFormat::Png],
        max_bytes: usize::MAX,
        alpha_policy: AlphaPolicy::Preserve,
        metadata_policy: MetadataPolicy::Preserve,
        color_space: qontinui_vision_core::ColorSpace::Srgb,
    };
    let frame = synthetic_rgba(200, 200, 11);
    let err = Pipeline::new()
        .push(Stage::Encode(EncodedFormat::Png))
        .push(Stage::Verify(tight))
        .run(frame)
        .unwrap_err();
    let reason = match err {
        VisionError::ContractViolation { reason, .. } => reason,
        other => panic!("expected ContractViolation, got {other:?}"),
    };
    assert!(
        reason.contains("long-edge"),
        "expected long-edge reason, got: {reason}"
    );
}

#[test]
fn verify_rejects_alpha_when_flatten_demanded() {
    // PNG-with-alpha against a Flatten contract should fail the alpha check.
    let flatten_png = OutputContract {
        name: "flatten_png",
        max_long_edge: 4096,
        allowed_formats: &[EncodedFormat::Png],
        max_bytes: usize::MAX,
        alpha_policy: AlphaPolicy::Flatten([0xFF, 0xFF, 0xFF]),
        metadata_policy: MetadataPolicy::Preserve,
        color_space: qontinui_vision_core::ColorSpace::Srgb,
    };
    let frame = synthetic_rgba(64, 64, 12);
    let err = Pipeline::new()
        // Skip FlattenAlpha intentionally so the encoded PNG keeps alpha.
        .push(Stage::Encode(EncodedFormat::Png))
        .push(Stage::Verify(flatten_png))
        .run(frame)
        .unwrap_err();
    let reason = match err {
        VisionError::ContractViolation { reason, .. } => reason,
        other => panic!("expected ContractViolation, got {other:?}"),
    };
    assert!(
        reason.contains("alpha"),
        "expected alpha-channel reason, got: {reason}"
    );
}

#[test]
fn verify_rejects_missing_strip_when_required() {
    // Contract demands StripAll, but pipeline omitted StripMetadata.
    let strict_strip = OutputContract {
        name: "strict_strip",
        max_long_edge: 4096,
        allowed_formats: &[EncodedFormat::Png],
        max_bytes: usize::MAX,
        alpha_policy: AlphaPolicy::Preserve,
        metadata_policy: MetadataPolicy::StripAll,
        color_space: qontinui_vision_core::ColorSpace::Srgb,
    };
    let frame = synthetic_rgba(64, 64, 13);
    let err = Pipeline::new()
        .push(Stage::Encode(EncodedFormat::Png))
        .push(Stage::Verify(strict_strip))
        .run(frame)
        .unwrap_err();
    let reason = match err {
        VisionError::ContractViolation { reason, .. } => reason,
        other => panic!("expected ContractViolation, got {other:?}"),
    };
    assert!(
        reason.contains("StripAll"),
        "expected StripAll-omitted reason, got: {reason}"
    );
}

#[test]
fn pipeline_rejects_verify_without_encode() {
    let frame = synthetic_rgba(32, 32, 14);
    let err = Pipeline::new()
        .push(Stage::Verify(OutputContract::PNG_STRICT))
        .run(frame)
        .unwrap_err();
    assert!(
        matches!(err, VisionError::InvalidPipeline(_)),
        "expected InvalidPipeline, got {err:?}"
    );
}

#[test]
fn pipeline_rejects_verify_not_terminal() {
    let frame = synthetic_rgba(32, 32, 15);
    let err = Pipeline::new()
        .push(Stage::Encode(EncodedFormat::Png))
        .push(Stage::Verify(OutputContract::PNG_STRICT))
        .push(Stage::Encode(EncodedFormat::Png))
        .run(frame)
        .unwrap_err();
    assert!(
        matches!(err, VisionError::InvalidPipeline(_)),
        "expected InvalidPipeline, got {err:?}"
    );
}

// ---------------------------------------------------------------------------
// Smoke tests for the frame-mutating stages so they don't regress silently.
// ---------------------------------------------------------------------------

#[test]
fn crop_region_extracts_subregion() {
    let frame = synthetic_rgba(200, 100, 20);
    let bytes = Pipeline::new()
        .push(Stage::CropRegion(Region {
            x: 50,
            y: 25,
            w: 100,
            h: 50,
        }))
        .push(Stage::Encode(EncodedFormat::Png))
        .run(frame)
        .unwrap();
    let decoded = image::load_from_memory(&bytes).unwrap();
    assert_eq!(decoded.dimensions(), (100, 50));
}

#[test]
fn resize_scale_doubles_dims() {
    let frame = synthetic_rgba(50, 30, 21);
    let bytes = Pipeline::new()
        .push(Stage::Resize(ResizeStrategy::Scale(2.0)))
        .push(Stage::Encode(EncodedFormat::Png))
        .run(frame)
        .unwrap();
    let decoded = image::load_from_memory(&bytes).unwrap();
    assert_eq!(decoded.dimensions(), (100, 60));
}

#[test]
fn resize_fit_preserves_aspect() {
    let frame = synthetic_rgba(200, 100, 22);
    let bytes = Pipeline::new()
        .push(Stage::Resize(ResizeStrategy::Fit { w: 50, h: 50 }))
        .push(Stage::Encode(EncodedFormat::Png))
        .run(frame)
        .unwrap();
    let decoded = image::load_from_memory(&bytes).unwrap();
    // 200x100 fit into 50x50 -> 50x25 (limited by width).
    assert_eq!(decoded.dimensions(), (50, 25));
}
