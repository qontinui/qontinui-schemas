//! Color analyzer — dominant-color extraction (sampling, not k-means in
//! Phase 6 first pass) + contrast distribution. Pure pixel math on the
//! frame buffer, no external CV deps.

use std::collections::BTreeMap;

use super::{AnalyzerResult, Finding, Severity};
use crate::coverage::SnapshotCoverage;
use crate::element_snapshot::{ElementSnapshot, Rgb};
use crate::frame::{Frame, Region};

pub fn run(frame: &Frame, snapshot: &ElementSnapshot) -> AnalyzerResult {
    let coverage = SnapshotCoverage::of(snapshot);
    // Text is this analyzer's entire candidate set — the loop below
    // `continue`s on every element whose `text` is `None`, so with none at
    // all it measures no contrast anywhere and returns the same empty list
    // a page with perfect contrast returns.
    if coverage.with_text == 0 {
        return AnalyzerResult::blocked(
            format!(
                "no element carries text (0/{}), so no foreground/background pair was \
                 sampled and no contrast ratio was computed.",
                coverage.elements
            ),
            Some(coverage),
            Vec::new(),
        );
    }
    let mut findings = Vec::new();

    // Only DECLARED-color elements feed the density roll-up. Pixel-sampled
    // ratios are approximate (see below) so they must never inflate the
    // density warning or emit Warning/Critical findings.
    let mut declared_low_contrast = 0usize;
    let mut declared_checked = 0usize;
    // Every element for which a contrast ratio was actually computed, by
    // either regime. Distinct from `declared_checked`, which gates the
    // density roll-up: this one answers "did this analyzer measure anything
    // at all?", and zero is the vacuous-pass condition.
    let mut ratios_computed = 0usize;
    for el in &snapshot.elements {
        if el.text.is_none() {
            continue;
        }
        // Prefer the snapshot's declared colors if present (cheaper, more
        // accurate than re-sampling pixels under text antialiasing).
        // Otherwise approximate from the rendered pixels.
        //
        // `declared` distinguishes the two contrast regimes:
        //   - declared == true  → both fg_color AND bg_color came from the
        //     snapshot's computed style. Trustworthy → full WCAG AA gating
        //     (Critical < 3.0, Warning < 4.5) and counts toward density.
        //   - declared == false → at least one color was missing, so we
        //     sampled the rendered pixels under the bbox via a two-mode
        //     histogram. For sparse-text elements both modes collapse to
        //     ~background (ratio ≈ 1.0), producing false-positive
        //     "low_contrast" findings. We therefore cap these at Info and
        //     exclude them from the density roll-up. This covers the
        //     fully-undeclared arm AND the partial-declaration arms
        //     (Some/None, None/Some): any element whose contrast required
        //     pixel sampling is informational only.
        // `sampled_confidence` is `None` on the declared arm and `Some` on
        // the sampled one — the two contrast regimes ARE the estimate /
        // deduction distinction [`Finding::confidence`] exists to state, so
        // it is carried straight through onto the finding rather than
        // recomputed or guessed at.
        //
        // `declared` is DERIVED from it rather than tracked alongside it,
        // the same way `AnalyzerResult::conclusive` is derived from its
        // verdict: two independently-maintained encodings of one fact can
        // drift when a third regime is added, and the drift would be a
        // finding silently losing its confidence with nothing failing.
        let (fg, bg, sampled_confidence) = match (el.fg_color, el.bg_color) {
            (Some(f), Some(b)) => (f, b, None),
            // At least one color undeclared: fall back to sampling the frame
            // under the element's bbox. Requires geometry — bbox-less
            // elements can't be sampled, so they're skipped (no finding).
            _ => match el
                .bbox
                .and_then(|bbox| sample_dominant_two_scored(frame, bbox))
            {
                Some((f, b, share)) => (f, b, Some(share)),
                None => continue,
            },
        };
        let declared = sampled_confidence.is_none();
        let ratio = wcag_contrast(fg, bg);
        ratios_computed += 1;

        if declared {
            declared_checked += 1;
            if ratio < 4.5 {
                declared_low_contrast += 1;
                findings.push(
                    Finding::new(
                        "low_contrast",
                        if ratio < 3.0 {
                            Severity::Critical
                        } else {
                            Severity::Warning
                        },
                        format!(
                            "element {} text contrast ratio {:.2}:1 (WCAG AA requires ≥4.5)",
                            el.id, ratio
                        ),
                    )
                    .with_elements(vec![el.id.clone()]),
                );
            }
        } else if ratio < 4.5 {
            // Pixel-sampled, no declared colors: informational only. Never
            // Warning/Critical and never counted toward density — the
            // two-mode histogram is unreliable for sparse text.
            //
            // This is the crate's one ESTIMATED analyzer observation, so it
            // is the one that carries a confidence. Everything else here is
            // exact arithmetic over declared values and states so by
            // leaving `confidence` at `None`.
            let share = sampled_confidence
                .expect("`declared` is derived from this Option, so the sampled arm always scores");
            findings.push(
                Finding::new(
                    "low_contrast",
                    Severity::Info,
                    format!(
                        "element {} text contrast sampled (no declared colors): \
                         ratio≈{:.2}:1 — informational, WCAG AA wants ≥4.5",
                        el.id, ratio
                    ),
                )
                .with_elements(vec![el.id.clone()])
                .with_confidence(share),
            );
        }
    }

    // Density warning is gated on declared-color elements only (sampled/Info
    // findings deliberately do not contribute — see the `declared` comment).
    if declared_checked > 0 && declared_low_contrast as f64 / declared_checked as f64 >= 0.25 {
        findings.push(Finding::new(
            "contrast_density",
            Severity::Warning,
            format!(
                "{}/{} text elements ({:.0}%) below WCAG AA contrast",
                declared_low_contrast,
                declared_checked,
                100.0 * declared_low_contrast as f64 / declared_checked as f64
            ),
        ));
    }

    verdict(coverage, ratios_computed, findings)
}

