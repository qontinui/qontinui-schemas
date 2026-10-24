//! The `image` analyzer and the L1 image assertions, against held fixtures and
//! their mutations (plan 2026-10-13-image-generation-and-visual-judgment,
//! Phase 15 step 4).
//!
//! `tests/fixtures/image_corpus/` holds a HELD image (`base.png`) and one
//! mutation per defect class: an off-palette recolour, a soft
//! down-then-up-scaled copy, JPEG at quality 10, a near-duplicate, and a flat
//! placeholder (the cropped-subject mutation is a change of object-position,
//! so it lives in the snapshot rather than in a file). `generate.py` wrote
//! them, once, and recorded Python `imagehash`'s pHash and dHash of each PNG
//! in `expected_hashes.json`.
//!
//! Every mutation test asserts BOTH halves of the plan's requirement: the
//! analyzer's VALUE moves between the held fixture and the mutation, and the
//! ASSERTION over that value flips from passed to failed. A test that checked
//! only the flip would pass against an assertion that ignored the value, and
//! one that checked only the value would pass against an assertion that
//! never fails.

use std::path::PathBuf;

use qontinui_vision_core::analyzers::image::{pixels, ImageClass, ImageValues, NaturalRegion};
use qontinui_vision_core::assertions::{delta_e_2000, delta_e_2000_lab};
use qontinui_vision_core::{
    analyzers, evaluate_assertion, AnalyzeInput, Analyzer, AnalyzerResult, AnalyzerVerdict,
    Assertion, AssertionOutcome, AssertionResult, Element, ElementMedia, ElementSnapshot,
    EvalContext, Frame, FrameSource, MediaKind, MediaSize, ObjectFit, Region, Rgb, Severity,
    UnknownCode, VisibilityReason,
};

// ---------------------------------------------------------------- fixtures

fn fixture_path(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/image_corpus")
        .join(name)
}

fn load(name: &str) -> image::RgbaImage {
    image::open(fixture_path(name))
        .unwrap_or_else(|e| panic!("fixture {name}: {e}"))
        .to_rgba8()
}

const PAGE: [u8; 4] = [0xf8, 0xfa, 0xfc, 0xff];
const CREAM: Rgb = Rgb::new(254, 243, 199);
const NAVY: Rgb = Rgb::new(30, 58, 138);
const ORANGE: Rgb = Rgb::new(249, 115, 22);

/// A page frame with each image drawn 1:1 at its position (DPR 1).
fn page(w: u32, h: u32, bg: [u8; 4], images: &[(&image::RgbaImage, i64, i64)]) -> Frame {
    let mut buf = image::RgbaImage::from_pixel(w, h, image::Rgba(bg));
    for (img, x, y) in images {
        image::imageops::overlay(&mut buf, *img, *x, *y);
    }
    Frame::from_rgba(buf, FrameSource::synthetic_now())
}

/// An `<img>` element drawn at its natural size with `object-fit: fill`.
fn img_el(id: &str, x: i32, y: i32, img: &image::RgbaImage) -> Element {
    let (w, h) = img.dimensions();
    Element {
        id: id.into(),
        bbox: Some(Region { x, y, w, h }),
        media: Some(ElementMedia {
            natural_px: Some(MediaSize::new(w as f32, h as f32)),
            rendered_px: Some(MediaSize::new(w as f32, h as f32)),
            object_fit: Some(ObjectFit::Fill),
            ..ElementMedia::new(MediaKind::Img)
        }),
        ..Default::default()
    }
}

fn snapshot(elements: Vec<Element>) -> ElementSnapshot {
    ElementSnapshot {
        elements,
        ..Default::default()
    }
}

fn run_image(frame: Option<&Frame>, snap: Option<&ElementSnapshot>) -> AnalyzerResult {
    analyzers::run(
        Analyzer::Image,
        &AnalyzeInput {
            frame,
            snapshot: snap,
            prior_frame: None,
        },
    )
}

fn values(frame: &Frame, snap: &ElementSnapshot) -> ImageValues {
    run_image(Some(frame), Some(snap))
        .image_values
        .expect("a framed image run carries values")
}

fn assert_on(frame: &Frame, snap: &ElementSnapshot, a: Assertion) -> AssertionResult {
    evaluate_assertion(
        &a,
        &EvalContext {
            snapshot: Some(snap),
            frame: Some(frame),
            ..Default::default()
        },
    )
}

/// A held fixture and its mutation, through one assertion: the held one must
/// PASS and the mutation must FAIL, each with a confidence in range.
fn flips(held: &AssertionResult, mutated: &AssertionResult) {
    assert_eq!(
        held.outcome,
        AssertionOutcome::Passed,
        "held fixture: {:?}",
        held.detail
    );
    assert_eq!(
        mutated.outcome,
        AssertionOutcome::Failed,
        "mutation: {:?}",
        mutated.detail
    );
}

fn conf_in_range(r: &AssertionResult) {
    let c = r
        .confidence
        .expect("a frame-reading verdict states its confidence");
    assert!((0.0..=1.0).contains(&c), "confidence {c}");
}

// ======================================================== hash cross-check

/// The declared tolerance: in-crate pHash agrees with Python
/// `imagehash.phash` within this many of 64 bits on every shared fixture.
/// Not 0, because Pillow's Lanczos resampling and the `image` crate's differ
/// numerically, and a coefficient that sits near the median flips on a
/// rounding difference.
const PHASH_TOLERANCE: u32 = 4;
/// The same for dHash, whose 9×8 thumbnail compares NEIGHBOURING pixels, so a
/// one-level resampling difference between two near-equal neighbours flips a
/// bit outright. Measured on these fixtures (Pillow 13.0.0, imagehash 4.3.2,
/// image 0.25.10): pHash Δ0 on every one, dHash at most Δ5 (`base.png`).
const DHASH_TOLERANCE: u32 = 8;

#[test]
fn phash_and_dhash_agree_with_python_imagehash_within_the_declared_tolerance() {
    let doc: serde_json::Value = serde_json::from_str(
        &std::fs::read_to_string(fixture_path("expected_hashes.json")).unwrap(),
    )
    .unwrap();
    let hashes = doc["hashes"].as_object().expect("hashes object");
    assert!(hashes.len() >= 6, "the cross-check needs its fixtures");
    let mut report = Vec::new();
    for (name, want) in hashes {
        let rgba = load(name);
        let rgb = image::DynamicImage::ImageRgba8(rgba).to_rgb8();
        let luma = pixels::luma_image(&rgb);
        let ph = pixels::phash(&luma).unwrap();
        let dh = pixels::dhash(&luma).unwrap();
        let want_ph = pixels::parse_hash_hex(want["phash"].as_str().unwrap()).unwrap();
        let want_dh = pixels::parse_hash_hex(want["dhash"].as_str().unwrap()).unwrap();
        let (dp, dd) = (pixels::hamming(ph, want_ph), pixels::hamming(dh, want_dh));
        report.push(format!("{name}: phash Δ{dp} dhash Δ{dd}"));
        assert!(
            dp <= PHASH_TOLERANCE,
            "{name}: pHash {} vs imagehash {} differ by {dp} > {PHASH_TOLERANCE}",
            pixels::hash_hex(ph),
            pixels::hash_hex(want_ph)
        );
        assert!(
            dd <= DHASH_TOLERANCE,
            "{name}: dHash {} vs imagehash {} differ by {dd} > {DHASH_TOLERANCE}",
            pixels::hash_hex(dh),
            pixels::hash_hex(want_dh)
        );
    }
    eprintln!("hash cross-check: {}", report.join("; "));
}

/// The tolerance must be tight enough to MEAN something: two different
/// fixtures stay further apart than it, or "agrees within tolerance" would
/// hold for any pair of images.
#[test]
fn the_hash_tolerance_separates_different_images() {
    let luma = |n: &str| {
        let rgb = image::DynamicImage::ImageRgba8(load(n)).to_rgb8();
        pixels::luma_image(&rgb)
    };
    for (a, b) in [
        ("base.png", "other.png"),
        ("base.png", "rings.png"),
        ("stripes.png", "rings.png"),
    ] {
        let (la, lb) = (luma(a), luma(b));
        let dp = pixels::hamming(pixels::phash(&la).unwrap(), pixels::phash(&lb).unwrap());
        let dd = pixels::hamming(pixels::dhash(&la).unwrap(), pixels::dhash(&lb).unwrap());
        assert!(
            dp > 4 * PHASH_TOLERANCE,
            "{a} vs {b}: pHash only {dp} bits apart"
        );
        assert!(
            dd > 2 * DHASH_TOLERANCE,
            "{a} vs {b}: dHash only {dd} bits apart"
        );
    }
}

// ======================================================== ΔE2000

/// Sharma, Wu & Dalal (2005), Table 1 — the reference test data for CIEDE2000.
#[test]
fn delta_e_2000_matches_the_sharma_reference_pairs() {
    let cases = [
        ((50.0, 2.6772, -79.7751), (50.0, 0.0, -82.7485), 2.0425),
        ((50.0, 3.1571, -77.2803), (50.0, 0.0, -82.7485), 2.8615),
        ((50.0, 2.8361, -74.0200), (50.0, 0.0, -82.7485), 3.4412),
        ((50.0, -1.3802, -84.2814), (50.0, 0.0, -82.7485), 1.0000),
        ((50.0, 0.0, 0.0), (50.0, -1.0, 2.0), 2.3669),
        // The two hue-averaging branches (pairs 13 and 15): a near-180° hue
        // difference lands either side of the wrap.
        ((50.0, 2.4900, -0.0010), (50.0, -2.4900, 0.0009), 7.1792),
        ((50.0, 2.4900, -0.0010), (50.0, -2.4900, 0.0011), 7.2195),
        ((50.0, 2.5, 0.0), (73.0, 25.0, -18.0), 27.1492),
        (
            (60.2574, -34.0099, 36.2677),
            (60.4626, -34.1751, 39.4387),
            1.2644,
        ),
    ];
    for (a, b, want) in cases {
        let got = delta_e_2000_lab(a, b);
        assert!((got - want).abs() < 1e-4, "{a:?} vs {b:?}: {got} != {want}");
        // Symmetric.
        assert!((delta_e_2000_lab(b, a) - got).abs() < 1e-9);
    }
    assert_eq!(delta_e_2000(ORANGE, ORANGE), 0.0);
}

// ======================================================== the no-gating rules

