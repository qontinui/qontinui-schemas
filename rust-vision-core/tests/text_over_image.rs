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
    text_contrast, GlyphColourSource, TextOverImageContrast,
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
        let tc = hero.text_contrast.as_ref().expect("this crate states one");
        let v = tc
            .value()
            .unwrap_or_else(|| panic!("no measured text contrast: {tc:?}"));
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
    let tc = v.images[0].text_contrast.as_ref().unwrap();
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
            v.images[0].text_contrast.as_ref().unwrap().status(),
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

/// The Phase 15 rules for the IMAGE, as they apply to text over it: an image
/// that paints nothing (`hidden`) is skipped and said so, and with nothing
/// left the answer is Unknown, never a vacuous pass. An image reported
/// OCCLUDED, or `visible: false` with no reason, is still MEASURED: frame B is
/// read under the glyphs and the image's crop never is.
#[test]
fn text_over_an_unrendered_image_is_skipped_and_over_an_occluded_one_measured() {
    let mut held = case("held");
    held.snap.elements[0].visible = Some(false);
    held.snap.elements[0].visibility_reason = Some(VisibilityReason::Hidden);
    let r = held.assert_aa();
    assert_eq!(r.outcome, AssertionOutcome::Unknown, "{:?}", r.detail);
    let d = r.detail.as_deref().unwrap();
    assert!(d.contains("no text lies over a rendered image"), "{d}");
    assert!(
        d.contains("unrendered image(s) with text over them skipped"),
        "{d}"
    );

    for reason in [None, Some(VisibilityReason::Occluded)] {
        let mut held = case("held");
        held.snap.elements[0].visible = Some(false);
        held.snap.elements[0].visibility_reason = reason.clone();
        let r = held.assert_aa();
        assert_eq!(
            r.outcome,
            AssertionOutcome::Passed,
            "{reason:?}: {:?}",
            r.detail
        );
    }
}

