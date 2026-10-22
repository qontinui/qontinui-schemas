//! Text over images, measured on real pixels (plan
//! 2026-10-13-image-generation-and-visual-judgment, Phase 16): the
//! `text_over_image_contrast` assertion and the image analyzer's per-image
//! `text_contrast` value, against the frame pairs in
//! `tests/fixtures/text_over_image/` (written once by its `generate.py`).
//!
//! Each fixture is a frame A (text rendered) and a frame B (the same page,
//! text hidden). The mutation pair the plan names — a low-contrast overlay
//! must FAIL, a solid dark scrim must PASS — sits beside a HELD case and a
//! wide heading whose box spans the hero while its glyphs do not. As in
//! Phase 15's corpus, each mutation test checks BOTH the analyzer's value
//! and the assertion's verdict: a test of only the flip would pass against an
//! assertion that ignored the value.

use std::path::PathBuf;

use qontinui_vision_core::analyzers::color::wcag_contrast;
use qontinui_vision_core::analyzers::image::{
    text_contrast, GlyphColourSource, SampleMethod, TextOverImageContrast,
};
use qontinui_vision_core::observation::ObservationStatus;
use qontinui_vision_core::{
    analyzers, evaluate_assertion, AnalyzeInput, Analyzer, AnalyzerVerdict, Assertion,
    AssertionOutcome, AssertionResult, Element, ElementMedia, ElementSnapshot, EvalContext, Frame,
    FrameSource, MediaKind, ObjectFit, Region, Rgb, Severity, UnknownCode, VisibilityReason,
};
use serde_json::Value;

// ---------------------------------------------------------------- fixtures

fn dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/text_over_image")
}

fn manifest() -> Value {
    let raw = std::fs::read(dir().join("manifest.json")).expect("manifest.json");
    serde_json::from_slice(&raw).expect("manifest parses")
}

fn frame(name: &str) -> Frame {
    let img = image::open(dir().join(name))
        .unwrap_or_else(|e| panic!("fixture {name}: {e}"))
        .to_rgba8();
    Frame::from_rgba(img, FrameSource::synthetic_now())
}

fn region(v: &Value) -> Region {
    let n = |k: &str| v[k].as_i64().unwrap();
    Region {
        x: n("x") as i32,
        y: n("y") as i32,
        w: n("w") as u32,
        h: n("h") as u32,
    }
}

/// One fixture: frame A, frame B, and the snapshot of its hero and heading.
struct Case {
    a: Frame,
    b: Frame,
    snap: ElementSnapshot,
}

fn hero(bbox: Region) -> Element {
    Element {
        id: "hero".into(),
        bbox: Some(bbox),
        media: Some(ElementMedia {
            object_fit: Some(ObjectFit::Fill),
            ..ElementMedia::new(MediaKind::Img)
        }),
        ..Default::default()
    }
}

fn case(name: &str) -> Case {
    let m = manifest();
    let c = &m["cases"][name];
    assert!(c.is_object(), "no case {name} in manifest.json");
    let fg = c["fg"].as_array().unwrap();
    let ch = |i: usize| fg[i].as_u64().unwrap() as u8;
    let heading = Element {
        id: "heading".into(),
        bbox: Some(region(&c["text"])),
        text: Some(m["text"].as_str().unwrap().to_string()),
        fg_color: Some(Rgb::new(ch(0), ch(1), ch(2))),
        ..Default::default()
    };
    Case {
        a: frame(c["frame"].as_str().unwrap()),
        b: frame(c["text_hidden"].as_str().unwrap()),
        snap: ElementSnapshot {
            elements: vec![hero(region(&c["image"])), heading],
            ..Default::default()
        },
    }
}

impl Case {
    fn without_fg(mut self) -> Self {
        for e in &mut self.snap.elements {
            e.fg_color = None;
        }
        self
    }

    fn assert_with(
        &self,
        b: Option<&Frame>,
        min: Option<f64>,
        pct: Option<f64>,
    ) -> AssertionResult {
        evaluate_assertion(
            &Assertion::TextOverImageContrast {
                min_ratio: min,
                percentile: pct,
            },
            &EvalContext {
                snapshot: Some(&self.snap),
                frame: Some(&self.a),
                text_hidden_frame: b,
                ..Default::default()
            },
        )
    }

    /// The assertion at WCAG AA (4.5:1), default percentile.
    fn assert_aa(&self) -> AssertionResult {
        self.assert_with(Some(&self.b), Some(4.5), None)
    }

    fn analyze(&self, b: Option<&Frame>) -> analyzers::AnalyzerResult {
        analyzers::run(
            Analyzer::Image,
            &AnalyzeInput {
                frame: Some(&self.a),
                snapshot: Some(&self.snap),
                text_hidden_frame: b,
                ..Default::default()
            },
        )
    }