/// Classify a completed color run.
///
/// # Blocked
///
/// Two arms, one condition: **no contrast ratio was computed for anything.**
/// `with_text == 0` is the coverage-level form, handled at the top of
/// [`run`] (the loop `continue`s on every element whose `text` is `None`, so
/// the candidate set is empty); `ratios_computed == 0` is the same fact one
/// level in — text elements existed, but not one carried declared colors and
/// not one could be sampled from the frame, so nothing was measured. Both
/// produce the empty finding list a page with flawless contrast produces.
///
/// The second arm is deliberately not `Degraded`: a run that measured
/// nothing has established nothing about the page, and calling that green
/// would leave open exactly the hole the first arm closes.
///
/// # Degraded
///
/// Some but not all text elements yielded a ratio. The findings stand for
/// the ones that did; the rest were not checked, and a reader deserves to be
/// told the sample was partial rather than infer full coverage from silence.
fn verdict(
    coverage: SnapshotCoverage,
    ratios_computed: usize,
    findings: Vec<Finding>,
) -> AnalyzerResult {
    if ratios_computed == 0 {
        return AnalyzerResult::blocked(
            format!(
                "no contrast ratio could be computed for any of the {} text element(s): \
                 none carried declared foreground/background colors and none could be \
                 sampled from the frame.",
                coverage.with_text
            ),
            Some(coverage),
            findings,
        );
    }
    if ratios_computed < coverage.with_text {
        return AnalyzerResult::degraded(
            format!(
                "contrast measured for {}/{} text element(s); the rest carried neither \
                 declared colors nor a sampleable bbox and were not checked.",
                ratios_computed, coverage.with_text
            ),
            Some(coverage),
            findings,
        );
    }
    AnalyzerResult::checked(Some(coverage), findings)
}

