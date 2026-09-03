//! Dynamic analyzer — compares two frames and flags large pixel deltas.
//! Pixel-level only; doesn't try to attribute changes to specific elements
//! (that's the `elements` analyzer's job once it gets baseline support).

use super::{AnalyzerResult, Finding, Severity};
use crate::frame::Frame;

/// Frame-to-frame diff.
///
/// **Carries no [`crate::SnapshotCoverage`] — by construction, not by
/// omission.** This analyzer takes two [`Frame`]s and no `ElementSnapshot`
/// at all, so there is no element list to measure coverage over and the
/// result's `coverage` is `None`. Its preconditions are frame availability,
/// which the dispatcher already checks: a call missing either frame is
/// [`crate::AnalyzerVerdict::Blocked`] there, with the same `skipped`
/// finding it has always emitted.
pub fn run(prior: &Frame, current: &Frame) -> AnalyzerResult {
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
        // A real, evidenced conclusion about the two frames — the diff below
        // is undefined across differing dimensions, and saying so is an
        // answer rather than a refusal.
        return AnalyzerResult::checked(None, findings);
    }

    let total_px = (prior.width as u64) * (prior.height as u64);
    if total_px == 0 {
        // Zero pixels to compare. Every delta threshold below is a ratio
        // over `total_px`, so none of them can fire, and the empty finding
        // list this used to return was indistinguishable from "the page did
        // not change" — the same vacuous pass the snapshot analyzers had.
        return AnalyzerResult::blocked(
            format!(
                "both frames are {}x{} — zero pixels to compare, so no frame delta was \
                 measured.",
                prior.width, prior.height
            ),
            None,
            findings,
        );
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

    AnalyzerResult::checked(None, findings)
}