    /// The analyzer's reading of the heading over the hero.
    fn reading(&self) -> TextOverImageContrast {
        let r = self.analyze(Some(&self.b));
        let values = r.image_values.expect("a framed image run carries values");
        let hero = values.images.iter().find(|i| i.element == "hero").unwrap();
        let v = hero
            .text_contrast
            .value()
            .unwrap_or_else(|| panic!("no measured text contrast: {:?}", hero.text_contrast));
        assert_eq!(v.len(), 1, "{v:?}");
        v[0].clone()
    }
}

fn conf(r: &AssertionResult) -> f64 {
    let c = r
        .confidence
        .unwrap_or_else(|| panic!("a pixel verdict states its confidence: {r:?}"));
    assert!((0.0..=1.0).contains(&c) && c > 0.0, "confidence {c}");
    c
}

// ---------------------------------------------------------------- mutations

/// The plan's mutation pair: light text over a light image FAILS, and the
/// same text over a solid dark scrim PASSES. The value moves with it.
#[test]
fn a_low_contrast_overlay_fails_and_a_dark_scrim_passes() {
    let low = case("low_contrast");
    let scrim = case("scrim");
    let (rl, rs) = (low.assert_aa(), scrim.assert_aa());
    assert_eq!(rl.outcome, AssertionOutcome::Failed, "{:?}", rl.detail);
    assert_eq!(rs.outcome, AssertionOutcome::Passed, "{:?}", rs.detail);
    assert_eq!(
        conf(&rl),
        1.0,
        "a declared colour under a glyph mask, fully in frame"
    );
    assert_eq!(conf(&rs), 1.0);

    let (vl, vs) = (low.reading(), scrim.reading());
    assert_eq!(vl.method, SampleMethod::GlyphMask);
    assert_eq!(vs.method, SampleMethod::GlyphMask);
    assert!(
        vl.percentile_ratio < 1.5,
        "low contrast p5 {}",
        vl.percentile_ratio
    );
    assert!(
        vs.percentile_ratio > 10.0,
        "scrim p5 {}",
        vs.percentile_ratio
    );
    assert_eq!(vl.percentile, text_contrast::DEFAULT_PERCENTILE);
}

/// The HELD case passes at p5, while its MINIMUM — a stroke over a bright
/// speck — is below AA: the percentile is what is gated on, the minimum is
/// reported, and `percentile: 0` gates on the minimum.
#[test]
fn the_held_case_passes_at_p5_and_fails_on_its_minimum() {
    let held = case("held");
    let r = held.assert_aa();
    assert_eq!(r.outcome, AssertionOutcome::Passed, "{:?}", r.detail);
    let detail = r.detail.as_deref().unwrap();
    assert!(
        detail.contains("p5") && detail.contains("minimum"),
        "{detail}"
    );

    let v = held.reading();
    assert!(
        v.min_ratio < 4.5,
        "a speck under a stroke: minimum {}",
        v.min_ratio
    );
    assert!(v.percentile_ratio >= 4.5, "p5 {}", v.percentile_ratio);

    let at_min = held.assert_with(Some(&held.b), Some(4.5), Some(0.0));
    assert_eq!(
        at_min.outcome,
        AssertionOutcome::Failed,
        "{:?}",
        at_min.detail
    );
}

/// A block-level heading's box spans the hero's full width; its glyphs
/// cover the dark left part. Sampled under the glyphs it passes; the whole
/// box, which is mostly over the light right part, would not — computed here
/// independently, so the fixture is known to discriminate.
#[test]
fn a_wide_heading_is_measured_under_its_glyphs_not_its_box() {
    let wide = case("wide_heading");
    let r = wide.assert_aa();
    assert_eq!(r.outcome, AssertionOutcome::Passed, "{:?}", r.detail);
    let v = wide.reading();
    assert_eq!(v.method, SampleMethod::GlyphMask);
    let area = (v.region.w * v.region.h) as u64;
    assert!(
        v.sampled_px * 5 < area,
        "{} of {area} sampled",
        v.sampled_px
    );

    let fg = Rgb::new(255, 255, 255);
    let mut whole: Vec<f64> = Vec::new();
    for y in v.region.y as u32..(v.region.y as u32 + v.region.h) {
        for x in v.region.x as u32..(v.region.x as u32 + v.region.w) {
            let p = wide.b.buffer.get_pixel(x, y).0;
            whole.push(wcag_contrast(fg, Rgb::new(p[0], p[1], p[2])));
        }
    }
    whole.sort_by(f64::total_cmp);
    let p5 = text_contrast::nearest_rank(&whole, 5.0);
    assert!(
        p5 < 4.5,
        "the whole box must fail for the fixture to mean anything: {p5}"
    );
}

// ---------------------------------------------------------------- unknowns