/// WCAG 2.x relative-luminance contrast ratio. Always returns a value in
/// `[1.0, 21.0]`. Order of arguments doesn't matter.
pub fn wcag_contrast(a: Rgb, b: Rgb) -> f64 {
    let la = a.relative_luminance();
    let lb = b.relative_luminance();
    let (lighter, darker) = if la >= lb { (la, lb) } else { (lb, la) };
    (lighter + 0.05) / (darker + 0.05)
}

/// Crude two-color summary of a region: histogram-mode + its
/// complementary "second-mode" color. Suitable for inferring foreground vs
/// background of a text element when the snapshot didn't supply computed
/// styles. Quantizes RGB to a 5-bit-per-channel cube (32×32×32 = 32k bins)
/// so background gradients still collapse to one mode in practice.
///
/// Thin wrapper over [`sample_dominant_two_scored`] that discards the score.
/// Prefer the scored form where the caller is producing an observation a
/// reader will act on — the score is what lets that reader tell an estimate
/// from a deduction.
pub fn sample_dominant_two(frame: &Frame, region: Region) -> Option<(Rgb, Rgb)> {
    sample_dominant_two_scored(frame, region).map(|(fg, bg, _)| (fg, bg))
}

/// [`sample_dominant_two`], plus **the share of the region's sampled pixels
/// falling in the two dominant quantization bins**.
///
/// Bins, not colours: the returned `Rgb`s are those bins' dequantized
/// representatives, so a counted pixel matches a returned colour to within
/// one 5-bit bin (up to ~7 per channel), not exactly. Saying "the two
/// returned colors account for" would overclaim by exactly that margin.
///
/// The value is in `0.0..=1.0`, but the reachable FLOOR is not 0.0: with
/// 32 768 bins the two dominant ones always hold at least `2/32768` of a
/// fully-scattered region, so calibrate a "low" threshold against real
/// regions rather than against zero.
///
/// That share is the honest confidence for anything derived from this pair,
/// and it is a MEASUREMENT rather than a judgement. Read it as exactly what
/// it says and nothing more:
///
/// - It answers *"how much of this region does a two-color model describe?"*
///   A flat label on a flat background scores near `1.0`; a region full of
///   gradient, imagery or antialiasing spreads across many quantization bins
///   and scores low, which is precisely when the sampled pair is least
///   trustworthy as a foreground/background reading.
/// - It does **not** claim the pair was correctly identified AS foreground
///   and background. A sparse-text element whose glyphs are a small minority
///   of pixels can score high while both modes collapse to background — the
///   known failure mode this analyzer caps at `Severity::Info` for.
///
/// A region with a single occupied bin has no second color: the pair
/// degenerates (both entries are the mode) and the score is that one bin's
/// share, i.e. `1.0`. The model describes the region perfectly; there is
/// simply no foreground in it.
pub fn sample_dominant_two_scored(frame: &Frame, region: Region) -> Option<(Rgb, Rgb, f64)> {
    if region.w == 0 || region.h == 0 {
        return None;
    }
    // The region's origin is signed and may sit outside the frame; narrow to
    // in-frame buffer indices before sampling. `None` = nothing to sample.
    let (rx, ry, rw, rh) = region.clamp_to_frame(frame.width, frame.height)?;

    let mut hist: BTreeMap<u16, u32> = BTreeMap::new();
    let mut sampled: u64 = 0;
    for y in ry..(ry + rh) {
        for x in rx..(rx + rw) {
            let p = frame.buffer.get_pixel(x, y).0;
            let key = quantize_rgb(p[0], p[1], p[2]);
            *hist.entry(key).or_default() += 1;
            sampled += 1;
        }
    }
    if hist.is_empty() || sampled == 0 {
        return None;
    }
    let mut entries: Vec<_> = hist.into_iter().collect();
    entries.sort_by_key(|e| std::cmp::Reverse(e.1));
    let primary = dequantize_rgb(entries[0].0);
    let (secondary, second_count) = if entries.len() > 1 {
        (dequantize_rgb(entries[1].0), u64::from(entries[1].1))
    } else {
        // Single-color region: reuse the mode as the "other" color so the
        // contrast math degenerates gracefully (ratio ≈ 1.0). The comment
        // here used to say "its complement", which the code has never done
        // — a complement would give a HIGH ratio and turn a blank region
        // into a clean bill of health.
        //
        // No second color was OBSERVED, so it contributes no pixels to the
        // score: the share is the mode's alone.
        (primary, 0)
    };
    let share = (u64::from(entries[0].1) + second_count) as f64 / sampled as f64;
    // The two bins are disjoint subsets of the sampled pixels, so this
    // cannot exceed 1.0 — and a `clamp` alone would LAUNDER the one likely
    // typo here (double-counting the mode in the degenerate arm) into a
    // passing 1.0. Assert first, then clamp for float slop.
    debug_assert!(
        share <= 1.0,
        "top-two bins ({share}) exceeded the sampled pixel count — a bin was counted twice"
    );
    Some((primary, secondary, share.clamp(0.0, 1.0)))
}

