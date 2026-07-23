//! Color analyzer — dominant-color extraction (sampling, not k-means in
//! Phase 6 first pass) + contrast distribution. Pure pixel math on the
//! frame buffer, no external CV deps.

use std::collections::BTreeMap;

use super::{Finding, Severity};
use crate::element_snapshot::{ElementSnapshot, Rgb};
use crate::frame::{Frame, Region};

pub fn run(frame: &Frame, snapshot: &ElementSnapshot) -> Vec<Finding> {
    let mut findings = Vec::new();

    // Only DECLARED-color elements feed the density roll-up. Pixel-sampled
    // ratios are approximate (see below) so they must never inflate the
    // density warning or emit Warning/Critical findings.
    let mut declared_low_contrast = 0usize;
    let mut declared_checked = 0usize;
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
        let (fg, bg, declared) = match (el.fg_color, el.bg_color) {
            (Some(f), Some(b)) => (f, b, true),
            // At least one color undeclared: fall back to sampling the frame
            // under the element's bbox. Requires geometry — bbox-less
            // elements can't be sampled, so they're skipped (no finding).
            _ => match el.bbox.and_then(|bbox| sample_dominant_two(frame, bbox)) {
                Some((f, b)) => (f, b, false),
                None => continue,
            },
        };
        let ratio = wcag_contrast(fg, bg);

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
                .with_elements(vec![el.id.clone()]),
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

    findings
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
pub fn sample_dominant_two(frame: &Frame, region: Region) -> Option<(Rgb, Rgb)> {
    if region.w == 0 || region.h == 0 {
        return None;
    }
    // The region's origin is signed and may sit outside the frame; narrow to
    // in-frame buffer indices before sampling. `None` = nothing to sample.
    let (rx, ry, rw, rh) = region.clamp_to_frame(frame.width, frame.height)?;

    let mut hist: BTreeMap<u16, u32> = BTreeMap::new();
    for y in ry..(ry + rh) {
        for x in rx..(rx + rw) {
            let p = frame.buffer.get_pixel(x, y).0;
            let key = quantize_rgb(p[0], p[1], p[2]);
            *hist.entry(key).or_default() += 1;
        }
    }
    if hist.is_empty() {
        return None;
    }
    let mut entries: Vec<_> = hist.into_iter().collect();
    entries.sort_by_key(|e| std::cmp::Reverse(e.1));
    let primary = dequantize_rgb(entries[0].0);
    let secondary = if entries.len() > 1 {
        dequantize_rgb(entries[1].0)
    } else {
        // Single-color region: use its complement as the "other" color so
        // contrast math degenerates gracefully (ratio ≈ 1.0).
        primary
    };
    Some((primary, secondary))
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
            parent_id: None,
            children_ids: vec![],
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
        };
        let findings = run(&f, &snap);
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
        };
        let findings = run(&f, &snap);
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
        };
        let r = wcag_contrast(Rgb::new(0x80, 0x80, 0x80), Rgb::new(255, 255, 255));
        assert!(
            (3.0..4.5).contains(&r),
            "fixture ratio {r} not in [3.0,4.5)"
        );
        let findings = run(&f, &snap);
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
        let snap = ElementSnapshot { elements };
        let findings = run(&f, &snap);
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
