//! Layout analyzer — overlaps, misalignments, off-screen elements, missing
//! gutters, etc. Pure geometry over [`ElementSnapshot`]; no pixels.

use super::{AnalyzerResult, Finding, Severity};
use crate::coverage::SnapshotCoverage;
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

/// Whether `ancestor` is an ancestor of `node`, following `parent_id` to the
/// root.
///
/// A free function rather than a closure: a closure taking `&str` parameters
/// AND capturing the borrowed map makes rustc infer `'static` for the map's
/// values and reject every borrow from the snapshot.
///
/// The depth cap terminates on a malformed snapshot containing a parent
/// cycle — the analyzer must not hang on hostile input, and 64 is far past
/// any real DOM nesting that carries registered elements.
fn is_ancestor_of(
    parent_of: &std::collections::HashMap<&str, &str>,
    ancestor: &str,
    node: &str,
) -> bool {
    let mut cur = node;
    for _ in 0..64 {
        match parent_of.get(cur) {
            Some(&p) if p == ancestor => return true,
            Some(&p) => cur = p,
            None => return false,
        }
    }
    false
}

pub fn run(snapshot: &ElementSnapshot) -> AnalyzerResult {
    let coverage = SnapshotCoverage::of(snapshot);
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

    // Ancestry, walked to the ROOT rather than one hop.
    //
    // A one-level `parent_id` check misses the common shape: card > header
    // > title, where the title carries its own `z-index` and overlaps the
    // card's box. That is ordinary containment two levels up, and reporting
    // it would put a false `occlusion` on essentially every nested layout
    // with a stacking context in it.
    // Type elided deliberately: an explicit `HashMap<&str, &str>` annotation
    // makes rustc infer `'static` for both elided lifetimes and reject the
    // borrows from `snapshot`. Inference gets it right.
    let parent_of = snapshot
        .elements
        .iter()
        .filter_map(|e| e.parent_id.as_deref().map(|p| (e.id.as_str(), p)))
        .collect::<std::collections::HashMap<_, _>>();
    // A producer that already attributed the occlusion is authoritative —
    // it hit-tested the live tree, which beats anything derivable from
    // bounding boxes here. Recorded so the derived pass below does not
    // report the same pair a second time from a weaker signal.
    // Same lifetime-elision rule as `parent_of` above: annotating the `&str`
    // pair here would pin both to `'static` and reject borrows from
    // `snapshot`.
    let mut attributed = std::collections::HashSet::new();
    for (el, _) in &positioned {
        if let Some(occluder) = &el.occluded_by {
            attributed.insert((occluder.as_str(), el.id.as_str()));
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

            // Skip ancestor/descendant pairs at any depth: a descendant
            // painting over an ancestor's box is what nesting IS.
            if is_ancestor_of(&parent_of, a.id.as_str(), b.id.as_str())
                || is_ancestor_of(&parent_of, b.id.as_str(), a.id.as_str())
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
            // Already reported from the producer's hit-test above, which is
            // the stronger signal — do not say it twice from geometry.
            if attributed.contains(&(over.id.as_str(), under.id.as_str())) {
                continue;
            }
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

    verdict(coverage, undirected_intersections, findings)
}

/// Classify a completed layout run.
///
/// # Blocked — `with_geometry == 0`
///
/// This analyzer is pure geometry, and with no bbox anywhere EVERY pass
/// above examined nothing: the interactive-overlap pass, the directed and
/// derived occlusion passes, zero-area and alignment all build their working
/// set from `e.bbox`. The empty finding list they produce is byte-identical
/// to a healthy page's, which is the vacuous pass this verdict exists to
/// make impossible to read as a pass.
///
/// The findings are still carried (see [`AnalyzerResult`]); what `Blocked`
/// withdraws is the claim that an empty list here means the page is clean.
///
/// # Degraded — `interactable == 0` with geometry present
///
/// **Not `Blocked`, and the corpus proves why.** Only ONE of this
/// analyzer's passes filters on `interactable` (the pairwise-overlap pass,
/// whose working set is `.filter(|e| e.interactable).filter_map(…bbox)`).
/// The directed-occlusion pass added later builds `positioned` from bbox
/// alone, as do the zero-area and alignment passes — so a snapshot with
/// nothing flagged interactable is not unexaminable, it is examined by four
/// passes out of five.
///
/// `tests/fixtures/analyzer_corpus/007-widget-occludes-label` is exactly
/// that snapshot: four elements, every one `interactable: false`, and it
/// yields the `Severity::Critical` `occlusion` finding that pins the
/// Terminal-page regression this crate was extended to catch. Blocking it
/// would stamp "no finding list from this run is a statement about the page"
/// on the most valuable finding the analyzer emits. What is genuinely
/// unmeasured is the overlap pass, and naming that is what `Degraded` is
/// for.
///
/// # Degraded — intersecting pairs with no usable stacking order
///
/// The condition that already emits `occlusion_unknown`, promoted onto the
/// verdict channel so a consumer reading verdicts rather than finding kinds
/// can see it. The finding itself is kept, at `Severity::Info`, because it
/// names the pair count and consumers already display it.
///
/// Deliberately NOT keyed on `with_stacking == 0` in general. A healthy
/// projected snapshot routinely carries no stacking order at all — the
/// projector emits `z_index` only for a computed `zIndex` that parses as an
/// integer, and `auto` does not — so a blanket degradation there would fire
/// on essentially every real capture while saying nothing about whether an
/// occlusion question was even asked. Boxes that do not intersect have no
/// occlusion question to be unknown about.
fn verdict(
    coverage: SnapshotCoverage,
    undirected_intersections: usize,
    findings: Vec<Finding>,
) -> AnalyzerResult {
    if coverage.with_geometry == 0 {
        return AnalyzerResult::blocked(
            format!(
                "no element carries a bbox (0/{}), so every geometric check — overlap, \
                 occlusion, zero-area, alignment — examined nothing. An empty finding \
                 list here is not a clean page.",
                coverage.elements
            ),
            Some(coverage),
            findings,
        );
    }

    // Both degradations can hold at once, and each names a different
    // unchecked defect class, so they are reported together rather than
    // one shadowing the other.
    let mut degradations: Vec<String> = Vec::new();
    if coverage.interactable == 0 {
        degradations.push(format!(
            "no element is flagged interactable (0/{}), so the pairwise-overlap pass \
             examined nothing; the occlusion, zero-area and alignment passes are \
             unaffected",
            coverage.elements
        ));
    }
    if undirected_intersections > 0 {
        degradations.push(format!(
            "{undirected_intersections} intersecting pair(s) carry no usable stacking \
             order, so it is UNKNOWN which element is on top and whether anything is \
             hidden ({}/{} elements carry a paint rank)",
            coverage.with_stacking, coverage.elements
        ));
    }
    if degradations.is_empty() {
        AnalyzerResult::checked(Some(coverage), findings)
    } else {
        AnalyzerResult::degraded(degradations.join("; "), Some(coverage), findings)
    }
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

    /// An element carrying TEXT and interactivity but NO geometry — the
    /// partial-projection shape. Distinct from `el_no_bbox` below, which
    /// carries neither text nor a real box; the text is load-bearing here,
    /// because it is what keeps the `elements` analyzer quiet and made this
    /// shape invisible.
    fn el_text_no_bbox(id: &str, text: &str, interactable: bool) -> Element {
        Element {
            id: id.to_string(),
            bbox: None,
            text: Some(text.to_string()),
            role: None,
            interactable,
            ..Default::default()
        }
    }

    // ---------------------------------------------------------------------
    // The vacuous-pass acceptance. Plan
    // `2026-09-03-a-degenerate-snapshot-audits-clean`: an empty finding list
    // used to be byte-identical whether the page was clean or the input was
    // too impoverished to check. These pin BOTH degenerate shapes by name so
    // the equivalence cannot silently come back.
    // ---------------------------------------------------------------------

    /// Shape 1: elements present, every `bbox: None`. Every geometric check
    /// examines nothing, so the finding list says nothing.
    #[test]
    fn a_zero_geometry_snapshot_is_blocked_not_clean() {
        let snap = ElementSnapshot {
            elements: vec![
                Element {
                    id: "a".into(),
                    bbox: None,
                    interactable: true,
                    ..Default::default()
                },
                Element {
                    id: "b".into(),
                    bbox: None,
                    interactable: true,
                    ..Default::default()
                },
            ],
            ..Default::default()
        };
        let result = run(&snap);

        assert!(
            result.verdict.is_blocked(),
            "zero-geometry snapshot must be Blocked, got {:?}",
            result.verdict
        );
        // The gate bit is the thing consumers read, and it must be false.
        assert!(
            !result.conclusive,
            "Blocked must not be conclusive — that is the whole defect"
        );
        // The reason must name the measurement, not merely assert failure.
        assert!(
            result.verdict.reason().is_some_and(|r| r.contains("bbox")),
            "Blocked must say WHY: {:?}",
            result.verdict.reason()
        );
        let cov = result
            .coverage
            .expect("a snapshot was supplied, so it was measured");
        assert_eq!(cov.elements, 2);
        assert_eq!(cov.with_geometry, 0);
    }

    /// Shape 2 — **the one this plan exists for.** Elements DO carry `text`
    /// and `interactable: true`, and only `bbox` is missing. Measured
    /// 2026-09-03 against the pre-fix crate, this produced output
    /// byte-identical to a healthy page: zero findings, exit 0. `elements`
    /// is content with it (`total > 0`, interactive > 0, text > 0), so no
    /// analyzer in the set said anything at all.
    #[test]
    fn a_partial_projection_with_text_and_interactivity_but_no_geometry_is_blocked() {
        let snap = ElementSnapshot {
            elements: vec![
                el_text_no_bbox("btn-save", "Save", true),
                el_text_no_bbox("btn-cancel", "Cancel", true),
                el_text_no_bbox("title", "Settings", false),
            ],
            ..Default::default()
        };
        let result = run(&snap);

        assert!(
            result.verdict.is_blocked(),
            "a partial projection must be Blocked, got {:?}",
            result.verdict
        );
        assert!(!result.conclusive);

        // The coverage must show the discriminator: text and interactivity
        // present, geometry absent. That combination is precisely what made
        // this shape invisible to every other signal.
        let cov = result.coverage.expect("a snapshot was supplied");
        assert_eq!(cov.elements, 3);
        assert_eq!(cov.with_geometry, 0, "the missing dimension");
        assert_eq!(cov.with_text, 3, "text WAS present — this is the trap");
        assert_eq!(cov.interactable, 2, "interactivity WAS present too");
    }

    /// The control: a healthy snapshot must stay green, and `Degraded` must
    /// NOT move the gate bit. Absent stacking order is the common case (the
    /// projector emits `z_index` only for an integer computed `zIndex`), so a
    /// Degraded-fails rule would fire on essentially every real page.
    #[test]
    fn a_healthy_snapshot_is_conclusive_and_degraded_stays_green() {
        let snap = ElementSnapshot {
            elements: vec![
                el("btn-a", 0, 0, 100, 50, true),
                el("btn-b", 200, 0, 100, 50, true),
            ],
            ..Default::default()
        };
        let result = run(&snap);

        assert!(
            !result.verdict.is_blocked(),
            "a measurable snapshot must never be Blocked: {:?}",
            result.verdict
        );
        assert!(
            result.conclusive,
            "Checked and Degraded are both green; only Blocked is not"
        );
        let cov = result.coverage.expect("a snapshot was supplied");
        assert_eq!(cov.with_geometry, 2);
        assert_eq!(cov.with_stacking, 0, "no z_index — the ordinary case");
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
        let findings = run(&snap).findings;
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
        let findings = run(&snap).findings;
        assert!(!findings.iter().any(|f| f.kind == "overlap"));
    }

    #[test]
    fn flags_zero_area_elements() {
        let snap = ElementSnapshot {
            elements: vec![el("hidden", 10, 10, 0, 50, false)],
            ..Default::default()
        };
        let findings = run(&snap).findings;
        assert!(findings.iter().any(|f| f.kind == "zero_area"));
    }

    #[test]
    fn no_false_overlap_when_separated() {
        let snap = ElementSnapshot {
            elements: vec![el("a", 0, 0, 50, 50, true), el("b", 100, 0, 50, 50, true)],
            ..Default::default()
        };
        let findings = run(&snap).findings;
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
        let findings = run(&snap).findings;
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
        let findings = run(&snap).findings;
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
        let f = run(&snap).findings;
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
        let f = run(&snap).findings;
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
        assert!(!run(&snap).findings.iter().any(|f| f.kind == "occlusion"));
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
        let f = run(&snap).findings;
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
            .findings
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
        assert!(!run(&snap).findings.iter().any(|f| f.kind == "occlusion"));
    }

    #[test]
    fn a_grandchild_painting_over_its_grandparent_is_not_occlusion() {
        // card > header > title, with the title carrying its own stacking
        // context. A one-hop `parent_id` check misses this and would put a
        // false `occlusion` on essentially every nested layout.
        let card = el_z("card", 0, 0, 400, 300, 0);
        let mut header = el_z("header", 0, 0, 400, 40, 1);
        header.parent_id = Some("card".into());
        let mut title = el_z("title", 0, 0, 200, 20, 5);
        title.parent_id = Some("header".into());
        title.text = Some("My Card".into());

        let snap = ElementSnapshot {
            elements: vec![card, header, title],
            ..Default::default()
        };
        assert!(
            !run(&snap).findings.iter().any(|f| f.kind == "occlusion"),
            "nesting is nesting however deep it goes"
        );
    }

    #[test]
    fn a_parent_cycle_terminates_instead_of_hanging() {
        // A malformed snapshot must not wedge the analyzer.
        let mut a = el_z("a", 0, 0, 100, 100, 1);
        a.parent_id = Some("b".into());
        let mut b = el_z("b", 0, 0, 100, 100, 2);
        b.parent_id = Some("a".into());
        let snap = ElementSnapshot {
            elements: vec![a, b],
            ..Default::default()
        };
        let _ = run(&snap); // must return at all
    }

    #[test]
    fn a_pair_is_not_reported_twice_from_two_sources() {
        // The producer's hit-test is the stronger signal. When geometry
        // derives the same pair, saying it again is noise.
        let mut header = el_z("header", 0, 0, 400, 20, 10);
        header.text = Some("Zone 8".into());
        header.occluded_by = Some("minimap".into());
        let widget = el_z("minimap", 0, 0, 400, 20, 30);
        let snap = ElementSnapshot {
            elements: vec![header, widget],
            ..Default::default()
        };
        let occ: Vec<_> = run(&snap)
            .findings
            .into_iter()
            .filter(|f| f.kind == "occlusion")
            .collect();
        assert_eq!(occ.len(), 1, "one pair, one finding: {occ:?}");
        assert!(
            occ[0].detail.contains("hit-test"),
            "the stronger source wins"
        );
    }
}