#[test]
fn without_the_text_hidden_frame_the_answer_is_needs_multi_frame_input() {
    let held = case("held");
    let r = held.assert_with(None, Some(4.5), None);
    assert_eq!(r.outcome, AssertionOutcome::Unknown);
    assert_eq!(r.code, Some(UnknownCode::NeedsMultiFrameInput));
    assert!(r.confidence.is_none());

    // The analyzer: the value is unknown with the same code, and its absence
    // changes no verdict — the run is Checked, not Degraded.
    let a = held.analyze(None);
    assert_eq!(a.verdict, AnalyzerVerdict::Checked, "{:?}", a.verdict);
    let v = a.image_values.unwrap();
    let tc = &v.images[0].text_contrast;
    assert_eq!(tc.status(), ObservationStatus::Unknown);
    assert_eq!(
        tc.unknown_info().unwrap().code,
        UnknownCode::NeedsMultiFrameInput
    );
}

/// Frames A and B must be two captures of one layout: the same size and the
/// same scale factor. Either mismatch is Unknown with a reason naming it, and
/// on the analyzer a green Degraded.
#[test]
fn frames_of_another_size_or_scale_are_not_paired() {
    let held = case("held");
    let small = Frame::from_rgba(
        image::imageops::crop_imm(&held.b.buffer, 0, 0, 319, 180).to_image(),
        FrameSource::synthetic_now(),
    );
    let mut scaled = held.b.clone();
    scaled.source.scale_factor = 2.0;
    for (b, says) in [(&small, "319x180"), (&scaled, "scale factor 2")] {
        let r = held.assert_with(Some(b), Some(4.5), None);
        assert_eq!(r.outcome, AssertionOutcome::Unknown, "{:?}", r.detail);
        assert_eq!(r.code, Some(UnknownCode::InputMissing));
        assert!(
            r.detail.as_deref().unwrap().contains(says),
            "{:?}",
            r.detail
        );

        let a = held.analyze(Some(b));
        assert!(a.verdict.is_degraded(), "{:?}", a.verdict);
        assert!(a.conclusive, "a degraded image run stays green");
        assert!(a.verdict.reason().unwrap().contains("text-hidden frame"));
        let v = a.image_values.unwrap();
        assert_eq!(
            v.images[0].text_contrast.status(),
            ObservationStatus::Unknown
        );
    }
}

#[test]
fn an_undeclared_threshold_or_a_bad_percentile_is_unknown() {
    let held = case("held");
    let r = held.assert_with(Some(&held.b), None, None);
    assert_eq!(r.code, Some(UnknownCode::InputMissing));
    assert!(r.detail.as_deref().unwrap().contains("min_ratio"));
    for p in [-1.0, 101.0, f64::NAN] {
        let r = held.assert_with(Some(&held.b), Some(4.5), Some(p));
        assert_eq!(r.outcome, AssertionOutcome::Unknown, "{p}");
        assert_eq!(r.code, Some(UnknownCode::InputMissing));
    }
}

/// The Phase 15 rules for the IMAGE: text over an image that paints nothing
/// or cannot be attributed is skipped (and said so), and with nothing left
/// the answer is Unknown, never a vacuous pass.
#[test]
fn text_over_an_unrendered_or_unreadable_image_is_skipped() {
    for (visible, reason) in [
        (Some(false), None),
        (Some(false), Some(VisibilityReason::Occluded)),
        (Some(false), Some(VisibilityReason::Hidden)),
    ] {
        let mut held = case("held");
        held.snap.elements[0].visible = visible;
        held.snap.elements[0].visibility_reason = reason.clone();
        let r = held.assert_aa();
        assert_eq!(
            r.outcome,
            AssertionOutcome::Unknown,
            "{reason:?}: {:?}",
            r.detail
        );
        let d = r.detail.as_deref().unwrap();
        assert!(d.contains("no text lies over a readable image"), "{d}");
        assert!(d.contains("skipped"), "{d}");
    }
}

/// A text box that only TOUCHES an image — its glyph-free line padding over
/// the image's top edge — is not text over it, and is not measured.
#[test]
fn a_box_that_touches_an_image_with_no_glyph_over_it_is_not_over_it() {
    let mut held = case("held");
    let t = held.snap.elements[1].bbox.unwrap();
    // The bottom 6 px of the line box are padding under the ink.
    held.snap.elements[0].bbox = Some(Region {
        x: 0,
        y: t.y + t.h as i32 - 3,
        w: 320,
        h: 40,
    });
    let r = held.assert_aa();
    assert_eq!(r.outcome, AssertionOutcome::Unknown, "{:?}", r.detail);
    assert!(
        r.detail.as_deref().unwrap().contains("glyphs do not reach"),
        "{:?}",
        r.detail
    );
    let a = held.analyze(Some(&held.b));
    let v = a.image_values.unwrap();
    assert_eq!(
        v.images[0].text_contrast.status(),
        ObservationStatus::Absent
    );
}

