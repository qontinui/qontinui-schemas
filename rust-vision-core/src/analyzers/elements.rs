//! Elements analyzer — coverage + interactivity statistics over an
//! [`ElementSnapshot`]. Pure structural; no pixels.

use super::{Finding, Severity};
use crate::element_snapshot::ElementSnapshot;

pub fn run(snapshot: &ElementSnapshot) -> Vec<Finding> {
    let mut findings = Vec::new();

    let total = snapshot.elements.len();
    if total == 0 {
        findings.push(Finding::new(
            "empty_snapshot",
            Severity::Critical,
            "ElementSnapshot has zero elements — page is unrendered, gated, or under-instrumented",
        ));
        return findings;
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

    findings
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
            parent_id: None,
            children_ids: vec![],
        }
    }

    #[test]
    fn empty_snapshot_critical() {
        let snap = ElementSnapshot { elements: vec![] };
        let f = run(&snap);
        assert_eq!(f[0].kind, "empty_snapshot");
        assert_eq!(f[0].severity, Severity::Critical);
    }

    #[test]
    fn tiny_target_warning() {
        let snap = ElementSnapshot {
            elements: vec![el("close", 16, 16, true, None)],
        };
        let f = run(&snap);
        assert!(f.iter().any(|x| x.kind == "tiny_target"));
    }

    #[test]
    fn bbox_none_interactive_counts_but_is_not_tiny_target() {
        // A bbox-less interactive element with text: it counts toward
        // interactive/text presence (so no `no_interactive`/`no_text`), but
        // has no geometry so it must NOT be flagged tiny_target.
        let mut e = el("hidden-btn", 0, 0, true, Some("Submit"));
        e.bbox = None;
        let snap = ElementSnapshot { elements: vec![e] };
        let f = run(&snap);
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
        };
        let f = run(&snap);
        assert!(f.is_empty());
    }
}