#[test]
fn without_a_frame_the_image_analyzer_blocks_with_an_info_finding_only() {
    let snap = snapshot(vec![img_el("hero", 0, 0, &load("base.png"))]);
    let r = run_image(None, Some(&snap));
    assert!(r.verdict.is_blocked(), "{:?}", r.verdict);
    assert_eq!(r.verdict.code(), Some(UnknownCode::InputMissing));
    assert!(
        r.image_values.is_none(),
        "nothing was read, so nothing is reported"
    );
    assert!(!r.findings.is_empty());
    assert!(r.findings.iter().all(|f| f.severity == Severity::Info));
}

#[test]
fn a_frame_with_no_media_is_degraded_and_green() {
    let frame = page(200, 100, PAGE, &[]);
    let snap = snapshot(vec![Element {
        id: "btn".into(),
        bbox: Some(Region {
            x: 10,
            y: 10,
            w: 40,
            h: 20,
        }),
        interactable: true,
        ..Default::default()
    }]);
    let r = run_image(Some(&frame), Some(&snap));
    assert!(
        matches!(
            r.verdict,
            AnalyzerVerdict::Degraded {
                code: UnknownCode::InputMissing,
                ..
            }
        ),
        "{:?}",
        r.verdict
    );
    assert!(r.conclusive, "an image-free page must stay green");
    let v = r.image_values.expect("page values are still measured");
    assert!(v.images.is_empty());
    assert!(v.page.colourfulness.value().is_some());
    assert!(v.near_duplicates.unknown_info().is_some());

    // And a frame with no snapshot at all: degraded, not blocked.
    let r = run_image(Some(&frame), None);
    assert!(r.verdict.is_degraded() && r.conclusive, "{:?}", r.verdict);
}

#[test]
fn every_image_finding_is_info_even_on_a_page_of_defects() {
    // Placeholders and a duplicate pair: the analyzer reports all of them,
    // and none above Info.
    let base = load("base.png");
    let dup = load("duplicate.png");
    let ph = load("placeholder.png");
    let frame = page(
        500,
        140,
        PAGE,
        &[(&base, 10, 10), (&dup, 160, 10), (&ph, 300, 10)],
    );
    let mut marked = img_el("slot", 300, 10, &ph);
    marked.media.as_mut().unwrap().placeholder = Some(true);
    let snap = snapshot(vec![
        img_el("a", 10, 10, &base),
        img_el("b", 160, 10, &dup),
        marked,
    ]);
    let r = run_image(Some(&frame), Some(&snap));
    assert_eq!(r.verdict, AnalyzerVerdict::Checked);
    let kinds: Vec<&str> = r.findings.iter().map(|f| f.kind.as_str()).collect();
    assert!(kinds.contains(&"image_placeholder"), "{kinds:?}");
    assert!(kinds.contains(&"image_near_duplicate"), "{kinds:?}");
    assert!(r.findings.iter().all(|f| f.severity == Severity::Info));
    assert!(r
        .findings
        .iter()
        .all(|f| f.analyzer == Some(Analyzer::Image)));
}

#[test]
fn a_media_element_off_the_frame_degrades_and_its_pixel_values_are_unknown() {
    let base = load("base.png");
    let frame = page(200, 120, PAGE, &[(&base, 10, 10)]);
    let snap = snapshot(vec![
        img_el("on", 10, 10, &base),
        img_el("off", 900, 900, &base),
    ]);
    let r = run_image(Some(&frame), Some(&snap));
    assert!(r.verdict.is_degraded() && r.conclusive, "{:?}", r.verdict);
    let v = r.image_values.unwrap();
    let off = v.images.iter().find(|i| i.element == "off").unwrap();
    assert_eq!(
        off.sharpness.unknown_info().map(|u| u.code),
        Some(UnknownCode::InputMissing)
    );
    let on = v.images.iter().find(|i| i.element == "on").unwrap();
    assert!(on.sharpness.value().is_some());
}

#[test]
fn image_values_survive_the_wire() {
    let base = load("base.png");
    let frame = page(200, 120, PAGE, &[(&base, 10, 10)]);
    let snap = snapshot(vec![img_el("hero", 10, 10, &base)]);
    let r = run_image(Some(&frame), Some(&snap));
    let wire = serde_json::to_value(&r).unwrap();
    let img = &wire["imageValues"]["images"][0];
    assert_eq!(img["element"], "hero");
    assert_eq!(img["sharpness"]["status"], "measured");
    assert_eq!(
        img["sharpness"]["provenance"]["producer"]["id"],
        "vision-core/image"
    );
    assert!(img["sharpness"]["provenance"]["confidence"].is_number());
    // A media-only deduction states that it is one.
    assert!(img["visibleRegion"]["provenance"]["confidence"].is_null());
    let back: AnalyzerResult = serde_json::from_value(wire).unwrap();
    assert_eq!(back.image_values, r.image_values);
    // Every other analyzer leaves the key off the wire.
    let layout = analyzers::run(
        Analyzer::Layout,
        &AnalyzeInput {
            frame: None,
            snapshot: Some(&snap),
            prior_frame: None,
        },
    );
    assert!(serde_json::to_value(&layout)
        .unwrap()
        .get("imageValues")
        .is_none());
}

// ======================================================== the mutation corpus

fn palette_assertion() -> Assertion {
    Assertion::ImagePaletteWithin {
        element: "hero".into(),
        palette: vec![CREAM, NAVY, ORANGE].into(),
        max_delta_e: Some(10.0),
        min_coverage: Some(0.85),
    }
}

/// The largest cluster's ΔE2000 to its nearest palette colour.
fn largest_cluster_delta(v: &ImageValues) -> f64 {
    let c = &v.images[0].palette.value().unwrap()[0];
    [CREAM, NAVY, ORANGE]
        .iter()
        .map(|p| delta_e_2000(c.rgb, *p))
        .fold(f64::INFINITY, f64::min)
}

#[test]
fn mutation_off_palette_recolour_moves_the_palette_and_flips_image_palette_within() {
    let (held_img, mut_img) = (load("base.png"), load("recolour.png"));
    let held_frame = page(200, 120, PAGE, &[(&held_img, 10, 10)]);
    let mut_frame = page(200, 120, PAGE, &[(&mut_img, 10, 10)]);
    let snap = snapshot(vec![img_el("hero", 10, 10, &held_img)]);

    let (dh, dm) = (
        largest_cluster_delta(&values(&held_frame, &snap)),
        largest_cluster_delta(&values(&mut_frame, &snap)),
    );
    assert!(
        dh < 5.0 && dm > 15.0,
        "palette did not move: held {dh:.1} mutated {dm:.1}"
    );

    let held = assert_on(&held_frame, &snap, palette_assertion());
    let mutated = assert_on(&mut_frame, &snap, palette_assertion());
    flips(&held, &mutated);
    conf_in_range(&held);
    conf_in_range(&mutated);
}

/// The cropped-subject mutation: the same image in a short banner, its
/// object-position moved so the subject (the orange disc, natural-frame
/// y ≈ 0.45..0.76) is cropped away.
fn banner(position: &str) -> ElementSnapshot {
    let mut el = img_el("hero", 10, 10, &load("base.png"));
    el.bbox = Some(Region {
        x: 10,
        y: 10,
        w: 128,
        h: 48,
    });
    let m = el.media.as_mut().unwrap();
    m.rendered_px = Some(MediaSize::new(128.0, 48.0));
    m.object_fit = Some(ObjectFit::Cover);
    m.object_position = Some(position.into());
    snapshot(vec![el])
}

#[test]
fn mutation_cropped_subject_moves_the_visible_region_and_flips_image_focal_visible() {
    let frame = page(200, 120, PAGE, &[(&load("base.png"), 10, -24)]);
    let held = banner("50% 70%");
    let mutated = banner("50% 0%");
    let focal = NaturalRegion {
        x: 0.58,
        y: 0.44,
        width: 0.28,
        height: 0.33,
    };

    let vh = *values(&frame, &held).images[0]
        .visible_region
        .value()
        .unwrap();
    let vm = *values(&frame, &mutated).images[0]
        .visible_region
        .value()
        .unwrap();
    assert!(
        (vh.y - 0.35).abs() < 1e-9 && (vm.y - 0.0).abs() < 1e-9,
        "{vh:?} {vm:?}"
    );
    assert!((vh.height - 0.5).abs() < 1e-9 && (vm.height - 0.5).abs() < 1e-9);

    let a = || Assertion::ImageFocalVisible {
        element: "hero".into(),
        focal,
    };
    let (rh, rm) = (
        assert_on(&frame, &held, a()),
        assert_on(&frame, &mutated, a()),
    );
    flips(&rh, &rm);
    // A deduction from media facts: no confidence, by design.
    assert_eq!(rh.confidence, None);
    assert_eq!(rm.confidence, None);
}

/// Thresholds for the two quality mutations sit between the held value and
/// the mutated one, as a visual system would set them; the tests below also
/// pin that the two values are on opposite sides by a wide margin.
const MIN_SHARPNESS: f64 = 60.0;
const MAX_BLOCKINESS: f64 = 1.5;

#[test]
fn mutation_down_then_up_scale_blur_moves_sharpness_and_flips_image_sharpness_at_least() {
    let (held_img, mut_img) = (load("base.png"), load("blur.png"));
    let held_frame = page(200, 120, PAGE, &[(&held_img, 10, 10)]);
    let mut_frame = page(200, 120, PAGE, &[(&mut_img, 10, 10)]);
    let snap = snapshot(vec![img_el("hero", 10, 10, &held_img)]);

    let sh = *values(&held_frame, &snap).images[0]
        .sharpness
        .value()
        .unwrap();
    let sm = *values(&mut_frame, &snap).images[0]
        .sharpness
        .value()
        .unwrap();
    eprintln!("sharpness held {sh:.1} blurred {sm:.1}");
    assert!(
        sh > 2.0 * MIN_SHARPNESS && sm < MIN_SHARPNESS / 2.0,
        "held {sh} blurred {sm}"
    );

    let a = || Assertion::ImageSharpnessAtLeast {
        element: "hero".into(),
        min_sharpness: Some(MIN_SHARPNESS),
    };
    let (rh, rm) = (
        assert_on(&held_frame, &snap, a()),
        assert_on(&mut_frame, &snap, a()),
    );
    flips(&rh, &rm);
    conf_in_range(&rh);
}