fn quantize_rgb(r: u8, g: u8, b: u8) -> u16 {
    ((r as u16 >> 3) << 10) | ((g as u16 >> 3) << 5) | (b as u16 >> 3)
}

fn dequantize_rgb(q: u16) -> Rgb {
    let r = ((q >> 10) & 0x1f) as u8;
    let g = ((q >> 5) & 0x1f) as u8;
    let b = (q & 0x1f) as u8;
    // Shift back to 8-bit by replicating the high bits — standard 5→8 expansion.
    Rgb::new(
        (r << 3) | (r >> 2),
        (g << 3) | (g >> 2),
        (b << 3) | (b >> 2),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::element_snapshot::Element;
    use crate::frame::{FrameSource, FrameSourceKind};

    fn solid_frame(w: u32, h: u32, color: [u8; 4]) -> Frame {
        let buf = image::RgbaImage::from_pixel(w, h, image::Rgba(color));
        Frame::from_rgba(
            buf,
            FrameSource {
                kind: FrameSourceKind::Synthetic,
                scale_factor: 1.0,
                captured_at: chrono::Utc::now(),
                capture_backend: None,
            },
        )
    }

    #[test]
    fn wcag_white_on_black_is_21() {
        let r = wcag_contrast(Rgb::new(255, 255, 255), Rgb::new(0, 0, 0));
        assert!((r - 21.0).abs() < 0.01);
    }

    #[test]
    fn wcag_low_contrast() {
        // light-gray on white: ~1.6:1
        let r = wcag_contrast(Rgb::new(200, 200, 200), Rgb::new(255, 255, 255));
        assert!(r < 2.0);
    }

    /// This is the SECOND-TERM guard: it is the only test that reaches the
    /// `entries.len() > 1` arm, so dropping `second_count` from the sum
    /// reddens here (0.50 against an expected 0.80) and nowhere else. The
    /// double-count mutation is caught elsewhere, by the `debug_assert!` on
    /// `share` reached through the single-bin test below — this fixture has
    /// six occupied bins and never enters the degenerate arm at all.
    ///
    /// It also pins WHICH pair comes back, so a mutation that picks
    /// `entries[2]` as the runner-up while still summing `entries[1]`'s
    /// count cannot slip through on the share alone.
    #[test]
    fn scored_share_counts_only_the_two_dominant_bins() {
        // 10x10 = 100 px. 50 of A, 30 of B, then 5 px each of four further
        // colours, all far enough apart to land in distinct 5-bit bins.
        // Every pixel is overwritten below, so the fill colour is arbitrary.
        let mut img = image::RgbaImage::from_pixel(10, 10, image::Rgba([0, 0, 0, 0xff]));
        let others = [
            [0x40, 0x00, 0x00, 0xff],
            [0x00, 0x40, 0x00, 0xff],
            [0x00, 0x00, 0x40, 0xff],
            [0x80, 0x80, 0x00, 0xff],
        ];
        for i in 0u32..100 {
            let (x, y) = (i % 10, i / 10);
            let px = if i < 50 {
                [0xF0, 0xF0, 0xF0, 0xff] // A: 50
            } else if i < 80 {
                [0x10, 0x10, 0x10, 0xff] // B: 30
            } else {
                others[((i - 80) / 5) as usize] // 4 x 5
            };
            img.put_pixel(x, y, image::Rgba(px));
        }
        let f = Frame::from_rgba(img, FrameSource::synthetic_now());
        let (p, s, share) = sample_dominant_two_scored(
            &f,
            Region {
                x: 0,
                y: 0,
                w: 10,
                h: 10,
            },
        )
        .unwrap();
        assert!(
            (share - 0.80).abs() < 1e-9,
            "top two bins hold 50+30 of 100 px, expected 0.80, got {share}"
        );
        // Round-tripped through the same quantization rather than
        // hardcoded, so this pins the SELECTION without also pinning the
        // 5->8 bit expansion.
        assert_eq!(
            p,
            dequantize_rgb(quantize_rgb(0xF0, 0xF0, 0xF0)),
            "A (50 px) is the mode"
        );
        assert_eq!(
            s,
            dequantize_rgb(quantize_rgb(0x10, 0x10, 0x10)),
            "B (30 px) is the runner-up"
        );
    }

    /// The degenerate arm: one occupied bin means no SECOND colour was
    /// observed, so it must contribute zero pixels rather than the mode's
    /// count a second time. Asserted alongside the bin count so "share is
    /// 1.0" cannot pass for the wrong reason.
    #[test]
    fn scored_share_on_a_single_bin_region_counts_the_mode_once() {
        let f = solid_frame(20, 20, [0x80, 0x80, 0x80, 0xff]);
        let r = Region {
            x: 0,
            y: 0,
            w: 20,
            h: 20,
        };
        let (p, s, share) = sample_dominant_two_scored(&f, r).unwrap();
        assert_eq!(p, s, "a single-bin region has no distinct second colour");
        assert!(
            (share - 1.0).abs() < 1e-9,
            "one bin holding every pixel is a share of 1.0, got {share}"
        );
    }

    #[test]
    fn sample_solid_region_returns_that_color() {
        let f = solid_frame(20, 20, [0x80, 0x80, 0x80, 0xff]);
        let (p, _) = sample_dominant_two(
            &f,
            Region {
                x: 0,
                y: 0,
                w: 20,
                h: 20,
            },
        )
        .unwrap();
        assert!(p.r >= 0x78 && p.r <= 0x88);
    }

    #[test]
    fn sample_wholly_offscreen_region_returns_none() {
        // A negative-origin region no longer aliases onto (0,0) — it samples
        // nothing, because none of it is inside the frame.
        let f = solid_frame(10, 10, [0xff, 0, 0, 0xff]);
        assert!(sample_dominant_two(
            &f,
            Region {
                x: -100,
                y: -100,
                w: 20,
                h: 20,
            },
        )
        .is_none());
    }

    #[test]
    fn sample_partly_offscreen_region_samples_the_visible_part() {
        let f = solid_frame(10, 10, [0x40, 0x80, 0xc0, 0xff]);
        let (p, _) = sample_dominant_two(
            &f,
            Region {
                x: -5,
                y: -5,
                w: 10,
                h: 10,
            },
        )
        .unwrap();
        assert!(p.r >= 0x38 && p.r <= 0x48);
    }

    #[test]
    fn sample_zero_area_returns_none() {
        let f = solid_frame(10, 10, [0xff, 0, 0, 0xff]);
        assert!(sample_dominant_two(
            &f,
            Region {
                x: 0,
                y: 0,
                w: 0,
                h: 0,
            },
        )
        .is_none());
    }

    /// Build a text element with optional declared colors and a 10x10 bbox.
    fn text_el(id: &str, fg: Option<Rgb>, bg: Option<Rgb>) -> Element {
        Element {
            id: id.to_string(),
            bbox: Some(Region {
                x: 0,
                y: 0,
                w: 10,
                h: 10,
            }),
            text: Some("hello".to_string()),
            role: Some("link".to_string()),
            interactable: true,
            fg_color: fg,
            bg_color: bg,
            font_size_px: None,
            font_family: None,
            line_height_px: None,
            ..Default::default()
        }
    }

    #[test]
    fn sampled_low_contrast_is_info_not_critical() {
        // Regression for the false-positive class: an element with NO declared
        // colors over a solid frame samples ~1.0:1 (both histogram modes are
        // the same background). It must surface at most Info — never
        // Warning/Critical — and must not feed the density roll-up.
        let f = solid_frame(10, 10, [0x20, 0x20, 0x20, 0xff]);
        let snap = ElementSnapshot {
            elements: vec![text_el("footer-link", None, None)],
            ..Default::default()
        };
        let findings = run(&f, &snap).findings;
        let lc: Vec<_> = findings
            .iter()
            .filter(|x| x.kind == "low_contrast")
            .collect();
        assert_eq!(lc.len(), 1, "expected exactly one sampled finding");
        assert_eq!(lc[0].severity, Severity::Info);
        // Sampled findings never count toward the density warning.
        assert!(!findings.iter().any(|x| x.kind == "contrast_density"));
    }

    #[test]
    fn declared_below_3_is_critical() {
        // Declared colors with ratio < 3.0 stay Critical (path untouched).
        let f = solid_frame(10, 10, [0xff, 0xff, 0xff, 0xff]);
        // light-gray on white ~1.6:1
        let snap = ElementSnapshot {
            elements: vec![text_el(
                "btn",
                Some(Rgb::new(200, 200, 200)),
                Some(Rgb::new(255, 255, 255)),
            )],
            ..Default::default()
        };
        let findings = run(&f, &snap).findings;
        let lc = findings
            .iter()
            .find(|x| x.kind == "low_contrast")
            .expect("declared low_contrast finding");
        assert_eq!(lc.severity, Severity::Critical);
    }

    #[test]
    fn declared_below_4_5_is_warning() {
        // Declared colors with 3.0 <= ratio < 4.5 stay Warning.
        // mid-gray (#808080) on white is ~3.95:1.
        let f = solid_frame(10, 10, [0xff, 0xff, 0xff, 0xff]);
        let snap = ElementSnapshot {
            elements: vec![text_el(
                "btn",
                Some(Rgb::new(0x80, 0x80, 0x80)),
                Some(Rgb::new(255, 255, 255)),
            )],
            ..Default::default()
        };
        let r = wcag_contrast(Rgb::new(0x80, 0x80, 0x80), Rgb::new(255, 255, 255));
        assert!(
            (3.0..4.5).contains(&r),
            "fixture ratio {r} not in [3.0,4.5)"
        );
        let findings = run(&f, &snap).findings;
        let lc = findings
            .iter()
            .find(|x| x.kind == "low_contrast")
            .expect("declared low_contrast finding");
        assert_eq!(lc.severity, Severity::Warning);
    }

    #[test]
    fn density_warning_unaffected_by_sampled_info() {
        // One declared good-contrast element plus many sampled (undeclared)
        // low-contrast elements: density must NOT fire, because sampled/Info
        // elements don't count toward the roll-up.
        let f = solid_frame(10, 10, [0x20, 0x20, 0x20, 0xff]);
        let mut elements = vec![text_el(
            "good",
            Some(Rgb::new(255, 255, 255)),
            Some(Rgb::new(0, 0, 0)),
        )];
        for i in 0..5 {
            elements.push(text_el(&format!("sampled-{i}"), None, None));
        }
        let snap = ElementSnapshot {
            elements,
            ..Default::default()
        };
        let findings = run(&f, &snap).findings;
        // 5 sampled Info findings, 1 declared (good contrast, no finding).
        assert!(!findings.iter().any(|x| x.kind == "contrast_density"));
        assert_eq!(
            findings
                .iter()
                .filter(|x| x.kind == "low_contrast" && x.severity == Severity::Info)
                .count(),
            5
        );
    }
}