/// No text over the image at all: the analyzer's value is ABSENT (a looked-at
/// image with nothing over it), not unknown, with or without frame B.
#[test]
fn an_image_with_no_text_over_it_reads_absent() {
    let mut held = case("held");
    held.snap.elements.truncate(1);
    for b in [None, Some(&held.b)] {
        let v = held.analyze(b).image_values.unwrap();
        assert_eq!(
            v.images[0].text_contrast.status(),
            ObservationStatus::Absent
        );
    }
    // An image with no box was not looked at: unknown, not absent.
    held.snap.elements[0].bbox = None;
    let v = held.analyze(Some(&held.b)).image_values.unwrap();
    assert_eq!(
        v.images[0].text_contrast.status(),
        ObservationStatus::Unknown
    );
}

// ---------------------------------------------------------------- confidence

/// With no `fg_color` the glyph colour is measured from frame A's glyph
/// core; the verdicts hold, and the reading says where its colour came from.
#[test]
fn the_glyph_colour_is_measured_when_fg_color_is_absent() {
    let held = case("held").without_fg();
    let r = held.assert_aa();
    assert_eq!(r.outcome, AssertionOutcome::Passed, "{:?}", r.detail);
    let v = held.reading();
    assert_eq!(v.glyph_colour_source, GlyphColourSource::Measured);
    let c = v.glyph_colour;
    assert!(
        c.r >= 240 && c.g >= 240 && c.b >= 240,
        "white text measured as {c:?}"
    );
    assert!(v.confidence > 0.0 && v.confidence <= 1.0);

    let low = case("low_contrast").without_fg();
    assert_eq!(low.assert_aa().outcome, AssertionOutcome::Failed);
}

/// A twin in which the text was NOT hidden (frame B identical to A) shows
/// no glyph mask: the whole box is sampled, at no more than the fallback's
/// confidence, and the detail says so. Without `fg_color` there is no
/// colour to measure, and the answer is Unknown.
#[test]
fn an_unhidden_twin_falls_back_to_the_box_at_lower_confidence() {
    let scrim = case("scrim");
    let r = scrim.assert_with(Some(&scrim.a), Some(4.5), None);
    assert!(conf(&r) <= text_contrast::BBOX_FALLBACK_CONFIDENCE, "{r:?}");
    assert!(
        r.detail.as_deref().unwrap().contains("fallback"),
        "{:?}",
        r.detail
    );

    let bare = case("scrim").without_fg();
    let r = bare.assert_with(Some(&bare.a), Some(4.5), None);
    assert_eq!(r.outcome, AssertionOutcome::Unknown, "{:?}", r.detail);
    assert_eq!(r.code, Some(UnknownCode::InputMissing));
}

/// A failure is evidence whatever else could not be measured; a pass needs
/// every text over a readable image measured.
#[test]
fn a_failure_stands_beside_an_unmeasured_text_and_a_pass_does_not() {
    // A second text element over the hero, with no colour of its own and no
    // glyphs drawn in either frame: no mask, so no colour can be measured.
    let unmeasurable = |id: &str| Element {
        id: id.into(),
        bbox: Some(Region {
            x: 200,
            y: 120,
            w: 100,
            h: 30,
        }),
        text: Some("caption".into()),
        ..Default::default()
    };
    let mut low = case("low_contrast");
    low.snap.elements.push(unmeasurable("caption"));
    let r = low.assert_aa();
    assert_eq!(r.outcome, AssertionOutcome::Failed, "{:?}", r.detail);
    assert!(r
        .detail
        .as_deref()
        .unwrap()
        .contains("could not be measured"));

    let mut scrim = case("scrim");
    scrim.snap.elements.push(unmeasurable("caption"));
    let r = scrim.assert_aa();
    assert_eq!(r.outcome, AssertionOutcome::Unknown, "{:?}", r.detail);
}

// ---------------------------------------------------------------- no gating

/// The image analyzer stays Info-only whatever frame B does: no finding
/// above Info, and the verdict is never Blocked, with no twin, a paired one
/// or an unpairable one.
#[test]
fn the_image_analyzer_never_gates_on_text_contrast() {
    let low = case("low_contrast");
    let small = Frame::from_rgba(
        image::imageops::crop_imm(&low.b.buffer, 0, 0, 100, 100).to_image(),
        FrameSource::synthetic_now(),
    );
    for b in [None, Some(&low.b), Some(&small)] {
        let r = low.analyze(b);
        assert!(r.conclusive, "{:?}", r.verdict);
        assert!(r.findings.iter().all(|f| f.severity == Severity::Info));
    }
}