#[test]
fn mutation_jpeg_q10_moves_blockiness_and_flips_image_blockiness_within() {
    let (held_img, mut_img) = (load("base.png"), load("jpeg_q10.jpg"));
    let held_frame = page(200, 120, PAGE, &[(&held_img, 10, 10)]);
    let mut_frame = page(200, 120, PAGE, &[(&mut_img, 10, 10)]);
    let snap = snapshot(vec![img_el("hero", 10, 10, &held_img)]);

    let bh = *values(&held_frame, &snap).images[0]
        .blockiness
        .value()
        .unwrap();
    let bm = *values(&mut_frame, &snap).images[0]
        .blockiness
        .value()
        .unwrap();
    eprintln!("blockiness held {bh:.2} jpeg-q10 {bm:.2}");
    // A clean image reads near 1 (its own edges land on some grid lines by
    // chance); quality 10 more than doubles it.
    assert!(bh < 1.35 && bm > 2.5, "held {bh} jpeg {bm}");

    let a = || Assertion::ImageBlockinessWithin {
        element: "hero".into(),
        max_blockiness: Some(MAX_BLOCKINESS),
    };
    let (rh, rm) = (
        assert_on(&held_frame, &snap, a()),
        assert_on(&mut_frame, &snap, a()),
    );
    flips(&rh, &rm);
    conf_in_range(&rm);
}

#[test]
fn mutation_duplicate_moves_the_pair_distance_and_flips_no_near_duplicate_images() {
    let base = load("base.png");
    let other = load("other.png");
    let dup = load("duplicate.png");
    let held_frame = page(320, 120, PAGE, &[(&base, 10, 10), (&other, 160, 10)]);
    let mut_frame = page(320, 120, PAGE, &[(&base, 10, 10), (&dup, 160, 10)]);
    let held = snapshot(vec![
        img_el("a", 10, 10, &base),
        img_el("b", 160, 10, &other),
    ]);
    let mutated = snapshot(vec![img_el("a", 10, 10, &base), img_el("b", 160, 10, &dup)]);

    let dist = |f: &Frame, s: &ElementSnapshot| {
        values(f, s).near_duplicates.value().unwrap()[0].phash_distance
    };
    let (dh, dm) = (dist(&held_frame, &held), dist(&mut_frame, &mutated));
    assert!(
        dh > 16 && dm <= 4,
        "pair distance did not move: held {dh} dup {dm}"
    );

    let a = || Assertion::NoNearDuplicateImages {
        max_hamming: Some(8),
    };
    let (rh, rm) = (
        assert_on(&held_frame, &held, a()),
        assert_on(&mut_frame, &mutated, a()),
    );
    flips(&rh, &rm);
    conf_in_range(&rm);
}

#[test]
fn mutation_flat_placeholder_moves_the_placeholder_signal_and_flips_no_placeholder_images() {
    let (held_img, mut_img) = (load("base.png"), load("placeholder.png"));
    let held_frame = page(200, 120, PAGE, &[(&held_img, 10, 10)]);
    let mut_frame = page(200, 120, PAGE, &[(&mut_img, 10, 10)]);
    let snap = snapshot(vec![img_el("hero", 10, 10, &held_img)]);

    let p = |f: &Frame| {
        values(f, &snap).images[0]
            .placeholder
            .value()
            .unwrap()
            .clone()
    };
    let (ph, pm) = (p(&held_frame), p(&mut_frame));
    assert!(!ph.placeholder && !ph.low_entropy_fill, "{ph:?}");
    assert!(pm.placeholder && pm.low_entropy_fill, "{pm:?}");

    let a = || Assertion::NoPlaceholderImages { element: None };
    let (rh, rm) = (
        assert_on(&held_frame, &snap, a()),
        assert_on(&mut_frame, &snap, a()),
    );
    flips(&rh, &rm);
    conf_in_range(&rm);
}

#[test]
fn the_slot_marker_alone_fails_no_placeholder_images_as_a_deduction() {
    let img = load("base.png");
    let frame = page(200, 120, PAGE, &[(&img, 10, 10)]);
    let mut el = img_el("hero", 10, 10, &img);
    el.media.as_mut().unwrap().placeholder = Some(true);
    let r = assert_on(
        &frame,
        &snapshot(vec![el]),
        Assertion::NoPlaceholderImages { element: None },
    );
    assert_eq!(r.outcome, AssertionOutcome::Failed);
    assert_eq!(r.confidence, None, "the marker is declared, not estimated");
}

// ======================================================== the other assertions

/// An icon's strokes on a dark-theme surround: near-black strokes vanish,
/// near-white ones read.
fn icon_page(stroke: [u8; 4]) -> (Frame, ElementSnapshot) {
    let bg = [0x12, 0x12, 0x12, 0xff];
    let icon = image::RgbaImage::from_fn(32, 32, |x, y| {
        let on = (x as i32 - y as i32).abs() < 2 || (x as i32 + y as i32 - 31).abs() < 2;
        image::Rgba(if on { stroke } else { bg })
    });
    let frame = page(120, 80, bg, &[(&icon, 40, 20)]);
    (frame, snapshot(vec![img_el("logo", 40, 20, &icon)]))
}

#[test]
fn transparent_image_legible_fails_strokes_that_vanish_on_the_surround() {
    let a = || Assertion::TransparentImageLegible {
        element: "logo".into(),
        min_ratio: Some(3.0),
    };
    let (hf, hs) = icon_page([0xf5, 0xf5, 0xf5, 0xff]);
    let (mf, ms) = icon_page([0x1f, 0x1f, 0x1f, 0xff]);
    let vh = *values(&hf, &hs).images[0].edge_contrast.value().unwrap();
    let vm = *values(&mf, &ms).images[0].edge_contrast.value().unwrap();
    assert!(
        vh > 10.0 && vm < 1.5,
        "edge contrast held {vh} mutated {vm}"
    );
    let (rh, rm) = (assert_on(&hf, &hs, a()), assert_on(&mf, &ms, a()));
    flips(&rh, &rm);
    conf_in_range(&rh);
}

/// A light page with an orange call to action and a second button. Held: the
/// second button is a quiet grey. Mutated: it is a loud black block that
/// out-draws the CTA.
fn cta_page(rival: [u8; 4]) -> (Frame, ElementSnapshot) {
    let bg = [0xe5, 0xe7, 0xeb, 0xff];
    let cta = image::RgbaImage::from_pixel(64, 24, image::Rgba([249, 115, 22, 255]));
    let other = image::RgbaImage::from_pixel(64, 24, image::Rgba(rival));
    let frame = page(320, 200, bg, &[(&cta, 40, 150), (&other, 200, 40)]);
    let button = |id: &str, x: i32, y: i32| Element {
        id: id.into(),
        bbox: Some(Region { x, y, w: 64, h: 24 }),
        interactable: true,
        ..Default::default()
    };
    (
        frame,
        snapshot(vec![button("cta", 40, 150), button("other", 200, 40)]),
    )
}

#[test]
fn salience_order_fails_when_another_element_out_draws_the_cta() {
    let a = || Assertion::SalienceOrder {
        cta_id: "cta".into(),
    };
    let (hf, hs) = cta_page([0xd8, 0xdb, 0xe0, 0xff]);
    let (mf, ms) = cta_page([0x11, 0x18, 0x27, 0xff]);
    let (rh, rm) = (assert_on(&hf, &hs, a()), assert_on(&mf, &ms, a()));
    flips(&rh, &rm);
    conf_in_range(&rh);
    conf_in_range(&rm);
}

#[test]
fn an_undeclared_threshold_is_unknown_never_a_pass() {
    let img = load("base.png");
    let frame = page(200, 120, PAGE, &[(&img, 10, 10)]);
    let snap = snapshot(vec![img_el("hero", 10, 10, &img)]);
    for a in [
        Assertion::ImagePaletteWithin {
            element: "hero".into(),
            palette: vec![CREAM].into(),
            max_delta_e: None,
            min_coverage: Some(0.5),
        },
        Assertion::NoNearDuplicateImages { max_hamming: None },
        Assertion::TransparentImageLegible {
            element: "hero".into(),
            min_ratio: None,
        },
        Assertion::ImageSharpnessAtLeast {
            element: "hero".into(),
            min_sharpness: None,
        },
        Assertion::ImageBlockinessWithin {
            element: "hero".into(),
            max_blockiness: None,
        },
    ] {
        let r = assert_on(&frame, &snap, a.clone());
        assert_eq!(r.outcome, AssertionOutcome::Unknown, "{a:?}");
        assert_eq!(r.code, Some(UnknownCode::InputMissing));
        assert!(r.detail.unwrap().contains("undeclared"));
    }
}

#[test]
fn a_frame_reading_assertion_without_a_frame_is_unknown() {
    let img = load("base.png");
    let snap = snapshot(vec![img_el("hero", 10, 10, &img)]);
    let r = evaluate_assertion(
        &palette_assertion(),
        &EvalContext {
            snapshot: Some(&snap),
            ..Default::default()
        },
    );
    assert_eq!(r.outcome, AssertionOutcome::Unknown);
    assert_eq!(r.code, Some(UnknownCode::InputMissing));
    // ...while the media-only focal check needs no frame at all.
    let r = evaluate_assertion(
        &Assertion::ImageFocalVisible {
            element: "hero".into(),
            focal: NaturalRegion {
                x: 0.1,
                y: 0.1,
                width: 0.2,
                height: 0.2,
            },
        },
        &EvalContext {
            snapshot: Some(&snap),
            ..Default::default()
        },
    );
    assert_eq!(r.outcome, AssertionOutcome::Passed, "{:?}", r.detail);
}

#[test]
fn the_image_assertions_round_trip_their_wire_spelling() {
    let json = serde_json::json!([
        {"type": "image_palette_within", "element": "h", "palette": [{"r":1,"g":2,"b":3}],
         "max_delta_e": 8.0, "min_coverage": 0.9},
        {"type": "no_near_duplicate_images", "max_hamming": 6},
        {"type": "no_placeholder_images"},
        {"type": "image_focal_visible", "element": "h",
         "focal": {"x": 0.05, "y": 0.2, "width": 0.45, "height": 0.6}},
        {"type": "transparent_image_legible", "element": "h", "min_ratio": 3.0},
        {"type": "salience_order", "cta_id": "buy"},
        {"type": "image_sharpness_at_least", "element": "h", "min_sharpness": 50.0},
        {"type": "image_blockiness_within", "element": "h", "max_blockiness": 1.5}
    ]);
    let parsed: Vec<Assertion> = serde_json::from_value(json.clone()).unwrap();
    assert_eq!(parsed.len(), 8);
    let back = serde_json::to_value(&parsed).unwrap();
    for (a, b) in json
        .as_array()
        .unwrap()
        .iter()
        .zip(back.as_array().unwrap())
    {
        assert_eq!(a["type"], b["type"]);
    }
}

