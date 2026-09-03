//! Elements analyzer — coverage + interactivity statistics over an
//! [`ElementSnapshot`]. Pure structural; no pixels.

use super::{AnalyzerResult, Finding, Severity};
use crate::coverage::SnapshotCoverage;
use crate::element_snapshot::ElementSnapshot;

pub fn run(snapshot: &ElementSnapshot) -> AnalyzerResult {
    let coverage = SnapshotCoverage::of(snapshot);
    let mut findings = Vec::new();

    let total = snapshot.elements.len();
    if total == 0 {
        findings.push(Finding::new(
            "empty_snapshot",
            Severity::Critical,
            "ElementSnapshot has zero elements — page is unrendered, gated, or under-instrumented",
        ));
        // `Checked`, not `Blocked`, and the distinction is the point: this
        // analyzer's input is the element LIST, and it read it, found it
        // empty, and said so at `Severity::Critical`. That is a conclusion
        // about the page, reached from evidence — the opposite of a refusal
        // to answer. A `Blocked` here would demote a real Critical finding
        // into "we could not tell", which is strictly less true.
        return AnalyzerResult::checked(Some(coverage), findings);
    }

    let interactive = snapshot.elements.iter().filter(|e| e.interactable).count();
    let text = snapshot
        .elements
        .iter()
        .filter(|e| e.text.is_some())
        .count();

    if interactive == 0 {
        findings.push(Finding::new(
            "no_interactive",
            Severity::Warning,
            format!(
                "{total} elements, none interactive — page may be a static dashboard or broken"
            ),
        ));
    }

    if text == 0 && total > 5 {
        findings.push(Finding::new(
            "no_text",
            Severity::Warning,
            format!("{total} elements but none carry text — snapshot source under-instrumented?"),
        ));
    }

    // Tiny target detection — interactive elements smaller than 24×24 are
    // below the WCAG 2.5.5 target-size minimum (Level AAA) and the more
    // relaxed 24×24 minimum from WCAG 2.5.8 (Level AA, 2.2). Only
    // positioned elements can be measured; bbox-less interactive elements
    // are skipped (no geometry → no target-size claim, not a tiny target).
    let tiny_targets: Vec<&_> = snapshot
        .elements
        .iter()
        .filter(|e| e.interactable && e.bbox.map(|b| b.w < 24 || b.h < 24).unwrap_or(false))
        .collect();
    if !tiny_targets.is_empty() {
        let ids: Vec<String> = tiny_targets.iter().map(|e| e.id.clone()).collect();
        findings.push(
            Finding::new(
                "tiny_target",
                Severity::Warning,
                format!(
                    "{} interactive elements below 24×24 px (WCAG 2.5.8 target size)",
                    tiny_targets.len()
                ),
            )
            .with_elements(ids),
        );
    }

    // This analyzer is never blocked on coverage. It is pure structural —
    // its checks are counts over the element list itself — and it stays
    // meaningful on a snapshot with no geometry at all, which is exactly
    // when `layout` cannot run. That asymmetry is deliberate and is why the
    // full analyzer set catches shapes a single-analyzer call does not.
    //
    // Geometry is read by ONE check, `tiny_target`, and without it no
    // target-size claim can be made either way. That is a named unmeasured
    // dimension, i.e. `Degraded` — green, findings intact.
    if coverage.with_geometry == 0 {
        return AnalyzerResult::degraded(
            format!(
                "no element carries a bbox (0/{}), so target size (`tiny_target`, WCAG \
                 2.5.8) was not evaluated for any of the {} interactive element(s). The \
                 structural checks above are unaffected.",
                coverage.elements, coverage.interactable
            ),
            Some(coverage),
            findings,
        );
    }
    AnalyzerResult::checked(Some(coverage), findings)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::element_snapshot::Element;
    use crate::frame::Region;

    fn el(id: &str, w: u32, h: u32, inter: bool, text: Option<&str>) -> Element {
        Element {
            id: id.to_string(),
            bbox: Some(Region { x: 0, y: 0, w, h }),
            text: text.map(String::from),
            role: None,
            interactable: inter,
            fg_color: None,
            bg_color: None,
            font_size_px: None,
            font_family: None,
            line_height_px: None,
            ..Default::default()
        }
    }

    #[test]
    fn empty_snapshot_critical() {
        let snap = ElementSnapshot {
            elements: vec![],
            ..Default::default()
        };
        let f = run(&snap).findings;
        assert_eq!(f[0].kind, "empty_snapshot");
        assert_eq!(f[0].severity, Severity::Critical);
    }

    #[test]
    fn tiny_target_warning() {
        let snap = ElementSnapshot {
            elements: vec![el("close", 16, 16, true, None)],
            ..Default::default()
        };
        let f = run(&snap).findings;
        assert!(f.iter().any(|x| x.kind == "tiny_target"));
    }

    #[test]
    fn bbox_none_interactive_counts_but_is_not_tiny_target() {
        // A bbox-less interactive element with text: it counts toward
        // interactive/text presence (so no `no_interactive`/`no_text`), but
        // has no geometry so it must NOT be flagged tiny_target.
        let mut e = el("hidden-btn", 0, 0, true, Some("Submit"));
        e.bbox = None;
        let snap = ElementSnapshot {
            elements: vec![e],
            ..Default::default()
        };
        let f = run(&snap).findings;
        assert!(!f.iter().any(|x| x.kind == "no_interactive"));
        assert!(!f.iter().any(|x| x.kind == "tiny_target"));
    }

    #[test]
    fn ok_when_normal() {
        let snap = ElementSnapshot {
            elements: vec![
                el("btn", 100, 40, true, Some("Save")),
                el("h", 200, 30, false, Some("Heading")),
            ],
            ..Default::default()
        };
        let f = run(&snap).findings;
        assert!(f.is_empty());
    }
}
