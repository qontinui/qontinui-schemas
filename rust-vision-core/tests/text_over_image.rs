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
    let p5 = text_contrast::nearest_rank(&whole, 5.0).unwrap();
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
    assert!(v.percentile_ratio_worst_case < 4.5, "{v:?}");
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
            assert!(d.contains("too near its ink rgb(255,255,255)"), "{d}");
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
    // The same with the glyph colour MEASURED: the removed glow is told
    // apart by the measured colours as well, so the verdicts do not depend on
    // the producer reporting `fg_color`.
    for (name, measured) in [
        ("glow_strong", false),
        ("glow_mid", false),
        ("glow_strong", true),
        ("glow_mid", true),
    ] {
        let mut banner = if measured {
            case(name).without_fg()
        } else {
            case(name)
        };
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
        assert_eq!(r.outcome, want, "{name} {measured}: {:?}", r.detail);
    }
    let r = case("glow_mid").without_fg().assert_aa();
    assert_eq!(r.outcome, AssertionOutcome::Passed, "{:?}", r.detail);
    let r = case("glow_light").without_fg().assert_aa();
    assert_eq!(r.outcome, AssertionOutcome::Failed, "{:?}", r.detail);
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
    assert!(v.percentile_ratio_worst_case < 4.5, "{v:?}");
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
    assert_eq!(
        control.outcome,
        AssertionOutcome::Passed,
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
    assert_eq!(control, AssertionOutcome::Passed);
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
    assert_eq!(r.outcome, AssertionOutcome::Passed);
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
    assert!(v.percentile_ratio_worst_case < 4.5, "{v:?}");
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
        assert!(v.percentile_ratio_worst_case < 4.5, "{name}: {v:?}");
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
    let d = r.detail.as_deref().unwrap();
    assert!(d.contains("displaced"), "{d}");
    // The refusal names where, so a producer can find the moving thing.
    assert!(
        d.contains("right beside the text, the first at buffer px ("),
        "{d}"
    );
    assert!(d.contains("pause animations"), "{d}");
    let control = c.assert_aa();
    assert_eq!(
        control.outcome,
        AssertionOutcome::Passed,
        "{:?}",
        control.detail
    );
}