#[test]
fn the_class_separates_a_flat_illustration_from_a_photograph() {
    let img = load("other.png");
    let frame = page(200, 120, PAGE, &[(&img, 10, 10)]);
    let snap = snapshot(vec![img_el("art", 10, 10, &img)]);
    let v = values(&frame, &snap);
    assert_eq!(v.images[0].class.value().unwrap().class, ImageClass::Flat);

    let noise = image::RgbaImage::from_fn(128, 96, |x, y| {
        let h =
            (x.wrapping_mul(2_654_435_761) ^ y.wrapping_mul(40_503)).wrapping_mul(2_246_822_519);
        image::Rgba([(h >> 8) as u8, (h >> 16) as u8, (h >> 24) as u8, 255])
    });
    let frame = page(200, 120, PAGE, &[(&noise, 10, 10)]);
    let v = values(&frame, &snapshot(vec![img_el("photo", 10, 10, &noise)]));
    assert_eq!(
        v.images[0].class.value().unwrap().class,
        ImageClass::Photographic
    );
}

#[test]
fn text_over_an_image_lowers_confidence_and_gets_its_own_busyness() {
    let img = load("base.png");
    let frame = page(200, 120, PAGE, &[(&img, 10, 10)]);
    let caption = Element {
        id: "caption".into(),
        bbox: Some(Region {
            x: 10,
            y: 10,
            w: 128,
            h: 24,
        }),
        text: Some("Hello".into()),
        ..Default::default()
    };
    let bare = values(&frame, &snapshot(vec![img_el("hero", 10, 10, &img)]));
    let over = values(
        &frame,
        &snapshot(vec![img_el("hero", 10, 10, &img), caption]),
    );
    let c = |v: &ImageValues| v.images[0].sharpness.provenance().confidence().unwrap();
    assert!((c(&bare) - 1.0).abs() < 1e-9);
    assert!(
        c(&over) < 0.8,
        "a quarter of the image is under text: {}",
        c(&over)
    );
    let busy = over.images[0].busyness.value().unwrap();
    assert_eq!(busy.under_text.len(), 1);
    assert_eq!(busy.under_text[0].element, "caption");
}

// ======================================================== review round 1

/// Draw `img` cropped to the CSS box `(bx, by, bw, bh)` as `object-fit`
/// would with the natural image's top-left at `(ox, oy)` relative to the box
/// (1:1 scale): the overflow is clipped by the box, as a browser clips it.
fn draw_clipped(
    buf: &mut image::RgbaImage,
    img: &image::RgbaImage,
    (bx, by, bw, bh): (i64, i64, i64, i64),
    (ox, oy): (i64, i64),
) {
    for y in 0..bh {
        for x in 0..bw {
            let (sx, sy) = (x - ox, y - oy);
            if sx >= 0 && sy >= 0 && (sx as u32) < img.width() && (sy as u32) < img.height() {
                let (fx, fy) = (bx + x, by + y);
                if fx >= 0 && fy >= 0 && (fx as u32) < buf.width() && (fy as u32) < buf.height() {
                    buf.put_pixel(fx as u32, fy as u32, *img.get_pixel(sx as u32, sy as u32));
                }
            }
        }
    }
}

fn fitted_el(id: &str, bbox: Region, natural: (u32, u32), fit: ObjectFit, pos: &str) -> Element {
    Element {
        id: id.into(),
        bbox: Some(bbox),
        media: Some(ElementMedia {
            natural_px: Some(MediaSize::new(natural.0 as f32, natural.1 as f32)),
            rendered_px: Some(MediaSize::new(bbox.w as f32, bbox.h as f32)),
            object_fit: Some(fit),
            object_position: Some(pos.into()),
            ..ElementMedia::new(MediaKind::Img)
        }),
        ..Default::default()
    }
}

// ---- M1 / R1: what paints nothing, what cannot be attributed, and stacks

fn with_visibility(mut el: Element, visible: Option<bool>, reason: Option<&str>) -> Element {
    el.visible = visible;
    el.visibility_reason = reason.map(|r| VisibilityReason::from(r.to_string()));
    el
}

#[test]
fn a_hidden_image_is_left_out_not_read_as_a_placeholder() {
    let base = load("base.png");
    let frame = page(320, 120, PAGE, &[(&base, 10, 10)]);
    for reason in ["hidden", "no-layout"] {
        let hidden = with_visibility(img_el("hidden", 160, 10, &base), Some(false), Some(reason));
        let snap = snapshot(vec![img_el("hero", 10, 10, &base), hidden]);
        let r = assert_on(
            &frame,
            &snap,
            Assertion::NoPlaceholderImages { element: None },
        );
        assert_eq!(
            r.outcome,
            AssertionOutcome::Passed,
            "{reason}: {:?}",
            r.detail
        );
        assert!(r.detail.as_deref().unwrap().contains("unrendered"));
        let res = run_image(Some(&frame), Some(&snap));
        assert_eq!(
            res.verdict,
            AnalyzerVerdict::Checked,
            "{reason}: {:?}",
            res.verdict
        );
        assert!(res.findings.iter().all(|f| f.kind != "image_placeholder"));
        let v = res.image_values.unwrap();
        let h = v.images.iter().find(|i| i.element == "hidden").unwrap();
        assert!(
            h.placeholder
                .unknown_info()
                .unwrap()
                .detail
                .contains("paints nothing"),
            "{reason}"
        );
    }
}

/// The SDK says `visible: false` for an image under a TRANSPARENT overlay (a
/// stretched-link card, an inset-0 scrim) exactly as for one under an opaque
/// one. The frame here shows the image fully painted, and the analyzer must
/// neither drop it silently nor call the page Checked.
#[test]
fn an_image_reported_occluded_is_unknown_never_silently_dropped() {
    let base = load("base.png");
    let other = load("other.png");
    let frame = page(320, 120, PAGE, &[(&base, 10, 10), (&other, 160, 10)]);
    let scrim = Element {
        id: "scrim".into(),
        bbox: Some(Region {
            x: 0,
            y: 0,
            w: 150,
            h: 120,
        }),
        interactable: true,
        ..Default::default()
    };
    let cases: [(&str, Element); 5] = [
        (
            "occluded",
            with_visibility(img_el("hero", 10, 10, &base), Some(false), Some("occluded")),
        ),
        (
            "off-screen",
            with_visibility(
                img_el("hero", 10, 10, &base),
                Some(false),
                Some("off-screen"),
            ),
        ),
        (
            "no reason",
            with_visibility(img_el("hero", 10, 10, &base), Some(false), None),
        ),
        (
            "unknown reason",
            with_visibility(img_el("hero", 10, 10, &base), Some(false), Some("clipped")),
        ),
        ("inside its occluder", {
            let mut e = img_el("hero", 10, 10, &base);
            e.occluded_by = Some("scrim".into());
            e
        }),
    ];
    for (label, hero) in cases {
        let snap = snapshot(vec![hero, img_el("other", 160, 10, &other), scrim.clone()]);
        let res = run_image(Some(&frame), Some(&snap));
        assert!(
            res.verdict.is_degraded() && res.conclusive,
            "{label}: must be a named gap, not Checked: {:?}",
            res.verdict
        );
        let v = res.image_values.unwrap();
        let h = v.images.iter().find(|i| i.element == "hero").unwrap();
        assert!(h.sharpness.unknown_info().is_some(), "{label}");
        for a in [
            Assertion::NoPlaceholderImages { element: None },
            Assertion::NoNearDuplicateImages {
                max_hamming: Some(8),
            },
        ] {
            let r = assert_on(&frame, &snap, a.clone());
            assert_eq!(
                r.outcome,
                AssertionOutcome::Unknown,
                "{label} {a:?}: {:?}",
                r.detail
            );
        }
    }
}

#[test]
fn a_display_none_variant_is_unrendered_not_outside_the_frame() {
    let base = load("base.png");
    let other = load("other.png");
    let frame = page(320, 120, PAGE, &[(&base, 10, 10), (&other, 160, 10)]);
    let mut variant = img_el("mobile-hero", 0, 0, &base);
    variant.bbox = Some(Region {
        x: 0,
        y: 0,
        w: 0,
        h: 0,
    });
    let snap = snapshot(vec![
        img_el("hero", 10, 10, &base),
        img_el("other", 160, 10, &other),
        variant,
    ]);
    let r = assert_on(
        &frame,
        &snap,
        Assertion::NoNearDuplicateImages {
            max_hamming: Some(8),
        },
    );
    assert_eq!(r.outcome, AssertionOutcome::Passed, "{:?}", r.detail);
    let res = run_image(Some(&frame), Some(&snap));
    assert_eq!(res.verdict, AnalyzerVerdict::Checked, "{:?}", res.verdict);
    let v = res.image_values.unwrap();
    let m = v
        .images
        .iter()
        .find(|i| i.element == "mobile-hero")
        .unwrap();
    let why = &m.sharpness.unknown_info().unwrap().detail;
    assert!(
        why.contains("zero-area") && !why.contains("outside"),
        "{why}"
    );
}

/// A carousel: the lower slide is partially covered (`occluded_pct`) by the
/// top one at the same box, so the frame shows the top one twice; the pair is
/// not compared. A distinct image elsewhere is.
#[test]
fn a_reported_stack_is_not_compared_with_itself() {
    let base = load("base.png");
    let other = load("other.png");
    let frame = page(320, 120, PAGE, &[(&base, 10, 10), (&other, 160, 10)]);
    let mut lower = img_el("slide2", 10, 10, &base);
    lower.occluded_pct = Some(60.0);
    let snap = snapshot(vec![
        img_el("slide1", 10, 10, &base),
        lower,
        img_el("other", 160, 10, &other),
    ]);
    let r = assert_on(
        &frame,
        &snap,
        Assertion::NoNearDuplicateImages {
            max_hamming: Some(8),
        },
    );
    assert_eq!(r.outcome, AssertionOutcome::Passed, "{:?}", r.detail);
    assert!(
        r.detail.as_deref().unwrap().contains("stacked"),
        "{:?}",
        r.detail
    );
    let res = run_image(Some(&frame), Some(&snap));
    assert!(res
        .findings
        .iter()
        .all(|f| f.kind != "image_near_duplicate"));

    // Every pair a stack: nothing was compared, which is not a pass.
    let snap = snapshot(vec![img_el("slide1", 10, 10, &base), {
        let mut e = img_el("slide2", 10, 10, &base);
        e.occluded_pct = Some(60.0);
        e
    }]);
    let r = assert_on(
        &frame,
        &snap,
        Assertion::NoNearDuplicateImages {
            max_hamming: Some(8),
        },
    );
    assert_eq!(r.outcome, AssertionOutcome::Unknown, "{:?}", r.detail);
}