/// The usual `<img>` hero: the image sits under a sibling `absolute inset-0`
/// scrim, so the SDK reports it occluded by that scrim. The scrim fixture,
/// with that snapshot, must still PASS — and the light overlay still FAIL.
#[test]
fn a_hero_occluded_by_its_scrim_is_measured() {
    fn occluded(mut c: Case) -> Case {
        let hero_box = c.snap.elements[0].bbox;
        c.snap.elements[0].visible = Some(false);
        c.snap.elements[0].visibility_reason = Some(VisibilityReason::Occluded);
        c.snap.elements[0].occluded_by = Some("scrim-layer".into());
        c.snap.elements.push(Element {
            id: "scrim-layer".into(),
            bbox: hero_box,
            ..Default::default()
        });
        c
    }
    let scrim = occluded(case("scrim"));
    let r = scrim.assert_aa();
    assert_eq!(r.outcome, AssertionOutcome::Passed, "{:?}", r.detail);
    assert!(scrim.reading().percentile_ratio > 10.0);
    let low = occluded(case("low_contrast"));
    assert_eq!(low.assert_aa().outcome, AssertionOutcome::Failed);
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
        v.images[0].text_contrast.as_ref().unwrap().status(),
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
            v.images[0].text_contrast.as_ref().unwrap().status(),
            ObservationStatus::Absent
        );
    }
    // An image with no box was not looked at: unknown, not absent.
    held.snap.elements[0].bbox = None;
    let v = held.analyze(Some(&held.b)).image_values.unwrap();
    assert_eq!(
        v.images[0].text_contrast.as_ref().unwrap().status(),
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
/// no glyph mask. That is UNKNOWN, never a failure: "not hidden" and "text
/// invisible against the image" read the same, and sampling the box of text
/// that is still painted reads its own glyphs at ~1:1. The scrim case, which
/// passes at ~18:1 when hidden, must not fail here.
#[test]
fn an_unhidden_twin_is_unknown_not_a_failure() {
    for c in [case("scrim"), case("scrim").without_fg(), case("held")] {
        let r = c.assert_with(Some(&c.a), Some(4.5), None);
        assert_eq!(r.outcome, AssertionOutcome::Unknown, "{:?}", r.detail);
        assert_eq!(r.code, Some(UnknownCode::InputMissing));
        assert!(r.confidence.is_none());
        let d = r.detail.as_deref().unwrap();
        assert!(d.contains("was not hidden"), "{d}");
    }
}

/// A failure is evidence whatever else could not be measured; a pass needs
/// every text over a rendered image measured.
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

// ---------------------------------------------------------------- registration

fn with_twin(c: &Case, b: image::RgbaImage) -> Frame {
    assert_eq!(b.dimensions(), c.a.buffer.dimensions());
    Frame::from_rgba(b, FrameSource::synthetic_now())
}

/// Frame B must be the same layout as frame A, not merely the same size: one
/// shifted by a pixel, zoomed, or captured from another page turns the image's
/// own texture into a "glyph mask". Each is Unknown with the measured share,
/// never a verdict.
#[test]
fn an_unregistered_twin_is_unknown_with_the_measured_share() {
    let wide = case("wide_heading");
    // Shifted right by one pixel (column 0 repeated).
    let shifted = image::RgbaImage::from_fn(320, 180, |x, y| {
        *wide.b.buffer.get_pixel(x.saturating_sub(1), y)
    });
    // Zoomed 2% about the centre.
    let held = case("held");
    let big = image::imageops::resize(
        &held.b.buffer,
        327,
        184,
        image::imageops::FilterType::Triangle,
    );
    let zoomed = image::imageops::crop_imm(&big, 3, 2, 320, 180).to_image();
    // Another page's text-hidden capture entirely.
    let other = case("low_contrast");
    for (c, b) in [
        (&wide, with_twin(&wide, shifted)),
        (&held, with_twin(&held, zoomed)),
        (&other, case("held").b),
    ] {
        let r = c.assert_with(Some(&b), Some(4.5), None);
        assert_eq!(r.outcome, AssertionOutcome::Unknown, "{:?}", r.detail);
        assert_eq!(r.code, Some(UnknownCode::InputMissing));
        let d = r.detail.as_deref().unwrap();
        assert!(d.contains("not registered") && d.contains('%'), "{d}");

        let v = c.analyze(Some(&b)).image_values.unwrap();
        let tc = v.images[0].text_contrast.as_ref().unwrap();
        assert_eq!(tc.status(), ObservationStatus::Unknown);
        assert!(tc.unknown_info().unwrap().detail.contains("not registered"));
    }
}

// ---------------------------------------------------------------- leaves and labels

/// A short label in a full-width block box covers a fraction of a percent of
/// it ("Hi" at 20 px in a 1280x28 box: about 0.3%). The mask floor is in CSS
/// px², not a share of the box, so it is measured.
#[test]
fn a_short_label_in_a_wide_box_is_measured() {
    let (w, h) = (1280u32, 120u32);
    let b = image::RgbaImage::from_fn(w, h, |x, y| {
        let v = (40 + (x * 7 + y * 13) % 30) as u8;
        image::Rgba([v, v, v.saturating_add(8), 255])
    });
    let mut a = b.clone();
    // "Hi": an H (two stems and a bar) and an i (stem and dot), white.
    let strokes = [
        (20, 46, 3, 15),
        (29, 46, 3, 15),
        (23, 52, 6, 3),
        (36, 50, 3, 11),
        (36, 46, 3, 2),
    ];
    for (x0, y0, sw, sh) in strokes {
        for y in y0..y0 + sh {
            for x in x0..x0 + sw {
                a.put_pixel(x, y, image::Rgba([255, 255, 255, 255]));
            }
        }
    }
    let label = Element {
        id: "label".into(),
        bbox: Some(Region {
            x: 0,
            y: 40,
            w,
            h: 28,
        }),
        text: Some("Hi".into()),
        fg_color: Some(Rgb::new(255, 255, 255)),
        ..Default::default()
    };
    let c = Case {
        a: Frame::from_rgba(a, FrameSource::synthetic_now()),
        b: Frame::from_rgba(b, FrameSource::synthetic_now()),
        snap: ElementSnapshot {
            elements: vec![hero(Region { x: 0, y: 0, w, h }), label],
            ..Default::default()
        },
    };
    let r = c.assert_aa();
    assert_eq!(r.outcome, AssertionOutcome::Passed, "{:?}", r.detail);
    let v = c.reading();
    assert!(
        v.sampled_px * 200 < (w * 28) as u64,
        "{} glyph px",
        v.sampled_px
    );
}

/// A wrapper whose innerText includes a child's (a link around a heading) is
/// not measured against its own colour: only the LEAF is. Here the wrapper
/// declares a dark colour that would fail on the dark scrim.
#[test]
fn only_leaf_text_elements_are_measured() {
    let mut scrim = case("scrim");
    let t = scrim.snap.elements[1].bbox.unwrap();
    scrim.snap.elements.push(Element {
        id: "link".into(),
        bbox: Some(Region {
            x: t.x - 4,
            y: t.y - 4,
            w: t.w + 8,
            h: t.h + 8,
        }),
        text: scrim.snap.elements[1].text.clone(),
        fg_color: Some(Rgb::new(20, 20, 20)),
        children_ids: vec!["heading".into()],
        ..Default::default()
    });
    let r = scrim.assert_aa();
    assert_eq!(r.outcome, AssertionOutcome::Passed, "{:?}", r.detail);
    assert_eq!(scrim.reading().element, "heading");
    // The wrapper is disclosed: text it paints beside its child is not measured.
    assert!(
        r.detail
            .as_deref()
            .unwrap()
            .contains("1 wrapper text element(s)"),
        "{:?}",
        r.detail
    );
}

// ---------------------------------------------------------------- round 3

/// A case's frame B, swapped for another file named in the manifest.
fn alt_twin(name: &str, key: &str) -> Frame {
    let m = manifest();
    frame(m["cases"][name][key].as_str().unwrap())
}

/// White text whose second word sits on a near-white (253 ± 1) region: the
/// A/B difference there is under GLYPH_DIFF_MIN, so the mask cannot see
/// those glyphs — and they are the invisible ones. Found as FAINT pixels
/// from frame B, they make it fail; it must never pass.
#[test]
fn glyphs_the_mask_cannot_see_never_pass() {
    let sky = case("sky");
    let r = sky.assert_aa();
    assert_eq!(r.outcome, AssertionOutcome::Failed, "{:?}", r.detail);
    let v = sky.reading();
    assert!(v.faint_px > 0, "{v:?}");
    assert!(v.percentile_ratio < 1.5, "{v:?}");
}

/// The same scene from a capture with sub-threshold noise: faint pixels are
/// not trusted, the invisible ink is ESTIMATED, and the answer is Unknown
/// naming it — never a pass.
#[test]
fn with_a_noisy_capture_invisible_glyphs_are_unknown_not_a_pass() {
    let mut sky = case("sky");
    for (i, p) in sky.b.buffer.pixels_mut().enumerate() {
        if i % 50 == 0 {
            p.0[0] = p.0[0].saturating_sub(1).max(1);
        }
    }
    let r = sky.assert_aa();
    assert_ne!(r.outcome, AssertionOutcome::Passed, "{:?}", r.detail);
    let v = sky.reading();
    assert_eq!(v.faint_px, 0, "faint pixels must not be trusted: {v:?}");
    assert!(v.hidden_px_estimate > 0, "{v:?}");
    assert!(v.percentile_ratio_with_hidden < 4.5, "{v:?}");
}

/// A wide heading whose box runs over a glyph-free white region.
///
/// Near-white (253) but not the glyph colour: on a noise-free capture a glyph
/// there would differ, and none does, so the region is MEASURED as empty and
/// the heading passes on its glyphs. Exactly the glyph colour (255): a glyph
/// there would be invisible to every difference, so the ink it could hold is
/// estimated at the glyphs' own density and the answer is Unknown — never a
/// pass.
#[test]
fn a_box_over_a_glyph_free_white_region_never_falsely_passes() {
    for (v, want) in [
        (253u8, AssertionOutcome::Passed),
        (255, AssertionOutcome::Unknown),
    ] {
        let mut wide = case("wide_heading");
        let t = wide.snap.elements[1].bbox.unwrap();
        for y in t.y as u32..(t.y as u32 + t.h) {
            for x in 240..300u32 {
                for f in [&mut wide.a, &mut wide.b] {
                    f.buffer.put_pixel(x, y, image::Rgba([v, v, v, 255]));
                }
            }
        }
        let r = wide.assert_aa();
        assert_eq!(r.outcome, want, "{v}: {:?}", r.detail);
        let est = wide.reading().hidden_px_estimate;
        if v == 255 {
            let d = r.detail.as_deref().unwrap();
            assert!(d.contains("matches its glyph colour"), "{d}");
            assert!(est > 0);
        } else {
            assert_eq!(est, 0, "a 253 region hides no undetectable ink");
        }
    }
}

/// A dark `text-shadow` glow under white text: the hide removes it, so every
/// halo pixel differs. The halo moved AWAY from the glyph colour, so it is
/// neither in the mask nor counted against registration: the text is judged
/// on its glyph pixels — a pass on a mid-tone hero, a failure on a light one,
/// never Unknown for the glow.
#[test]
fn a_glow_text_shadow_is_judged_on_the_glyph_pixels() {
    let mid = case("glow_mid");
    let r = mid.assert_aa();
    assert_eq!(r.outcome, AssertionOutcome::Passed, "{:?}", r.detail);
    let light = case("glow_light");
    let r = light.assert_aa();
    assert_eq!(r.outcome, AssertionOutcome::Failed, "{:?}", r.detail);
    // The glyph pixels sampled are about as many as without the glow.
    let plain = case("scrim").reading().sampled_px as f64;
    let glowing = mid.reading().sampled_px as f64;
    assert!((glowing / plain - 1.0).abs() < 0.25, "{glowing} vs {plain}");

    // A banner-sized image just around the text: under the heavy glow every
    // tile of it away from the text lies in the halo, so registration passes
    // (and the light case FAILS on its glyphs) only because a pixel
    // that moved AWAY from the text colour is a removed glow, not
    // misregistration.
    for name in ["glow_strong", "glow_mid"] {
        let mut banner = case(name);
        let t = banner.snap.elements[1].bbox.unwrap();
        banner.snap.elements[0].bbox = Some(Region {
            x: t.x - 20,
            y: t.y - 20,
            w: t.w + 40,
            h: t.h + 40,
        });
        let r = banner.assert_aa();
        let want = if name == "glow_mid" {
            AssertionOutcome::Passed
        } else {
            AssertionOutcome::Failed
        };
        assert_eq!(r.outcome, want, "{name}: {:?}", r.detail);
    }
}

/// The differing pixels moved AWAY from the declared colour (it is not what
/// paints): Unknown, saying so — not "not hidden".
#[test]
fn a_declared_colour_nothing_moved_toward_is_unknown_saying_so() {
    let mut scrim = case("scrim");
    scrim.snap.elements[1].fg_color = Some(Rgb::new(20, 20, 20));
    let r = scrim.assert_aa();
    assert_eq!(r.outcome, AssertionOutcome::Unknown, "{:?}", r.detail);
    let d = r.detail.as_deref().unwrap();
    assert!(d.contains("moved toward its glyph colour"), "{d}");
}

/// Untracked text over the image (a 12 px photo credit absent from the
/// snapshot) is a compact cluster of tiles, not misregistration.
#[test]
fn an_untracked_credit_line_does_not_unregister_the_frames() {
    let c = case("credit");
    let r = c.assert_aa();
    assert_eq!(r.outcome, AssertionOutcome::Passed, "{:?}", r.detail);
}

/// An unhidden twin whose glyphs were re-rasterised a quarter pixel over: its
/// edges differ, but frame B still paints the glyphs. Unknown, not a
/// full-confidence failure.
#[test]
fn a_twin_that_still_paints_the_glyphs_is_unknown() {
    let scrim = case("scrim");
    let b = alt_twin("scrim", "text_hidden_rerastered");
    let r = scrim.assert_with(Some(&b), Some(4.5), None);
    assert_eq!(r.outcome, AssertionOutcome::Unknown, "{:?}", r.detail);
    assert!(
        r.detail.as_deref().unwrap().contains("still paints"),
        "{:?}",
        r.detail
    );
}

// ---------------------------------------------------------------- round 4

/// A white heading in a FULL-WIDTH block box, with a clipped 255 highlight
/// under its last ~12% of ink. Averaged over the box's blank run, the hidden
/// ink's density was diluted into a pass; taken in the densest column band
/// (where the glyphs are), it is not. Must never pass.
#[test]
fn a_clipped_highlight_under_the_last_word_never_passes_in_a_full_width_box() {
    let c = case("clipped");
    let r = c.assert_aa();
    assert_ne!(r.outcome, AssertionOutcome::Passed, "{:?}", r.detail);
    let v = c.reading();
    assert!(v.hidden_px_estimate > 0, "{v:?}");
    assert!(v.percentile_ratio_with_hidden < 4.5, "{v:?}");
}

/// A parallax layer — the left 45% of the hero, under the text — moved 2 px
/// between the captures. Under half the image's tiles, so the global check
/// passes it; the LOCAL ring around the text sees content a shift lines up,
/// and the answer is Unknown. The registered twin of the same page is judged.
#[test]
fn a_parallax_layer_under_the_text_is_unknown() {
    let c = case("parallax");
    // 2 px, and 4 and 6 px: beyond the shift search, caught because moved
    // texture does not change toward one colour the way text does.
    for key in [
        "text_hidden_parallax",
        "text_hidden_parallax4",
        "text_hidden_parallax6",
    ] {
        let moved = alt_twin("parallax", key);
        let r = c.assert_with(Some(&moved), Some(4.5), None);
        assert_eq!(
            r.outcome,
            AssertionOutcome::Unknown,
            "{key}: {:?}",
            r.detail
        );
        assert!(
            r.detail.as_deref().unwrap().contains("displaced"),
            "{key}: {:?}",
            r.detail
        );
    }
    let control = c.assert_aa();
    assert_ne!(
        control.outcome,
        AssertionOutcome::Unknown,
        "{:?}",
        control.detail
    );
}

/// Glyph-coloured speckle that belongs to the IMAGE (a 253 / 225 mottle under
/// white text) is not a glyph frame B still paints: it is unchanged in frame
/// A. The text is judged — and fails, as it should.
#[test]
fn glyph_coloured_speckle_in_the_image_is_not_a_still_painted_glyph() {
    let c = case("mottled");
    let r = c.assert_aa();
    assert_eq!(r.outcome, AssertionOutcome::Failed, "{:?}", r.detail);
}

/// One flat tile that differs (untracked content on a flat patch of an
/// otherwise textured hero) is no evidence of a different capture: the flat
/// arm of the global check needs at least four flat tiles. The `parallax`
/// hero (registered twin) is finely textured everywhere, so the patch is its
/// only flat tile, and the verdict must be the one the page has without it.
#[test]
fn a_single_differing_flat_tile_does_not_unregister_the_frames() {
    let mut c = case("parallax");
    let control = c.assert_aa().outcome;
    assert_ne!(control, AssertionOutcome::Unknown);
    // A tile of the image's 16 px grid (the hero starts at y = 10), far from
    // the text and its local ring, made flat in both frames.
    for y in 106..122u32 {
        for x in 288..304u32 {
            for f in [&mut c.a, &mut c.b] {
                f.buffer.put_pixel(x, y, image::Rgba([60, 60, 60, 255]));
            }
        }
    }
    // Untracked content in frame A only: a 4x4 mark, 6% of the tile.
    for y in 112..116u32 {
        for x in 294..298u32 {
            c.a.buffer
                .put_pixel(x, y, image::Rgba([255, 255, 255, 255]));
        }
    }
    let r = c.assert_aa();
    assert_eq!(r.outcome, control, "{:?}", r.detail);
}

// ---------------------------------------------------------------- round 5

/// An UNTRACKED tagline just under the heading, inside its local ring, is
/// text that only frame A carries: every differing pixel moved toward its
/// colour, so it is tolerated, and the heading's verdict is the one the page
/// has without it.
#[test]
fn an_untracked_tagline_inside_the_ring_is_tolerated() {
    let control = case("parallax").assert_aa().outcome;
    let r = case("tagline").assert_aa();
    assert_eq!(r.outcome, control, "{:?}", r.detail);
    assert_ne!(r.outcome, AssertionOutcome::Unknown);
}

/// A re-rasterised UNHIDDEN twin on a plain dark card. Its glyph cores lie in
/// the box and are glyph-coloured; estimating the image's own glyph-coloured
/// rate from inside the box counted them and measured glyph against glyph (a
/// false FAIL). Read from the ring around the text, the rate is 0: Unknown.
#[test]
fn an_unhidden_twin_on_a_plain_card_is_unknown() {
    let card = case("card");
    let r = card.assert_aa();
    assert_eq!(r.outcome, AssertionOutcome::Passed, "{:?}", r.detail);
    let b = alt_twin("card", "text_hidden_rerastered");
    let r = card.assert_with(Some(&b), Some(4.5), None);
    assert_eq!(r.outcome, AssertionOutcome::Unknown, "{:?}", r.detail);
    assert!(
        r.detail.as_deref().unwrap().contains("still paints"),
        "{:?}",
        r.detail
    );
}

/// A long regular line, then a HEAVY word wholly on a clipped 255 patch, in
/// a full-width box: the hidden word is denser than any visible band. At the
/// regular line's densest band density its ink is 4% of the samples (p5
/// passes); the bands over the patch are too blind to qualify, so their
/// hidden pixels all count as ink (8%). Must never pass.
#[test]
fn a_heavy_word_on_a_clipped_patch_never_passes() {
    let c = case("heavy");
    let r = c.assert_aa();
    assert_ne!(r.outcome, AssertionOutcome::Passed, "{:?}", r.detail);
    let v = c.reading();
    assert!(v.percentile_ratio_with_hidden < 4.5, "{v:?}");
}

// ---------------------------------------------------------------- round 6

/// A heavy "M" (stroke 5) or "W" (stroke 4) on a TIGHT 255 patch after a
/// 71-character regular line, in a full-width box. At the regular line's
/// band density (~0.2) the hidden ink is under 5% of the samples and p5
/// passes; with every hidden pixel counted as ink it is well over, and it
/// does not. Must never pass.
#[test]
fn a_heavy_glyph_on_a_tight_patch_never_passes() {
    for name in ["heavy_m", "heavy_w"] {
        let c = case(name);
        let r = c.assert_aa();
        assert_ne!(
            r.outcome,
            AssertionOutcome::Passed,
            "{name}: {:?}",
            r.detail
        );
        let v = c.reading();
        assert!(v.percentile_ratio_with_hidden < 4.5, "{name}: {v:?}");
    }
}

/// A textured panel behind the first letters, reaching 8 px past the text
/// box on one side, moved 5 px between the captures: it touches only a few
/// tiles of the ring's FIRST row — under the ring share — and one displaced
/// tile there is enough. The registered twin is judged.
#[test]
fn a_panel_moving_just_under_the_text_is_unknown() {
    let c = case("panel");
    let moved = alt_twin("panel", "text_hidden_moved");
    let r = c.assert_with(Some(&moved), Some(4.5), None);
    assert_eq!(r.outcome, AssertionOutcome::Unknown, "{:?}", r.detail);
    assert!(
        r.detail.as_deref().unwrap().contains("displaced"),
        "{:?}",
        r.detail
    );
    let control = c.assert_aa();
    assert_ne!(
        control.outcome,
        AssertionOutcome::Unknown,
        "{:?}",
        control.detail
    );
}

/// Only pixels the glyph covers at least half are sampled. Here most of the
/// "glyph" is a 30%-coverage veil over a LIGHT region (a low-coverage edge
/// clears the difference threshold easily far from the glyph colour), while
/// its fully covered core sits over a mid-grey one. Sampled whole, the light
/// region dominates and p5 passes; sampled by coverage, the core decides and
/// it fails at about 4:1.
#[test]
fn low_coverage_edge_pixels_are_not_sampled() {
    let (w, h) = (200u32, 100u32);
    let b = image::RgbaImage::from_fn(w, h, |x, _| {
        let v = if x < 100 { 110 } else { 240 };
        image::Rgba([v, v, v, 255])
    });
    let mut a = b.clone();
    // Fully covered black core over the mid-grey half: 30 px.
    for y in 40..46u32 {
        for x in 20..25u32 {
            a.put_pixel(x, y, image::Rgba([0, 0, 0, 255]));
        }
    }
    // A 30%-coverage veil over the light half: 0.7 * 240 = 168, 1200 px.
    for y in 35..55u32 {
        for x in 120..180u32 {
            a.put_pixel(x, y, image::Rgba([168, 168, 168, 255]));
        }
    }
    let text = Element {
        id: "label".into(),
        bbox: Some(Region {
            x: 10,
            y: 30,
            w: 180,
            h: 30,
        }),
        text: Some("label".into()),
        fg_color: Some(Rgb::new(0, 0, 0)),
        ..Default::default()
    };
    let c = Case {
        a: Frame::from_rgba(a, FrameSource::synthetic_now()),
        b: Frame::from_rgba(b, FrameSource::synthetic_now()),
        snap: ElementSnapshot {
            elements: vec![hero(Region { x: 0, y: 0, w, h }), text],
            ..Default::default()
        },
    };
    let r = c.assert_aa();
    assert_eq!(r.outcome, AssertionOutcome::Failed, "{:?}", r.detail);
    let v = c.reading();
    assert_eq!(v.sampled_px, 30, "only the fully covered core: {v:?}");
}

/// The producer's REPORT of what it hid (`text_hidden_ids`). A text element
/// not in it is Unknown "not among the elements the producer reports it
/// hid"; one in it is measured, and the still-painted inference — which
/// exists only to guess the hide — is skipped.
#[test]
fn a_reported_hide_list_decides_which_text_is_measured() {
    let scrim = case("scrim");
    let eval = |ids: Option<&[String]>, b: &Frame| {
        evaluate_assertion(
            &Assertion::TextOverImageContrast {
                min_ratio: Some(4.5),
                percentile: None,
            },
            &EvalContext {
                snapshot: Some(&scrim.snap),
                frame: Some(&scrim.a),
                text_hidden_frame: Some(b),
                text_hidden_ids: ids,
                ..Default::default()
            },
        )
    };
    let listed = vec!["heading".to_string()];
    let other = vec!["caption".to_string()];
    assert_eq!(
        eval(Some(&listed), &scrim.b).outcome,
        AssertionOutcome::Passed
    );
    let r = eval(Some(&other), &scrim.b);
    assert_eq!(r.outcome, AssertionOutcome::Unknown, "{:?}", r.detail);
    assert!(
        r.detail.as_deref().unwrap().contains("not among"),
        "{:?}",
        r.detail
    );
    let r = eval(Some(&[]), &scrim.b);
    assert_eq!(r.outcome, AssertionOutcome::Unknown, "{:?}", r.detail);

    // The analyzer value follows the same list.
    let v = analyzers::run(
        Analyzer::Image,
        &AnalyzeInput {
            frame: Some(&scrim.a),
            snapshot: Some(&scrim.snap),
            text_hidden_frame: Some(&scrim.b),
            text_hidden_ids: Some(&other),
            ..Default::default()
        },
    )
    .image_values
    .unwrap();
    let tc = v.images[0].text_contrast.as_ref().unwrap();
    assert_eq!(tc.status(), ObservationStatus::Unknown);
}