/// An anti-aliased EDGE is not sampled: a pixel covered less than half as
/// well as the best pixel within one CSS px of it is the fringe of that
/// stroke, and a low-coverage fringe clears the difference threshold more
/// readily far from the glyph colour, weighting the sample toward high
/// contrast. Here a fully covered black core over mid-grey (about 4:1) has a
/// 30% fringe around it: only the core is sampled, and it fails. A 30% VEIL
/// with no better-covered pixel near it is no edge: it is FAINT ink (a fade,
/// a translucent word), sampled at its own coverage. Over a light region it
/// passes in the best case (opaque black, 18:1) and reads no better than
/// the 1.9:1 of the 30% ink it shows, so the page is Unknown — dropping it
/// would have passed the light half unseen.
#[test]
fn low_coverage_edge_pixels_are_not_sampled() {
    let (w, h) = (200u32, 100u32);
    let bg = |x: u32| if x < 100 { 110u8 } else { 240 };
    let b = image::RgbaImage::from_fn(w, h, |x, _| {
        let v = bg(x);
        image::Rgba([v, v, v, 255])
    });
    let mut a = b.clone();
    // A 30% fringe around the core, then the fully covered black core over
    // the mid-grey half: 30 px of core, 26 of fringe.
    for y in 39..47u32 {
        for x in 19..26u32 {
            let v = (0.7 * bg(x) as f64).round() as u8;
            a.put_pixel(x, y, image::Rgba([v, v, v, 255]));
        }
    }
    for y in 40..46u32 {
        for x in 20..25u32 {
            a.put_pixel(x, y, image::Rgba([0, 0, 0, 255]));
        }
    }
    let label = |w| Element {
        id: "label".into(),
        bbox: Some(Region {
            x: 10,
            y: 30,
            w,
            h: 30,
        }),
        text: Some("label".into()),
        fg_color: Some(Rgb::new(0, 0, 0)),
        ..Default::default()
    };
    let core_only = Case {
        a: Frame::from_rgba(a.clone(), FrameSource::synthetic_now()),
        b: Frame::from_rgba(b.clone(), FrameSource::synthetic_now()),
        snap: ElementSnapshot {
            elements: vec![hero(Region { x: 0, y: 0, w, h }), label(80)],
            ..Default::default()
        },
    };
    let r = core_only.assert_aa();
    assert_eq!(r.outcome, AssertionOutcome::Failed, "{:?}", r.detail);
    let v = core_only.reading();
    assert_eq!(v.sampled_px, 30, "only the fully covered core: {v:?}");

    // A 30%-coverage veil over the light half: 0.7 * 240 = 168, 1200 px.
    for y in 35..55u32 {
        for x in 120..180u32 {
            a.put_pixel(x, y, image::Rgba([168, 168, 168, 255]));
        }
    }
    let veiled = Case {
        a: Frame::from_rgba(a, FrameSource::synthetic_now()),
        b: Frame::from_rgba(b, FrameSource::synthetic_now()),
        snap: ElementSnapshot {
            elements: vec![hero(Region { x: 0, y: 0, w, h }), label(180)],
            ..Default::default()
        },
    };
    let r = veiled.assert_aa();
    assert_eq!(r.outcome, AssertionOutcome::Unknown, "{:?}", r.detail);
    let v = veiled.reading();
    assert_eq!(
        v.sampled_px,
        30 + 1200,
        "core and veil, not the fringe: {v:?}"
    );
    assert!(v.percentile_ratio > 4.5, "{v:?}");
    // The veil is most of the glyph pixels, so the one fill may be any grey
    // from what it shows to black, and the mid-grey half may hide such ink:
    // the worst case is at most what the veil shows.
    let shown = wcag_contrast(Rgb::new(168, 168, 168), Rgb::new(240, 240, 240));
    assert!(v.percentile_ratio_worst_case <= shown + 0.05, "{v:?}");
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

// ---------------------------------------------------------------- final

/// A true hairline face: no ink pixel reaches 50% coverage (here every glyph
/// pixel is 45% white over a dark card). The coverage floor is relative to
/// the mask's best-covered pixel, so it is MEASURED — every glyph pixel is
/// sampled. But 45% white on every pixel is also exactly what a 45%-opaque
/// white paints, and the pixels cannot tell the two apart: the best case
/// (opaque white, 11:1) and the worst (148 over 60, 3.6:1) bracket AA, so AA
/// is Unknown naming the coverage, while a threshold under the worst case
/// passes and one over the best case fails.
#[test]
fn a_hairline_face_whose_ink_peaks_under_half_coverage_is_measured() {
    let (w, h) = (200u32, 100u32);
    let bg = 60u8;
    let b = image::RgbaImage::from_pixel(w, h, image::Rgba([bg, bg, bg, 255]));
    let mut a = b.clone();
    // 45% coverage of white over 60: 60 + 0.45 * 195 = 148.
    let ink = image::Rgba([148, 148, 148, 255]);
    for (x0, y0, sw, sh) in [
        (20u32, 40u32, 1u32, 16u32),
        (30, 40, 1, 16),
        (20, 47, 11, 1),
        (50, 40, 1, 16),
        (60, 40, 1, 16),
        (50, 40, 11, 1),
    ] {
        for y in y0..y0 + sh {
            for x in x0..x0 + sw {
                a.put_pixel(x, y, ink);
            }
        }
    }
    let text = Element {
        id: "label".into(),
        bbox: Some(Region {
            x: 14,
            y: 34,
            w: 56,
            h: 28,
        }),
        text: Some("HH".into()),
        fg_color: Some(Rgb::new(255, 255, 255)),
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
    assert_eq!(r.outcome, AssertionOutcome::Unknown, "{:?}", r.detail);
    let d = r.detail.as_deref().unwrap();
    assert!(d.contains("cover at most 45% of a pixel"), "{d}");
    assert!(!d.contains("could not be measured"), "{d}");
    let v = c.reading();
    assert!(v.sampled_px >= 60, "{v:?}");
    assert!((v.glyph_alpha.unwrap() - 0.45).abs() < 0.01, "{v:?}");
    assert!(v.percentile_ratio > 10.0, "{v:?}");
    let shown = wcag_contrast(Rgb::new(148, 148, 148), Rgb::new(60, 60, 60));
    assert!(
        (v.percentile_ratio_worst_case - shown).abs() < 0.05,
        "{v:?}"
    );
    let low = c.assert_with(Some(&c.b), Some(3.0), None);
    assert_eq!(low.outcome, AssertionOutcome::Passed, "{:?}", low.detail);
    let high = c.assert_with(Some(&c.b), Some(12.0), None);
    assert_eq!(high.outcome, AssertionOutcome::Failed, "{:?}", high.detail);
}

// ---------------------------------------------------------------- review of #257

/// A frame pair built in the test: `bg(x, y)` is frame B; frame A is frame B
/// with each `(x0, y0, w, h)` stroke painted `ink`.
fn painted(
    (w, h): (u32, u32),
    bg: impl Fn(u32, u32) -> u8,
    strokes: &[(u32, u32, u32, u32)],
    ink: u8,
) -> (Frame, Frame) {
    let b = image::RgbaImage::from_fn(w, h, |x, y| {
        let v = bg(x, y);
        image::Rgba([v, v, v, 255])
    });
    let mut a = b.clone();
    for &(x0, y0, sw, sh) in strokes {
        for y in y0..y0 + sh {
            for x in x0..x0 + sw {
                a.put_pixel(x, y, image::Rgba([ink, ink, ink, 255]));
            }
        }
    }
    (
        Frame::from_rgba(a, FrameSource::synthetic_now()),
        Frame::from_rgba(b, FrameSource::synthetic_now()),
    )
}

/// A white-text element with the given id and box.
fn white_text(id: &str, bbox: Region) -> Element {
    Element {
        id: id.into(),
        bbox: Some(bbox),
        text: Some(id.into()),
        fg_color: Some(Rgb::new(255, 255, 255)),
        ..Default::default()
    }
}

/// Vertical stems 3 px wide, `n` of them every 10 px from `x0`.
fn stems(x0: u32, y0: u32, n: u32, h: u32) -> Vec<(u32, u32, u32, u32)> {
    stems_of(3, x0, y0, n, h)
}

/// Vertical stems `width` px wide, `n` of them every 10 px from `x0`.
fn stems_of(width: u32, x0: u32, y0: u32, n: u32, h: u32) -> Vec<(u32, u32, u32, u32)> {
    (0..n).map(|i| (x0 + 10 * i, y0, width, h)).collect()
}

/// The image analyzer's `text_contrast` value for the hero.
fn hero_value(
    c: &Case,
) -> qontinui_vision_core::observation::Observation<Vec<TextOverImageContrast>> {
    let v = c.analyze(Some(&c.b)).image_values.unwrap();
    let hero = v.images.iter().find(|i| i.element == "hero").unwrap();
    hero.text_contrast.clone().unwrap()
}

/// A second text over the hero that may paint but is reported not visible
/// itself (occluded, off-screen, an uninterpreted reason, `visible: false`
/// with no reason) is unmeasured: beside a passing heading the answer is
/// Unknown naming it, never Passed; beside a failing one the failure stands.
/// Text that paints NOTHING (`hidden`) is not over the image at all, and the
/// pass stands.
#[test]
fn text_reported_not_visible_over_the_image_blocks_a_pass() {
    let badge = |reason: Option<VisibilityReason>| Element {
        id: "badge".into(),
        bbox: Some(Region {
            x: 200,
            y: 120,
            w: 100,
            h: 30,
        }),
        text: Some("New".into()),
        fg_color: Some(Rgb::new(250, 250, 250)),
        visible: Some(false),
        visibility_reason: reason,
        ..Default::default()
    };
    for reason in [
        Some(VisibilityReason::Occluded),
        Some(VisibilityReason::OffScreen),
        Some(VisibilityReason::Other("clipped".into())),
        None,
    ] {
        let mut scrim = case("scrim");
        scrim.snap.elements.push(badge(reason.clone()));
        let r = scrim.assert_aa();
        assert_eq!(
            r.outcome,
            AssertionOutcome::Unknown,
            "{reason:?}: {:?}",
            r.detail
        );
        let d = r.detail.as_deref().unwrap();
        assert!(
            d.contains("1 text element(s) over an image could not be measured")
                && d.contains("reported not visible"),
            "{reason:?}: {d}"
        );
        // The analyzer half already counted it unmeasured.
        let v = hero_value(&scrim);
        assert!(
            !v.provenance().coverage.unmeasured.is_empty(),
            "{reason:?}: {v:?}"
        );

        let mut low = case("low_contrast");
        low.snap.elements.push(badge(reason.clone()));
        let r = low.assert_aa();
        assert_eq!(
            r.outcome,
            AssertionOutcome::Failed,
            "{reason:?}: {:?}",
            r.detail
        );
        assert!(
            r.detail
                .as_deref()
                .unwrap()
                .contains("1 other text element(s) could not be measured"),
            "{reason:?}: {:?}",
            r.detail
        );
    }
    let mut scrim = case("scrim");
    scrim
        .snap
        .elements
        .push(badge(Some(VisibilityReason::Hidden)));
    let r = scrim.assert_aa();
    assert_eq!(r.outcome, AssertionOutcome::Passed, "{:?}", r.detail);
}

/// A text box straddling the image's top edge: its glyphs differ over the
/// page above, and over the image they sit on their own colour (white on a
/// blown-out 255 region), where no difference shows. Too few MASK pixels
/// reach the image, but the image under the box could hide every glyph: that
/// is Unknown, not "not over it" — so a passing heading beside it does not
/// make the page pass, and the analyzer does not read the straddler as
/// measured.
#[test]
fn a_box_straddling_the_image_edge_over_its_own_colour_is_unknown() {
    let (w, h) = (200u32, 100u32);
    // The page (y < 50) is dark; the image (y >= 50) is white on the left,
    // dark on the right.
    let bg = |x: u32, y: u32| if y >= 50 && x < 100 { 255 } else { 40 };
    let mut strokes = stems(15, 42, 6, 17); // the straddler, rows 42..59
    strokes.extend(stems(125, 64, 5, 16)); // the heading, over the dark part
    let (a, b) = painted((w, h), bg, &strokes, 255);
    let c = Case {
        a,
        b,
        snap: ElementSnapshot {
            elements: vec![
                hero(Region {
                    x: 0,
                    y: 50,
                    w,
                    h: 50,
                }),
                white_text(
                    "straddler",
                    Region {
                        x: 10,
                        y: 40,
                        w: 80,
                        h: 20,
                    },
                ),
                white_text(
                    "heading",
                    Region {
                        x: 120,
                        y: 60,
                        w: 60,
                        h: 24,
                    },
                ),
            ],
            ..Default::default()
        },
    };
    let r = c.assert_aa();
    assert_eq!(r.outcome, AssertionOutcome::Unknown, "{:?}", r.detail);
    let d = r.detail.as_deref().unwrap();
    assert!(
        d.contains("straddler") && d.contains("whether its glyphs reach the image is unknown"),
        "{d}"
    );

    let v = hero_value(&c);
    let values = v.value().expect("the heading is measured");
    assert_eq!(values.len(), 1);
    assert_eq!(values[0].element, "heading");
    assert!(
        !v.provenance().coverage.unmeasured.is_empty(),
        "the straddler is unmeasured, not ruled out: {v:?}"
    );
}

/// An inset-0 overlay whose text node is its own: a LEAF text element whose
/// box IS the image's. It was skipped as a container; its glyphs lie over the
/// image, and here they are white on a light part of it (about 1.25:1). It is
/// measured, and the page fails rather than passing on the heading beside it.
#[test]
fn a_leaf_overlay_whose_box_is_the_image_is_measured() {
    let (w, h) = (200u32, 100u32);
    let bg = |x: u32, _: u32| if x < 100 { 40 } else { 230 };
    let mut strokes = stems(15, 23, 4, 14); // the heading, over the dark half
    strokes.extend(stems(130, 70, 6, 20)); // the overlay's own glyphs
    let (a, b) = painted((w, h), bg, &strokes, 255);
    let image = Region { x: 0, y: 0, w, h };
    let c = Case {
        a,
        b,
        snap: ElementSnapshot {
            elements: vec![
                hero(image),
                white_text(
                    "heading",
                    Region {
                        x: 10,
                        y: 20,
                        w: 80,
                        h: 20,
                    },
                ),
                white_text("overlay", image),
            ],
            ..Default::default()
        },
    };
    let r = c.assert_aa();
    assert_eq!(r.outcome, AssertionOutcome::Failed, "{:?}", r.detail);
    assert!(
        r.detail.as_deref().unwrap().contains("text 'overlay'"),
        "{:?}",
        r.detail
    );
}

/// A scale factor past `MAX_SCALE_FACTOR` is Unknown naming it, on the
/// assertion and on the analyzer: the shift search grows with its square and
/// the tile arithmetic overflowed past 4096. It must return, not hang.
#[test]
fn a_scale_factor_past_the_cap_is_unknown() {
    let mut held = case("held");
    for f in [&mut held.a, &mut held.b] {
        f.source.scale_factor = 5000.0;
    }
    let r = held.assert_aa();
    assert_eq!(r.outcome, AssertionOutcome::Unknown, "{:?}", r.detail);
    assert_eq!(r.code, Some(UnknownCode::InputMissing));
    assert!(
        r.detail.as_deref().unwrap().contains("scale factor 5000"),
        "{:?}",
        r.detail
    );
    let v = hero_value(&held);
    assert_eq!(v.status(), ObservationStatus::Unknown, "{v:?}");
    assert!(v
        .unknown_info()
        .unwrap()
        .detail
        .contains("scale factor 5000"));
}

/// Partly covered ink on a background just past the mask's threshold: white
/// at 57% coverage over 248 grey reads 252, a difference of 6.9 (under
/// GLYPH_DIFF_MIN), while 248 lies 12.1 from white (past it). Such a pixel
/// was neither masked nor blind, so the light half of the word was never
/// sampled and the heading passed on its dark half. A glyph at the sampling
/// floor's coverage is missed on any background within GLYPH_DIFF_MIN over
/// that coverage, so those are blind too: FAINT here, and the text fails.
#[test]
fn partly_covered_ink_just_past_the_threshold_is_not_ignored() {
    let (w, h) = (200u32, 100u32);
    let bg = |x: u32, _: u32| if x < 100 { 40 } else { 248 };
    let (mut a, b) = painted((w, h), bg, &stems(15, 25, 4, 20), 255);
    for (x0, y0, sw, sh) in stems(115, 25, 8, 20) {
        for y in y0..y0 + sh {
            for x in x0..x0 + sw {
                a.buffer.put_pixel(x, y, image::Rgba([252, 252, 252, 255]));
            }
        }
    }
    let c = Case {
        a,
        b,
        snap: ElementSnapshot {
            elements: vec![
                hero(Region { x: 0, y: 0, w, h }),
                white_text(
                    "heading",
                    Region {
                        x: 10,
                        y: 20,
                        w: 180,
                        h: 30,
                    },
                ),
            ],
            ..Default::default()
        },
    };
    let r = c.assert_aa();
    assert_eq!(r.outcome, AssertionOutcome::Failed, "{:?}", r.detail);
    let v = c.reading();
    assert!(v.faint_px >= 480, "{v:?}");
}

// ---------------------------------------------------------------- review round 2

fn region_at(x: i32, y: i32, w: u32, h: u32) -> Region {
    Region { x, y, w, h }
}

/// A CSS-background hero `<div>` paints its OWN text node over its own
/// background image. That text was never paired (media was not text, and the
/// image was never over itself), so the page read "no text" and the analyzer
/// Absent. It is measured over itself — and with a child heading inside it,
/// with the child's box cut out, so the hero's own word is what decides.
#[test]
fn a_css_background_heros_own_text_is_measured() {
    let (w, h) = (200u32, 100u32);
    let bg = |x: u32, _: u32| if x < 100 { 230 } else { 40 };
    let mut strokes = stems(20, 30, 6, 20); // "Welcome": the hero's own, on 230
    strokes.extend(stems(120, 30, 6, 20)); // "Sale": the child heading, on 40
    let (a, b) = painted((w, h), bg, &strokes, 255);
    let hero_div = |text: &str, children: Vec<String>| Element {
        id: "hero".into(),
        bbox: Some(region_at(0, 0, w, h)),
        text: Some(text.into()),
        fg_color: Some(Rgb::new(255, 255, 255)),
        media: Some(ElementMedia::new(MediaKind::CssBackground)),
        children_ids: children,
        ..Default::default()
    };
    // The hero's own text alone (the right half's glyphs are untracked).
    let alone = Case {
        a: a.clone(),
        b: b.clone(),
        snap: ElementSnapshot {
            elements: vec![hero_div("Welcome", vec![])],
            ..Default::default()
        },
    };
    let r = alone.assert_aa();
    assert_eq!(r.outcome, AssertionOutcome::Failed, "{:?}", r.detail);
    assert!(
        r.detail.as_deref().unwrap().contains("text 'hero'"),
        "{:?}",
        r.detail
    );
    let v = hero_value(&alone);
    assert!(
        v.value()
            .is_some_and(|v| v.iter().any(|x| x.element == "hero")),
        "{v:?}"
    );

    // With a child heading that passes on its own: the hero's own word fails.
    let mut sale = white_text("sale", region_at(110, 25, 80, 30));
    sale.text = Some("Sale".into());
    let mixed = Case {
        a,
        b,
        snap: ElementSnapshot {
            elements: vec![hero_div("Welcome Sale", vec!["sale".into()]), sale],
            ..Default::default()
        },
    };
    let r = mixed.assert_aa();
    assert_eq!(r.outcome, AssertionOutcome::Failed, "{:?}", r.detail);
    assert!(
        r.detail.as_deref().unwrap().contains("text 'hero'"),
        "{:?}",
        r.detail
    );
}

/// `<h1>Hello <span>world</span></h1>`: "Hello" is the wrapper's own text,
/// never sampled when a wrapper was measured only through its children. It
/// is measured, against the wrapper's colour, with the span's box cut out.
/// Here only the wrapper's box reaches the image (the span lies beside it):
/// the page read "no text" and the analyzer Absent; now both see "Hello"
/// fail on the light image.
#[test]
fn a_wrappers_own_text_beside_its_child_is_measured() {
    let (w, h) = (200u32, 100u32);
    let bg = |x: u32, _: u32| if x < 100 { 230 } else { 40 };
    let mut strokes = stems(20, 25, 5, 20); // "Hello", over the image
    strokes.extend(stems(120, 25, 5, 20)); // "world", beside it
    let (a, b) = painted((w, h), bg, &strokes, 255);
    let mut h1 = white_text("h1", region_at(10, 20, 180, 30));
    h1.text = Some("Hello world".into());
    h1.children_ids = vec!["world".into()];
    let mut span = white_text("world", region_at(110, 20, 80, 30));
    span.text = Some("world".into());
    let c = Case {
        a,
        b,
        snap: ElementSnapshot {
            elements: vec![hero(region_at(0, 0, 100, h)), h1, span],
            ..Default::default()
        },
    };
    let r = c.assert_aa();
    assert_eq!(r.outcome, AssertionOutcome::Failed, "{:?}", r.detail);
    assert!(
        r.detail.as_deref().unwrap().contains("text 'h1'"),
        "{:?}",
        r.detail
    );
    let v = hero_value(&c);
    assert_eq!(v.status(), ObservationStatus::Measured, "{v:?}");
    assert_eq!(v.value().unwrap()[0].element, "h1");
}

/// A `<button>` whose own "Buy" label sits beside a black icon inside its
/// box: the label's box CONTAINS the icon, no glyph reaches it, and every
/// icon pixel equals the label's black. When the icon is the button's own
/// DESCENDANT those are not hidden glyphs (text is laid out beside an inline
/// icon), so the button is not text over the icon, and the page — a heading
/// that passes over its hero — passes. The same pixels with the image NOT the
/// text's descendant are a word laid over a patch of its own colour, which
/// may hide its glyphs: Unknown. A box that only straddles an image still
/// counts every blind pixel (the straddling test above).
#[test]
fn a_button_label_beside_its_icon_is_not_unknown() {
    let (w, h) = (200u32, 100u32);
    let bg = |x: u32, y: u32| {
        if x < 100 {
            40 // the hero
        } else if (110..126).contains(&x) && (40..56).contains(&y) {
            0 // the icon
        } else {
            230 // the button
        }
    };
    let (mut a, b) = painted((w, h), bg, &stems(15, 25, 4, 14), 255);
    for (x0, y0, sw, sh) in stems(140, 42, 3, 12) {
        for y in y0..y0 + sh {
            for x in x0..x0 + sw {
                a.buffer.put_pixel(x, y, image::Rgba([0, 0, 0, 255]));
            }
        }
    }
    let mut button = white_text("buy", region_at(104, 34, 90, 28));
    button.fg_color = Some(Rgb::new(0, 0, 0));
    button.children_ids = vec!["icon".into()];
    let icon = Element {
        id: "icon".into(),
        bbox: Some(region_at(110, 40, 16, 16)),
        media: Some(ElementMedia::new(MediaKind::Svg)),
        ..Default::default()
    };
    let mut c = Case {
        a,
        b,
        snap: ElementSnapshot {
            elements: vec![
                hero(region_at(0, 0, 100, h)),
                white_text("heading", region_at(10, 20, 80, 20)),
                button,
                icon,
            ],
            ..Default::default()
        },
    };
    let r = c.assert_aa();
    assert_eq!(r.outcome, AssertionOutcome::Passed, "{:?}", r.detail);

    // The same image, laid under the word rather than inside it.
    c.snap.elements[2].children_ids.clear();
    let r = c.assert_aa();
    assert_eq!(r.outcome, AssertionOutcome::Unknown, "{:?}", r.detail);
    assert!(
        r.detail
            .as_deref()
            .unwrap()
            .contains("whether its glyphs reach the image is unknown"),
        "{:?}",
        r.detail
    );
}

/// A rendered image with no box: whether text lies over it is unknown, as
/// the analyzer already says, so the page cannot pass. A painted text with no
/// box is the same unknown from the other side — whether IT lies over an
/// image — and blocks a pass too (a found failure still stands), and the
/// analyzer's hero value carries it as an unmeasured `text_box` dimension
/// rather than claiming its text complete.
#[test]
fn a_rendered_image_with_no_box_blocks_a_pass() {
    let mut scrim = case("scrim");
    scrim.snap.elements.push(Element {
        id: "thumb".into(),
        media: Some(ElementMedia::new(MediaKind::Img)),
        ..Default::default()
    });
    let r = scrim.assert_aa();
    assert_eq!(r.outcome, AssertionOutcome::Unknown, "{:?}", r.detail);
    assert!(
        r.detail
            .as_deref()
            .unwrap()
            .contains("1 rendered image(s) carry no box"),
        "{:?}",
        r.detail
    );
    let mut low = case("low_contrast");
    low.snap.elements.push(Element {
        id: "thumb".into(),
        media: Some(ElementMedia::new(MediaKind::Img)),
        ..Default::default()
    });
    assert_eq!(low.assert_aa().outcome, AssertionOutcome::Failed);

    let caption = Element {
        id: "caption".into(),
        text: Some("caption".into()),
        ..Default::default()
    };
    let mut scrim = case("scrim");
    scrim.snap.elements.push(caption.clone());
    let r = scrim.assert_aa();
    assert_eq!(r.outcome, AssertionOutcome::Unknown, "{:?}", r.detail);
    assert!(
        r.detail
            .as_deref()
            .unwrap()
            .contains("1 painted text element(s) carry no box"),
        "{:?}",
        r.detail
    );
    let v = hero_value(&scrim);
    assert_eq!(v.status(), ObservationStatus::Measured, "{v:?}");
    let un = &v.provenance().coverage.unmeasured;
    assert!(
        un.iter().any(|d| d.dimension == "text_box" && d.count == 1),
        "{un:?}"
    );
    let mut low = case("low_contrast");
    low.snap.elements.push(caption.clone());
    assert_eq!(low.assert_aa().outcome, AssertionOutcome::Failed);
    // An image with no text over it is not ABSENT of text past a boxless one.
    let mut alone = case("scrim");
    alone.snap.elements.retain(|e| e.id == "hero");
    assert_eq!(hero_value(&alone).status(), ObservationStatus::Absent);
    alone.snap.elements.push(caption);
    assert_eq!(hero_value(&alone).status(), ObservationStatus::Unknown);
}

/// Neighbouring text boxes (here four LEAF texts of their own around the
/// heading) cover the whole ring the LOCAL registration check reads around
/// the heading: no ring tile is left to compare, which confirms nothing.
/// Unknown, as the global check answers the same thing over a whole image —
/// never a vacuous pass. The same four boxes as WRAPPERS of the heading,
/// whose text is all the heading's, paint no glyph of their own, are no
/// holes, and leave the ring to read: the heading is measured and passes.
#[test]
fn a_ring_covered_by_neighbouring_text_is_unknown() {
    let (w, h) = (320u32, 180u32);
    let (a, b) = painted((w, h), |_, _| 40, &stems(104, 73, 5, 14), 255);
    let heading = white_text("heading", region_at(100, 70, 60, 20));
    let around = [
        ("top", region_at(60, 30, 140, 36)),
        ("bottom", region_at(60, 94, 140, 36)),
        ("left", region_at(60, 66, 36, 28)),
        ("right", region_at(164, 66, 36, 28)),
    ];
    let mut elements = vec![hero(region_at(0, 0, w, h))];
    elements.extend(around.iter().map(|(id, bbox)| white_text(id, *bbox)));
    elements.push(heading.clone());
    let c = Case {
        a: a.clone(),
        b: b.clone(),
        snap: ElementSnapshot {
            elements,
            ..Default::default()
        },
    };
    let r = c.assert_aa();
    assert_eq!(r.outcome, AssertionOutcome::Unknown, "{:?}", r.detail);
    assert!(
        r.detail
            .as_deref()
            .unwrap()
            .contains("registration with the frame around it cannot be confirmed"),
        "{:?}",
        r.detail
    );

    let wrapper = |id: &str, child: &str, bbox: Region| Element {
        children_ids: vec![child.into()],
        text: Some("heading".into()),
        ..white_text(id, bbox)
    };
    let nested = ["bottom", "left", "right", "heading"];
    let mut elements = vec![hero(region_at(0, 0, w, h))];
    elements.extend(
        around
            .iter()
            .zip(nested)
            .map(|((id, bbox), child)| wrapper(id, child, *bbox)),
    );
    elements.push(heading);
    let c = Case {
        a,
        b,
        snap: ElementSnapshot {
            elements,
            ..Default::default()
        },
    };
    let r = c.assert_aa();
    assert_eq!(r.outcome, AssertionOutcome::Passed, "{:?}", r.detail);
}

// ---------------------------------------------------------------- review round 3

/// `(x0, y0, w, h)` strokes, all painted one colour.
type Strokes<'a> = (&'a [(u32, u32, u32, u32)], [u8; 3]);

/// Frame A is frame B with each `(x0, y0, w, h)` stroke painted the matching
/// colour: [`painted`] with a colour per stroke group.
fn painted_rgb(
    (w, h): (u32, u32),
    bg: impl Fn(u32, u32) -> u8,
    strokes: &[Strokes<'_>],
) -> (Frame, Frame) {
    let b = image::RgbaImage::from_fn(w, h, |x, y| {
        let v = bg(x, y);
        image::Rgba([v, v, v, 255])
    });
    let mut a = b.clone();
    for (group, [r, g, bl]) in strokes {
        for &(x0, y0, sw, sh) in *group {
            for y in y0..y0 + sh {
                for x in x0..x0 + sw {
                    a.put_pixel(x, y, image::Rgba([*r, *g, *bl, 255]));
                }
            }
        }
    }
    (
        Frame::from_rgba(a, FrameSource::synthetic_now()),
        Frame::from_rgba(b, FrameSource::synthetic_now()),
    )
}

/// `<h1>Hello <span>world wraps</span></h1>` with the span wrapped onto a
/// second line: an inline box spans every line it touches, so the span's box
/// covers the h1's own "Hello". Cut away by box, the h1 lost every glyph and
/// the page read Unknown over a real failure. Attributed by colour, the h1's
/// white "Hello" on the light half is the h1's, and fails; the span's yellow
/// glyphs on the dark half are the span's, and pass on their own.
#[test]
fn a_wrapped_inline_span_does_not_erase_its_parents_glyphs() {
    let (w, h) = (200u32, 100u32);
    let bg = |x: u32, _: u32| if x < 100 { 230 } else { 40 };
    let hello = stems(20, 30, 5, 14);
    let world = stems(120, 30, 5, 14);
    let (a, b) = painted_rgb(
        (w, h),
        bg,
        &[(&hello, [255, 255, 255]), (&world, [230, 200, 40])],
    );
    let mut h1 = white_text("h1", region_at(10, 20, 180, 40));
    h1.text = Some("Hello world wraps".into());
    h1.children_ids = vec!["world".into()];
    let mut span = white_text("world", region_at(10, 20, 180, 40));
    span.text = Some("world wraps".into());
    span.fg_color = Some(Rgb::new(230, 200, 40));
    let c = Case {
        a,
        b,
        snap: ElementSnapshot {
            elements: vec![hero(region_at(0, 0, w, h)), h1, span],
            ..Default::default()
        },
    };
    let r = c.assert_aa();
    assert_eq!(r.outcome, AssertionOutcome::Failed, "{:?}", r.detail);
    assert!(
        r.detail.as_deref().unwrap().contains("text 'h1'"),
        "{:?}",
        r.detail
    );
    let v = hero_value(&c);
    let v = v.value().expect("both measured");
    let h1 = v.iter().find(|x| x.element == "h1").unwrap();
    let world = v.iter().find(|x| x.element == "world").unwrap();
    // Every one of the 5 white stems' 3x14 pixels, and none of the yellow.
    assert_eq!(h1.sampled_px, 5 * 3 * 14, "{h1:?}");
    assert!(h1.percentile_ratio < 1.5, "{h1:?}");
    assert_eq!(world.sampled_px, 5 * 3 * 14, "{world:?}");
    assert!(world.percentile_ratio > 4.5, "{world:?}");
}

/// An overlay card that paints text of its own around a heading, with
/// padding between: the card's box was a hole in the heading's local ring,
/// covering all of it, so the heading was Unknown "cannot be confirmed" on a
/// static page. The ring is read through any box that CONTAINS the heading
/// (the card's own glyphs there read as text, not moved image): measured,
/// and passing.
#[test]
fn a_padded_overlay_card_around_a_heading_is_measured() {
    let (w, h) = (320u32, 180u32);
    let mut strokes = stems(104, 73, 5, 14); // the heading
    strokes.extend(stems(70, 110, 4, 10)); // the card's own text, below it
    let (a, b) = painted((w, h), |_, _| 40, &strokes, 255);
    let heading = white_text("heading", region_at(100, 70, 60, 20));
    let mut card = white_text("card", region_at(60, 30, 200, 120));
    card.text = Some("heading Read more".into());
    card.children_ids = vec!["heading".into()];
    let c = Case {
        a,
        b,
        snap: ElementSnapshot {
            elements: vec![hero(region_at(0, 0, w, h)), card, heading],
            ..Default::default()
        },
    };
    let r = c.assert_aa();
    assert_eq!(r.outcome, AssertionOutcome::Passed, "{:?}", r.detail);
    let v = hero_value(&c);
    assert_eq!(v.status(), ObservationStatus::Measured, "{v:?}");
    assert!(v.provenance().coverage.unmeasured.is_empty(), "{v:?}");
    // With no declared colours the heading's is measured inside the card's
    // box, which contains it: still measured, and passing.
    let c = c.without_fg();
    let r = c.assert_aa();
    assert_eq!(r.outcome, AssertionOutcome::Passed, "{:?}", r.detail);
    let v = hero_value(&c);
    let v = v.value().expect("measured");
    let heading = v.iter().find(|x| x.element == "heading").unwrap();
    assert_eq!(heading.glyph_colour_source, GlyphColourSource::Measured);
    assert_eq!(heading.glyph_colour, Rgb::new(255, 255, 255));
}

/// A declared `fg_color` that is not what paints (gradient text, a
/// `-webkit-text-fill-color`, a blend mode): white declared, red glyphs on a
/// dark card. Red moves every pixel toward white too, so the mask fills, and
/// the ratio against the declared white passed at 10:1 while the red on
/// screen reads 3.3:1. The colour must explain the glyph core: Unknown,
/// naming it. Declared red, the same pixels fail.
#[test]
fn a_declared_colour_that_does_not_paint_the_glyphs_is_unknown() {
    let (w, h) = (200u32, 100u32);
    let glyphs = stems(20, 30, 6, 16);
    let (a, b) = painted_rgb((w, h), |_, _| 40, &[(&glyphs, [230, 40, 40])]);
    let label = white_text("label", region_at(10, 25, 80, 26));
    let mut c = Case {
        a,
        b,
        snap: ElementSnapshot {
            elements: vec![hero(region_at(0, 0, w, h)), label],
            ..Default::default()
        },
    };
    let r = c.assert_aa();
    assert_eq!(r.outcome, AssertionOutcome::Unknown, "{:?}", r.detail);
    assert!(
        r.detail
            .as_deref()
            .unwrap()
            .contains("do not paint its declared colour rgb(255,255,255)"),
        "{:?}",
        r.detail
    );
    let v = hero_value(&c);
    assert_eq!(v.status(), ObservationStatus::Unknown, "{v:?}");

    c.snap.elements[1].fg_color = Some(Rgb::new(230, 40, 40));
    let r = c.assert_aa();
    assert_eq!(r.outcome, AssertionOutcome::Failed, "{:?}", r.detail);
    let v = c.reading();
    assert!(v.colour_fit > 0.99, "{v:?}");
    let shown = wcag_contrast(Rgb::new(230, 40, 40), Rgb::new(40, 40, 40));
    assert!((v.percentile_ratio - shown).abs() < 0.01, "{v:?}");
    // Measured instead of declared, the colour is the red that paints.
    let c = c.without_fg();
    assert_eq!(c.assert_aa().outcome, AssertionOutcome::Failed);
    assert_eq!(c.reading().glyph_colour, Rgb::new(230, 40, 40));
}

/// A visually hidden (`.sr-only`) label over the hero: a 1x1 box whose text
/// is clipped away. No legible glyph fits in it, so it is not text over the
/// image — it was Unknown ("not hidden, or invisible"), turning every hero
/// with an accessible label Unknown. Disclosed, not measured, not blocking.
#[test]
fn a_visually_hidden_label_does_not_block_a_pass() {
    let mut scrim = case("scrim");
    let hero_box = scrim.snap.elements[0].bbox.unwrap();
    scrim.snap.elements.push(white_text(
        "sr-only",
        region_at(hero_box.x + 5, hero_box.y + 5, 1, 1),
    ));
    // 8 CSS px² holds the glyph floor's 6 px but not within the mask's share
    // ceiling: it could never be measured, and is not measured either.
    scrim.snap.elements.push(white_text(
        "tiny",
        region_at(hero_box.x + 20, hero_box.y + 5, 2, 4),
    ));
    let r = scrim.assert_aa();
    assert_eq!(r.outcome, AssertionOutcome::Passed, "{:?}", r.detail);
    assert!(
        r.detail
            .as_deref()
            .unwrap()
            .contains("2 text element(s) over an image have a box under"),
        "{:?}",
        r.detail
    );
    let v = hero_value(&scrim);
    assert_eq!(v.status(), ObservationStatus::Measured, "{v:?}");
    assert!(v.provenance().coverage.unmeasured.is_empty(), "{v:?}");
    assert_eq!(v.value().unwrap().len(), 1);
}

/// The text index scales with the page: built in one pass per element over
/// its ancestors, not by comparing every text with every other. 20000 list
/// items under three nested wrappers (a quadratic build is 4e8 steps here)
/// index in well under the bound, and the wrappers are told apart from the
/// leaves.
#[test]
fn the_text_index_is_not_quadratic_in_the_page() {
    let n = 20_000;
    let mut elements = vec![
        Element {
            id: "main".into(),
            text: Some("x".repeat(n)),
            bbox: Some(region_at(0, 0, 100, 100)),
            children_ids: vec!["ul".into()],
            ..Default::default()
        },
        Element {
            id: "ul".into(),
            text: Some("x".repeat(n)),
            bbox: Some(region_at(0, 0, 100, 100)),
            ..Default::default()
        },
    ];
    elements.extend((0..n).map(|i| Element {
        id: format!("li{i}"),
        text: Some("x".into()),
        bbox: Some(region_at(0, 0, 10, 10)),
        parent_id: Some("ul".into()),
        ..Default::default()
    }));
    let snap = ElementSnapshot {
        elements,
        ..Default::default()
    };
    let t = std::time::Instant::now();
    let index = text_contrast::TextIndex::of(&snap);
    let took = t.elapsed();
    assert!(took.as_secs_f64() < 5.0, "{took:?}");
    assert!(index.is_leaf("li7"));
    assert!(!index.is_leaf("ul") && !index.paints_own_text("ul"));
    assert!(!index.is_leaf("main") && !index.paints_own_text("main"));
    assert_eq!(index.painters().len(), n);
}

// ---------------------------------------------------------------- review round 4

/// One grey level per stem: `level(i)` for stem `i` of `stems(x0, y0, n, h)`.
fn graded(
    (w, h): (u32, u32),
    bg: u8,
    strokes: &[(u32, u32, u32, u32)],
    level: impl Fn(usize) -> u8,
) -> (Frame, Frame) {
    let b = image::RgbaImage::from_pixel(w, h, image::Rgba([bg, bg, bg, 255]));
    let mut a = b.clone();
    for (i, &(x0, y0, sw, sh)) in strokes.iter().enumerate() {
        let v = level(i);
        for y in y0..y0 + sh {
            for x in x0..x0 + sw {
                a.put_pixel(x, y, image::Rgba([v, v, v, 255]));
            }
        }
    }
    (
        Frame::from_rgba(a, FrameSource::synthetic_now()),
        Frame::from_rgba(b, FrameSource::synthetic_now()),
    )
}

/// White at `alpha` over grey `bg`, as a browser blends it.
fn white_at(bg: u8, alpha: f64) -> u8 {
    (bg as f64 + alpha * (255.0 - bg as f64)).round() as u8
}

/// A heading whose ink is opaque on half its glyphs and FAINT on the other
/// half (a gradient fading out, a `mask-image`, a translucent word beside an
/// opaque one). The faint glyphs were dropped as low-coverage "edges" of the
/// opaque ones, and the heading passed at 14.7:1 while half of it showed
/// 2.7:1. Each glyph is sampled at its OWN local peak coverage: the best case
/// passes, the worst case is what the faint half shows, and the reading is
/// Unknown. So is a continuous fade, while the same stems all opaque pass.
#[test]
fn faint_glyphs_beside_opaque_ones_are_sampled() {
    let (w, h) = (300u32, 100u32);
    let glyphs = stems(20, 40, 16, 14);
    let label = white_text("label", region_at(10, 30, 180, 34));
    let case_with = |level: &dyn Fn(usize) -> u8| {
        let (a, b) = graded((w, h), 40, &glyphs, level);
        Case {
            a,
            b,
            snap: ElementSnapshot {
                elements: vec![hero(region_at(0, 0, w, h)), label.clone()],
                ..Default::default()
            },
        }
    };
    let halves = case_with(&|i| if i < 8 { 255 } else { white_at(40, 0.3) });
    let r = halves.assert_aa();
    assert_eq!(r.outcome, AssertionOutcome::Unknown, "{:?}", r.detail);
    let v = halves.reading();
    assert_eq!(v.sampled_px, 16 * 3 * 14, "every stem: {v:?}");
    assert!(v.percentile_ratio > 10.0, "{v:?}");
    let shown = wcag_contrast(
        Rgb::new(white_at(40, 0.3), white_at(40, 0.3), white_at(40, 0.3)),
        Rgb::new(40, 40, 40),
    );
    assert!(
        (v.percentile_ratio_worst_case - shown).abs() < 0.05,
        "{v:?}"
    );

    let fade = case_with(&|i| white_at(40, 1.0 - 0.05 * i as f64));
    let r = fade.assert_aa();
    assert_eq!(r.outcome, AssertionOutcome::Unknown, "{:?}", r.detail);

    let opaque = case_with(&|_| 255);
    let r = opaque.assert_aa();
    assert_eq!(r.outcome, AssertionOutcome::Passed, "{:?}", r.detail);
    let v = opaque.reading();
    assert_eq!(v.percentile_ratio, v.percentile_ratio_worst_case, "{v:?}");
    assert_eq!(v.glyph_alpha, Some(1.0), "{v:?}");
}

/// `<p class="text-white/40">Muted <strong class="text-white">bold</strong></p>`:
/// both report rgb(255,255,255) (a colour carries no alpha). The strong's
/// opaque ink inside the p's box is the same ink and counts for the p — and
/// it set the p's coverage and outnumbered its own word, so the p passed on
/// the strong's pixels while "Muted" showed 2.3:1. Coverage is local, and the
/// p's unshared pixels are read on their own as well: Unknown.
#[test]
fn a_translucent_parent_is_not_read_through_its_opaque_child() {
    let (w, h) = (300u32, 100u32);
    let muted = stems(20, 40, 3, 14);
    let bold = stems(120, 40, 12, 14);
    let mut all = muted.clone();
    all.extend(bold.iter().copied());
    let (a, b) = graded((w, h), 100, &all, |i| {
        if i < muted.len() {
            white_at(100, 0.4)
        } else {
            255
        }
    });
    let mut p = white_text("p", region_at(10, 30, 260, 34));
    p.text = Some("Muted bold".into());
    p.children_ids = vec!["strong".into()];
    let mut strong = white_text("strong", region_at(115, 30, 125, 34));
    strong.text = Some("bold".into());
    let c = Case {
        a,
        b,
        snap: ElementSnapshot {
            elements: vec![hero(region_at(0, 0, w, h)), p, strong],
            ..Default::default()
        },
    };
    let r = c.assert_aa();
    assert_eq!(r.outcome, AssertionOutcome::Unknown, "{:?}", r.detail);
    assert!(
        r.detail.as_deref().unwrap().contains("text 'p'"),
        "{:?}",
        r.detail
    );
    let v = hero_value(&c);
    let v = v.value().expect("measured");
    let p = v.iter().find(|x| x.element == "p").unwrap();
    let shown = wcag_contrast(
        Rgb::new(white_at(100, 0.4), white_at(100, 0.4), white_at(100, 0.4)),
        Rgb::new(100, 100, 100),
    );
    assert!(
        (p.percentile_ratio_worst_case - shown).abs() < 0.05,
        "{p:?}"
    );
}

/// Hidden ink where the rendered colour is not the one the glyph colour
/// names. MEASURED (no `fg_color`): strokes that never cover a whole pixel
/// (80% white over 40 reads 212) measure as rgb(212,212,212), so a word over
/// a clipped 255 highlight — invisible — was neither faint nor hidden, and
/// the page passed. The measured colour is only what the best-covered pixels
/// show; the ink may lie anywhere on the ray past it, up to 255, and the
/// highlight is undetectable: Unknown. DECLARED white rendered 250 (a
/// `filter`, a text-fill colour within tolerance), half the word on a 250
/// patch: the best-covered pixels' own colour is ink too, and it is Unknown.
#[test]
fn hidden_ink_is_judged_against_the_colours_the_glyph_may_render() {
    let (w, h) = (300u32, 100u32);
    let left = stems(20, 40, 6, 14);
    let right = stems(170, 40, 6, 14);
    let mut strokes = left.clone();
    strokes.extend(right.iter().copied());
    let paint = |bg_right: u8, ink: &dyn Fn(u8) -> u8| {
        let bg = |x: u32| if x < 150 { 40u8 } else { bg_right };
        let b = image::RgbaImage::from_fn(w, h, |x, _| {
            let v = bg(x);
            image::Rgba([v, v, v, 255])
        });
        let mut a = b.clone();
        for &(x0, y0, sw, sh) in &strokes {
            for y in y0..y0 + sh {
                for x in x0..x0 + sw {
                    let v = ink(bg(x));
                    a.put_pixel(x, y, image::Rgba([v, v, v, 255]));
                }
            }
        }
        (
            Frame::from_rgba(a, FrameSource::synthetic_now()),
            Frame::from_rgba(b, FrameSource::synthetic_now()),
        )
    };
    let label = |fg: Option<Rgb>| Element {
        fg_color: fg,
        ..white_text("label", region_at(10, 30, 260, 34))
    };
    let (a, b) = paint(255, &|bg| white_at(bg, 0.8));
    let measured = Case {
        a,
        b,
        snap: ElementSnapshot {
            elements: vec![hero(region_at(0, 0, w, h)), label(None)],
            ..Default::default()
        },
    };
    let r = measured.assert_aa();
    assert_eq!(r.outcome, AssertionOutcome::Unknown, "{:?}", r.detail);
    let v = measured.reading();
    assert_eq!(v.glyph_colour, Rgb::new(212, 212, 212), "{v:?}");
    assert!(v.hidden_px_estimate > 0, "{v:?}");

    let (a, b) = paint(250, &|_| 250);
    let declared = Case {
        a,
        b,
        snap: ElementSnapshot {
            elements: vec![
                hero(region_at(0, 0, w, h)),
                label(Some(Rgb::new(255, 255, 255))),
            ],
            ..Default::default()
        },
    };
    let r = declared.assert_aa();
    assert_eq!(r.outcome, AssertionOutcome::Unknown, "{:?}", r.detail);
    assert!(declared.reading().hidden_px_estimate > 0);
}

/// Frames A and B upscaled by `k` (nearest neighbour) and stamped with that
/// scale factor: the same page at a higher device pixel ratio.
fn at_scale(c: &Case, k: u32) -> Case {
    let up = |f: &Frame| {
        let img = image::imageops::resize(
            &f.buffer,
            f.width * k,
            f.height * k,
            image::imageops::FilterType::Nearest,
        );
        let mut g = Frame::from_rgba(img, FrameSource::synthetic_now());
        g.source.scale_factor = k as f64;
        g
    };
    Case {
        a: up(&c.a),
        b: up(&c.b),
        snap: c.snap.clone(),
    }
}

/// The measurement at device pixel ratios 2 and 3: every threshold is in CSS
/// px and scales with the ratio, so the plan's mutation pair keeps its
/// verdicts and its readings.
#[test]
fn the_mutation_pair_holds_at_higher_scale_factors() {
    for k in [2u32, 3] {
        let (low, scrim) = (
            at_scale(&case("low_contrast"), k),
            at_scale(&case("scrim"), k),
        );
        let (rl, rs) = (low.assert_aa(), scrim.assert_aa());
        assert_eq!(
            rl.outcome,
            AssertionOutcome::Failed,
            "x{k}: {:?}",
            rl.detail
        );
        assert_eq!(
            rs.outcome,
            AssertionOutcome::Passed,
            "x{k}: {:?}",
            rs.detail
        );
        let (one, many) = (case("scrim").reading(), scrim.reading());
        assert!(
            (one.percentile_ratio - many.percentile_ratio).abs() < 0.5,
            "x{k}: {one:?} vs {many:?}"
        );
    }
}

/// The local registration check's shift search is bounded per tile: a
/// CSS-background hero whose own text's ring is the whole textured image, at
/// scale factor 4, is measured in bounded time (it took 13.8 s in a debug
/// build when every differing pixel was read at each of its 9408 non-zero offsets).
#[test]
fn the_local_shift_search_is_bounded_at_scale_factor_4() {
    let (w, h, k) = (320u32, 180u32, 4u32);
    let texture = |x: u32, y: u32| (((x / k) * 37 + (y / k) * 91) % 160 + 40) as u8;
    let b = image::RgbaImage::from_fn(w * k, h * k, |x, y| {
        let v = texture(x, y);
        image::Rgba([v, v, v, 255])
    });
    let mut a = b.clone();
    for (x0, y0, sw, sh) in stems(10, 30, 12, 100) {
        for y in y0 * k..(y0 + sh) * k {
            for x in x0 * k..(x0 + sw) * k {
                a.put_pixel(x, y, image::Rgba([255, 255, 255, 255]));
            }
        }
    }
    let mut frames = [a, b].map(|img| Frame::from_rgba(img, FrameSource::synthetic_now()));
    for f in &mut frames {
        f.source.scale_factor = k as f64;
    }
    let [a, b] = frames;
    let hero_div = Element {
        id: "hero".into(),
        bbox: Some(region_at(0, 0, w, h)),
        text: Some("Welcome".into()),
        fg_color: Some(Rgb::new(255, 255, 255)),
        media: Some(ElementMedia::new(MediaKind::CssBackground)),
        ..Default::default()
    };
    let c = Case {
        a,
        b,
        snap: ElementSnapshot {
            elements: vec![hero_div],
            ..Default::default()
        },
    };
    let t = std::time::Instant::now();
    let r = c.assert_aa();
    let took = t.elapsed();
    assert_ne!(r.outcome, AssertionOutcome::Passed, "{:?}", r.detail);
    assert!(took.as_secs_f64() < 12.0, "{took:?}: {:?}", r.detail);
}

/// A glyph too faint to move its pixels past `GLYPH_DIFF_MIN` is read on a
/// noise-free capture by the difference it does make, at its own coverage,
/// where it was dropped and the heading passed on its opaque half. A word at
/// 2% opacity over grey 100, which opaque white would pass, is Unknown; one
/// at 10% over grey 220, which even opaque white fails, is a failure. The
/// faint anti-aliasing fringe of a visible stroke is not such a glyph: the
/// opaque half with only that fringe passes.
#[test]
fn a_glyph_too_faint_for_the_mask_is_read_on_a_noise_free_capture() {
    let (w, h) = (300u32, 100u32);
    let frames = |right: u8, ink: &dyn Fn(u32, u32, u8) -> Option<u8>| {
        let bg = |x: u32| if x < 150 { 40u8 } else { right };
        let b = image::RgbaImage::from_fn(w, h, |x, _| {
            let v = bg(x);
            image::Rgba([v, v, v, 255])
        });
        let mut a = b.clone();
        for y in 0..h {
            for x in 0..w {
                if let Some(v) = ink(x, y, bg(x)) {
                    a.put_pixel(x, y, image::Rgba([v, v, v, 255]));
                }
            }
        }
        (
            Frame::from_rgba(a, FrameSource::synthetic_now()),
            Frame::from_rgba(b, FrameSource::synthetic_now()),
        )
    };
    let in_stems = |strokes: &[(u32, u32, u32, u32)], x: u32, y: u32| {
        strokes
            .iter()
            .any(|&(x0, y0, sw, sh)| x >= x0 && x < x0 + sw && y >= y0 && y < y0 + sh)
    };
    let (opaque, faint) = (stems(20, 40, 8, 14), stems(170, 40, 8, 14));
    let label = white_text("label", region_at(10, 30, 260, 34));
    let run = |(a, b): (Frame, Frame)| {
        let c = Case {
            a,
            b,
            snap: ElementSnapshot {
                elements: vec![hero(region_at(0, 0, w, h)), label.clone()],
                ..Default::default()
            },
        };
        (c.assert_aa(), c.reading())
    };
    let (opaque, faint) = (&opaque, &faint);
    let two_words = |alpha: f64| {
        move |x: u32, y: u32, bg: u8| {
            if in_stems(opaque, x, y) {
                Some(255)
            } else if in_stems(faint, x, y) {
                Some(white_at(bg, alpha))
            } else {
                None
            }
        }
    };

    let (r, v) = run(frames(100, &two_words(0.02)));
    assert_eq!(r.outcome, AssertionOutcome::Unknown, "{:?}", r.detail);
    assert_eq!(v.faint_px, 8 * 3 * 14, "{v:?}");
    assert!(v.percentile_ratio_worst_case < 1.1, "{v:?}");
    assert!(v.percentile_ratio > 4.5, "{v:?}");

    let (r, v) = run(frames(220, &two_words(0.1)));
    assert_eq!(r.outcome, AssertionOutcome::Failed, "{:?}", r.detail);
    assert_eq!(v.faint_px, 8 * 3 * 14, "{v:?}");

    // The fringe: one column each side of every opaque stem, at 2%.
    let (r, v) = run(frames(100, &|x, y, bg| {
        if in_stems(opaque, x, y) {
            Some(255)
        } else if in_stems(opaque, x + 1, y) || x > 0 && in_stems(opaque, x - 1, y) {
            Some(white_at(bg, 0.02))
        } else {
            None
        }
    }));
    assert_eq!(r.outcome, AssertionOutcome::Passed, "{:?}", r.detail);
    assert_eq!(v.faint_px, 0, "{v:?}");
}

/// Gradient text whose best-covered quarter is the declared colour passes
/// the colour fit; its other end is another colour. White to magenta over
/// black: the magenta glyphs project onto white at 2/3 coverage, a grey that
/// reads 9.0:1, while magenta itself shows 6.7:1. Against 7:1 the worst case
/// reads the off-blend pixels at what they render: Unknown, not a pass.
#[test]
fn a_gradients_off_colour_end_is_read_at_what_it_renders() {
    let (w, h) = (300u32, 100u32);
    let (white, magenta) = (stems(20, 40, 8, 14), stems(120, 40, 8, 14));
    let b = image::RgbaImage::from_pixel(w, h, image::Rgba([0, 0, 0, 255]));
    let mut a = b.clone();
    for (strokes, ink) in [(&white, [255u8, 255, 255]), (&magenta, [255, 0, 255])] {
        for &(x0, y0, sw, sh) in strokes {
            for y in y0..y0 + sh {
                for x in x0..x0 + sw {
                    a.put_pixel(x, y, image::Rgba([ink[0], ink[1], ink[2], 255]));
                }
            }
        }
    }
    let c = Case {
        a: Frame::from_rgba(a, FrameSource::synthetic_now()),
        b: Frame::from_rgba(b, FrameSource::synthetic_now()),
        snap: ElementSnapshot {
            elements: vec![
                hero(region_at(0, 0, w, h)),
                white_text("label", region_at(10, 30, 210, 34)),
            ],
            ..Default::default()
        },
    };
    let r = c.assert_with(Some(&c.b), Some(7.0), None);
    assert_eq!(r.outcome, AssertionOutcome::Unknown, "{:?}", r.detail);
    assert!(
        r.detail.as_deref().unwrap().contains("paint off the blend"),
        "{:?}",
        r.detail
    );
    let v = c.reading();
    assert!(v.off_colour_px > 0, "{v:?}");
    // The magenta end projects onto white at 2/3 coverage, but what a pixel
    // off the blend renders says nothing of how much white covers it: the
    // faintest glyphs reported are the white end's, opaque.
    assert_eq!(v.glyph_alpha, Some(1.0), "{v:?}");
    let shown = wcag_contrast(Rgb::new(255, 0, 255), Rgb::new(0, 0, 0));
    assert!(
        (v.percentile_ratio_worst_case - shown).abs() < 0.05,
        "{shown}: {v:?}"
    );
    // At 4.5:1 both ends pass.
    let r = c.assert_aa();
    assert_eq!(r.outcome, AssertionOutcome::Passed, "{:?}", r.detail);
}

// ------------------------------------------------- review round 2 findings

/// A frame pair over an RGB background: frame B is `bg`, frame A paints
/// `ink(x, y, bg)` wherever it answers `Some`.
fn rgb_frames(
    (w, h): (u32, u32),
    bg: impl Fn(u32, u32) -> [u8; 3],
    ink: impl Fn(u32, u32, [u8; 3]) -> Option<[u8; 3]>,
) -> (Frame, Frame) {
    let b = image::RgbaImage::from_fn(w, h, |x, y| {
        let [r, g, bl] = bg(x, y);
        image::Rgba([r, g, bl, 255])
    });
    let mut a = b.clone();
    for y in 0..h {
        for x in 0..w {
            if let Some([r, g, bl]) = ink(x, y, bg(x, y)) {
                a.put_pixel(x, y, image::Rgba([r, g, bl, 255]));
            }
        }
    }
    (
        Frame::from_rgba(a, FrameSource::synthetic_now()),
        Frame::from_rgba(b, FrameSource::synthetic_now()),
    )
}

fn in_any(strokes: &[(u32, u32, u32, u32)], x: u32, y: u32) -> bool {
    strokes
        .iter()
        .any(|&(x0, y0, sw, sh)| x >= x0 && x < x0 + sw && y >= y0 && y < y0 + sh)
}

/// One text over one hero, the frames given.
fn one_text((a, b): (Frame, Frame), (w, h): (u32, u32), text: Element) -> Case {
    Case {
        a,
        b,
        snap: ElementSnapshot {
            elements: vec![hero(region_at(0, 0, w, h)), text],
            ..Default::default()
        },
    }
}

/// A word too faint for the mask beside a translucent dot it can see: the
/// dot's 20% coverage set the blind radius, the word fell inside it, and the
/// word's faint pixels were read at their BEST case, opaque white, so a word
/// at 2% over grey 105 passed at 5.5:1 while it shows 1.05:1. Each faint
/// glyph pixel is read at no better than it renders: Unknown, over grey 105
/// with a 20% dot and over grey 50 with a 14% one, and with one visible stem
/// beside many faint ones.
#[test]
fn a_faint_word_beside_a_translucent_dot_is_read_at_what_it_renders() {
    let (w, h) = (300u32, 100u32);
    let dot = [(20u32, 40u32, 3u32, 3u32)];
    let word = stems(40, 40, 16, 14);
    for (bg, dot_alpha) in [(105u8, 0.2), (110, 0.2), (50, 0.14), (60, 0.14)] {
        for staggered in [false, true] {
            let (visible, faint): (Vec<_>, Vec<_>) = if staggered {
                (stems(20, 40, 1, 14), stems(40, 40, 16, 14))
            } else {
                (dot.to_vec(), word.clone())
            };
            let frames = rgb_frames(
                (w, h),
                |_, _| [bg; 3],
                |x, y, _| {
                    let v = if in_any(&visible, x, y) {
                        white_at(bg, dot_alpha)
                    } else if in_any(&faint, x, y) {
                        white_at(bg, 0.02)
                    } else {
                        return None;
                    };
                    Some([v; 3])
                },
            );
            let c = one_text(
                frames,
                (w, h),
                white_text("label", region_at(10, 30, 210, 34)),
            );
            let r = c.assert_aa();
            assert_eq!(
                r.outcome,
                AssertionOutcome::Unknown,
                "bg {bg} staggered {staggered}: {:?}",
                r.detail
            );
            let v = c.reading();
            assert!(v.faint_px >= 16 * 3 * 14, "{v:?}");
            assert!(v.percentile_ratio_worst_case < 1.1, "{v:?}");
        }
    }
}

/// White declared, ink a few units off it (a `filter`, a text-fill colour
/// within tolerance): the left word over grey 40 shows it, the right word
/// lies on a near-white patch it barely differs from. Where that patch is
/// tinted (rgb(250,238,250), rgb(243,255,255)) the right word's pixels
/// moved toward the ink but AWAY from white, so the mask left them out, no
/// rule counted them, and the page passed at 14.7:1 while the right word
/// shows 1.02:1. They are read as off-colour glyph pixels: a failure. So is
/// ink 250 on a 253-grey-ish patch, read as faint. And ink 250 over a 250
/// patch with a 1 px underline in the declared white under the left word:
/// the best-covered pixels were the underline's, which narrowed the ink to
/// white and left the invisible right word uncounted. The stroke interiors'
/// own colour is ink as well, and the right word is Unknown.
///
/// Review round 3: stems 1 or 2 px wide have no interior, and the ink stayed
/// white — every one of these passed again. One fill paints the glyphs, and
/// an ink renders nothing beyond itself: stems that never reach white bound
/// the fill from their own colour up, and an underline (1 or 3 px tall)
/// bounds only its own colour. Each is a failure or Unknown, as at 3 px.
#[test]
fn ink_off_its_declared_colour_is_read_where_it_renders() {
    let (w, h) = (300u32, 100u32);
    let label = || white_text("label", region_at(10, 30, 260, 34));
    for width in [1u32, 2, 3] {
        let (left, right) = (
            stems_of(width, 20, 40, 6, 14),
            stems_of(width, 170, 40, 6, 14),
        );
        for (ink, patch) in [
            (244u8, [250u8, 238, 250]),
            (244, [243, 255, 255]),
            (250, [253, 247, 253]),
        ] {
            let frames = rgb_frames(
                (w, h),
                |x, _| if x < 150 { [40; 3] } else { patch },
                |x, y, _| (in_any(&left, x, y) || in_any(&right, x, y)).then_some([ink; 3]),
            );
            let c = one_text(frames, (w, h), label());
            let r = c.assert_aa();
            assert_eq!(
                r.outcome,
                AssertionOutcome::Failed,
                "{width} px, ink {ink} over {patch:?}: {:?}",
                r.detail
            );
            let v = c.reading();
            assert_eq!(v.sampled_px, (2 * 6 * width * 14) as u64, "{v:?}");
        }

        for thick in [1u32, 3] {
            let underline = [(20u32, 56u32, 53u32, thick)];
            let frames = rgb_frames(
                (w, h),
                |x, _| if x < 150 { [40; 3] } else { [250; 3] },
                |x, y, _| {
                    if in_any(&underline, x, y) {
                        Some([255; 3])
                    } else {
                        (in_any(&left, x, y) || in_any(&right, x, y)).then_some([250; 3])
                    }
                },
            );
            let c = one_text(frames, (w, h), label());
            let r = c.assert_aa();
            assert_eq!(
                r.outcome,
                AssertionOutcome::Unknown,
                "{width} px stems, {thick} px underline: {:?}",
                r.detail
            );
            assert!(c.reading().hidden_px_estimate > 0, "{width} px, {thick} px");
        }
    }
}

/// `<p class="text-white">… <span class="text-white/40">wraps</span></p>`
/// with the span wrapped: an inline box spans every line it touches, so each
/// text's box holds the other's glyphs, and neither has a pixel of its own.
/// One faint stem per line is under 5% of the pixels, so both read the p's
/// opaque glyphs and passed at 5.9:1 while the span shows 2.3:1. The p wraps
/// the span, and the span's colour explains every pixel of the p: the p's
/// own glyphs were not seen, and it is Unknown. The span is measured: every
/// pixel shared, a colour that carries no opacity cannot say which glyphs
/// are whose, the reading is taken per opacity band as well, and its bracket
/// spans the threshold.
#[test]
fn a_wrapped_translucent_span_inside_its_parent_is_unknown() {
    let (w, h) = (300u32, 120u32);
    let lines = [30u32, 70];
    let opaque: Vec<_> = lines
        .iter()
        .flat_map(|&y| stems(15, y, 26, 14))
        .enumerate()
        .filter(|(i, _)| i % 26 != 13)
        .map(|(_, s)| s)
        .collect();
    let faint: Vec<_> = lines.iter().map(|&y| (145u32, y, 3u32, 14u32)).collect();
    let frames = rgb_frames(
        (w, h),
        |_, _| [100; 3],
        |x, y, _| {
            if in_any(&opaque, x, y) {
                Some([255; 3])
            } else if in_any(&faint, x, y) {
                Some([white_at(100, 0.4); 3])
            } else {
                None
            }
        },
    );
    let mut p = white_text("p", region_at(10, 20, 270, 80));
    p.text = Some("Lead text that wraps".into());
    p.children_ids = vec!["span".into()];
    let mut span = white_text("span", region_at(10, 20, 270, 80));
    span.text = Some("wraps".into());
    let (a, b) = frames;
    let c = Case {
        a,
        b,
        snap: ElementSnapshot {
            elements: vec![hero(region_at(0, 0, w, h)), p, span],
            ..Default::default()
        },
    };
    let r = c.assert_aa();
    assert_eq!(r.outcome, AssertionOutcome::Unknown, "{:?}", r.detail);
    let d = r.detail.as_deref().unwrap();
    assert!(
        d.contains("text 'p'") && d.contains("none of its inner texts"),
        "{d}"
    );
    let v = hero_value(&c);
    let v = v.value().expect("measured");
    assert_eq!(
        v.iter().map(|t| t.element.as_str()).collect::<Vec<_>>(),
        ["span"],
        "{v:?}"
    );
    let shown = wcag_contrast(
        Rgb::new(white_at(100, 0.4), white_at(100, 0.4), white_at(100, 0.4)),
        Rgb::new(100, 100, 100),
    );
    for t in v {
        assert_eq!(t.shared_px, t.sampled_px, "{t:?}");
        assert!(t.percentile_ratio > 4.5, "{t:?}");
        assert!(
            (t.percentile_ratio_worst_case - shown).abs() < 0.05,
            "{t:?}"
        );
    }
}

/// White text on a tight dark highlight, with a white glow `text-shadow`
/// spreading onto the grey-200 photo around it. The glow's ramp moves the
/// photo toward white like faint glyphs, its inner ring was sampled at the
/// stroke's coverage, and the heading FAILED at 1.7:1 while its glyphs show
/// 18:1. A pixel covered under half as well as a stroke within `HALO_CSS`
/// is a glow and a faint glyph alike: it counts in the worst case only, and
/// the reading is Unknown, never a failure. Without the glow it passes.
///
/// Review round 3: a stronger ramp (60/50/35/20%) has pixels covered half
/// as well as the stroke, which read as glyphs and failed it again; and over
/// a near-white photo (250) the glow moves pixels a unit or two, FAINT, which
/// were read as glyphs too. Beside a stroke both are optional: Unknown.
#[test]
fn a_glow_of_the_glyph_colour_is_not_read_as_glyphs() {
    let (w, h) = (300u32, 100u32);
    let glyphs = stems(40, 40, 12, 14);
    let highlight: Vec<_> = glyphs
        .iter()
        .map(|&(x, y, sw, sh)| (x - 1, y - 1, sw + 2, sh + 2))
        .collect();
    // Chebyshev distance from the nearest glyph pixel, up to 5.
    let reach = |x: u32, y: u32| {
        (0..=5u32).find(|&d| {
            glyphs.iter().any(|&(x0, y0, sw, sh)| {
                x + d >= x0 && x < x0 + sw + d && y + d >= y0 && y < y0 + sh + d
            })
        })
    };
    let label = || white_text("label", region_at(20, 25, 150, 46));
    let ramps: [Option<[f64; 4]>; 3] = [
        None,
        Some([0.4, 0.3, 0.2, 0.1]),
        Some([0.6, 0.5, 0.35, 0.2]),
    ];
    for photo in [200u8, 250] {
        let bg = |x: u32, y: u32| {
            if in_any(&highlight, x, y) {
                [30u8; 3]
            } else {
                [photo; 3]
            }
        };
        for ramp in ramps {
            let frames = rgb_frames((w, h), bg, |x, y, under| match (reach(x, y), ramp) {
                (Some(0), _) => Some([255; 3]),
                (Some(d), Some(ramp)) if d <= 4 => {
                    Some([white_at(under[0], ramp[d as usize - 1]); 3])
                }
                _ => None,
            });
            let c = one_text(frames, (w, h), label());
            let r = c.assert_aa();
            let want = if ramp.is_some() {
                AssertionOutcome::Unknown
            } else {
                AssertionOutcome::Passed
            };
            assert_eq!(
                r.outcome, want,
                "photo {photo} ramp {ramp:?}: {:?}",
                r.detail
            );
            let v = c.reading();
            assert!(v.percentile_ratio > 10.0, "{v:?}");
            if ramp.is_some() {
                assert!(v.halo_px > 0, "{v:?}");
                assert_eq!(v.faint_px, 0, "{v:?}");
            }
        }
    }
}

/// A MEASURED colour (no `fg_color`) from strokes with no interior — every
/// pixel 80% white over grey 118, 2 px wide — is what a hairline of opaque
/// white shows as well, so its best case reads white too: 4.5:1 at best,
/// 3.6:1 at worst, Unknown, as with white declared. The same strokes 3 px
/// wide have an interior, a hairline cannot paint one, and both readings
/// left (opaque rgb(228,228,228), white at 80%) render 3.6:1: a failure.
#[test]
fn a_measured_colour_with_no_stroke_interior_may_be_a_hairline() {
    let (w, h) = (300u32, 100u32);
    let ink = white_at(118, 0.8);
    for width in [2u32, 3] {
        let strokes: Vec<_> = (0..16)
            .map(|i| (20 + 10 * i, 40u32, width, 14u32))
            .collect();
        let frames = rgb_frames(
            (w, h),
            |_, _| [118; 3],
            |x, y, _| in_any(&strokes, x, y).then_some([ink; 3]),
        );
        let c = one_text(
            frames,
            (w, h),
            white_text("label", region_at(10, 30, 180, 34)),
        )
        .without_fg();
        let r = c.assert_aa();
        let want = if width == 2 {
            AssertionOutcome::Unknown
        } else {
            AssertionOutcome::Failed
        };
        assert_eq!(r.outcome, want, "width {width}: {:?}", r.detail);
        let v = c.reading();
        assert_eq!(v.glyph_colour, Rgb::new(ink, ink, ink), "{v:?}");
        assert!(v.percentile_ratio_worst_case < 3.7, "{v:?}");
        // Review round 3: the detail named only the measured colour while the
        // best case read the hairline's ink. It names both.
        let d = r.detail.as_deref().unwrap();
        if width == 2 {
            assert_eq!(v.hairline_ink, Some(Rgb::new(255, 255, 255)), "{v:?}");
            assert!(d.contains("hairline ink rgb(255,255,255)"), "{d}");
            assert!(d.contains("hairline of rgb(255,255,255)"), "{d}");
        } else {
            assert_eq!(v.hairline_ink, None, "{v:?}");
            assert!(!d.contains("hairline"), "{d}");
        }
    }
}

/// A faint glyph on a BLIND background — one near the ink, where the mask
/// cannot see a glyph — was read at its best case. A MEASURED grey 100 with
/// no stroke interior may be a hairline of anything on the ray up to white,
/// so a faint word over grey 95 just outside that ink has a best case of
/// 6.4:1, which is what it was read at while it shows 1.03:1. It is read at
/// no better than it renders: Unknown, not a pass.
#[test]
fn a_faint_glyph_on_a_blind_background_is_read_at_what_it_renders() {
    let (w, h) = (300u32, 100u32);
    let seen = [(20u32, 40u32, 2u32, 14u32)];
    let faint = stems(60, 40, 16, 14);
    let frames = rgb_frames(
        (w, h),
        |x, _| if x < 40 { [0; 3] } else { [95; 3] },
        |x, y, _| {
            if in_any(&seen, x, y) {
                Some([100; 3])
            } else {
                in_any(&faint, x, y).then_some([97; 3])
            }
        },
    );
    let c = one_text(
        frames,
        (w, h),
        white_text("label", region_at(10, 30, 230, 34)),
    )
    .without_fg();
    let r = c.assert_aa();
    assert_eq!(r.outcome, AssertionOutcome::Unknown, "{:?}", r.detail);
    let v = c.reading();
    assert_eq!(v.glyph_colour, Rgb::new(100, 100, 100), "{v:?}");
    assert_eq!(v.faint_px, 16 * 3 * 14, "{v:?}");
    assert_eq!(v.hidden_px_estimate, 0, "{v:?}");
    assert!(v.percentile_ratio > 4.5, "{v:?}");
    assert!(v.percentile_ratio_worst_case < 1.1, "{v:?}");
}

/// A glyph pixel that moved toward the ink but AWAY from the glyph colour —
/// the mask leaves it out — is read at no better than it renders. A MEASURED
/// grey 160 with no stroke interior may be a hairline of anything up to
/// white; a second word over magenta-grey rgb(180,60,180) paints
/// rgb(240,160,240), inside that ink but further from 160. Its best case is
/// white's 4.9:1; it shows 2.6:1: Unknown, not a pass.
#[test]
fn an_off_colour_glyph_pixel_is_read_at_what_it_renders() {
    let (w, h) = (300u32, 100u32);
    let seen: Vec<_> = (0..8).map(|i| (20 + 10 * i, 40u32, 2u32, 14u32)).collect();
    let off = stems(110, 40, 8, 14);
    let frames = rgb_frames(
        (w, h),
        |x, _| if x < 105 { [0; 3] } else { [180, 60, 180] },
        |x, y, _| {
            if in_any(&seen, x, y) {
                Some([160; 3])
            } else {
                in_any(&off, x, y).then_some([240, 160, 240])
            }
        },
    );
    let c = one_text(
        frames,
        (w, h),
        white_text("label", region_at(10, 30, 200, 34)),
    )
    .without_fg();
    let r = c.assert_aa();
    assert_eq!(r.outcome, AssertionOutcome::Unknown, "{:?}", r.detail);
    let v = c.reading();
    assert_eq!(v.glyph_colour, Rgb::new(160, 160, 160), "{v:?}");
    assert_eq!(v.off_colour_px, 8 * 3 * 14, "{v:?}");
    // Review round 3: off-colour pixels carried the coverage of the colour
    // they do not paint into `glyph_alpha`. They are read at what they
    // render, not at a coverage: only the seen stems' α is reported.
    assert_eq!(v.glyph_alpha, Some(1.0), "{v:?}");
    let shown = wcag_contrast(Rgb::new(240, 160, 240), Rgb::new(180, 60, 180));
    assert!(v.percentile_ratio > 4.5, "{v:?}");
    assert!(
        (v.percentile_ratio_worst_case - shown).abs() < 0.05,
        "{shown}: {v:?}"
    );
}

// ------------------------------------------------- review round 3 findings

/// A ring at 49% beside each bold stem — a glow, or faint glyphs — and one
/// faint word far off, 7.7% of the glyphs, showing 2.7:1. The ring was
/// ADDED to the worst case, which then held so many pixels that the faint
/// word fell under the 5th percentile and the page passed at 4.7:1. A pixel
/// that may or may not be a glyph is taken only where it LOWERS the worst
/// case: Unknown, at the faint word.
#[test]
fn a_ring_beside_the_strokes_never_outnumbers_a_faint_word() {
    let (w, h) = (300u32, 100u32);
    let bold = stems(20, 40, 8, 14);
    let faint = [(200u32, 40u32, 2u32, 14u32)];
    let ring_at = |x: u32, y: u32| {
        bold.iter().any(|&(x0, y0, sw, sh)| {
            let inside = |m: u32| x + m >= x0 && x < x0 + sw + m && y + m >= y0 && y < y0 + sh + m;
            inside(2) && !inside(1)
        })
    };
    let frames = rgb_frames(
        (w, h),
        |_, _| [40; 3],
        |x, y, _| {
            if in_any(&bold, x, y) {
                Some([255; 3])
            } else if in_any(&faint, x, y) {
                Some([white_at(40, 0.3); 3])
            } else if ring_at(x, y) {
                Some([white_at(40, 0.49); 3])
            } else {
                None
            }
        },
    );
    let c = one_text(
        frames,
        (w, h),
        white_text("label", region_at(10, 30, 220, 34)),
    );
    let r = c.assert_aa();
    assert_eq!(r.outcome, AssertionOutcome::Unknown, "{:?}", r.detail);
    let v = c.reading();
    assert_eq!(v.halo_px, 8 * 46, "{v:?}");
    let shown = wcag_contrast(
        Rgb::new(white_at(40, 0.3), white_at(40, 0.3), white_at(40, 0.3)),
        Rgb::new(40, 40, 40),
    );
    assert!(
        (v.percentile_ratio_worst_case - shown).abs() < 0.05,
        "{v:?}"
    );
    assert!(v.percentile_ratio > 10.0, "{v:?}");
}

/// A noisy capture (frame B one unit off at every 50th pixel) cannot tell a
/// faint glyph from noise, so every blind pixel may hide ink. That ink was
/// read at the text's own peak coverage, 1.08:1 over grey 246 — and passed
/// a 1.06:1 check. Hidden ink may sit at any coverage: it paints 1:1, and
/// the answer is Unknown.
#[test]
fn hidden_ink_on_a_noisy_capture_paints_one_to_one() {
    let (w, h) = (300u32, 100u32);
    let glyphs = stems(20, 40, 8, 14);
    let (a, mut b) = rgb_frames(
        (w, h),
        |x, _| if x < 150 { [40; 3] } else { [246; 3] },
        |x, y, _| in_any(&glyphs, x, y).then_some([255; 3]),
    );
    for (i, p) in b.buffer.pixels_mut().enumerate() {
        if i % 50 == 0 {
            p.0[0] = p.0[0].saturating_sub(1).max(1);
        }
    }
    let c = one_text(
        (a, b),
        (w, h),
        white_text("label", region_at(10, 30, 260, 34)),
    );
    let r = c.assert_with(Some(&c.b), Some(1.06), None);
    assert_eq!(r.outcome, AssertionOutcome::Unknown, "{:?}", r.detail);
    let v = c.reading();
    assert_eq!(v.faint_px, 0, "{v:?}");
    assert!(v.hidden_px_estimate > 0, "{v:?}");
    assert_eq!(v.percentile_ratio_worst_case, 1.0, "{v:?}");
}

/// The wrapped translucent span of round 2, at 95% and 92% opacity over a
/// lighter patch where it shows 2.3:1. Read by opacity band, a span at 90%
/// or more sat in the parent's opaque band, its 4% of the pixels fell under
/// the 5th percentile, and the page passed at 5.9:1. Every pixel shared,
/// any of them may be the span's: Unknown, at the span.
#[test]
fn a_nearly_opaque_wrapped_span_inside_its_parent_is_unknown() {
    let (w, h) = (300u32, 120u32);
    let lines = [30u32, 70];
    let opaque: Vec<_> = lines
        .iter()
        .flat_map(|&y| stems(15, y, 26, 14))
        .enumerate()
        .filter(|(i, _)| i % 26 != 13)
        .map(|(_, s)| s)
        .collect();
    let span: Vec<_> = lines.iter().map(|&y| (145u32, y, 3u32, 14u32)).collect();
    let patch: Vec<_> = span
        .iter()
        .map(|&(x, y, sw, sh)| (x - 1, y - 1, sw + 2, sh + 2))
        .collect();
    for alpha in [0.95, 0.92] {
        let frames = rgb_frames(
            (w, h),
            |x, y| {
                if in_any(&patch, x, y) {
                    [170; 3]
                } else {
                    [100; 3]
                }
            },
            |x, y, under| {
                if in_any(&opaque, x, y) {
                    Some([255; 3])
                } else if in_any(&span, x, y) {
                    Some([white_at(under[0], alpha); 3])
                } else {
                    None
                }
            },
        );
        let mut p = white_text("p", region_at(10, 20, 270, 80));
        p.text = Some("Lead text that wraps".into());
        p.children_ids = vec!["span".into()];
        let mut span_el = white_text("span", region_at(10, 20, 270, 80));
        span_el.text = Some("wraps".into());
        let (a, b) = frames;
        let c = Case {
            a,
            b,
            snap: ElementSnapshot {
                elements: vec![hero(region_at(0, 0, w, h)), p, span_el],
                ..Default::default()
            },
        };
        let r = c.assert_aa();
        assert_eq!(
            r.outcome,
            AssertionOutcome::Unknown,
            "{alpha}: {:?}",
            r.detail
        );
        let v = hero_value(&c);
        let shown = wcag_contrast(
            Rgb::new(
                white_at(170, alpha),
                white_at(170, alpha),
                white_at(170, alpha),
            ),
            Rgb::new(170, 170, 170),
        );
        for t in v.value().expect("measured") {
            assert_eq!(t.shared_px, t.sampled_px, "{t:?}");
            assert!(t.percentile_ratio > 4.5, "{t:?}");
            assert!(
                (t.percentile_ratio_worst_case - shown).abs() < 0.05,
                "{alpha}: {t:?}"
            );
        }
    }
}

/// White over rgb(154,108,90) is 4.499999851:1, which `f32` rounds to 4.5:
/// the readings were held in `f32` and the page passed WCAG AA. Held in
/// `f64` it fails.
#[test]
fn a_ratio_a_hair_under_the_threshold_fails() {
    let (w, h) = (300u32, 100u32);
    let glyphs = stems(20, 40, 8, 14);
    let frames = rgb_frames(
        (w, h),
        |_, _| [154, 108, 90],
        |x, y, _| in_any(&glyphs, x, y).then_some([255; 3]),
    );
    let c = one_text(
        frames,
        (w, h),
        white_text("label", region_at(10, 30, 100, 34)),
    );
    let r = c.assert_aa();
    assert_eq!(r.outcome, AssertionOutcome::Failed, "{:?}", r.detail);
    let v = c.reading();
    assert!(v.percentile_ratio < 4.5, "{v:?}");
    assert!(v.percentile_ratio > 4.4999, "{v:?}");
}

/// A card that paints its own grey text around a white heading, one of whose
/// five stems sits on a light patch it shows 1.4:1 on. Every heading pixel
/// lay in the card's box and was SHARED, so the best case could drop the
/// failing stem. Grey cannot paint white: a pixel the other text's colour
/// does not explain is this text's own, and the heading fails.
#[test]
fn a_containing_text_of_another_colour_does_not_share_the_glyphs() {
    let (w, h) = (320u32, 180u32);
    let heading_stems = stems(104, 73, 5, 14);
    let card_stems = stems(70, 110, 4, 10);
    let light = (142u32, 72u32, 5u32, 16u32);
    let frames = rgb_frames(
        (w, h),
        |x, y| {
            if in_any(&[light], x, y) {
                [210; 3]
            } else {
                [40; 3]
            }
        },
        |x, y, _| {
            if in_any(&heading_stems, x, y) {
                Some([255; 3])
            } else {
                in_any(&card_stems, x, y).then_some([140; 3])
            }
        },
    );
    let heading = white_text("heading", region_at(100, 70, 60, 20));
    let mut card = white_text("card", region_at(60, 30, 200, 120));
    card.fg_color = Some(Rgb::new(140, 140, 140));
    card.text = Some("heading Read more".into());
    card.children_ids = vec!["heading".into()];
    let (a, b) = frames;
    let c = Case {
        a,
        b,
        snap: ElementSnapshot {
            elements: vec![hero(region_at(0, 0, w, h)), card, heading],
            ..Default::default()
        },
    };
    let r = c.assert_aa();
    assert_eq!(r.outcome, AssertionOutcome::Failed, "{:?}", r.detail);
    assert!(
        r.detail.as_deref().unwrap().contains("text 'heading'"),
        "{:?}",
        r.detail
    );
    let v = hero_value(&c);
    let heading = v
        .value()
        .expect("measured")
        .iter()
        .find(|t| t.element == "heading")
        .unwrap()
        .clone();
    assert_eq!(heading.shared_px, 0, "{heading:?}");
}

// ------------------------------------------------- review round 4 findings

/// Sub-threshold noise in frame B: every `every`-th pixel one unit darker in
/// red, so frame A reads one unit toward white there.
fn add_noise(b: &mut Frame, every: usize) {
    for (i, p) in b.buffer.pixels_mut().enumerate() {
        if i % every == 0 {
            p.0[0] = p.0[0].saturating_sub(1).max(1);
        }
    }
}

/// A word too faint for the mask (sub-threshold, on a background the blind
/// radius does not reach) beside an opaque one. On a noise-free capture it
/// is read as faint glyphs; on a NOISY one the noise and the word paint
/// alike, and both were dropped, so the text passed on the opaque word
/// (14.7:1) while the faint one shows about 1.05:1. Each such pixel is
/// optional there, noise or glyph: Unknown, saying so — never a pass.
#[test]
fn on_a_noisy_capture_a_word_too_faint_for_the_mask_is_never_a_pass() {
    let (w, h) = (300u32, 100u32);
    let (opaque, faint) = (stems(20, 40, 8, 14), stems(170, 40, 8, 14));
    for (right, alpha) in [(100u8, 0.02), (200, 0.08), (40, 0.02)] {
        let (a, mut b) = rgb_frames(
            (w, h),
            |x, _| if x < 150 { [40; 3] } else { [right; 3] },
            |x, y, bg| {
                if in_any(&opaque, x, y) {
                    Some([255; 3])
                } else if in_any(&faint, x, y) {
                    Some([white_at(bg[0], alpha); 3])
                } else {
                    None
                }
            },
        );
        let quiet = one_text(
            (a.clone(), b.clone()),
            (w, h),
            white_text("label", region_at(10, 30, 260, 34)),
        );
        assert_ne!(
            quiet.assert_aa().outcome,
            AssertionOutcome::Passed,
            "{right}/{alpha} noise-free"
        );
        assert_eq!(quiet.reading().noise_px, 0, "{right}/{alpha}");
        add_noise(&mut b, 50);
        let c = one_text(
            (a, b),
            (w, h),
            white_text("label", region_at(10, 30, 260, 34)),
        );
        let r = c.assert_aa();
        assert_eq!(
            r.outcome,
            AssertionOutcome::Unknown,
            "{right}/{alpha}: {:?}",
            r.detail
        );
        let d = r.detail.as_deref().unwrap();
        assert!(d.contains("sub-threshold noise"), "{d}");
        let v = c.reading();
        assert!(v.noise_px > 0, "{right}/{alpha}: {v:?}");
        assert_eq!(v.faint_px, 0, "faint pixels are not trusted: {v:?}");
        let shown = wcag_contrast(
            Rgb::new(
                white_at(right, alpha),
                white_at(right, alpha),
                white_at(right, alpha),
            ),
            Rgb::new(right, right, right),
        );
        assert!(
            v.percentile_ratio_worst_case <= shown + 0.05,
            "{right}/{alpha}: {shown}: {v:?}"
        );
    }
}

/// A box whose visible glyphs stay above the image, on a NOISY capture: too
/// few mask pixels reach it, but a word too faint for the mask could lie
/// over it where the noise paints alike. That is Unknown, not "not over the
/// image"; noise-free, the same box is not over it.
#[test]
fn on_a_noisy_capture_too_few_glyphs_over_the_image_is_unknown() {
    let (w, h) = (200u32, 100u32);
    let strokes = stems(15, 41, 6, 7); // rows 41..48, above the image
    let build = |noisy: bool| {
        let (a, mut b) = painted((w, h), |_, _| 100, &strokes, 255);
        if noisy {
            add_noise(&mut b, 5);
        }
        Case {
            a,
            b,
            snap: ElementSnapshot {
                elements: vec![
                    hero(Region {
                        x: 0,
                        y: 50,
                        w,
                        h: 50,
                    }),
                    white_text(
                        "label",
                        Region {
                            x: 10,
                            y: 40,
                            w: 80,
                            h: 20,
                        },
                    ),
                ],
                ..Default::default()
            },
        }
    };
    let quiet = build(false);
    let r = quiet.assert_aa();
    assert!(
        r.detail.as_deref().unwrap().contains("no text lies over"),
        "{:?}",
        r.detail
    );
    let noisy = build(true);
    let r = noisy.assert_aa();
    assert_eq!(r.outcome, AssertionOutcome::Unknown, "{:?}", r.detail);
    let d = r.detail.as_deref().unwrap();
    assert!(
        d.contains("whether its glyphs reach the image is unknown")
            && d.contains("sub-threshold noise"),
        "{d}"
    );
}

/// A same-colour RING one pixel round bold strokes (a 1 px glow at 55% or
/// 80%) shares the strokes' local peak. Read as the strokes themselves it
/// outnumbered a faint word far off (28 px against 336 bold), and the text
/// passed while that word shows 2.7:1. A pixel covered under INK_COVERAGE of
/// its local peak — an edge, or such a ring — is optional: Unknown, as
/// without the ring.
#[test]
fn a_ring_within_one_pixel_of_the_strokes_does_not_outnumber_a_faint_word() {
    let (w, h) = (300u32, 100u32);
    let bold = stems(20, 40, 8, 14);
    let faint = [(200u32, 40u32, 2u32, 14u32)];
    let ring_at = |x: u32, y: u32| {
        bold.iter().any(|&(x0, y0, sw, sh)| {
            let inside = |m: u32| x + m >= x0 && x < x0 + sw + m && y + m >= y0 && y < y0 + sh + m;
            inside(1) && !inside(0)
        })
    };
    for ring in [0.0, 0.55, 0.8] {
        let frames = rgb_frames(
            (w, h),
            |_, _| [40; 3],
            |x, y, _| {
                if in_any(&bold, x, y) {
                    Some([255; 3])
                } else if in_any(&faint, x, y) {
                    Some([white_at(40, 0.3); 3])
                } else if ring > 0.0 && ring_at(x, y) {
                    Some([white_at(40, ring); 3])
                } else {
                    None
                }
            },
        );
        let c = one_text(
            frames,
            (w, h),
            white_text("label", region_at(10, 30, 220, 34)),
        );
        let r = c.assert_aa();
        assert_eq!(
            r.outcome,
            AssertionOutcome::Unknown,
            "{ring}: {:?}",
            r.detail
        );
        let v = c.reading();
        let shown = wcag_contrast(
            Rgb::new(white_at(40, 0.3), white_at(40, 0.3), white_at(40, 0.3)),
            Rgb::new(40, 40, 40),
        );
        assert!(
            v.percentile_ratio_worst_case <= shown + 0.05,
            "{ring}: {shown}: {v:?}"
        );
        assert!(v.percentile_ratio > 4.5, "{ring}: {v:?}");
        if ring > 0.0 {
            assert!(v.halo_px > 0, "{ring}: {v:?}");
        }
    }
}

/// Declared white, glyphs rendering 240 (a `-webkit-text-fill-color`) in
/// strokes too thin to hold an interior; a DOTTED underline in the declared
/// colour under the left word. Its 2x2 dots are no line, so they joined the
/// one-fill bound and pinned it to white: the right word, at 240 over a 248
/// patch (1.05:1), was not ink the box could hide, and the text passed. The
/// bound the majority of the glyph pixels set is kept too: Unknown, with the
/// dots, with a solid underline, and with neither.
#[test]
fn a_dotted_underline_in_the_declared_colour_does_not_pin_the_fill() {
    let (w, h) = (300u32, 100u32);
    for width in [1u32, 2] {
        let (left, right) = (
            stems_of(width, 20, 40, 6, 14),
            stems_of(width, 170, 40, 6, 14),
        );
        for deco in ["none", "dotted", "solid"] {
            let dots: Vec<(u32, u32, u32, u32)> = match deco {
                "dotted" => (0..14).map(|i| (20 + 4 * i, 57u32, 2u32, 2u32)).collect(),
                "solid" => vec![(20u32, 57u32, 53u32, 2u32)],
                _ => vec![],
            };
            let frames = rgb_frames(
                (w, h),
                |x, _| if x < 150 { [40; 3] } else { [248; 3] },
                |x, y, _| {
                    if in_any(&dots, x, y) {
                        Some([255; 3])
                    } else if in_any(&left, x, y) || in_any(&right, x, y) {
                        Some([240; 3])
                    } else {
                        None
                    }
                },
            );
            let c = one_text(
                frames,
                (w, h),
                white_text("label", region_at(10, 30, 260, 34)),
            );
            let r = c.assert_aa();
            assert_eq!(
                r.outcome,
                AssertionOutcome::Unknown,
                "{width}/{deco}: {:?}",
                r.detail
            );
            assert!(c.reading().hidden_px_estimate > 0, "{width}/{deco}");
        }
    }
}

/// The same through a child: `<p>` declared white rendering 240 and a child
/// `<a>` over its left word — with a dotted underline in currentColor, or
/// rendering white itself. The child's pixels count for the parent (the
/// colours agree) and pinned its one-fill bound to white, so the parent's
/// right word at 240 over 248 passed unseen. The parent's fill is bounded by
/// its OWN pixels too (its words beside the link, `more`, at 240 over dark
/// grey), which the child's cannot outvote, though the link's are more: the
/// parent is Unknown, the link, all white over dark grey, passes.
#[test]
fn a_child_rendering_the_declared_colour_does_not_pin_its_parents_fill() {
    let (w, h) = (300u32, 100u32);
    for (child, width) in [("dotted", 2u32), ("white", 1), ("white", 2)] {
        let left = stems_of(width, 20, 40, 6, 14);
        let right = stems_of(width, 170, 40, 6, 14);
        let more = stems_of(width, 90, 40, 4, 14);
        let dots: Vec<(u32, u32, u32, u32)> =
            (0..14).map(|i| (20 + 4 * i, 57u32, 2u32, 2u32)).collect();
        let (a, b) = rgb_frames(
            (w, h),
            |x, _| if x < 150 { [40; 3] } else { [248; 3] },
            |x, y, _| {
                if child == "dotted" && in_any(&dots, x, y) {
                    Some([255; 3])
                } else if in_any(&left, x, y) {
                    Some([if child == "white" { 255 } else { 240 }; 3])
                } else if in_any(&right, x, y) || in_any(&more, x, y) {
                    Some([240; 3])
                } else {
                    None
                }
            },
        );
        let mut p = white_text("p", region_at(10, 30, 260, 34));
        p.text = Some("Visit our shop today".into());
        p.children_ids = vec!["link".into()];
        let mut link = white_text("link", region_at(16, 36, 64, 26));
        link.text = Some("our shop".into());
        let c = Case {
            a,
            b,
            snap: ElementSnapshot {
                elements: vec![hero(region_at(0, 0, w, h)), p, link],
                ..Default::default()
            },
        };
        let r = c.assert_aa();
        assert_eq!(
            r.outcome,
            AssertionOutcome::Unknown,
            "{child}/{width}: {:?}",
            r.detail
        );
        assert!(
            r.detail.as_deref().unwrap().contains("text 'p'"),
            "{child}/{width}: {:?}",
            r.detail
        );
        let v = hero_value(&c);
        let values = v.value().expect("measured");
        let of = |id: &str| values.iter().find(|t| t.element == id).unwrap().clone();
        assert!(
            of("p").hidden_px_estimate > 0,
            "{child}/{width}: {values:?}"
        );
        if child == "white" {
            let link = of("link");
            assert!(link.percentile_ratio_worst_case >= 4.5, "{link:?}");
        }
    }
}

/// A white glow WIDER than HALO_CSS onto a near-white photo, round white
/// glyphs on a tight dark highlight (the glyphs show 18:1). The glow's ramp
/// past HALO_CSS was read as faint glyphs, and the text FAILED on its glow.
/// A faint pixel joined to the mask by pixels that moved toward the ink is
/// optional at any distance: never a failure. A glow that FLATTENS (a wide
/// blur's plateau) peaks locally past HALO_CSS, so only the join tells it
/// from a glyph there: over 245 those pixels are under the mask's threshold
/// but not blind, and over 250 blind. Over a mid photo (150 to 230) the
/// plateau moves its pixels past the threshold, into the mask, joined to the
/// strokes: a mask pixel whose local peak falls under INK_COVERAGE of the
/// peak of the stroke it joins is beside that stroke, as a glow is.
#[test]
fn a_glow_wider_than_the_halo_onto_a_near_white_photo_is_not_a_failure() {
    let (w, h) = (300u32, 100u32);
    let glyphs = stems(40, 40, 12, 14);
    let highlight: Vec<_> = glyphs
        .iter()
        .map(|&(x, y, sw, sh)| (x - 1, y - 1, sw + 2, sh + 2))
        .collect();
    let reach = |x: u32, y: u32| {
        (0..=10u32).find(|&d| {
            glyphs.iter().any(|&(x0, y0, sw, sh)| {
                x + d >= x0 && x < x0 + sw + d && y + d >= y0 && y < y0 + sh + d
            })
        })
    };
    let smooth = [0.6, 0.5, 0.4, 0.35, 0.3, 0.25, 0.2, 0.15, 0.0, 0.0];
    let flat = [0.6, 0.5, 0.4, 0.3, 0.3, 0.3, 0.3, 0.3, 0.3, 0.3];
    for (photo, n, ramp) in [
        (250u8, 6usize, smooth),
        (250, 8, smooth),
        (245, 8, smooth),
        (200, 8, smooth),
        (245, 10, flat),
        (250, 10, flat),
        (230, 10, flat),
        (200, 10, flat),
        (150, 10, flat),
    ] {
        let bg = |x: u32, y: u32| {
            if in_any(&highlight, x, y) {
                [30u8; 3]
            } else {
                [photo; 3]
            }
        };
        let frames = rgb_frames((w, h), bg, |x, y, under| match reach(x, y) {
            Some(0) => Some([255; 3]),
            Some(d) if (d as usize) <= n => Some([white_at(under[0], ramp[d as usize - 1]); 3]),
            _ => None,
        });
        let c = one_text(
            frames,
            (w, h),
            white_text("label", region_at(20, 20, 150, 56)),
        );
        let r = c.assert_aa();
        assert_ne!(
            r.outcome,
            AssertionOutcome::Failed,
            "{photo}/{n}: {:?}",
            r.detail
        );
        let v = c.reading();
        assert!(v.percentile_ratio > 4.5, "{photo}/{n}: {v:?}");
        assert_eq!(v.faint_px, 0, "the glow is no glyph of its own: {v:?}");
    }
}

/// Thin white strokes (85% coverage, so no pixel is covered whole) over
/// dark grey, with bright specks (235) between them. Such strokes widen the
/// ink down to what they render, so a speck in that range is a pixel where
/// ink may hide: Unknown, estimating it. Without specks, the text passes.
#[test]
fn bright_specks_between_thin_strokes_may_hide_ink() {
    let (w, h) = (300u32, 100u32);
    let glyphs = stems_of(2, 20, 40, 16, 14);
    for specks in [0u32, 4] {
        let at: Vec<(u32, u32, u32, u32)> = (0..specks)
            .map(|i| (25 + 20 * i, 45u32, 3u32, 3u32))
            .collect();
        let frames = rgb_frames(
            (w, h),
            |x, y| if in_any(&at, x, y) { [235; 3] } else { [40; 3] },
            |x, y, under| in_any(&glyphs, x, y).then_some([white_at(under[0], 0.85); 3]),
        );
        let c = one_text(
            frames,
            (w, h),
            white_text("label", region_at(10, 30, 180, 34)),
        );
        let r = c.assert_aa();
        let v = c.reading();
        if specks == 0 {
            assert_eq!(r.outcome, AssertionOutcome::Passed, "{:?}", r.detail);
            assert_eq!(v.hidden_px_estimate, 0, "{v:?}");
        } else {
            assert_eq!(r.outcome, AssertionOutcome::Unknown, "{:?}", r.detail);
            assert_eq!(v.hidden_px_estimate, 9 * specks as u64, "{v:?}");
        }
    }
}

// ------------------------------------------------- review round 5 findings

/// A static texture: a deterministic hash in `base ± amp`.
fn texture(x: u32, y: u32, base: i32, amp: i32) -> u8 {
    let mut h = x.wrapping_mul(374_761_393) ^ y.wrapping_mul(668_265_263);
    h = (h ^ (h >> 13)).wrapping_mul(1_274_126_177);
    h ^= h >> 16;
    let v = base + (h % (2 * amp as u32 + 1)) as i32 - amp;
    v.clamp(0, 255) as u8
}

/// How much of pixel `(x, y)` a glyph set covers, supersampled 4 × 4: rings
/// `(cx, cy, r_out, r_in)` and bars `(x, y, w, h)` in fractional px.
fn supersampled(
    x: u32,
    y: u32,
    rings: &[(f64, f64, f64, f64)],
    bars: &[(f64, f64, f64, f64)],
) -> f64 {
    let mut hit = 0;
    for sy in 0..4 {
        for sx in 0..4 {
            let px = x as f64 + (sx as f64 + 0.5) / 4.0;
            let py = y as f64 + (sy as f64 + 0.5) / 4.0;
            let in_ring = rings.iter().any(|&(cx, cy, ro, ri)| {
                let d = ((px - cx).powi(2) + (py - cy).powi(2)).sqrt();
                d <= ro && d >= ri
            });
            let in_bar = bars
                .iter()
                .any(|&(bx, by, bw, bh)| px >= bx && px < bx + bw && py >= by && py < by + bh);
            hit += (in_ring || in_bar) as u32;
        }
    }
    hit as f64 / 16.0
}

/// Ordinary anti-aliased letters — rings and fractional stems, supersampled —
/// in opaque white over a static textured photo. Their edges and the thin
/// joins beside their stems are read as a whole, with the cores or not at
/// all: taken pixel by pixel, the worst case picked the edges that happen
/// to lie over the lightest texture and read every such text Unknown just
/// over the threshold. Near the threshold it is decided both ways: over
/// 90 ± 15 and 100 ± 10 it passes (the glyphs show about 5:1), over 115 ± 8
/// it fails (the lightest twentieth of the texture under it shows under
/// 4.5:1 even opaque).
#[test]
fn anti_aliased_text_over_a_textured_photo_is_decided_at_the_threshold() {
    let (w, h) = (300u32, 100u32);
    let (mut rings, mut bars) = (Vec::new(), Vec::new());
    for i in 0..6 {
        let cx = 30.0 + 22.0 * i as f64 + 0.37 * i as f64;
        rings.push((cx, 50.3, 8.2, 6.6));
        bars.push((cx + 9.4 + 0.21 * i as f64, 38.6, 1.7, 20.3));
    }
    for (base, amp, want) in [
        (40, 20, AssertionOutcome::Passed),
        (90, 15, AssertionOutcome::Passed),
        (100, 10, AssertionOutcome::Passed),
        (115, 8, AssertionOutcome::Failed),
    ] {
        let frames = rgb_frames(
            (w, h),
            |x, y| [texture(x, y, base, amp); 3],
            |x, y, under| {
                let cv = supersampled(x, y, &rings, &bars);
                (cv > 0.0).then(|| [white_at(under[0], cv); 3])
            },
        );
        let c = one_text(
            frames,
            (w, h),
            white_text("label", region_at(15, 30, 180, 40)),
        );
        let r = c.assert_aa();
        assert_eq!(r.outcome, want, "{base}±{amp}: {:?}", r.detail);
        let v = c.reading();
        assert!(v.halo_px > 0, "edges were read: {v:?}");
    }
}

/// A `<p>` declared white whose OWN words show nowhere apart from its child
/// `<a>`'s box: they render 240 over a 248 patch (moved away from white, so
/// the mask holds none of them), or one of them lies inside the link's
/// widened box. Every glyph pixel the parent's mask holds is then the link's
/// too, and the link's white bounded the parent's fill, so the 240 words
/// went unseen and the parent passed. Its own glyphs were not seen: Unknown.
/// The link, all white over dark grey, is measured and passes.
#[test]
fn a_parent_none_of_whose_own_glyphs_show_apart_from_its_child_is_unknown() {
    let (w, h) = (300u32, 100u32);
    for (variant, width) in [("on_patch", 2u32), ("on_patch", 1), ("inside_link_box", 2)] {
        let left = stems_of(width, 20, 40, 6, 14);
        let right = stems_of(width, 170, 40, 6, 14);
        let near = stems_of(width, 76, 40, 1, 14);
        let (a, b) = rgb_frames(
            (w, h),
            |x, _| if x < 150 { [40; 3] } else { [248; 3] },
            |x, y, _| {
                if in_any(&left, x, y) {
                    Some([255; 3])
                } else if in_any(&right, x, y)
                    || (variant == "inside_link_box" && in_any(&near, x, y))
                {
                    Some([240; 3])
                } else {
                    None
                }
            },
        );
        let mut p = white_text("p", region_at(10, 30, 260, 34));
        p.text = Some("Visit our shop today".into());
        p.children_ids = vec!["link".into()];
        let mut link = white_text("link", region_at(16, 36, 64, 26));
        link.text = Some("our shop".into());
        let c = Case {
            a,
            b,
            snap: ElementSnapshot {
                elements: vec![hero(region_at(0, 0, w, h)), p, link],
                ..Default::default()
            },
        };
        let r = c.assert_aa();
        assert_eq!(
            r.outcome,
            AssertionOutcome::Unknown,
            "{variant}/{width}: {:?}",
            r.detail
        );
        let d = r.detail.as_deref().unwrap();
        assert!(
            d.contains("text 'p'") && d.contains("none of its inner texts"),
            "{variant}/{width}: {d}"
        );
        let v = hero_value(&c);
        let values = v.value().expect("the link is measured");
        assert!(values.iter().all(|t| t.element != "p"), "{values:?}");
        let link = values.iter().find(|t| t.element == "link").unwrap();
        assert!(link.percentile_ratio_worst_case >= 4.5, "{link:?}");
    }
}

/// Hidden ink is optional in EVERY extent a reading takes, the cores alone
/// included. Here 3 px stems render 65% / 100% / 65%, so their edges are
/// twice their cores, and a 4 × 4 patch of the ink's own colour in the box
/// may hide a glyph: under the 5th percentile's share of all the glyph
/// pixels, over it of the cores alone. Read with the cores alone, the patch
/// may be ink and the text is Unknown; without the patch it passes.
#[test]
fn hidden_ink_counts_against_the_cores_alone() {
    let (w, h) = (300u32, 100u32);
    let stems: Vec<u32> = (0..12).map(|i| 20 + 8 * i).collect();
    let patch = [(114u32, 44u32, 4u32, 4u32)];
    for with_patch in [false, true] {
        let frames = rgb_frames(
            (w, h),
            |x, y| {
                if with_patch && in_any(&patch, x, y) {
                    [255; 3]
                } else {
                    [40; 3]
                }
            },
            |x, y, under| {
                let col = stems.iter().find(|&&s| x >= s && x < s + 3)?;
                (40..54).contains(&y).then(|| {
                    let cv = if x == col + 1 { 1.0 } else { 0.65 };
                    [white_at(under[0], cv); 3]
                })
            },
        );
        let c = one_text(
            frames,
            (w, h),
            white_text("label", region_at(10, 30, 120, 34)),
        );
        let r = c.assert_aa();
        let v = c.reading();
        assert!(v.halo_px >= 2 * 12 * 14, "the edges are read: {v:?}");
        if with_patch {
            assert_eq!(r.outcome, AssertionOutcome::Unknown, "{:?}", r.detail);
            assert_eq!(v.hidden_px_estimate, 16, "{v:?}");
            assert!(v.percentile_ratio_worst_case < 4.5, "{v:?}");
        } else {
            assert_eq!(r.outcome, AssertionOutcome::Passed, "{:?}", r.detail);
        }
    }
}

// ------------------------------------------------- review round 6 findings

/// The `<p>` of the test above whose child `<a>` is not laid strictly inside
/// it: a link with vertical padding overhangs its parent's line box, and a
/// child may have its parent's exact box. Whether a text wraps another is
/// read from the DOM, not from their boxes, so its own glyphs unseen apart
/// from the link's are Unknown whatever the boxes. Where its own words are
/// white over the dark grey too, they show apart from the link and it is
/// measured, except where the link's box is its own: there every pixel may
/// be the link's.
#[test]
fn a_parent_whose_child_overhangs_or_matches_its_box_is_unknown() {
    let (w, h) = (300u32, 100u32);
    for (variant, link_box) in [
        ("overhang", region_at(16, 28, 64, 38)),
        ("overhang_top", region_at(16, 29, 64, 30)),
        ("identical", region_at(10, 30, 260, 34)),
    ] {
        for own_words_seen in [false, true] {
            let left = stems_of(2, 20, 40, 6, 14);
            let right = stems_of(2, 170, 40, 6, 14);
            let (a, b) = rgb_frames(
                (w, h),
                |x, _| {
                    if x < 150 || own_words_seen {
                        [40; 3]
                    } else {
                        [248; 3]
                    }
                },
                |x, y, _| {
                    if in_any(&left, x, y) || (own_words_seen && in_any(&right, x, y)) {
                        Some([255; 3])
                    } else if in_any(&right, x, y) {
                        Some([240; 3])
                    } else {
                        None
                    }
                },
            );
            let mut p = white_text("p", region_at(10, 30, 260, 34));
            p.text = Some("Visit our shop today".into());
            p.children_ids = vec!["link".into()];
            let mut link = white_text("link", link_box);
            link.text = Some("our shop".into());
            let c = Case {
                a,
                b,
                snap: ElementSnapshot {
                    elements: vec![hero(region_at(0, 0, w, h)), p, link],
                    ..Default::default()
                },
            };
            let r = c.assert_aa();
            let v = hero_value(&c);
            let values = v.value().expect("the link is measured");
            let link = values.iter().find(|t| t.element == "link").unwrap();
            assert!(link.percentile_ratio_worst_case >= 4.5, "{link:?}");
            if own_words_seen && variant != "identical" {
                assert_eq!(
                    r.outcome,
                    AssertionOutcome::Passed,
                    "{variant}: {:?}",
                    r.detail
                );
                assert!(values.iter().any(|t| t.element == "p"), "{values:?}");
            } else {
                assert_eq!(
                    r.outcome,
                    AssertionOutcome::Unknown,
                    "{variant}/{own_words_seen}: {:?}",
                    r.detail
                );
                let d = r.detail.as_deref().unwrap();
                assert!(
                    d.contains("text 'p'") && d.contains("none of its inner texts"),
                    "{variant}/{own_words_seen}: {d}"
                );
                assert!(values.iter().all(|t| t.element != "p"), "{values:?}");
            }
        }
    }
}

/// A `<p>` whose own words all lie inside the box of the link it wraps (a
/// link broken over two lines spans both lines' width), in another colour:
/// the link's blue explains none of the parent's white pixels, so they are
/// the parent's own, and it is measured — passing over dark and mid grey,
/// where the blue link itself fails over mid grey.
#[test]
fn a_parent_whose_words_lie_in_its_childs_box_in_another_colour_is_measured() {
    let (w, h) = (300u32, 100u32);
    let blue = [110u8, 170, 255];
    let p_words: Vec<_> = stems_of(2, 20, 26, 5, 14)
        .into_iter()
        .chain(stems_of(2, 170, 60, 7, 14))
        .collect();
    let l_words: Vec<_> = stems_of(2, 80, 26, 17, 14)
        .into_iter()
        .chain(stems_of(2, 20, 60, 13, 14))
        .collect();
    for (bg, link_passes) in [(40u8, true), (90, false)] {
        let (a, b) = rgb_frames(
            (w, h),
            |_, _| [bg; 3],
            |x, y, _| {
                if in_any(&p_words, x, y) {
                    Some([255; 3])
                } else if in_any(&l_words, x, y) {
                    Some(blue)
                } else {
                    None
                }
            },
        );
        let mut p = white_text("p", region_at(10, 20, 260, 64));
        p.text = Some("Read our terms of service today".into());
        p.children_ids = vec!["link".into()];
        let mut link = white_text("link", region_at(14, 22, 250, 58));
        link.fg_color = Some(Rgb::new(blue[0], blue[1], blue[2]));
        link.text = Some("terms of service".into());
        let c = Case {
            a,
            b,
            snap: ElementSnapshot {
                elements: vec![hero(region_at(0, 0, w, h)), p, link],
                ..Default::default()
            },
        };
        let r = c.assert_aa();
        let expected = if link_passes {
            AssertionOutcome::Passed
        } else {
            AssertionOutcome::Failed
        };
        assert_eq!(r.outcome, expected, "bg {bg}: {:?}", r.detail);
        let v = hero_value(&c);
        let values = v.value().expect("both are measured");
        let p = values.iter().find(|t| t.element == "p").expect("p");
        assert!(p.percentile_ratio_worst_case >= 4.5, "{p:?}");
        assert_eq!(p.shared_px, 0, "{p:?}");
    }
}

/// A `<p>` and the link it wraps whose glyphs both lie below the image (the
/// boxes meet it; the glyphs do not): not over the image, so the image has
/// no text over it — Absent, not Unknown — however few of the parent's
/// glyph pixels lie apart from the link's.
#[test]
fn a_wrapper_whose_glyphs_miss_the_image_is_not_over_it() {
    let (w, h) = (300u32, 100u32);
    let words = stems_of(2, 20, 46, 6, 14);
    let (a, b) = rgb_frames(
        (w, h),
        |_, _| [40; 3],
        |x, y, _| in_any(&words, x, y).then_some([255; 3]),
    );
    let mut p = white_text("p", region_at(10, 30, 260, 34));
    p.text = Some("Visit our shop".into());
    p.children_ids = vec!["link".into()];
    let mut link = white_text("link", region_at(16, 40, 64, 22));
    link.text = Some("our shop".into());
    let c = Case {
        a,
        b,
        snap: ElementSnapshot {
            elements: vec![hero(region_at(0, 0, w, 40)), p, link],
            ..Default::default()
        },
    };
    assert_eq!(hero_value(&c).status(), ObservationStatus::Absent);
}

/// One word whose letters are joined (a connected script, an underline
/// drawn through them) and fade from opaque to `end` across it. The faint
/// letters are not beside a better covered stroke: they are the word, and
/// `glyph_alpha` reports them — not the opaque letters the join ties them
/// to.
#[test]
fn glyph_alpha_reads_the_faint_end_of_a_joined_word() {
    let (w, h) = (300u32, 100u32);
    let stems_x: Vec<u32> = (0..12).map(|i| 20 + 12 * i).collect();
    for (end, bg) in [(1.0f64, 40u8), (0.3, 40), (0.5, 90)] {
        let frames = rgb_frames(
            (w, h),
            |_, _| [bg; 3],
            |x, y, under| {
                let on_stem = stems_x.iter().any(|&s| x >= s && x < s + 3) && (40..56).contains(&y);
                let on_join = (20..20 + 12 * 11 + 3).contains(&x) && (54..56).contains(&y);
                (on_stem || on_join).then(|| {
                    let t = (x as f64 - 20.0) / (12.0 * 11.0 + 3.0);
                    [white_at(under[0], 1.0 + (end - 1.0) * t.clamp(0.0, 1.0)); 3]
                })
            },
        );
        let c = one_text(
            frames,
            (w, h),
            white_text("label", region_at(10, 30, 200, 34)),
        );
        let r = c.assert_aa();
        let v = c.reading();
        let alpha = v.glyph_alpha.expect("cores were read");
        if end == 1.0 {
            assert_eq!(r.outcome, AssertionOutcome::Passed, "{:?}", r.detail);
            assert!(alpha > 0.99, "{v:?}");
        } else {
            assert_eq!(r.outcome, AssertionOutcome::Unknown, "{:?}", r.detail);
            assert!(alpha < end + 0.1, "end {end}: {v:?}");
        }
    }
}

// ------------------------------------------------- review round 7 findings

/// The parent/child case with the other text UNRELATED to the `<p>`: a
/// badge or caption laid inside or over its box (absolutely positioned, a
/// sibling in the DOM, listed before or after it). Where the `<p>`'s own
/// words render 240 over a 248 patch, unseen, every pixel its mask holds may
/// be the badge's: Unknown. Two unrelated texts with one box are each the
/// other's inner text, and both are refused. Where the `<p>`'s own words are
/// white over dark grey too, they show apart from the badge, and both are
/// measured and pass; the badge, the smaller box, is read whatever the
/// heading under it shows.
#[test]
fn a_text_whose_glyphs_show_only_inside_an_unrelated_badge_is_unknown() {
    let (w, h) = (300u32, 100u32);
    for (variant, badge_box) in [
        ("inside", region_at(16, 36, 64, 26)),
        ("overhang", region_at(16, 28, 64, 38)),
        ("identical", region_at(10, 30, 260, 34)),
    ] {
        for (own_words_seen, badge_first) in [(false, false), (false, true), (true, false)] {
            let left = stems_of(2, 20, 40, 6, 14);
            let right = stems_of(2, 170, 40, 6, 14);
            let (a, b) = rgb_frames(
                (w, h),
                |x, _| {
                    if x < 150 || own_words_seen {
                        [40; 3]
                    } else {
                        [248; 3]
                    }
                },
                |x, y, _| {
                    if in_any(&left, x, y) || (own_words_seen && in_any(&right, x, y)) {
                        Some([255; 3])
                    } else if in_any(&right, x, y) {
                        Some([240; 3])
                    } else {
                        None
                    }
                },
            );
            let mut p = white_text("p", region_at(10, 30, 260, 34));
            p.text = Some("Visit our shop today".into());
            let mut badge = white_text("badge", badge_box);
            badge.text = Some("New".into());
            let texts = if badge_first { [badge, p] } else { [p, badge] };
            let mut elements = vec![hero(region_at(0, 0, w, h))];
            elements.extend(texts);
            let c = Case {
                a,
                b,
                snap: ElementSnapshot {
                    elements,
                    ..Default::default()
                },
            };
            let r = c.assert_aa();
            let tag = format!("{variant}/seen={own_words_seen}/first={badge_first}");
            let v = hero_value(&c);
            let measured: Vec<&str> = v
                .value()
                .map(|vs| vs.iter().map(|t| t.element.as_str()).collect())
                .unwrap_or_default();
            if own_words_seen && variant != "identical" {
                assert_eq!(r.outcome, AssertionOutcome::Passed, "{tag}: {:?}", r.detail);
                assert_eq!(measured.len(), 2, "{tag}: {measured:?}");
                continue;
            }
            assert_eq!(
                r.outcome,
                AssertionOutcome::Unknown,
                "{tag}: {:?}",
                r.detail
            );
            let d = r.detail.as_deref().unwrap();
            assert!(
                d.contains("text 'p'") && d.contains("none of its inner texts"),
                "{tag}: {d}"
            );
            assert!(!measured.contains(&"p"), "{tag}: {measured:?}");
            if variant == "identical" {
                assert!(measured.is_empty(), "{tag}: {measured:?}");
            } else {
                assert_eq!(measured, ["badge"], "{tag}");
            }
        }
    }
}

// ------------------------------------------------- review round 8 findings

/// An absolutely positioned child larger than its parent and not enclosed
/// by it — a tooltip outgrowing its `<label>` — whose own words render 240
/// over a 248 patch, unseen, while the label's white glyphs lie partly
/// inside its box. The label's box covers under half of the tooltip's, so
/// the label is an inner text of it: the tooltip's mask is all the label's
/// ink, and it is Unknown. With its words visible it is measured and
/// passes. A padded inline link whose box overhangs its paragraph's, both
/// visible, is measured and passes: the paragraph's box covers most of it.
#[test]
fn a_positioned_child_outgrowing_its_label_is_read_apart_from_the_labels_ink() {
    let (w, h) = (300u32, 100u32);
    for child_seen in [false, true] {
        let label_words: Vec<_> = stems_of(2, 15, 34, 3, 12)
            .into_iter()
            .chain(stems_of(2, 60, 34, 4, 12))
            .collect();
        let tip_words = stems_of(2, 160, 40, 6, 14);
        let (a, b) = rgb_frames(
            (w, h),
            |x, _| {
                if x >= 150 && !child_seen {
                    [248; 3]
                } else {
                    [40; 3]
                }
            },
            |x, y, _| {
                if in_any(&label_words, x, y) {
                    Some([255; 3])
                } else if in_any(&tip_words, x, y) {
                    Some(if child_seen { [255; 3] } else { [240; 3] })
                } else {
                    None
                }
            },
        );
        let mut label = white_text("label", region_at(10, 30, 100, 20));
        label.text = Some("Email Enter your work email".into());
        label.children_ids = vec!["tip".into()];
        let mut tip = white_text("tip", region_at(50, 25, 200, 40));
        tip.text = Some("Enter your work email".into());
        tip.parent_id = Some("label".into());
        let c = Case {
            a,
            b,
            snap: ElementSnapshot {
                elements: vec![hero(region_at(0, 0, w, h)), label, tip],
                ..Default::default()
            },
        };
        let r = c.assert_aa();
        let v = hero_value(&c);
        let measured: Vec<&str> = v
            .value()
            .expect("the label is measured")
            .iter()
            .map(|t| t.element.as_str())
            .collect();
        if child_seen {
            assert_eq!(r.outcome, AssertionOutcome::Passed, "{:?}", r.detail);
            assert_eq!(measured.len(), 2, "{measured:?}");
        } else {
            assert_eq!(r.outcome, AssertionOutcome::Unknown, "{:?}", r.detail);
            let d = r.detail.as_deref().unwrap();
            assert!(
                d.contains("text 'tip'") && d.contains("none of its inner texts"),
                "{d}"
            );
            assert_eq!(measured, ["label"]);
        }
    }

    let p_words: Vec<_> = stems_of(2, 15, 34, 3, 12)
        .into_iter()
        .chain(stems_of(2, 240, 34, 3, 12))
        .collect();
    let link_words = stems_of(2, 60, 34, 16, 12);
    let (a, b) = rgb_frames(
        (w, h),
        |_, _| [40; 3],
        |x, y, _| (in_any(&p_words, x, y) || in_any(&link_words, x, y)).then_some([255; 3]),
    );
    let mut p = white_text("p", region_at(10, 30, 260, 20));
    p.text = Some("Read our terms of service now".into());
    p.children_ids = vec!["a".into()];
    let mut link = white_text("a", region_at(50, 24, 180, 32));
    link.text = Some("our terms of service".into());
    let c = Case {
        a,
        b,
        snap: ElementSnapshot {
            elements: vec![hero(region_at(0, 0, w, h)), p, link],
            ..Default::default()
        },
    };
    let r = c.assert_aa();
    assert_eq!(r.outcome, AssertionOutcome::Passed, "{:?}", r.detail);
    assert_eq!(hero_value(&c).value().unwrap().len(), 2);
}

/// The inner-text rule's two edges, pinned so that a change to them is
/// deliberate. A caption unrelated to the `<p>`, overlapping it from below,
/// whose white glyphs lie in the `<p>`'s box, while the `<p>`'s own words
/// render 240 over a 248 patch, unseen:
///
/// - with the caption's box LARGER than the `<p>`'s, it is not an inner
///   text of the `<p>` (the larger box is assumed laid under the smaller),
///   and the `<p>` is read on the caption's glyphs and passes — the KNOWN
///   LIMIT the module docs name;
/// - with the two boxes of one area, each is the other's inner text: the
///   `<p>` is Unknown, and the caption, whose glyphs show below the `<p>`'s
///   box too, is measured.
///
/// Where the `<p>`'s own words are visible above the caption's box, it is
/// measured either way.
#[test]
fn a_larger_unrelated_text_is_not_refused_on_and_one_of_equal_area_is() {
    let (w, h) = (300u32, 120u32);
    for (variant, cap_box) in [
        ("larger", region_at(0, 40, 300, 50)),
        ("equal_area", region_at(10, 40, 260, 34)),
    ] {
        for p_seen in [false, true] {
            let cap_in = stems_of(2, 20, 44, 6, 14);
            let cap_out = stems_of(2, 20, 66, 6, 6);
            let p_words = stems_of(2, 170, 32, 6, 6);
            let (a, b) = rgb_frames(
                (w, h),
                |x, y| {
                    if !p_seen && x >= 150 && y < 64 {
                        [248; 3]
                    } else {
                        [40; 3]
                    }
                },
                |x, y, _| {
                    if in_any(&cap_in, x, y) || in_any(&cap_out, x, y) {
                        Some([255; 3])
                    } else if in_any(&p_words, x, y) {
                        Some(if p_seen { [255; 3] } else { [240; 3] })
                    } else {
                        None
                    }
                },
            );
            let mut p = white_text("p", region_at(10, 30, 260, 34));
            p.text = Some("Visit our shop today".into());
            let mut cap = white_text("caption", cap_box);
            cap.text = Some("A caption".into());
            let c = Case {
                a,
                b,
                snap: ElementSnapshot {
                    elements: vec![hero(region_at(0, 0, w, h)), p, cap],
                    ..Default::default()
                },
            };
            let r = c.assert_aa();
            let tag = format!("{variant}/seen={p_seen}");
            let v = hero_value(&c);
            let values = v.value().expect("the caption is measured");
            assert!(
                values.iter().any(|t| t.element == "caption"),
                "{tag}: {values:?}"
            );
            let p = values.iter().find(|t| t.element == "p");
            if p_seen || variant == "larger" {
                assert_eq!(r.outcome, AssertionOutcome::Passed, "{tag}: {:?}", r.detail);
                let p = p.expect("p is measured");
                if !p_seen {
                    assert_eq!(p.shared_px, p.sampled_px, "{tag}: {p:?}");
                }
            } else {
                assert_eq!(
                    r.outcome,
                    AssertionOutcome::Unknown,
                    "{tag}: {:?}",
                    r.detail
                );
                assert!(p.is_none(), "{tag}: {values:?}");
            }
        }
    }
}

// ------------------------------------------------- review round 9 findings

/// A 300 x 100 hero under a `<label>` (box `label_box`, white glyphs at
/// x 115-136 and 155-166) and its positioned tooltip (box 150,20 100x40),
/// whose own words render 240 over a 248 patch from x 212, unseen.
fn tooltip_over_its_label(label_box: Region) -> Case {
    let (w, h) = (300u32, 100u32);
    let label_words: Vec<_> = stems_of(2, 115, 30, 3, 14)
        .into_iter()
        .chain(stems_of(2, 155, 30, 2, 14))
        .collect();
    let tip_words = stems_of(2, 220, 32, 3, 14);
    let (a, b) = rgb_frames(
        (w, h),
        |x, _| if x >= 212 { [248; 3] } else { [40; 3] },
        |x, y, _| {
            if in_any(&label_words, x, y) {
                Some([255; 3])
            } else if in_any(&tip_words, x, y) {
                Some([240; 3])
            } else {
                None
            }
        },
    );
    let mut label = white_text("label", label_box);
    label.text = Some("Email Enter your work email".into());
    label.children_ids = vec!["tip".into()];
    let mut tip = white_text("tip", region_at(150, 20, 100, 40));
    tip.text = Some("Enter your work email".into());
    tip.parent_id = Some("label".into());
    Case {
        a,
        b,
        snap: ElementSnapshot {
            elements: vec![hero(region_at(0, 0, w, h)), label, tip],
            ..Default::default()
        },
    }
}

/// The ancestor half-cover edge, pinned so that a change to it is
/// deliberate. A positioned tooltip whose own words render 240 over a 248
/// patch, unseen, while its `<label>`'s white glyphs lie partly inside its
/// box. With the label's box of the tooltip's area:
///
/// - covering just under half of the tooltip's box, the label is an inner
///   text of it, and the tooltip is Unknown;
/// - covering exactly half, or more, it is not: the tooltip is read on the
///   label's ink and passes — the KNOWN LIMIT the module docs name.
///
/// A label LARGER than the tooltip, covering far under half of it, is not
/// an inner text either (the larger box is assumed laid under the smaller):
/// the same KNOWN LIMIT. The label is measured in every case.
#[test]
fn an_ancestor_covering_at_least_half_of_a_text_is_not_refused_on() {
    for (variant, label_box, refused) in [
        ("under_half", region_at(99, 20, 100, 40), true),
        ("half", region_at(100, 20, 100, 40), false),
        ("over_half", region_at(110, 20, 100, 40), false),
        ("larger_offset", region_at(0, 20, 170, 40), false),
    ] {
        let c = tooltip_over_its_label(label_box);
        let r = c.assert_aa();
        let v = hero_value(&c);
        let values = v.value().expect("the label is measured");
        assert!(
            values.iter().any(|t| t.element == "label"),
            "{variant}: {values:?}"
        );
        let tip = values.iter().find(|t| t.element == "tip");
        if refused {
            assert_eq!(
                r.outcome,
                AssertionOutcome::Unknown,
                "{variant}: {:?}",
                r.detail
            );
            let d = r.detail.as_deref().unwrap();
            assert!(
                d.contains("text 'tip'") && d.contains("none of its inner texts"),
                "{variant}: {d}"
            );
            assert!(tip.is_none(), "{variant}: {values:?}");
        } else {
            assert_eq!(
                r.outcome,
                AssertionOutcome::Passed,
                "{variant}: {:?}",
                r.detail
            );
            let tip = tip.expect("the tooltip is measured");
            assert_eq!(tip.shared_px, tip.sampled_px, "{variant}: {tip:?}");
        }
    }
}

/// The half-cover test compares box areas near `u32::MAX` square without
/// overflowing: a text and its ancestor sharing one such box (an area over
/// half of `u64::MAX`) are paired, the ancestor not an inner text of the
/// child (it covers all of it) and the child an inner text of the ancestor
/// (it descends from it).
#[test]
fn boxes_near_u32_max_square_do_not_overflow_the_half_cover_test() {
    let huge = region_at(-2_000_000_000, -2_000_000_000, 4_000_000_000, 4_000_000_000);
    assert!(huge.w as u64 * huge.h as u64 > u64::MAX / 2);
    let mut p = white_text("p", huge);
    p.text = Some("Read our terms".into());
    p.children_ids = vec!["a".into()];
    let mut link = white_text("a", huge);
    link.text = Some("terms".into());
    link.parent_id = Some("p".into());
    let image = hero(region_at(0, 0, 300, 100));
    let snap = ElementSnapshot {
        elements: vec![image.clone(), p, link],
        ..Default::default()
    };
    let index = text_contrast::TextIndex::of(&snap);
    let over = text_contrast::texts_over(&snap, &image, &index);
    let flags = |id: &str| {
        let pair = over
            .pairs
            .iter()
            .find(|pair| pair.text.id == id)
            .unwrap_or_else(|| panic!("'{id}' is paired"));
        pair.others
            .iter()
            .map(|o| o.id.as_str())
            .zip(pair.inner.iter().copied())
            .collect::<Vec<_>>()
    };
    assert_eq!(flags("a"), [("p", false)]);
    assert_eq!(flags("p"), [("a", true)]);
}

/// A pair built by hand with fewer `inner` flags than `others` reads each
/// missing flag as inner, the cautious answer. The tooltip under a label
/// covering just under half of it is refused with the flag `texts_over`
/// computes and with no flag at all, and is read on the label's ink only
/// when the flag says the label is not an inner text.
#[test]
fn a_missing_inner_flag_reads_as_inner() {
    let c = tooltip_over_its_label(region_at(99, 20, 100, 40));
    let index = text_contrast::TextIndex::of(&c.snap);
    let image = c.snap.elements.iter().find(|e| e.id == "hero").unwrap();
    let over = text_contrast::texts_over(&c.snap, image, &index);
    let pair = over.pairs.iter().find(|p| p.text.id == "tip").unwrap();
    assert_eq!(pair.inner, [true]);
    text_contrast::pair_frames(&c.a, &c.b).unwrap();
    let reg = text_contrast::registered(&c.a, &c.b, image, &index).unwrap();
    let read = |inner: Vec<bool>| {
        let pair = text_contrast::TextOverImage {
            inner,
            ..pair.clone()
        };
        text_contrast::measure(
            &c.a,
            &c.b,
            &pair,
            text_contrast::DEFAULT_PERCENTILE,
            &reg,
            false,
        )
    };
    for inner in [vec![true], vec![]] {
        let (_, d) = read(inner.clone()).expect_err("the tooltip is refused");
        assert!(d.contains("none of its inner texts"), "{inner:?}: {d}");
    }
    let v = read(vec![false])
        .expect("measured")
        .expect("over the image");
    assert_eq!(v.shared_px, v.sampled_px, "{v:?}");
}