/// Two images at one place that the producer reports fully visible are a
/// real duplicate drawn twice, not a stack: they are compared.
#[test]
fn an_unreported_overlap_is_compared_not_skipped() {
    let base = load("base.png");
    let frame = page(200, 120, PAGE, &[(&base, 10, 10)]);
    let snap = snapshot(vec![
        with_visibility(img_el("a", 10, 10, &base), Some(true), None),
        with_visibility(img_el("b", 10, 10, &base), Some(true), None),
    ]);
    let r = assert_on(
        &frame,
        &snap,
        Assertion::NoNearDuplicateImages {
            max_hamming: Some(8),
        },
    );
    assert_eq!(r.outcome, AssertionOutcome::Failed, "{:?}", r.detail);
}

#[test]
fn a_single_image_page_has_no_duplicate_pair() {
    let base = load("base.png");
    let frame = page(200, 120, PAGE, &[(&base, 10, 10)]);
    let snap = snapshot(vec![img_el("hero", 10, 10, &base)]);
    let r = assert_on(
        &frame,
        &snap,
        Assertion::NoNearDuplicateImages {
            max_hamming: Some(8),
        },
    );
    assert_eq!(r.outcome, AssertionOutcome::Passed, "{:?}", r.detail);
    assert!(r.detail.as_deref().unwrap().contains("no pair is possible"));
    conf_in_range(&r);
}

/// A section with a CSS background, or a `role=img` wrapper, CONTAINS the
/// image: it is behind it, not over it, and must not drive its confidence
/// to zero.
#[test]
fn a_containing_media_element_is_not_an_overlay() {
    let base = load("base.png");
    let frame = page(200, 120, PAGE, &[(&base, 10, 10)]);
    let section = Element {
        id: "section".into(),
        bbox: Some(Region {
            x: 0,
            y: 0,
            w: 200,
            h: 120,
        }),
        media: Some(ElementMedia::new(MediaKind::CssBackground)),
        ..Default::default()
    };
    let v = values(
        &frame,
        &snapshot(vec![img_el("hero", 10, 10, &base), section]),
    );
    let hero = v.images.iter().find(|i| i.element == "hero").unwrap();
    let c = hero.sharpness.provenance().confidence().unwrap();
    assert!(
        (c - 1.0).abs() < 1e-9,
        "a container read as an overlay: {c}"
    );
}

// ---- N1: which media kinds have a drawn rect

/// An inline SVG (or a `role=img` element) paints its own box, and the SDK
/// records no object-fit for it: the box is read whole, not refused. A CSS
/// background is unreadable, and the Degraded reason says why.
#[test]
fn an_svg_is_read_whole_and_a_css_background_is_named_unreadable() {
    let base = load("base.png");
    let frame = page(200, 120, PAGE, &[(&base, 10, 10)]);
    for kind in [MediaKind::Svg, MediaKind::RoleImg] {
        let el = Element {
            id: "logo".into(),
            bbox: Some(Region {
                x: 10,
                y: 10,
                w: 128,
                h: 96,
            }),
            media: Some(ElementMedia::new(kind)),
            ..Default::default()
        };
        let res = run_image(Some(&frame), Some(&snapshot(vec![el])));
        assert_eq!(
            res.verdict,
            AnalyzerVerdict::Checked,
            "{kind:?}: {:?}",
            res.verdict
        );
        let v = res.image_values.unwrap();
        assert!(v.images[0].sharpness.value().is_some(), "{kind:?}");
    }
    let section = Element {
        id: "section".into(),
        bbox: Some(Region {
            x: 0,
            y: 0,
            w: 200,
            h: 120,
        }),
        media: Some(ElementMedia::new(MediaKind::CssBackground)),
        ..Default::default()
    };
    let res = run_image(
        Some(&frame),
        Some(&snapshot(vec![img_el("hero", 10, 10, &base), section])),
    );
    let reason = res.verdict.reason().unwrap_or_default().to_string();
    assert!(res.verdict.is_degraded(), "{:?}", res.verdict);
    assert!(
        reason.contains("CSS background") && reason.contains("background-size"),
        "the reason must name the gap: {reason}"
    );
}

// ---- N2: unreadable rivals and overlays are not measured

#[test]
fn an_unreadable_rival_does_not_out_draw_the_cta() {
    let (frame, mut snap) = cta_page([0x11, 0x18, 0x27, 0xff]);
    // The loud rival is reported not visible with no reason: whether the
    // frame shows IT at that place is unknown, so it is not a rival.
    snap.elements[1].visible = Some(false);
    let r = assert_on(
        &frame,
        &snap,
        Assertion::SalienceOrder {
            cta_id: "cta".into(),
        },
    );
    assert_eq!(r.outcome, AssertionOutcome::Unknown, "{:?}", r.detail);
    // ...and text that cannot be attributed is not "over" the image.
    let base = load("base.png");
    let frame = page(200, 120, PAGE, &[(&base, 10, 10)]);
    let caption = Element {
        id: "caption".into(),
        bbox: Some(Region {
            x: 10,
            y: 10,
            w: 128,
            h: 24,
        }),
        text: Some("Hello".into()),
        visible: Some(false),
        ..Default::default()
    };
    let v = values(
        &frame,
        &snapshot(vec![img_el("hero", 10, 10, &base), caption]),
    );
    assert!(v.images[0].busyness.value().unwrap().under_text.is_empty());
}

// ---- M2: the coding grid is placed where the encoder put it

fn blockiness_of(frame: &Frame, el: Element) -> f64 {
    *values(frame, &snapshot(vec![el])).images[0]
        .blockiness
        .value()
        .expect("blockiness measured")
}

#[test]
fn blockiness_finds_the_grid_of_an_image_starting_off_frame() {
    // The image starts 3 px left of and 5 px above the frame, so the crop's
    // origin is not the image's: the grid is 3 and 5 px into the image.
    let (held, jpeg) = (load("base.png"), load("jpeg_q10.jpg"));
    let at = |img: &image::RgbaImage| page(200, 120, PAGE, &[(img, -3, -5)]);
    let el = || img_el("hero", -3, -5, &held);
    let (bh, bm) = (
        blockiness_of(&at(&held), el()),
        blockiness_of(&at(&jpeg), el()),
    );
    eprintln!("off-frame blockiness held {bh:.2} jpeg {bm:.2}");
    assert!(bh < 1.35 && bm > 2.5, "held {bh} jpeg {bm}");
    let a = || Assertion::ImageBlockinessWithin {
        element: "hero".into(),
        max_blockiness: Some(MAX_BLOCKINESS),
    };
    let snap = snapshot(vec![el()]);
    flips(
        &assert_on(&at(&held), &snap, a()),
        &assert_on(&at(&jpeg), &snap, a()),
    );
}

#[test]
fn blockiness_places_the_grid_under_cover_and_contain() {
    let (held, jpeg) = (load("base.png"), load("jpeg_q10.jpg"));
    // (box, natural top-left inside the box, object-fit, object-position)
    let cases = [
        // cover, cropped vertically: the image starts 12 px above the box,
        // half a block off a grid sought at the crop's origin.
        (
            Region {
                x: 10,
                y: 10,
                w: 128,
                h: 48,
            },
            (0, -12),
            ObjectFit::Cover,
            "50% 25%",
        ),
        // cover, cropped horizontally to a 96-px box: a period taken from the
        // width axis would be 8 × 96/128 = 6, which meets the true 8-px grid
        // on one line in four. (A 64-px box would give 4, a divisor of 8 that
        // still hits every true line, so it could not tell the two apart.)
        (
            Region {
                x: 10,
                y: 10,
                w: 96,
                h: 96,
            },
            (-8, 0),
            ObjectFit::Cover,
            "25% 50%",
        ),
        // contain, pushed 30 px down by a px offset: letterbox above and
        // below, and the grid starts at 30, not at the box's top.
        (
            Region {
                x: 10,
                y: 10,
                w: 128,
                h: 160,
            },
            (0, 30),
            ObjectFit::Contain,
            "50% 30px",
        ),
    ];
    for (bbox, off, fit, pos) in cases {
        let draw = |img: &image::RgbaImage| {
            let mut buf = image::RgbaImage::from_pixel(220, 200, image::Rgba(PAGE));
            draw_clipped(
                &mut buf,
                img,
                (bbox.x as i64, bbox.y as i64, bbox.w as i64, bbox.h as i64),
                off,
            );
            Frame::from_rgba(buf, FrameSource::synthetic_now())
        };
        let el = || fitted_el("hero", bbox, (128, 96), fit, pos);
        let (bh, bm) = (
            blockiness_of(&draw(&held), el()),
            blockiness_of(&draw(&jpeg), el()),
        );
        eprintln!("{fit:?} {pos}: blockiness held {bh:.2} jpeg {bm:.2}");
        // Clear of the gate threshold on both sides. (A grid sought at the
        // crop's origin or at the width axis's period reads these jpeg crops
        // at or below 1.5, i.e. as clean.)
        assert!(
            bh < 1.35 && bm > MAX_BLOCKINESS + 0.25,
            "{fit:?} {pos}: held {bh} jpeg {bm}"
        );
    }
}

#[test]
fn blockiness_without_drawn_geometry_is_unknown() {
    let held = load("base.png");
    let frame = page(200, 120, PAGE, &[(&held, 10, 10)]);
    let mut el = img_el("hero", 10, 10, &held);
    el.media.as_mut().unwrap().natural_px = None;
    let v = values(&frame, &snapshot(vec![el]));
    assert_eq!(
        v.images[0].blockiness.unknown_info().map(|u| u.code),
        Some(UnknownCode::InputMissing)
    );
}

// ---- M3 / R2: the crossed-box placeholder at any size, and nothing else

/// A line segment, endpoints as fractions of the box.
type Segment = ((f64, f64), (f64, f64));

/// A segment-drawn picture: `ground` everywhere, `ink` within `width / 2` of
/// any segment (endpoints as fractions of the box), plus an optional border.
fn segments(
    w: u32,
    h: u32,
    segs: &[Segment],
    width: f64,
    border: f64,
    ground: [u8; 3],
    ink: [u8; 3],
) -> image::RgbImage {
    let (wf, hf) = (w as f64, h as f64);
    image::RgbImage::from_fn(w, h, |x, y| {
        let p = (x as f64 + 0.5, y as f64 + 0.5);
        let near = segs.iter().any(|&((ax, ay), (bx, by))| {
            let (ax, ay, bx, by) = (ax * wf, ay * hf, bx * wf, by * hf);
            let (dx, dy) = (bx - ax, by - ay);
            let t = (((p.0 - ax) * dx + (p.1 - ay) * dy) / (dx * dx + dy * dy)).clamp(0.0, 1.0);
            (p.0 - (ax + t * dx)).hypot(p.1 - (ay + t * dy)) <= width / 2.0
        });
        let edge = p.0.min(p.1).min(wf - p.0).min(hf - p.1);
        image::Rgb(if near || edge <= border { ink } else { ground })
    })
}

