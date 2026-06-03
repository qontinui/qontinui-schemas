//! Color analyzer — dominant-color extraction (sampling, not k-means in
//! Phase 6 first pass) + contrast distribution. Pure pixel math on the
//! frame buffer, no external CV deps.

use std::collections::BTreeMap;

use super::{Finding, Severity};
use crate::element_snapshot::{ElementSnapshot, Rgb};
use crate::frame::{Frame, Region};

pub fn run(frame: &Frame, snapshot: &ElementSnapshot) -> Vec<Finding> {
    let mut findings = Vec::new();

    let mut low_contrast = 0usize;
    let mut checked = 0usize;
    for el in &snapshot.elements {
        if el.text.is_none() {
            continue;
        }
        // Prefer the snapshot's declared colors if present (cheaper, more
        // accurate than re-sampling pixels under text antialiasing).
        // Otherwise approximate from the rendered pixels.
        let (fg, bg) = match (el.fg_color, el.bg_color) {
            (Some(f), Some(b)) => (f, b),
            // No declared colors: fall back to sampling the frame under the
            // element's bbox. Requires geometry — bbox-less elements can't be
            // sampled, so they're skipped (no contrast finding).
            _ => match el.bbox.and_then(|bbox| sample_dominant_two(frame, bbox)) {
                Some(pair) => pair,
                None => continue,
            },
        };
        checked += 1;
        let ratio = wcag_contrast(fg, bg);
        if ratio < 4.5 {
            low_contrast += 1;
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
    }

    if checked > 0 && low_contrast as f64 / checked as f64 >= 0.25 {
        findings.push(Finding::new(
            "contrast_density",
            Severity::Warning,
            format!(
                "{}/{} text elements ({:.0}%) below WCAG AA contrast",
                low_contrast,
                checked,
                100.0 * low_contrast as f64 / checked as f64
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
    let region = clamp_region(region, frame.width, frame.height)?;

    let mut hist: BTreeMap<u16, u32> = BTreeMap::new();
    for y in region.y..(region.y + region.h) {
        for x in region.x..(region.x + region.w) {
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

fn clamp_region(r: Region, fw: u32, fh: u32) -> Option<Region> {
    if r.x >= fw || r.y >= fh {
        return None;
    }
    let x = r.x;
    let y = r.y;
    let w = r.w.min(fw - x);
    let h = r.h.min(fh - y);
    if w == 0 || h == 0 {
        None
    } else {
        Some(Region { x, y, w, h })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::frame::{FrameSource, FrameSourceKind};

    fn solid_frame(w: u32, h: u32, color: [u8; 4]) -> Frame {
        let buf = image::RgbaImage::from_pixel(w, h, image::Rgba(color));
        Frame::from_rgba(
            buf,
            FrameSource {
                kind: FrameSourceKind::Synthetic,
                scale_factor: 1.0,
                captured_at: chrono::Utc::now(),
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
}
