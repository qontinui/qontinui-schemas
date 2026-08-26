//! Layout analyzer — overlaps, misalignments, off-screen elements, missing
//! gutters, etc. Pure geometry over [`ElementSnapshot`]; no pixels.

use super::{Finding, Severity};
use crate::element_snapshot::{intersection, ElementSnapshot};

/// Minimum share of the covered element's own area that must be hidden
/// before a derived occlusion is reported, in percent.
///
/// Not zero: sub-pixel rounding and 1px borders routinely produce hairline
/// intersections between correctly-laid-out siblings, and a check that
/// fires on those gets muted, which costs more than the tail it catches.
/// Low enough that clipping the end of a label still reports — the
/// motivating defect covered a corner of a tile header, not the whole
/// thing.
const OCCLUSION_MIN_PCT: f64 = 2.0;

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

    // 1b. Directed OCCLUSION — which element is on top, and what it hides.
    //
    // Deliberately a separate pass from the `overlap` check above, which
    // answers a different question ("do two clickable targets collide?")
    // and whose semantics are pinned by CI specs. Three differences, each
    // required for this pass to see the bug class it exists for:
    //
    //   * ALL bbox-bearing elements participate, not just `interactable`
    //     ones. The thing a floating widget hides is usually a label — a
    //     name, a status, a count — and labels are never interactive.
    //   * FULL containment is the strongest signal, not an exemption. The
    //     `overlap` pass skips nesting as intentional layout (a button
    //     inside its container); for occlusion, an element completely
    //     covering another is the worst case, so nesting is scored, not
    //     skipped. The two passes disagree on purpose.
    //   * Direction comes from `z_index`, so the finding names the
    //     OCCLUDER and the OCCLUDED rather than an unordered pair.
    //
    // Ancestry is the one exemption: a child painting over its own
    // ancestor's box is ordinary containment (every element covers part of
    // its parent), so pairs related by `parent_id` are skipped.
    let positioned: Vec<(&_, crate::frame::Region)> = snapshot
        .elements
        .iter()
        .filter_map(|e| e.bbox.map(|b| (e, b)))
        .collect();

    // A producer that already attributed the occlusion is authoritative —
    // it hit-tested the live tree, which beats anything derivable from
    // bounding boxes here.
    for (el, _) in &positioned {
        if let Some(occluder) = &el.occluded_by {
            findings.push(
                Finding::new(
                    "occlusion",
                    Severity::Critical,
                    format!(
                        "{} is covered by {} (reported by the snapshot producer's hit-test)",
                        el.id, occluder
                    ),
                )
                .with_elements(vec![el.id.clone(), occluder.clone()]),
            );
        }
    }

    // Derived occlusion needs stacking order. Where it is missing, this
    // pass must say so rather than return quietly — a snapshot with no
    // `z_index` would otherwise render as "no occlusion found".
    //
    // Reported only for pairs that ACTUALLY INTERSECT, not for every
    // z-less snapshot. Boxes that do not touch have no occlusion question
    // to be unknown about, and firing on those would put an unresolvable
    // Info on every legacy capture — noise that gets the whole finding
    // muted, which is how a real UNKNOWN ends up being ignored.
    let mut undirected_intersections = 0usize;

    for i in 0..positioned.len() {
        for j in (i + 1)..positioned.len() {
            let (a, a_bbox) = positioned[i];
            let (b, b_bbox) = positioned[j];

            // Skip ancestor/descendant pairs: a child painting over its
            // own parent's box is what nesting IS, not a defect.
            if a.parent_id.as_deref() == Some(b.id.as_str())
                || b.parent_id.as_deref() == Some(a.id.as_str())
            {
                continue;
            }

            // Geometry first: no intersection means there is nothing to
            // be occluded, and nothing to be unknown about either.
            let Some(inter) = intersection(a_bbox, b_bbox) else {
                continue;
            };

            let (Some(az), Some(bz)) = (a.z_index, b.z_index) else {
                undirected_intersections += 1;
                continue;
            };
            // Equal stacking = document order decides, which a snapshot
            // does not carry reliably. Leave those to the `overlap` pass
            // rather than guess a direction.
            if az == bz {
                undirected_intersections += 1;
                continue;
            }
            let (over, under, under_bbox) = if az > bz {
                (a, b, b_bbox)
            } else {
                (b, a, a_bbox)
            };
            let under_area = under_bbox.w as u64 * under_bbox.h as u64;
            if under_area == 0 {
                continue; // zero_area reports this separately
            }
            let covered = inter.w as u64 * inter.h as u64;
            let pct = (covered as f64 / under_area as f64) * 100.0;
            if pct < OCCLUSION_MIN_PCT {
                continue;
            }

            // Hiding text is worse than hiding a blank box: a covered
            // label destroys information the user has no other way to
            // recover, which is exactly the reported defect.
            let hides_text = under.text.as_deref().is_some_and(|t| !t.trim().is_empty());
            let severity = if hides_text {
                Severity::Critical
            } else {
                Severity::Warning
            };
            let what = match under.text.as_deref().map(str::trim) {
                Some(t) if !t.is_empty() => format!(" (text: {t:?})"),
                _ => String::new(),
            };
            findings.push(
                Finding::new(
                    "occlusion",
                    severity,
                    format!(
                        "{} (z={}) covers {:.0}% of {} (z={}){}",
                        over.id,
                        over.z_index.unwrap_or_default(),
                        pct,
                        under.id,
                        under.z_index.unwrap_or_default(),
                        what
                    ),
                )
                .with_region(inter)
                .with_elements(vec![over.id.clone(), under.id.clone()]),
            );
        }
    }

    if undirected_intersections > 0 {
        findings.push(Finding::new(
            "occlusion_unknown",
            Severity::Info,
            format!(
                "{undirected_intersections} intersecting pair(s) carry no usable stacking \
                 order, so it is UNKNOWN which element is on top and whether anything is \
                 hidden. This is not a clean result — populate `z_index` in the snapshot \
                 projection to resolve it."
            ),
        ));
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
    let mut sorted_y: Vec<(&_, i32)> = snapshot
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
            // Drift = spread within the group. `abs_diff` on the signed
            // baselines yields the u32 pixel jitter and can't overflow even
            // for a group that straddles the (now-signed) origin.
            let min = group.iter().map(|(_, y)| *y).min().unwrap();
            let max = group.iter().map(|(_, y)| *y).max().unwrap();
            let jitter = max.abs_diff(min);
            if jitter > 0 {
                let ids: Vec<String> = group.iter().map(|(e, _)| e.id.clone()).collect();
                findings.push(
                    Finding::new(
                        "alignment_jitter",
                        Severity::Info,
                        format!(
                            "{} elements share a near-y baseline with {jitter}px jitter",
                            group.len(),
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

    fn el(id: &str, x: i32, y: i32, w: u32, h: u32, interactable: bool) -> Element {
        Element {
            id: id.to_string(),
            bbox: Some(Region { x, y, w, h }),
            text: None,
            role: None,
            interactable,
            ..Default::default()
        }
    }

    /// `el()` plus a stacking order — the occlusion pass needs a direction.
    fn el_z(id: &str, x: i32, y: i32, w: u32, h: u32, z: i32) -> Element {
        Element {
            z_index: Some(z),
            ..el(id, x, y, w, h, false)
        }
    }

    #[test]
    fn detects_overlap_between_interactive_elements() {
        let snap = ElementSnapshot {
            elements: vec![
                el("btn-a", 0, 0, 100, 50, true),
                el("btn-b", 50, 25, 100, 50, true),
            ],
            ..Default::default()
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
            ..Default::default()
        };
        let findings = run(&snap);
        assert!(!findings.iter().any(|f| f.kind == "overlap"));
    }

    #[test]
    fn flags_zero_area_elements() {
        let snap = ElementSnapshot {
            elements: vec![el("hidden", 10, 10, 0, 50, false)],
            ..Default::default()
        };
        let findings = run(&snap);
        assert!(findings.iter().any(|f| f.kind == "zero_area"));
    }

    #[test]
    fn no_false_overlap_when_separated() {
        let snap = ElementSnapshot {
            elements: vec![el("a", 0, 0, 50, 50, true), el("b", 100, 0, 50, 50, true)],
            ..Default::default()
        };
        let findings = run(&snap);
        assert!(!findings.iter().any(|f| f.kind == "overlap"));
    }

    #[test]
    fn offscreen_elements_do_not_fabricate_overlaps() {
        // Two a11y-hidden nodes parked at `left: -9999px`, plus a real
        // on-screen button. Under the old clamped-to-0 encoding all three
        // collapsed onto the viewport origin and reported as mutually
        // overlapping; with a signed origin they are correctly disjoint.
        let snap = ElementSnapshot {
            elements: vec![
                el("skip-link", -9999, 0, 120, 32, true),
                el("sr-only", -9999, 40, 120, 32, true),
                el("real-button", 10, 10, 100, 40, true),
            ],
            ..Default::default()
        };
        let findings = run(&snap);
        assert!(
            !findings.iter().any(|f| f.kind == "overlap"),
            "off-screen elements must not overlap each other or on-screen ones: {findings:?}"
        );
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
            ..Default::default()
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

    // --- Directed occlusion (the ZoneMinimap class of defect) ---------------

    #[test]
    fn reports_which_element_is_on_top_and_what_it_hides() {
        let mut header = el_z("zone-header-8", 0, 0, 400, 20, 10);
        header.text = Some("Zone 8: qontinui-web (a3f2c1d0)".into());
        // A small widget parked over the right end of that header — the
        // shape of the reported bug.
        let widget = el_z("zone-minimap", 260, 0, 128, 88, 30);
        let snap = ElementSnapshot {
            elements: vec![header, widget],
            ..Default::default()
        };
        let f = run(&snap);
        let occ = f
            .iter()
            .find(|f| f.kind == "occlusion")
            .expect("a widget covering a header must report occlusion");
        // Direction: the finding must name the OCCLUDER first.
        assert_eq!(occ.elements[0], "zone-minimap");
        assert_eq!(occ.elements[1], "zone-header-8");
        // Hiding text is Critical, not a Warning.
        assert_eq!(occ.severity, Severity::Critical);
        assert!(
            occ.detail.contains("Zone 8: qontinui-web"),
            "the finding must quote the text being hidden: {}",
            occ.detail
        );
    }

    #[test]
    fn full_containment_is_occlusion_not_an_exemption() {
        // The `overlap` pass skips this as intentional nesting. For
        // occlusion it is the WORST case, so the two passes must disagree.
        let mut label = el_z("label", 100, 100, 50, 10, 1);
        label.text = Some("important".into());
        let cover = el_z("cover", 0, 0, 400, 400, 99);
        let snap = ElementSnapshot {
            elements: vec![label, cover],
            ..Default::default()
        };
        let f = run(&snap);
        assert!(
            !f.iter().any(|f| f.kind == "overlap"),
            "the legacy overlap pass still exempts nesting"
        );
        let occ = f.iter().find(|f| f.kind == "occlusion").expect("occluded");
        assert!(occ.detail.contains("100%"), "detail: {}", occ.detail);
    }

    #[test]
    fn a_child_painting_over_its_own_parent_is_not_occlusion() {
        let parent = el_z("card", 0, 0, 200, 100, 0);
        let mut child = el_z("card-title", 0, 0, 200, 20, 5);
        child.parent_id = Some("card".into());
        child.text = Some("title".into());
        let snap = ElementSnapshot {
            elements: vec![parent, child],
            ..Default::default()
        };
        assert!(!run(&snap).iter().any(|f| f.kind == "occlusion"));
    }

    #[test]
    fn missing_z_index_reports_unknown_rather_than_clean() {
        // Absence of stacking data must not read as "nothing is covered".
        let snap = ElementSnapshot {
            elements: vec![
                el("a", 0, 0, 100, 100, false),
                el("b", 0, 0, 100, 100, false),
            ],
            ..Default::default()
        };
        let f = run(&snap);
        assert!(f.iter().any(|f| f.kind == "occlusion_unknown"));
        assert!(!f.iter().any(|f| f.kind == "occlusion"));
    }

    #[test]
    fn a_producer_attributed_occlusion_is_reported_directly() {
        let mut covered = el("session-name", 0, 0, 200, 20, false);
        covered.text = Some("my-session".into());
        covered.occluded_by = Some("toast".into());
        let snap = ElementSnapshot {
            elements: vec![covered],
            ..Default::default()
        };
        let occ = run(&snap)
            .into_iter()
            .find(|f| f.kind == "occlusion")
            .expect("occluded_by is authoritative");
        assert_eq!(occ.severity, Severity::Critical);
        assert!(occ.detail.contains("toast"));
    }

    #[test]
    fn hairline_intersections_do_not_fire() {
        // A 1px seam between correctly-laid-out siblings must stay quiet.
        let a = el_z("row-1", 0, 0, 400, 100, 1);
        let b = el_z("row-2", 0, 99, 400, 100, 2);
        let snap = ElementSnapshot {
            elements: vec![a, b],
            ..Default::default()
        };
        assert!(!run(&snap).iter().any(|f| f.kind == "occlusion"));
    }
}