const GREY: [u8; 3] = [0xe5, 0xe7, 0xeb];
const GREY_INK: [u8; 3] = [0x9c, 0xa3, 0xaf];
const CROSS: [Segment; 2] = [((0.0, 0.0), (1.0, 1.0)), ((1.0, 0.0), (0.0, 1.0))];

fn crossed(w: u32, h: u32, line: f64, border: bool) -> image::RgbImage {
    segments(
        w,
        h,
        &CROSS,
        line,
        if border { line } else { -1.0 },
        GREY,
        GREY_INK,
    )
}

/// Lucide's `x` (`M18 6 6 18 M6 6l12 12` in a 24 viewBox, stroke 2): padded
/// 25% inside its box, dark on a light page.
fn lucide_x(side: u32) -> image::RgbImage {
    segments(
        side,
        side,
        &[((0.25, 0.25), (0.75, 0.75)), ((0.75, 0.25), (0.25, 0.75))],
        side as f64 / 12.0,
        -1.0,
        [0xff, 0xff, 0xff],
        [0x0f, 0x17, 0x2a],
    )
}

#[test]
fn the_crossed_box_is_recognised_at_any_size_aspect_and_line_weight() {
    for (w, h, line, border) in [
        (97, 64, 1.0, false),
        (200, 150, 2.0, false),
        (640, 360, 1.0, false),
        (400, 300, 8.0, false),
        (96, 64, 2.0, true),
        (64, 64, 3.0, true),
    ] {
        assert!(
            pixels::is_crossed_box(&crossed(w, h, line, border), 1.0),
            "{w}x{h} line {line} border {border} not recognised"
        );
    }
}

/// Tinted placeholders: one hue at two lightnesses (Tailwind 100 under 300/400).
#[test]
fn a_tinted_crossed_box_is_recognised() {
    for (label, ground, ink) in [
        (
            "blue-100 / blue-300",
            [0xdb, 0xea, 0xfe],
            [0x93, 0xc5, 0xfd],
        ),
        (
            "yellow-100 / amber-400",
            [0xfe, 0xf9, 0xc3],
            [0xfb, 0xbf, 0x24],
        ),
        (
            "pink-100 / pink-300",
            [0xfc, 0xe7, 0xf3],
            [0xf9, 0xa8, 0xd4],
        ),
    ] {
        let img = segments(240, 160, &CROSS, 2.0, 2.0, ground, ink);
        assert!(pixels::is_crossed_box(&img, 1.0), "{label} not recognised");
    }
}

/// A placeholder that names its slot in a centred caption is still one.
#[test]
fn a_captioned_crossed_box_is_recognised() {
    let mut img = crossed(600, 315, 2.0, false);
    // "Hero image 1200 × 630": a row of glyph-like bars, 24% × 8% of the box,
    // centred.
    let (x0, x1, y0, y1) = (228u32, 372u32, 145u32, 170u32);
    for y in y0..y1 {
        for x in x0..x1 {
            if (x / 2) % 3 != 0 && (y - y0) % 12 < 9 {
                img.put_pixel(x, y, image::Rgb([0x4b, 0x55, 0x63]));
            }
        }
    }
    assert!(
        pixels::is_crossed_box(&img, 1.0),
        "the caption hid the pattern"
    );
}

#[test]
fn icons_flags_and_charts_are_not_crossed_boxes() {
    // Close icons at 48 and 72 CSS px, at DPR 1 and DPR 2 (so 96 and 144
    // buffer px — above any buffer-px floor).
    for css in [48u32, 72] {
        for dpr in [1u32, 2] {
            assert!(
                !pixels::is_crossed_box(&lucide_x(css * dpr), dpr as f64),
                "Lucide x at {css} CSS px, DPR {dpr}, read as a placeholder"
            );
        }
    }
    // A checkbox's corner-to-corner X: small, at DPR 1 and 2.
    for dpr in [1u32, 2] {
        let side = 20 * dpr;
        let check = segments(
            side,
            side,
            &CROSS,
            dpr as f64 * 2.0,
            dpr as f64,
            [255; 3],
            [0x11; 3],
        );
        assert!(
            !pixels::is_crossed_box(&check, dpr as f64),
            "checkbox X at DPR {dpr}"
        );
    }
    // A saltire flag: a corner-to-corner white X on a saturated field.
    let saltire = segments(200, 120, &CROSS, 24.0, -1.0, [0x00, 0x5e, 0xb8], [255; 3]);
    assert!(!pixels::is_crossed_box(&saltire, 1.0), "saltire flag");
    // A thin crimson saltire on white (Alabama's flag): corner to corner, a
    // sixth of the box in ink, neutral ground — only the ink's colour says it
    // is not a grey wireframe cross.
    let crimson = segments(200, 120, &CROSS, 8.0, -1.0, [255; 3], [0xb0, 0x1c, 0x2e]);
    assert!(
        !pixels::is_crossed_box(&crimson, 1.0),
        "crimson saltire on white"
    );
    // A supply/demand chart: two crossing lines inset 10% from the axes.
    let chart = segments(
        300,
        200,
        &[((0.1, 0.1), (0.9, 0.9)), ((0.9, 0.1), (0.1, 0.9))],
        2.0,
        -1.0,
        [255; 3],
        [0x33; 3],
    );
    assert!(!pixels::is_crossed_box(&chart, 1.0), "supply/demand chart");
    // Real images, and one diagonal only.
    let rgb = |n: &str| image::DynamicImage::ImageRgba8(load(n)).to_rgb8();
    for n in [
        "base.png",
        "stripes.png",
        "rings.png",
        "other.png",
        "recolour.png",
    ] {
        assert!(
            !pixels::is_crossed_box(&rgb(n), 1.0),
            "{n} read as a crossed box"
        );
    }
    let one = segments(200, 150, &CROSS[1..], 2.0, -1.0, GREY, GREY_INK);
    assert!(!pixels::is_crossed_box(&one, 1.0), "a single diagonal");
}

/// Through the analyzer on a DPR-2 frame: a 200×150-CSS placeholder (400×300
/// buffer px) is recognised; a 48-CSS one (96 buffer px) is under the CSS
/// floor and is not.
#[test]
fn the_crossed_box_floor_is_in_css_px_on_a_hidpi_frame() {
    let mut buf = image::RgbaImage::from_pixel(560, 340, image::Rgba(PAGE));
    let big = image::DynamicImage::ImageRgb8(crossed(400, 300, 4.0, true)).to_rgba8();
    let small = image::DynamicImage::ImageRgb8(crossed(96, 96, 4.0, true)).to_rgba8();
    image::imageops::overlay(&mut buf, &big, 20, 20);
    image::imageops::overlay(&mut buf, &small, 440, 20);
    let mut source = FrameSource::synthetic_now();
    source.scale_factor = 2.0;
    let frame = Frame::from_rgba(buf, source);
    let el = |id: &str, x: i32, side_w: u32, side_h: u32| Element {
        id: id.into(),
        bbox: Some(Region {
            x,
            y: 10,
            w: side_w,
            h: side_h,
        }),
        media: Some(ElementMedia {
            natural_px: Some(MediaSize::new((side_w * 2) as f32, (side_h * 2) as f32)),
            rendered_px: Some(MediaSize::new(side_w as f32, side_h as f32)),
            object_fit: Some(ObjectFit::Fill),
            ..ElementMedia::new(MediaKind::Img)
        }),
        ..Default::default()
    };
    let v = values(
        &frame,
        &snapshot(vec![el("big", 10, 200, 150), el("small", 220, 48, 48)]),
    );
    let pattern = |id: &str| {
        v.images
            .iter()
            .find(|i| i.element == id)
            .unwrap()
            .placeholder
            .value()
            .unwrap()
            .known_pattern
            .clone()
    };
    assert_eq!(pattern("big").as_deref(), Some("crossed_box"));
    assert_eq!(pattern("small"), None);
}

#[test]
fn a_crossed_box_on_the_page_fails_no_placeholder_images_by_its_pixels() {
    let ph = image::DynamicImage::ImageRgb8(crossed(200, 150, 2.0, true)).to_rgba8();
    let frame = page(240, 180, PAGE, &[(&ph, 10, 10)]);
    let snap = snapshot(vec![img_el("slot", 10, 10, &ph)]);
    let v = values(&frame, &snap);
    let p = v.images[0].placeholder.value().unwrap();
    assert_eq!(p.known_pattern.as_deref(), Some("crossed_box"), "{p:?}");
    let r = assert_on(
        &frame,
        &snap,
        Assertion::NoPlaceholderImages { element: None },
    );
    assert_eq!(r.outcome, AssertionOutcome::Failed, "{:?}", r.detail);
    assert!(r.detail.as_deref().unwrap().contains("crossed_box"));
    conf_in_range(&r);
}

// ---- M4: a contain letterbox is page, not image

