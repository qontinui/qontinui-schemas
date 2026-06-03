//! Layout analyzer — overlaps, misalignments, off-screen elements, missing
//! gutters, etc. Pure geometry over [`ElementSnapshot`]; no pixels.

use super::{Finding, Severity};
use crate::element_snapshot::{intersection, ElementSnapshot};

pub fn run(snapshot: &ElementSnapshot) -> Vec<Finding> {
    let mut findings = Vec::new();

    // Only positioned (bbox-bearing) interactive elements participate in the
    // geometry checks; bbox-less elements (e.g. unmeasured mobile-discover
    // nodes) are silently skipped here — they carry no geometry to reason
    // about and must not produce spurious findings.
    let interactive: Vec<(&_, crate::frame::Region)> = snapshot
        .elements
        .iter()
        .filter(|e| e.interactable)
        .filter_map(|e| e.bbox.map(|b| (e, b)))
        .collect();

    // 1. Pairwise overlap among interactive elements (the high-leverage check
    // from the plan's intro — the terminal-tab overlap bug that triggered
    // the whole Phase 6 design).
    for i in 0..interactive.len() {
        for j in (i + 1)..interactive.len() {
            let (a, a_bbox) = interactive[i];
            let (b, b_bbox) = interactive[j];
            if let Some(inter) = intersection(a_bbox, b_bbox) {
                // Skip cases where one is fully nested in the other — that's
                // intentional layout (button inside container), not a bug.
                let nested = inter == a_bbox || inter == b_bbox;
                if nested {
                    continue;
                }
                findings.push(
                    Finding::new(
                        "overlap",
                        Severity::Warning,
                        format!(
                            "interactive elements {} and {} overlap by {} px²",
                            a.id,
                            b.id,
                            inter.w * inter.h
                        ),
                    )
                    .with_region(inter)
                    .with_elements(vec![a.id.clone(), b.id.clone()]),
                );
            }
        }
    }

    // 2. Zero-area elements — usually layout regressions ("display:none on a
    // visible-by-spec element" or "width: 0"). bbox-less elements are NOT
    // zero-area; they simply have no measured geometry, so we skip them
    // rather than fabricate a `zero_area` finding.
    for el in &snapshot.elements {
        if let Some(bbox) = el.bbox {
            if bbox.w == 0 || bbox.h == 0 {
                findings.push(
                    Finding::new(
                        "zero_area",
                        Severity::Warning,
                        format!("element {} has zero area ({}x{})", el.id, bbox.w, bbox.h),
                    )
                    .with_elements(vec![el.id.clone()]),
                );
            }
        }
    }

    // 3. Alignment groups: when 3+ elements share a near-horizontal y but
    // differ by 1-2 px, the +/- jitter is usually a layout bug. Only
    // positioned elements participate.
    let mut sorted_y: Vec<(&_, u32)> = snapshot
        .elements
        .iter()
        .filter_map(|e| e.bbox.map(|b| (e, b.y)))
        .collect();
    sorted_y.sort_by_key(|(_, y)| *y);
    let tol_px = 3u32;
    let mut i = 0;
    while i < sorted_y.len() {
        let group_y = sorted_y[i].1;
        let mut j = i + 1;
        while j < sorted_y.len() && sorted_y[j].1.abs_diff(group_y) <= tol_px {
            j += 1;
        }
        let group = &sorted_y[i..j];
        if group.len() >= 3 {
            // Drift = max - min within the group.
            let min = group.iter().map(|(_, y)| *y).min().unwrap();
            let max = group.iter().map(|(_, y)| *y).max().unwrap();
            if max - min > 0 {
                let ids: Vec<String> = group.iter().map(|(e, _)| e.id.clone()).collect();
                findings.push(
                    Finding::new(
                        "alignment_jitter",
                        Severity::Info,
                        format!(
                            "{} elements share a near-y baseline with {}px jitter",
                            group.len(),
                            max - min
                        ),
                    )
                    .with_elements(ids),
                );
            }
        }
        i = j;
    }

    findings
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::element_snapshot::Element;
    use crate::frame::Region;

    fn el(id: &str, x: u32, y: u32, w: u32, h: u32, interactable: bool) -> Element {
        Element {
            id: id.to_string(),
            bbox: Some(Region { x, y, w, h }),
            text: None,
            role: None,
            interactable,
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
    fn detects_overlap_between_interactive_elements() {
        let snap = ElementSnapshot {
            elements: vec![
                el("btn-a", 0, 0, 100, 50, true),
                el("btn-b", 50, 25, 100, 50, true),
            ],
        };
        let findings = run(&snap);
        assert!(findings.iter().any(|f| f.kind == "overlap"));
    }

    #[test]
    fn ignores_nested_layouts() {
        // child fully inside parent — not an overlap finding
        let snap = ElementSnapshot {
            elements: vec![
                el("container", 0, 0, 200, 200, true),
                el("nested-btn", 50, 50, 50, 50, true),
            ],
        };
        let findings = run(&snap);
        assert!(!findings.iter().any(|f| f.kind == "overlap"));
    }

    #[test]
    fn flags_zero_area_elements() {
        let snap = ElementSnapshot {
            elements: vec![el("hidden", 10, 10, 0, 50, false)],
        };
        let findings = run(&snap);
        assert!(findings.iter().any(|f| f.kind == "zero_area"));
    }

    #[test]
    fn no_false_overlap_when_separated() {
        let snap = ElementSnapshot {
            elements: vec![el("a", 0, 0, 50, 50, true), el("b", 100, 0, 50, 50, true)],
        };
        let findings = run(&snap);
        assert!(!findings.iter().any(|f| f.kind == "overlap"));
    }

    fn el_no_bbox(id: &str, interactable: bool) -> Element {
        let mut e = el(id, 0, 0, 0, 0, interactable);
        e.bbox = None;
        e
    }

    #[test]
    fn bbox_none_elements_skipped_no_spurious_findings() {
        // A mix of positioned and bbox-less elements: the positioned pair
        // overlaps (1 finding); the bbox-less elements must NOT yield
        // overlap/zero_area/alignment findings and must not panic.
        let snap = ElementSnapshot {
            elements: vec![
                el("btn-a", 0, 0, 100, 50, true),
                el("btn-b", 50, 25, 100, 50, true),
                el_no_bbox("hidden-1", true),
                el_no_bbox("hidden-2", false),
                el_no_bbox("hidden-3", true),
            ],
        };
        let findings = run(&snap);
        assert!(findings.iter().any(|f| f.kind == "overlap"));
        // None of the bbox-less ids appear in any finding.
        for f in &findings {
            for id in &f.elements {
                assert!(
                    !id.starts_with("hidden-"),
                    "bbox-less element {id} leaked into finding {:?}",
                    f.kind
                );
            }
        }
        // No zero_area fabricated for the bbox-less elements.
        assert!(!findings.iter().any(|f| f.kind == "zero_area"));
    }
}
