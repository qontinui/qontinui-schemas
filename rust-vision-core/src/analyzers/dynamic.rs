//! Dynamic analyzer — compares two frames and flags large pixel deltas.
//! Pixel-level only; doesn't try to attribute changes to specific elements
//! (that's the `elements` analyzer's job once it gets baseline support).

use super::{Finding, Severity};
use crate::frame::Frame;

pub fn run(prior: &Frame, current: &Frame) -> Vec<Finding> {
    let mut findings = Vec::new();

    if prior.width != current.width || prior.height != current.height {
        findings.push(Finding::new(
            "frame_dimension_change",
            Severity::Warning,
            format!(
                "frame dimensions changed: {}x{} → {}x{}",
                prior.width, prior.height, current.width, current.height
            ),
        ));
        return findings;
    }

    let total_px = (prior.width as u64) * (prior.height as u64);
    if total_px == 0 {
        return findings;
    }

    let mut diff_px = 0u64;
    // Per-channel tolerance of 8 (~3% of 0-255) — below that we treat
    // pixels as equal to absorb compression/AA noise.
    const TOL: u8 = 8;
    for (a, b) in prior.buffer.pixels().zip(current.buffer.pixels()) {
        let differs =
            a.0.iter()
                .zip(b.0.iter())
                .any(|(x, y)| x.abs_diff(*y) > TOL);
        if differs {
            diff_px += 1;
        }
    }

    let ratio = diff_px as f64 / total_px as f64;
    if ratio > 0.5 {
        findings.push(Finding::new(
            "large_frame_delta",
            Severity::Warning,
            format!(
                "{:.1}% of pixels changed since prior frame — likely a navigation, not an in-page update",
                ratio * 100.0
            ),
        ));
    } else if ratio > 0.1 {
        findings.push(Finding::new(
            "frame_delta",
            Severity::Info,
            format!("{:.1}% of pixels changed since prior frame", ratio * 100.0),
        ));
    }

    findings
}