#[test]
fn a_contained_image_is_judged_on_its_pixels_not_its_letterbox() {
    let base = load("base.png");
    let bbox = Region {
        x: 10,
        y: 10,
        w: 128,
        h: 200,
    };
    let mut buf = image::RgbaImage::from_pixel(160, 230, image::Rgba(PAGE));
    draw_clipped(&mut buf, &base, (10, 10, 128, 200), (0, 52));
    let frame = Frame::from_rgba(buf, FrameSource::synthetic_now());
    let snap = snapshot(vec![fitted_el(
        "hero",
        bbox,
        (128, 96),
        ObjectFit::Contain,
        "50% 50%",
    )]);
    // PAGE (the letterbox) is not on the palette, and is half the box.
    assert!(delta_e_2000(Rgb::new(PAGE[0], PAGE[1], PAGE[2]), CREAM) > 10.0);
    let r = assert_on(&frame, &snap, palette_assertion());
    assert_eq!(r.outcome, AssertionOutcome::Passed, "{:?}", r.detail);
    conf_in_range(&r);

    // Without the facts that place the drawn image, the box's pixels cannot
    // be told from the letterbox: every pixel value, and every assertion that
    // reads pixels, is Unknown rather than a judgement of page background.
    // Each way the placement can be lost is checked.
    let unplaced: [(&str, Element); 4] = [
        ("contain, no natural_px", {
            let mut e = fitted_el("hero", bbox, (128, 96), ObjectFit::Contain, "50% 50%");
            e.media.as_mut().unwrap().natural_px = None;
            e
        }),
        ("contain, no object_position", {
            let mut e = fitted_el("hero", bbox, (128, 96), ObjectFit::Contain, "50% 50%");
            e.media.as_mut().unwrap().object_position = None;
            e
        }),
        (
            "none",
            fitted_el("hero", bbox, (128, 96), ObjectFit::None, "50% 50%"),
        ),
        (
            "scale-down",
            fitted_el("hero", bbox, (128, 96), ObjectFit::ScaleDown, "50% 50%"),
        ),
    ];
    for (label, el) in unplaced {
        let snap = snapshot(vec![el]);
        let v = values(&frame, &snap);
        assert!(
            v.images[0].palette.unknown_info().is_some(),
            "{label}: palette"
        );
        for a in [
            palette_assertion(),
            Assertion::ImageSharpnessAtLeast {
                element: "hero".into(),
                min_sharpness: Some(MIN_SHARPNESS),
            },
            Assertion::ImageBlockinessWithin {
                element: "hero".into(),
                max_blockiness: Some(MAX_BLOCKINESS),
            },
            Assertion::TransparentImageLegible {
                element: "hero".into(),
                min_ratio: Some(3.0),
            },
        ] {
            let r = assert_on(&frame, &snap, a.clone());
            assert_eq!(
                r.outcome,
                AssertionOutcome::Unknown,
                "{label} {a:?}: {:?}",
                r.detail
            );
            assert!(
                r.detail.as_deref().unwrap().contains("letterbox"),
                "{label}: {:?}",
                r.detail
            );
        }
    }
}

/// Effective resolution depends on the drawn SCALE, not on where the image is
/// placed: a missing object_position must not make it unknown.
#[test]
fn effective_resolution_needs_no_object_position() {
    let frame = page(200, 220, PAGE, &[]);
    let mut el = fitted_el(
        "hero",
        Region {
            x: 10,
            y: 10,
            w: 64,
            h: 200,
        },
        (128, 96),
        ObjectFit::Contain,
        "50% 50%",
    );
    el.media.as_mut().unwrap().object_position = None;
    let v = values(&frame, &snapshot(vec![el]));
    // contain: scale min(64/128, 200/96) = 0.5 → 2 source px per device px.
    assert_eq!(v.images[0].effective_resolution.value(), Some(&2.0));
}

#[test]
fn a_px_offset_under_contain_can_push_part_of_the_image_out_of_view() {
    let snap = snapshot(vec![fitted_el(
        "hero",
        Region {
            x: 0,
            y: 0,
            w: 128,
            h: 96,
        },
        (128, 96),
        ObjectFit::Contain,
        "64px 0px",
    )]);
    let frame = page(200, 120, PAGE, &[]);
    let vis = *values(&frame, &snap).images[0]
        .visible_region
        .value()
        .unwrap();
    assert!((vis.width - 0.5).abs() < 1e-9 && vis.x == 0.0, "{vis:?}");
}

// ---- the f64 / non-finite guard

#[test]
fn extreme_media_sizes_stay_finite_and_round_trip() {
    let base = load("base.png");
    let frame = page(200, 120, PAGE, &[(&base, 10, 10)]);
    let mut el = img_el("hero", 10, 10, &base);
    {
        let m = el.media.as_mut().unwrap();
        // An f32 ratio of these overflows to inf; in f64 it is finite.
        m.natural_px = Some(MediaSize::new(1.0, 1.0));
        m.rendered_px = Some(MediaSize::new(3.0e38, 1.0e-30));
    }
    let mut inf = img_el("inf", 150, 10, &base);
    inf.media.as_mut().unwrap().effective_resolution = Some(f32::INFINITY);
    let r = run_image(Some(&frame), Some(&snapshot(vec![el, inf])));
    let v = r.image_values.as_ref().unwrap();
    let d = *v.images[0]
        .aspect_distortion
        .value()
        .expect("finite in f64");
    assert!(d.is_finite() && d > 1e60, "{d}");
    assert!(
        v.images[1].effective_resolution.unknown_info().is_some(),
        "a non-finite producer value must be refused, not carried"
    );
    let wire = serde_json::to_value(&r).unwrap();
    let back: AnalyzerResult = serde_json::from_value(wire).expect("the values read back");
    assert_eq!(back.image_values, r.image_values);
}

// ======================================================== review round 5

fn flat_el(id: &str, kind: MediaKind, bbox: Region) -> Element {
    Element {
        id: id.into(),
        bbox: Some(bbox),
        media: Some(ElementMedia {
            natural_px: Some(MediaSize::new(bbox.w as f32, bbox.h as f32)),
            rendered_px: Some(MediaSize::new(bbox.w as f32, bbox.h as f32)),
            object_fit: Some(ObjectFit::Fill),
            ..ElementMedia::new(kind)
        }),
        ..Default::default()
    }
}

/// A frame holding one flat rectangle per box.
fn flat_page(boxes: &[Region]) -> Frame {
    let mut buf = image::RgbaImage::from_pixel(520, 360, image::Rgba(PAGE));
    for b in boxes {
        for y in b.y..b.y + b.h as i32 {
            for x in b.x..b.x + b.w as i32 {
                buf.put_pixel(x as u32, y as u32, image::Rgba([0x25, 0x63, 0xeb, 0xff]));
            }
        }
    }
    Frame::from_rgba(buf, FrameSource::synthetic_now())
}

fn is_placeholder(frame: &Frame, el: Element) -> bool {
    values(frame, &snapshot(vec![el])).images[0]
        .placeholder
        .value()
        .unwrap()
        .placeholder
}

// ---- P1: flatness is a placeholder only for a slot-sized raster

#[test]
fn a_flat_data_graphic_or_a_small_fill_is_not_a_placeholder() {
    let bar = Region {
        x: 10,
        y: 10,
        w: 300,
        h: 8,
    };
    let swatch = Region {
        x: 10,
        y: 40,
        w: 64,
        h: 64,
    };
    let dot = Region {
        x: 100,
        y: 40,
        w: 32,
        h: 32,
    };
    let frame = flat_page(&[bar, swatch, dot]);
    // A 100% progress bar drawn as role=img, and as an inline SVG.
    assert!(!is_placeholder(
        &frame,
        flat_el("bar", MediaKind::RoleImg, bar)
    ));
    assert!(!is_placeholder(&frame, flat_el("bar", MediaKind::Svg, bar)));
    // A solid brand-colour swatch SVG, slot-sized.
    assert!(!is_placeholder(
        &frame,
        flat_el("swatch", MediaKind::Svg, swatch)
    ));
    // A raster flat fill smaller than a slot.
    assert!(!is_placeholder(&frame, flat_el("dot", MediaKind::Img, dot)));
}

#[test]
fn a_flat_slot_sized_raster_is_still_a_placeholder() {
    let slot = Region {
        x: 10,
        y: 10,
        w: 400,
        h: 300,
    };
    let frame = flat_page(&[slot]);
    assert!(is_placeholder(
        &frame,
        flat_el("slot", MediaKind::Img, slot)
    ));
    let snap = snapshot(vec![flat_el("slot", MediaKind::Img, slot)]);
    let r = assert_on(
        &frame,
        &snap,
        Assertion::NoPlaceholderImages { element: None },
    );
    assert_eq!(r.outcome, AssertionOutcome::Failed, "{:?}", r.detail);
}

// ---- L-a / L-b: the caption box is wide, and sparse

#[test]
fn a_long_caption_does_not_hide_the_crossed_box() {
    let mut img = crossed(600, 315, 2.0, false);
    // 55% of the width: longer than half the box.
    let (x0, x1, y0, y1) = (135u32, 465u32, 145u32, 170u32);
    for y in y0..y1 {
        for x in x0..x1 {
            if (x / 2) % 3 != 0 && (y - y0) % 12 < 9 {
                img.put_pixel(x, y, image::Rgb([0x4b, 0x55, 0x63]));
            }
        }
    }
    assert!(
        pixels::is_crossed_box(&img, 1.0),
        "the long caption hid the pattern"
    );
}

#[test]
fn a_filled_centre_is_not_a_caption() {
    // A hallway: perspective lines to the corners, a filled back wall.
    let mut hallway = crossed(300, 200, 2.0, false);
    for y in 70..130 {
        for x in 105..195 {
            hallway.put_pixel(x, y, image::Rgb(GREY_INK));
        }
    }
    assert!(
        !pixels::is_crossed_box(&hallway, 1.0),
        "hallway read as a placeholder"
    );
    // A central subject with four rays to the corners.
    let mut hub = crossed(300, 200, 2.0, false);
    for y in 0..200u32 {
        for x in 0..300u32 {
            if (x as f64 - 150.0).hypot(y as f64 - 100.0) <= 50.0 {
                hub.put_pixel(x, y, image::Rgb(GREY_INK));
            }
        }
    }
    assert!(
        !pixels::is_crossed_box(&hub, 1.0),
        "hub-and-rays read as a placeholder"
    );
}

// ---- L-d: one image read through two nested elements

#[test]
fn a_wrapper_and_its_own_svg_are_not_duplicates() {
    let base = load("base.png");
    let other = load("other.png");
    let frame = page(320, 120, PAGE, &[(&base, 10, 10), (&other, 160, 10)]);
    let mut wrapper = flat_el(
        "arrow",
        MediaKind::RoleImg,
        Region {
            x: 10,
            y: 10,
            w: 128,
            h: 96,
        },
    );
    wrapper.children_ids = vec!["arrow-svg".into()];
    let mut svg = flat_el(
        "arrow-svg",
        MediaKind::Svg,
        Region {
            x: 10,
            y: 10,
            w: 128,
            h: 96,
        },
    );
    svg.parent_id = Some("arrow".into());
    let snap = snapshot(vec![wrapper, svg, img_el("other", 160, 10, &other)]);
    let r = assert_on(
        &frame,
        &snap,
        Assertion::NoNearDuplicateImages {
            max_hamming: Some(8),
        },
    );
    assert_eq!(r.outcome, AssertionOutcome::Passed, "{:?}", r.detail);
    assert!(
        r.detail.as_deref().unwrap().contains("nested"),
        "{:?}",
        r.detail
    );
    let res = run_image(Some(&frame), Some(&snap));
    assert!(res
        .findings
        .iter()
        .all(|f| f.kind != "image_near_duplicate"));
}

// ---- CSS backgrounds: excluded with a note, marker still read

fn css_bg(id: &str, marked: bool) -> Element {
    let mut e = flat_el(
        id,
        MediaKind::CssBackground,
        Region {
            x: 0,
            y: 0,
            w: 320,
            h: 120,
        },
    );
    e.media.as_mut().unwrap().placeholder = Some(marked);
    e
}

#[test]
fn css_backgrounds_are_left_out_with_a_note_not_made_unknown() {
    let base = load("base.png");
    let other = load("other.png");
    let frame = page(320, 120, PAGE, &[(&base, 10, 10), (&other, 160, 10)]);
    let snap = snapshot(vec![
        css_bg("section", false),
        img_el("hero", 10, 10, &base),
        img_el("other", 160, 10, &other),
    ]);
    for a in [
        Assertion::NoPlaceholderImages { element: None },
        Assertion::NoNearDuplicateImages {
            max_hamming: Some(8),
        },
    ] {
        let r = assert_on(&frame, &snap, a.clone());
        assert_eq!(r.outcome, AssertionOutcome::Passed, "{a:?}: {:?}", r.detail);
        assert!(
            r.detail
                .as_deref()
                .unwrap()
                .contains("1 CSS-background image(s) not examined"),
            "{a:?}: {:?}",
            r.detail
        );
    }
    // The analyzer still says why, per row.
    let res = run_image(Some(&frame), Some(&snap));
    assert!(res
        .verdict
        .reason()
        .unwrap_or_default()
        .contains("CSS background"));
}

#[test]
fn a_css_background_marked_placeholder_still_fails() {
    let base = load("base.png");
    let frame = page(320, 120, PAGE, &[(&base, 10, 10)]);
    // Guard 1: page-wide, the marker is read although the pixels are not.
    let snap = snapshot(vec![css_bg("section", true), img_el("hero", 10, 10, &base)]);
    let r = assert_on(
        &frame,
        &snap,
        Assertion::NoPlaceholderImages { element: None },
    );
    assert_eq!(r.outcome, AssertionOutcome::Failed, "{:?}", r.detail);
    assert_eq!(r.confidence, None, "the marker is a deduction");
    // Guard 2: the single-element form is Unknown unless the row is marked.
    let one = |marked: bool| {
        let snap = snapshot(vec![css_bg("section", marked)]);
        assert_on(
            &frame,
            &snap,
            Assertion::NoPlaceholderImages {
                element: Some("section".into()),
            },
        )
    };
    assert_eq!(one(true).outcome, AssertionOutcome::Failed);
    let unmarked = one(false);
    assert_eq!(
        unmarked.outcome,
        AssertionOutcome::Unknown,
        "{:?}",
        unmarked.detail
    );
    assert!(unmarked.detail.unwrap().contains("CSS background"));
}

// ======================================================== review round 6

/// A compact central subject (~24% × 28%) with rays to the corners: diluted
/// over the 60%-wide caption box it read 0.31 and matched; over the 30% core
/// it is a filled subject.
#[test]
fn a_compact_hub_with_corner_rays_is_not_a_caption() {
    let mut hub = crossed(300, 200, 2.0, false);
    for y in 0..200u32 {
        for x in 0..300u32 {
            let (dx, dy) = ((x as f64 - 150.0) / 36.0, (y as f64 - 100.0) / 28.0);
            if dx * dx + dy * dy <= 1.0 {
                hub.put_pixel(x, y, image::Rgb(GREY_INK));
            }
        }
    }
    assert!(
        !pixels::is_crossed_box(&hub, 1.0),
        "a compact hub read as a placeholder"
    );
}

/// A slot marked as its placeholder fails even when another image on the
/// page cannot be read: the marker is a deduction, and the unreadable image's
/// Unknown must not mask it.
#[test]
fn a_marked_placeholder_fails_despite_an_unreadable_image() {
    let base = load("base.png");
    let frame = page(320, 120, PAGE, &[(&base, 10, 10)]);
    let mut unreadable = img_el("hero", 10, 10, &base);
    unreadable.visible = Some(false);
    let snap = snapshot(vec![css_bg("section", true), unreadable]);
    let r = assert_on(
        &frame,
        &snap,
        Assertion::NoPlaceholderImages { element: None },
    );
    assert_eq!(r.outcome, AssertionOutcome::Failed, "{:?}", r.detail);
    assert_eq!(r.confidence, None);
}

/// A page whose only images are a wrapper and its own SVG has ONE image, so
/// no pair is possible: a pass, not an Unknown.
#[test]
fn a_wrapper_and_its_svg_alone_are_one_image() {
    let base = load("base.png");
    let frame = page(200, 120, PAGE, &[(&base, 10, 10)]);
    let bbox = Region {
        x: 10,
        y: 10,
        w: 128,
        h: 96,
    };
    let mut wrapper = flat_el("arrow", MediaKind::RoleImg, bbox);
    wrapper.children_ids = vec!["arrow-svg".into()];
    let mut svg = flat_el("arrow-svg", MediaKind::Svg, bbox);
    svg.parent_id = Some("arrow".into());
    let snap = snapshot(vec![wrapper, svg]);
    let r = assert_on(
        &frame,
        &snap,
        Assertion::NoNearDuplicateImages {
            max_hamming: Some(8),
        },
    );
    assert_eq!(r.outcome, AssertionOutcome::Passed, "{:?}", r.detail);
    let d = r.detail.unwrap();
    assert!(d.contains("fewer than two") && d.contains("nested"), "{d}");
}

/// A cyclic parent chain ends (no hang), and two images that are each
/// other's "ancestor" are a producer error, not a nesting: both are kept.
#[test]
fn a_cyclic_parent_chain_ends_and_keeps_both_images() {
    let base = load("base.png");
    let other = load("other.png");
    let frame = page(320, 120, PAGE, &[(&base, 10, 10), (&other, 160, 10)]);
    let mut a = img_el("a", 10, 10, &base);
    a.parent_id = Some("b".into());
    let mut b = img_el("b", 160, 10, &other);
    b.parent_id = Some("a".into());
    let r = assert_on(
        &frame,
        &snapshot(vec![a, b]),
        Assertion::NoNearDuplicateImages {
            max_hamming: Some(8),
        },
    );
    assert_eq!(r.outcome, AssertionOutcome::Passed, "{:?}", r.detail);
    assert!(r.detail.unwrap().contains("1 pair(s) compared"));
}

// ======================================================== review round 7

/// Rays twice the diagonal tolerance wide must not excuse the hub between
/// them; the thin-stroked 64-px bordered placeholder must still match.
#[test]
fn thick_rays_do_not_excuse_a_central_hub() {
    // 300 × 200: tolerance max(2.5, 4% × 200) = 8, so rays 16 px wide.
    let mut hub = crossed(300, 200, 16.0, false);
    for y in 0..200u32 {
        for x in 0..300u32 {
            let (dx, dy) = ((x as f64 - 150.0) / 36.0, (y as f64 - 100.0) / 28.0);
            if dx * dx + dy * dy <= 1.0 {
                hub.put_pixel(x, y, image::Rgb(GREY_INK));
            }
        }
    }
    assert!(
        !pixels::is_crossed_box(&hub, 1.0),
        "thick rays excused the hub"
    );
    assert!(pixels::is_crossed_box(&crossed(64, 64, 3.0, true), 1.0));
}

/// A wrapper around SEVERAL child images is a group: it goes, and its
/// children are compared, so a duplicate pair among them fails.
#[test]
fn a_wrapper_with_two_children_hides_no_duplicate() {
    let base = load("base.png");
    let frame = page(320, 120, PAGE, &[(&base, 10, 10), (&base, 160, 10)]);
    let mut group = flat_el(
        "gallery",
        MediaKind::RoleImg,
        Region {
            x: 0,
            y: 0,
            w: 320,
            h: 120,
        },
    );
    group.children_ids = vec!["a".into(), "b".into()];
    let mut a = img_el("a", 10, 10, &base);
    a.parent_id = Some("gallery".into());
    let mut b = img_el("b", 160, 10, &base);
    b.parent_id = Some("gallery".into());
    let r = assert_on(
        &frame,
        &snapshot(vec![group, a, b]),
        Assertion::NoNearDuplicateImages {
            max_hamming: Some(8),
        },
    );
    assert_eq!(r.outcome, AssertionOutcome::Failed, "{:?}", r.detail);
    assert!(r.detail.unwrap().contains("'a' and 'b'"));
}

// ======================================================== review round 8

/// Icons are not slot images: five identical 20-CSS-px SVG check marks are
/// not duplicated content, while two identical 200 × 150 rasters are.
#[test]
fn identical_icons_are_not_near_duplicates_but_identical_slot_images_are() {
    let icon = image::RgbaImage::from_fn(20, 20, |x, y| {
        let on = (x as i32 - y as i32).abs() < 2 || (x + y) % 7 == 0;
        image::Rgba(if on { [0x16, 0xa3, 0x4a, 0xff] } else { PAGE })
    });
    let placed: Vec<(&image::RgbaImage, i64, i64)> =
        (0..5).map(|i| (&icon, 10 + 30 * i as i64, 10)).collect();
    let frame = page(200, 60, PAGE, &placed);
    let icons: Vec<Element> = (0..5)
        .map(|i| {
            flat_el(
                &format!("check{i}"),
                MediaKind::Svg,
                Region {
                    x: 10 + 30 * i,
                    y: 10,
                    w: 20,
                    h: 20,
                },
            )
        })
        .collect();
    let icons = snapshot(icons);
    let r = assert_on(
        &frame,
        &icons,
        Assertion::NoNearDuplicateImages {
            max_hamming: Some(8),
        },
    );
    assert_ne!(r.outcome, AssertionOutcome::Failed, "{:?}", r.detail);
    assert!(r
        .detail
        .unwrap()
        .contains("5 image(s) under 48 CSS px left out"));
    // The analyzer leaves them out of its pairs too, and says how many.
    let v = values(&frame, &icons);
    assert!(v.near_duplicates.value().unwrap().is_empty());
    assert_eq!(
        v.near_duplicates.provenance().source.as_ref().unwrap()["smallImagesSkipped"],
        5
    );

    let base = load("base.png");
    let slot = image::imageops::resize(&base, 200, 150, image::imageops::FilterType::Triangle);
    let frame = page(440, 170, PAGE, &[(&slot, 10, 10), (&slot, 225, 10)]);
    let snap = snapshot(vec![
        img_el("a", 10, 10, &slot),
        img_el("b", 225, 10, &slot),
    ]);
    let r = assert_on(
        &frame,
        &snap,
        Assertion::NoNearDuplicateImages {
            max_hamming: Some(8),
        },
    );
    assert_eq!(r.outcome, AssertionOutcome::Failed, "{:?}", r.detail);
}
