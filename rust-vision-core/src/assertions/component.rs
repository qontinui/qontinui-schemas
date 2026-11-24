// SPDX-License-Identifier: MIT
//! [`Assertion::MatchesIsolatedRender`]: does a component render in situ the
//! way it renders in isolation? (plan
//! `2026-11-13-ui-bridge-matches-isolated-render-assertion`, Phase 1.)
//!
//! Compares two [`ComponentCapture`]s — the isolated one stored as a
//! [`ComponentBaseline`] in [`EvalContext::component_baselines`], the in-situ
//! one in [`EvalContext::component_captures`] — by STRUCTURE, never pixels.
//! Nodes are paired by tree path; node geometry is root-relative, so a
//! different viewport or page offset is reported and never fails on its own.
//!
//! The failure detail is a JSON [`ComponentDiffReport`] (every outcome that
//! reached the comparison carries one), so a consumer can parse the typed
//! findings back out with `serde_json::from_str`.

use std::collections::{BTreeMap, HashMap};

use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

use super::{Assertion, AssertionResult, EvalContext};
use crate::component_capture::{
    CaptureRect, CaptureViewport, ClipAncestor, ComponentBaseline, ComponentCapture,
    ComponentCaptureNode, OverflowAxis, COMPONENT_CAPTURE_SCHEMA_VERSION,
};
use crate::observation::UnknownCode;

/// Default [`Assertion::MatchesIsolatedRender`] `size_tolerance_pct`.
pub const DEFAULT_SIZE_TOLERANCE_PCT: f64 = 5.0;

/// Default [`Assertion::MatchesIsolatedRender`] `style_props`.
pub const DEFAULT_STYLE_PROPS: [&str; 4] = ["display", "fontSize", "color", "overflow"];

/// Sub-pixel slack for the `clipped` check: layout rounding must not read as
/// clipping.
const CLIP_EPSILON_PX: f64 = 0.5;

/// The six typed diff classes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ComponentFindingKind {
    /// Stretched or constrained beyond tolerance (absolute size AND fill
    /// ratio of the parent content box both differ on one axis).
    Size,
    /// A node extends past a clip ancestor in situ that did not clip it in
    /// isolation.
    Clipped,
    /// A component name or tag differs at the same tree path, or a node is
    /// present on one side only.
    ChildIdentity,
    /// A whitelisted computed style differs at the same tree path.
    StyleDelta,
    /// A top-level prop differs (only with `compare_props`).
    PropsDelta,
    /// A portal's mount target differs, or a portal exists on one side only.
    PortalMismatch,
}

/// One typed divergence between the isolated and the in-situ render.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ComponentFinding {
    pub kind: ComponentFindingKind,
    /// The tree path the finding is about (`""` = the root). For
    /// `props_delta`, the prop key prefixed with `props.`.
    pub path: String,
    pub message: String,
    /// The baseline (isolated) side's value; `null` when absent there.
    pub baseline: Value,
    /// The in-situ side's value; `null` when absent there.
    pub in_situ: Value,
}

/// The viewports of both captures. Reported, never failed on.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ViewportComparison {
    pub baseline: Option<CaptureViewport>,
    pub in_situ: Option<CaptureViewport>,
    pub baseline_dpr: Option<f64>,
    pub in_situ_dpr: Option<f64>,
    /// `Some(false)` when any KNOWN pair differs (both viewports known and
    /// different, or both DPRs known and different); `Some(true)` only when
    /// all four are known and equal; `None` otherwise.
    pub matches: Option<bool>,
}

/// The JSON body of a [`Assertion::MatchesIsolatedRender`] result's
/// `detail` once both captures were compared.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ComponentDiffReport {
    pub summary: String,
    pub component: String,
    pub baseline: String,
    pub findings: Vec<ComponentFinding>,
    /// Checks that could not be performed (unreadable names, a missing parent
    /// content box, …) plus the captures' own `unknownChecks`.
    pub unknown_checks: Vec<String>,
    pub viewport: ViewportComparison,
}

/// Dispatch target for [`Assertion::MatchesIsolatedRender`]; called by
/// [`super::evaluate`] for that variant only.
pub(super) fn evaluate(a: Assertion, ctx: &EvalContext<'_>) -> AssertionResult {
    let Assertion::MatchesIsolatedRender {
        component,
        baseline,
        size_tolerance_pct,
        style_props,
        compare_props,
    } = a.clone()
    else {
        unreachable!("not a component assertion: {a:?}")
    };

    let Some(entry) = ctx.component_baselines.and_then(|m| m.get(&*baseline)) else {
        let detail = if ctx.component_baselines.is_none() {
            "no component baselines were supplied to the evaluator, so the in-situ render \
             was not compared"
                .to_string()
        } else {
            format!(
                "component baseline '{baseline}' is not registered, so the in-situ render was \
                 not compared — store it with `vision/component-baseline` first"
            )
        };
        return AssertionResult::unknown(a, UnknownCode::InputMissing, detail);
    };
    let Some(in_situ) = ctx.component_captures.and_then(|m| m.get(&*component)) else {
        return AssertionResult::unknown(
            a,
            UnknownCode::InputMissing,
            format!(
                "no in-situ capture for component '{component}' (the component was not found, \
                 or the capture route refused), so nothing was compared"
            ),
        );
    };
    for (side, c) in [("baseline", &entry.capture), ("in-situ", in_situ)] {
        if c.schema_version.is_none() {
            return AssertionResult::unknown(
                a,
                UnknownCode::InputMissing,
                format!(
                    "the {side} capture carries no schemaVersion, so it cannot be read as \
                     schemaVersion {COMPONENT_CAPTURE_SCHEMA_VERSION} and the renders were \
                     not compared"
                ),
            );
        }
    }
    let (b_ver, s_ver) = (
        entry.capture.schema_version.unwrap_or_default(),
        in_situ.schema_version.unwrap_or_default(),
    );
    if b_ver != COMPONENT_CAPTURE_SCHEMA_VERSION || s_ver != COMPONENT_CAPTURE_SCHEMA_VERSION {
        return AssertionResult::unknown(
            a,
            UnknownCode::InputMissing,
            format!(
                "capture schema version mismatch: baseline '{baseline}' is schemaVersion \
                 {b_ver} and the in-situ capture is schemaVersion {s_ver}, but this reader \
                 understands only schemaVersion {COMPONENT_CAPTURE_SCHEMA_VERSION}, so the \
                 renders were not compared"
            ),
        );
    }
    if let (Some(b_sha), Some(s_sha)) =
        (entry.effective_source_sha(), in_situ.source_sha.as_deref())
    {
        if b_sha != s_sha {
            return AssertionResult::unknown(
                a,
                UnknownCode::StaleInput,
                format!(
                    "baseline '{baseline}' was captured at source sha {b_sha} but the in-situ \
                     render is at {s_sha}; re-capture the baseline at the current sha"
                ),
            );
        }
    }

    // From here every check that has its inputs runs, and a definite failure
    // outranks a check that could not run. `undecided` collects the checks
    // that could not run: with no finding, any of them makes the answer
    // unknown; with a finding, they are notes on the failure.
    let mut undecided: Vec<String> = Vec::new();
    let tolerance = if size_tolerance_pct.is_finite() && size_tolerance_pct >= 0.0 {
        Some(size_tolerance_pct)
    } else {
        undecided.push(format!(
            "size not compared: invalid tolerance (size_tolerance_pct {size_tolerance_pct} \
             is not a finite non-negative percentage)"
        ));
        None
    };
    // Root geometry is what `size` measures and what turns root-relative node
    // rects into page coordinates for `clipped`; without it on a side, those
    // two checks cannot run, and the rest still do.
    let roots = match (entry.capture.root_bbox(), in_situ.root_bbox()) {
        (Some(b), Some(s)) => Some((b, s)),
        (b, s) => {
            let what = match (b, s) {
                (None, None) => "neither capture has root geometry",
                (None, Some(_)) => "the baseline capture has no root geometry",
                _ => "the in-situ capture has no root geometry",
            };
            undecided.push(format!("size/clipped not compared: {what}"));
            None
        }
    };
    // A capture with no node tree (or no root node) would pass with every
    // per-node check skipped, or fail with every node "missing": neither is
    // an answer, so the per-node checks are skipped as undecided while `size`
    // and `props_delta` still run.
    let has_root_node = |c: &ComponentCapture| c.nodes.iter().any(|n| n.path.is_empty());
    let missing_tree = |c: &ComponentCapture| {
        if c.nodes.is_empty() {
            Some("has no nodes")
        } else if !has_root_node(c) {
            Some("has no root node (no node with path \"\")")
        } else {
            None
        }
    };
    let mut tree_ok = true;
    for (side, c) in [("baseline", &entry.capture), ("in-situ", in_situ)] {
        if let Some(why) = missing_tree(c) {
            tree_ok = false;
            undecided.push(format!(
                "per-node checks skipped: the {side} capture {why}, so clipped, \
                 child_identity, style_delta and portal_mismatch were not compared"
            ));
        }
    }
    let mut props_ok = compare_props;
    if compare_props {
        let missing = match (entry.capture.props.is_none(), in_situ.props.is_none()) {
            (true, true) => Some("neither capture has props"),
            (true, false) => Some("the baseline capture has no props"),
            (false, true) => Some("the in-situ capture has no props"),
            (false, false) => None,
        };
        if let Some(what) = missing {
            props_ok = false;
            undecided.push(format!(
                "props not compared: compare_props was requested but {what}"
            ));
        }
    }
    // A repeated style prop must not yield a repeated finding.
    let mut props: Vec<String> = Vec::with_capacity(style_props.len());
    for p in style_props.iter() {
        if !props.contains(p) {
            props.push(p.clone());
        }
    }
    let opts = DiffOptions {
        size_tolerance_pct: tolerance,
        style_props: &props,
        compare_props: props_ok,
        per_node: tree_ok,
    };
    let (findings, notes) = diff(entry, in_situ, roots, &opts, &mut undecided);
    let viewport = compare_viewports(&entry.capture, in_situ);
    let failed = !findings.is_empty();
    let unknown = !failed && !undecided.is_empty();
    let summary = if unknown {
        format!(
            "component '{component}' could not be fully compared with baseline '{baseline}' \
             and no compared check failed: {}",
            undecided.join("; ")
        )
    } else {
        summarize(&component, &baseline, &findings)
    };
    let mut unknown_checks = undecided;
    unknown_checks.extend(notes);
    let report = ComponentDiffReport {
        summary,
        component: component.into(),
        baseline: baseline.into(),
        findings,
        unknown_checks,
        viewport,
    };
    let detail = serde_json::to_string(&report).expect("a ComponentDiffReport always serializes");
    if unknown {
        AssertionResult::unknown(a, UnknownCode::InputMissing, detail)
    } else if failed {
        AssertionResult::fail(a, detail)
    } else {
        AssertionResult::pass_with(a, detail)
    }
}

/// The report's one-line summary. Each finding kind is named once, in order
/// of first appearance, however the findings interleave.
fn summarize(component: &str, baseline: &str, findings: &[ComponentFinding]) -> String {
    if findings.is_empty() {
        return format!("component '{component}' renders in situ like baseline '{baseline}'");
    }
    let mut kinds: Vec<&str> = Vec::new();
    for k in findings.iter().map(|f| kind_name(f.kind)) {
        if !kinds.contains(&k) {
            kinds.push(k);
        }
    }
    format!(
        "component '{component}' diverges in situ from baseline '{baseline}': {} finding(s) ({})",
        findings.len(),
        kinds.join(", ")
    )
}

fn kind_name(k: ComponentFindingKind) -> &'static str {
    match k {
        ComponentFindingKind::Size => "size",
        ComponentFindingKind::Clipped => "clipped",
        ComponentFindingKind::ChildIdentity => "child_identity",
        ComponentFindingKind::StyleDelta => "style_delta",
        ComponentFindingKind::PropsDelta => "props_delta",
        ComponentFindingKind::PortalMismatch => "portal_mismatch",
    }
}

struct DiffOptions<'a> {
    /// `None` when the requested tolerance was invalid: `size` is skipped.
    size_tolerance_pct: Option<f64>,
    style_props: &'a [String],
    /// Compare props: requested AND both sides carry them.
    compare_props: bool,
    /// Run the per-node checks: both sides have a node tree with a root node.
    per_node: bool,
}

/// Both captures' root bboxes (baseline, in situ), when both are usable.
type Roots = Option<(CaptureRect, CaptureRect)>;

fn diff(
    entry: &ComponentBaseline,
    in_situ: &ComponentCapture,
    roots: Roots,
    opts: &DiffOptions<'_>,
    undecided: &mut Vec<String>,
) -> (Vec<ComponentFinding>, Vec<String>) {
    let base = &entry.capture;
    let mut findings = Vec::new();
    let mut notes = Vec::new();

    if let (Some((b_root, s_root)), Some(tol)) = (roots, opts.size_tolerance_pct) {
        size_check(
            base,
            in_situ,
            (b_root, s_root),
            tol,
            &mut findings,
            &mut notes,
            undecided,
        );
    }
    if opts.per_node {
        if let Some((b_root, s_root)) = roots {
            clip_check(
                base,
                in_situ,
                b_root,
                s_root,
                &mut findings,
                &mut notes,
                undecided,
            );
        }
        let b_nodes = by_path(base);
        let s_nodes = by_path(in_situ);
        let retagged = identity_check(base, in_situ, &b_nodes, &s_nodes, &mut findings, &mut notes);
        style_check(
            in_situ,
            &b_nodes,
            &retagged,
            opts.style_props,
            &mut findings,
            &mut notes,
        );
        portal_check(in_situ, &b_nodes, &retagged, &mut findings);
    }
    if opts.compare_props {
        props_check(base, in_situ, &mut findings);
    }

    // Checks the SDK itself could not perform are checks this comparison
    // could not run either: undecided, not mere notes.
    for n in &base.unknown_checks {
        undecided.push(format!("baseline capture: {n}"));
    }
    for n in &in_situ.unknown_checks {
        undecided.push(format!("in-situ capture: {n}"));
    }
    (findings, notes)
}

fn by_path(c: &ComponentCapture) -> BTreeMap<&str, &ComponentCaptureNode> {
    c.nodes.iter().map(|n| (n.path.as_str(), n)).collect()
}

/// `path` lies strictly below `ancestor` in the tree.
fn is_descendant(path: &str, ancestor: &str) -> bool {
    if ancestor.is_empty() {
        !path.is_empty()
    } else {
        path.len() > ancestor.len()
            && path.starts_with(ancestor)
            && path.as_bytes()[ancestor.len()] == b'/'
    }
}

/// The parent of a tree path; `None` for the root (`""`).
fn parent_path(path: &str) -> Option<&str> {
    if path.is_empty() {
        None
    } else {
        Some(path.rsplit_once('/').map_or("", |(p, _)| p))
    }
}

fn below_any(path: &str, reported: &[String]) -> bool {
    reported.iter().any(|r| is_descendant(path, r))
}

fn at_or_below_any(path: &str, reported: &[String]) -> bool {
    reported.iter().any(|r| r == path || is_descendant(path, r))
}

fn rect_json(r: &CaptureRect) -> Value {
    json!({"x": r.x, "y": r.y, "width": r.width, "height": r.height})
}

/// Relative difference of `other` from `reference`, as a fraction.
fn rel_diff(reference: f64, other: f64) -> f64 {
    if reference == 0.0 {
        if other == 0.0 {
            0.0
        } else {
            f64::INFINITY
        }
    } else {
        (other - reference).abs() / reference.abs()
    }
}

/// `pct` is beyond `tol_pct`, with a relative epsilon so a difference that
/// is exactly the tolerance in decimal (100 -> 107 at 7%, which computes as
/// 7.000000000000001) is not read as beyond it. The tolerance is exclusive.
fn exceeds_tolerance(pct: f64, tol_pct: f64) -> bool {
    pct > tol_pct + tol_pct.max(1.0) * 1e-9
}

fn ratio(num: f64, den: Option<f64>) -> Option<f64> {
    den.filter(|d| *d > 0.0).map(|d| num / d)
}

/// `size`: raised only when, on one axis, BOTH the absolute root size AND the
/// root-to-parent-content-box fill ratio differ beyond tolerance. A component
/// that fills its parent on both sides (a fluid component shrink-wrapped in a
/// centered story and filling a page column) or keeps its absolute size on
/// both sides is not stretched.
///
/// Without a usable parent content box on a side (absent, or zero-sized) the
/// fill ratio cannot be computed. An axis whose absolute size is within
/// tolerance is still decided (not stretched; a note says the fill ratio was
/// not compared). An axis whose absolute size differs is UNDECIDED: absolute
/// size alone would read every fluid component as stretched, so it is pushed
/// to `undecided` as `size: fill ratio not compared`, never failed.
fn size_check(
    base: &ComponentCapture,
    in_situ: &ComponentCapture,
    (b, s): (CaptureRect, CaptureRect),
    tol_pct: f64,
    findings: &mut Vec<ComponentFinding>,
    notes: &mut Vec<String>,
    undecided: &mut Vec<String>,
) {
    let bp = base.parent_content_box();
    let sp = in_situ.parent_content_box();
    let axes = [
        (
            "width",
            b.width,
            s.width,
            bp.map(|p| p.width),
            sp.map(|p| p.width),
        ),
        (
            "height",
            b.height,
            s.height,
            bp.map(|p| p.height),
            sp.map(|p| p.height),
        ),
    ];
    let mut reasons = Vec::new();
    let mut fill_unknown = Vec::new();
    let mut size_undecided = Vec::new();
    for (axis, b_abs, s_abs, b_par, s_par) in axes {
        let abs_pct = rel_diff(b_abs, s_abs) * 100.0;
        let abs_differs = exceeds_tolerance(abs_pct, tol_pct);
        match (ratio(b_abs, b_par), ratio(s_abs, s_par)) {
            (Some(bf), Some(sf)) => {
                if abs_differs && exceeds_tolerance(rel_diff(bf, sf) * 100.0, tol_pct) {
                    let verb = if s_abs > b_abs {
                        "stretched"
                    } else {
                        "constrained"
                    };
                    reasons.push(format!(
                        "{verb} on {axis}: {b_abs}px -> {s_abs}px, fill ratio {bf:.2} -> \
                         {sf:.2} of the parent content box"
                    ));
                }
            }
            _ if abs_differs => {
                size_undecided.push(format!("{axis} {b_abs}px -> {s_abs}px ({abs_pct:.1}%)"))
            }
            _ => fill_unknown.push(axis),
        }
    }
    let side = match (bp.is_some(), sp.is_some()) {
        (false, false) => "either side",
        (false, true) => "the baseline side",
        (true, false) => "the in-situ side",
        _ => "one side (zero-sized)",
    };
    if !size_undecided.is_empty() {
        undecided.push(format!(
            "size: fill ratio not compared: no usable parentContentBox on {side}, and the \
             absolute size differs beyond tolerance ({}), so whether the component is \
             stretched is unknown",
            size_undecided.join("; ")
        ));
    }
    if !fill_unknown.is_empty() {
        notes.push(format!(
            "size: no usable parentContentBox on {side} for {}; the fill ratio was not \
             compared, and the absolute size is within tolerance there",
            fill_unknown.join(", ")
        ));
    }
    if reasons.is_empty() {
        return;
    }
    let side_json = |r: &CaptureRect, p: Option<CaptureRect>| {
        json!({
            "width": r.width,
            "height": r.height,
            "parentContentBox": p.as_ref().map(rect_json),
            "fillRatio": {
                "width": ratio(r.width, p.map(|p| p.width)),
                "height": ratio(r.height, p.map(|p| p.height)),
            },
        })
    };
    let parent = sp
        .map(|p| format!("in-situ parent content box {}x{}", p.width, p.height))
        .unwrap_or_else(|| "no in-situ parent content box".to_string());
    findings.push(ComponentFinding {
        kind: ComponentFindingKind::Size,
        path: String::new(),
        message: format!(
            "root {} (tolerance {tol_pct}%; {parent})",
            reasons.join("; ")
        ),
        baseline: side_json(&b, bp),
        in_situ: side_json(&s, sp),
    });
}

/// The axes (x, y) on which some clip ancestor of `c` clips `node_page`.
fn clipped_axes(c: &ComponentCapture, node_page: &CaptureRect) -> (bool, bool) {
    c.clip_ancestors.iter().fold((false, false), |(x, y), anc| {
        let Some(rect) = anc.rect.filter(CaptureRect::is_finite) else {
            return (x, y);
        };
        let past_x = anc.clips_x()
            && (node_page.x < rect.x - CLIP_EPSILON_PX
                || node_page.right() > rect.right() + CLIP_EPSILON_PX);
        let past_y = anc.clips_y()
            && (node_page.y < rect.y - CLIP_EPSILON_PX
                || node_page.bottom() > rect.bottom() + CLIP_EPSILON_PX);
        (x || past_x, y || past_y)
    })
}

/// The first clip ancestor in `c` that `node_page` extends past, if any.
fn clipping_ancestor(
    c: &ComponentCapture,
    node_page: &CaptureRect,
) -> Option<(usize, CaptureRect)> {
    c.clip_ancestors.iter().enumerate().find_map(|(i, anc)| {
        let rect = anc.rect.filter(CaptureRect::is_finite)?;
        let past_x = anc.clips_x()
            && (node_page.x < rect.x - CLIP_EPSILON_PX
                || node_page.right() > rect.right() + CLIP_EPSILON_PX);
        let past_y = anc.clips_y()
            && (node_page.y < rect.y - CLIP_EPSILON_PX
                || node_page.bottom() > rect.bottom() + CLIP_EPSILON_PX);
        (past_x || past_y).then_some((i, rect))
    })
}

/// The first clip ancestor in `c` that `node_page` extends past along an
/// axis whose overflow is `kind` (one that scrolls, or one that is
/// unreported), if any, with the axis or axes it extends past on.
fn overflow_past<'c>(
    c: &'c ComponentCapture,
    node_page: &CaptureRect,
    kind: OverflowAxis,
) -> Option<(&'c ClipAncestor, &'static str)> {
    let soft = |axis: OverflowAxis| axis == kind;
    c.clip_ancestors.iter().find_map(|anc| {
        let rect = anc.rect.filter(CaptureRect::is_finite)?;
        let past_x = soft(anc.axis_x())
            && (node_page.x < rect.x - CLIP_EPSILON_PX
                || node_page.right() > rect.right() + CLIP_EPSILON_PX);
        let past_y = soft(anc.axis_y())
            && (node_page.y < rect.y - CLIP_EPSILON_PX
                || node_page.bottom() > rect.bottom() + CLIP_EPSILON_PX);
        let axis = match (past_x, past_y) {
            (true, true) => "x and y",
            (true, false) => "x",
            (false, true) => "y",
            (false, false) => return None,
        };
        Some((anc, axis))
    })
}

/// Whether an ancestor might clip on some axis: one axis clips (`hidden`,
/// `clip`) or is unreported. A scroll container (`auto`, `scroll`, `overlay`)
/// or a `visible` axis never clips.
fn may_clip(anc: &ClipAncestor) -> bool {
    let might = |axis| matches!(axis, OverflowAxis::Clips | OverflowAxis::Unreported);
    might(anc.axis_x()) || might(anc.axis_y())
}

/// How the baseline (isolated) render treated the node at a path the in-situ
/// render clips.
enum BaselineClip {
    /// Clipped in isolation too: not a finding.
    Clipped,
    /// Known not clipped in isolation (or absent there): a finding.
    NotClipped,
    /// Cannot be told, with the reason.
    Undecided(String),
}

/// `situ_axes` are the axes (x, y) the in-situ render clips the node on. A
/// baseline clip covers the in-situ one only on the same axis: clipped on x
/// in isolation does not excuse a new clip on y.
fn baseline_clip(
    base: &ComponentCapture,
    node: Option<&&ComponentCaptureNode>,
    b_root: CaptureRect,
    situ_axes: (bool, bool),
) -> BaselineClip {
    let Some(node) = node else {
        // Absent in isolation: `child_identity` reports that; the clip is new.
        return BaselineClip::NotClipped;
    };
    let Some(rel) = node.bbox.filter(CaptureRect::is_finite) else {
        return BaselineClip::Undecided("the node has no bbox in the baseline capture".into());
    };
    let page = rel.offset_by(&b_root);
    let (bx, by) = clipped_axes(base, &page);
    let (sx, sy) = situ_axes;
    // An in-situ clip axis the baseline did not also clip is a new clip.
    let uncovered = (sx && !bx) || (sy && !by);
    if !uncovered {
        return BaselineClip::Clipped;
    }
    if let Some((anc, axis)) = overflow_past(base, &page, OverflowAxis::Unreported) {
        return BaselineClip::Undecided(format!(
            "in isolation it extends outside clip ancestor '{}' on {axis}, whose overflow is \
             unreported or unrecognized",
            anc.descriptor
        ));
    }
    if base
        .clip_ancestors
        .iter()
        .any(|a| may_clip(a) && a.rect.filter(CaptureRect::is_finite).is_none())
    {
        return BaselineClip::Undecided(
            "a baseline clip ancestor has no rect, so whether it clipped the node in \
             isolation is unknown"
                .into(),
        );
    }
    BaselineClip::NotClipped
}

/// `clipped`: a node extends past a clip ancestor in situ, while in isolation
/// no clip ancestor clipped the node at that path. Only overflow `hidden` and
/// `clip` count as clipping: a node that extends outside a scroll container
/// (`auto`, `scroll`, `overlay`) is scrolled to, not cut off, so it is noted
/// and never a `clipped` finding.
///
/// What cannot be decided is pushed to `undecided`, which makes the answer
/// unknown when nothing else failed: a node extending past an ancestor whose
/// overflow is unreported (absent or unrecognized), an in-situ clip ancestor
/// with no rect, an in-situ node with no bbox, and an in-situ clip whose
/// baseline counterpart cannot be judged ([`baseline_clip`], which is where a
/// baseline clip ancestor with no rect is consulted). Only the topmost clipped
/// (or noted, or undecided) node of a subtree is reported, and each kind
/// keeps its own list: a scroll note or an undecided node never hides a
/// descendant that is definitely clipped.
fn clip_check(
    base: &ComponentCapture,
    in_situ: &ComponentCapture,
    b_root: CaptureRect,
    s_root: CaptureRect,
    findings: &mut Vec<ComponentFinding>,
    notes: &mut Vec<String>,
    undecided: &mut Vec<String>,
) {
    for (i, anc) in in_situ.clip_ancestors.iter().enumerate() {
        if may_clip(anc) && anc.rect.filter(CaptureRect::is_finite).is_none() {
            undecided.push(format!(
                "clipped not decided: in-situ clip ancestor {i} ('{}') has no rect, so \
                 whether it clips the component there is unknown",
                anc.descriptor
            ));
        }
    }
    if in_situ.clip_ancestors.is_empty() {
        return;
    }
    let b_nodes = by_path(base);
    let mut reported: Vec<String> = Vec::new();
    let mut undecided_paths: Vec<String> = Vec::new();
    // (path, reason) of each baseline-undecided entry: a descendant repeating
    // the same reason is folded into its ancestor's entry; a different reason
    // is recorded on its own.
    let mut baseline_undecided: Vec<(String, String)> = Vec::new();
    let mut scroll_noted: Vec<String> = Vec::new();
    let mut no_bbox: Vec<String> = Vec::new();
    let overflow = |anc: &ClipAncestor| {
        format!(
            "(overflowX {}, overflowY {})",
            anc.overflow_x.as_deref().unwrap_or("absent"),
            anc.overflow_y.as_deref().unwrap_or("absent"),
        )
    };
    for node in &in_situ.nodes {
        if below_any(&node.path, &reported) {
            continue;
        }
        let Some(rel) = node.bbox.filter(CaptureRect::is_finite) else {
            no_bbox.push(format!("'{}'", node.path));
            continue;
        };
        let page = rel.offset_by(&s_root);
        let Some((i, clip_rect)) = clipping_ancestor(in_situ, &page) else {
            if !below_any(&node.path, &undecided_paths) {
                if let Some((anc, axis)) = overflow_past(in_situ, &page, OverflowAxis::Unreported) {
                    undecided.push(format!(
                    "clipped not decided: <{}> at path '{}' extends outside clip ancestor '{}' \
                     on {axis} in situ {}, and that overflow value is unreported or \
                     unrecognized, so whether it is clipped is unknown",
                    node.tag,
                    node.path,
                    anc.descriptor,
                    overflow(anc),
                ));
                    undecided_paths.push(node.path.clone());
                    continue;
                }
            }
            if below_any(&node.path, &scroll_noted) {
                continue;
            }
            if let Some((anc, axis)) = overflow_past(in_situ, &page, OverflowAxis::Scrolls) {
                notes.push(format!(
                    "clipped: <{}> at path '{}' extends outside '{}' on {axis} in situ {}; \
                     that axis scrolls rather than clipping, so it is not a clipped finding",
                    node.tag,
                    node.path,
                    anc.descriptor,
                    overflow(anc),
                ));
                scroll_noted.push(node.path.clone());
            }
            continue;
        };
        let anc = &in_situ.clip_ancestors[i];
        let situ_axes = clipped_axes(in_situ, &page);
        match baseline_clip(base, b_nodes.get(node.path.as_str()), b_root, situ_axes) {
            BaselineClip::Clipped => continue,
            BaselineClip::Undecided(why) => {
                // Descendants are still judged: one definitely clipped below
                // this node is a finding. Only a repeat undecided entry for
                // the same subtree is suppressed.
                let repeat = baseline_undecided
                    .iter()
                    .any(|(p, r)| *r == why && is_descendant(&node.path, p));
                if !repeat {
                    undecided.push(format!(
                        "clipped not decided: <{}> at path '{}' extends past clip ancestor \
                         '{}' in situ, but {why}",
                        node.tag, node.path, anc.descriptor,
                    ));
                }
                baseline_undecided.push((node.path.clone(), why));
                continue;
            }
            BaselineClip::NotClipped => {}
        }
        findings.push(ComponentFinding {
            kind: ComponentFindingKind::Clipped,
            path: node.path.clone(),
            message: format!(
                "<{}> at path '{}' extends past clip ancestor '{}' (overflowX {}, overflowY {}) \
                 in situ; no clip ancestor clipped it in isolation",
                node.tag,
                node.path,
                anc.descriptor,
                anc.overflow_x.as_deref().unwrap_or("?"),
                anc.overflow_y.as_deref().unwrap_or("?"),
            ),
            baseline: Value::Null,
            in_situ: json!({
                "nodeRect": rect_json(&page),
                "clipAncestor": anc.descriptor,
                "clipRect": rect_json(&clip_rect),
            }),
        });
        reported.push(node.path.clone());
    }
    // A node with no bbox matters only when something in situ might clip it.
    if !no_bbox.is_empty() && in_situ.clip_ancestors.iter().any(may_clip) {
        undecided.push(format!(
            "clipped not decided: {} in-situ node(s) have no bbox ({}), so whether they are \
             clipped is unknown",
            no_bbox.len(),
            no_bbox.join(", ")
        ));
    }
}

fn node_identity_json(n: &ComponentCaptureNode) -> Value {
    json!({"tag": n.tag, "componentName": n.component_name, "registeredId": n.registered_id})
}

/// `child_identity`: tag or component name differs at the same path, or a
/// node exists on one side only. A null name on either side is UNREADABLE:
/// the name comparison is skipped and noted, never reported as a mismatch.
///
/// Only the topmost divergence of a subtree is reported: nothing beneath a
/// tag-changed path (nor beneath a one-sided path) is reported again. Returns
/// the tag-changed paths, at and beneath which the style and portal checks
/// are skipped too — the subtree is a different render, not a restyled one,
/// so the style and portal checks add no findings there (a `clipped` or root
/// `size` finding is judged independently and can still land at or beneath it).
///
/// The suppression relies on `nodes` being depth-first pre-order (parents
/// before children), as the capture contract states. Out-of-order input can
/// only OVER-report (a descendant seen before its retagged ancestor is still
/// compared), which is the safe direction.
fn identity_check(
    base: &ComponentCapture,
    in_situ: &ComponentCapture,
    b_nodes: &BTreeMap<&str, &ComponentCaptureNode>,
    s_nodes: &BTreeMap<&str, &ComponentCaptureNode>,
    findings: &mut Vec<ComponentFinding>,
    notes: &mut Vec<String>,
) -> Vec<String> {
    let mut unreadable = 0usize;
    let mut retagged: Vec<String> = Vec::new();
    for bn in &base.nodes {
        let Some(sn) = s_nodes.get(bn.path.as_str()) else {
            continue;
        };
        if below_any(&bn.path, &retagged) {
            continue;
        }
        if !bn.tag.eq_ignore_ascii_case(&sn.tag) {
            retagged.push(bn.path.clone());
            findings.push(ComponentFinding {
                kind: ComponentFindingKind::ChildIdentity,
                path: bn.path.clone(),
                message: format!(
                    "tag changed at path '{}': <{}> in isolation, <{}> in situ",
                    bn.path, bn.tag, sn.tag
                ),
                baseline: node_identity_json(bn),
                in_situ: node_identity_json(sn),
            });
            continue;
        }
        match (&bn.component_name, &sn.component_name) {
            (Some(b), Some(s)) if b != s => findings.push(ComponentFinding {
                kind: ComponentFindingKind::ChildIdentity,
                path: bn.path.clone(),
                message: format!(
                    "component changed at path '{}': {b} in isolation, {s} in situ",
                    bn.path
                ),
                baseline: node_identity_json(bn),
                in_situ: node_identity_json(sn),
            }),
            (Some(_), Some(_)) => {}
            _ => unreadable += 1,
        }
    }
    if unreadable > 0 {
        notes.push(format!(
            "child_identity: componentName unreadable (minified or redacted) on at least one \
             side at {unreadable} path(s); name comparison skipped there, tag and geometry \
             still decided"
        ));
    }

    let mut one_sided = |nodes: &[ComponentCaptureNode],
                         other: &BTreeMap<&str, &ComponentCaptureNode>,
                         missing_in_situ: bool| {
        let mut reported: Vec<String> = Vec::new();
        for n in nodes {
            if other.contains_key(n.path.as_str())
                || below_any(&n.path, &reported)
                || below_any(&n.path, &retagged)
            {
                continue;
            }
            let what = n
                .component_name
                .as_deref()
                .map(|c| format!("<{}> ({c})", n.tag))
                .unwrap_or_else(|| format!("<{}>", n.tag));
            let (message, baseline, in_situ) = if missing_in_situ {
                (
                    format!(
                        "{what} at path '{}' renders in isolation but is missing in situ",
                        n.path
                    ),
                    node_identity_json(n),
                    Value::Null,
                )
            } else {
                (
                    format!(
                        "{what} at path '{}' renders in situ but not in isolation",
                        n.path
                    ),
                    Value::Null,
                    node_identity_json(n),
                )
            };
            findings.push(ComponentFinding {
                kind: ComponentFindingKind::ChildIdentity,
                path: n.path.clone(),
                message,
                baseline,
                in_situ,
            });
            reported.push(n.path.clone());
        }
    };
    one_sided(&base.nodes, s_nodes, true);
    one_sided(&in_situ.nodes, b_nodes, false);
    retagged
}

/// CSS properties that inherit by default, in the camelCase spelling the
/// capture uses. Only these cascade from a parent to its children, so only
/// these get cascade suppression in [`style_check`]; every `font*` longhand
/// is matched by prefix in [`is_inherited`].
const INHERITED_STYLE_PROPS: [&str; 20] = [
    "color",
    "lineHeight",
    "letterSpacing",
    "wordSpacing",
    "textAlign",
    "textIndent",
    "textTransform",
    "textShadow",
    "visibility",
    "whiteSpace",
    "wordBreak",
    "overflowWrap",
    "direction",
    "writingMode",
    "cursor",
    "listStyle",
    "listStyleType",
    "listStylePosition",
    "quotes",
    "tabSize",
];

/// Whether `prop` (camelCase) inherits by default.
fn is_inherited(prop: &str) -> bool {
    prop == "font" || prop.starts_with("font") || INHERITED_STYLE_PROPS.contains(&prop)
}

/// `style_delta`: a `style_props` key whose (trimmed) value differs at the
/// same path. For an INHERITED property ([`is_inherited`]) a delta is not
/// repeated on a node whose PARENT carried the same delta (the value
/// cascading down); a non-inherited one (`display`, `overflow`, …) is
/// reported at every node that carries it; since that holds at every
/// step, a node is suppressed only when every path between it and the
/// reported ancestor carried the same delta, so a reset in between lets a
/// re-divergence report. Nothing at or beneath a tag-changed path is
/// compared.
///
/// The cascade suppression relies on `nodes` being depth-first pre-order
/// (parents before children), as the capture contract states: a parent's
/// delta must be recorded before its child is visited. Out-of-order input can
/// only OVER-report (a child visited first is reported rather than
/// suppressed), which is the safe direction.
fn style_check(
    in_situ: &ComponentCapture,
    b_nodes: &BTreeMap<&str, &ComponentCaptureNode>,
    retagged: &[String],
    style_props: &[String],
    findings: &mut Vec<ComponentFinding>,
    notes: &mut Vec<String>,
) {
    // (path, prop) -> the trimmed (baseline, in-situ) delta that path carried.
    let mut carried: HashMap<(&str, &str), (&str, &str)> = HashMap::new();
    let mut one_sided = 0usize;
    for sn in &in_situ.nodes {
        let Some(bn) = b_nodes.get(sn.path.as_str()) else {
            continue;
        };
        if at_or_below_any(&sn.path, retagged) {
            continue;
        }
        for prop in style_props {
            match (bn.styles.get(prop), sn.styles.get(prop)) {
                (Some(bv), Some(sv)) if bv.trim() != sv.trim() => {
                    let (bv, sv) = (bv.trim(), sv.trim());
                    carried.insert((sn.path.as_str(), prop.as_str()), (bv, sv));
                    let inherited = is_inherited(prop)
                        && parent_path(&sn.path)
                            .and_then(|p| carried.get(&(p, prop.as_str())))
                            .is_some_and(|d| *d == (bv, sv));
                    if inherited {
                        continue;
                    }
                    findings.push(ComponentFinding {
                        kind: ComponentFindingKind::StyleDelta,
                        path: sn.path.clone(),
                        message: format!(
                            "{prop} at path '{}' (<{}>): {bv} in isolation, {sv} in situ",
                            sn.path, sn.tag
                        ),
                        baseline: json!({ prop.as_str(): bv }),
                        in_situ: json!({ prop.as_str(): sv }),
                    });
                }
                (Some(_), None) | (None, Some(_)) => one_sided += 1,
                _ => {}
            }
        }
    }
    if one_sided > 0 {
        notes.push(format!(
            "style_delta: {one_sided} style value(s) captured on one side only; not compared"
        ));
    }
}

/// `props_delta`: top-level prop keys whose (scrubbed) values differ.
fn props_check(
    base: &ComponentCapture,
    in_situ: &ComponentCapture,
    findings: &mut Vec<ComponentFinding>,
) {
    // `diff` calls this only when both sides carry props (`evaluate` records
    // `props not compared` as an undecided check otherwise), so this
    // early return is defensive.
    let (Some(bp), Some(sp)) = (&base.props, &in_situ.props) else {
        return;
    };
    let (Some(bo), Some(so)) = (bp.as_object(), sp.as_object()) else {
        if bp != sp {
            findings.push(ComponentFinding {
                kind: ComponentFindingKind::PropsDelta,
                path: "props".to_string(),
                message: "props differ".to_string(),
                baseline: bp.clone(),
                in_situ: sp.clone(),
            });
        }
        return;
    };
    let mut keys: Vec<&String> = bo.keys().chain(so.keys()).collect();
    keys.sort();
    keys.dedup();
    for k in keys {
        let (bv, sv) = (bo.get(k), so.get(k));
        if bv != sv {
            findings.push(ComponentFinding {
                kind: ComponentFindingKind::PropsDelta,
                path: format!("props.{k}"),
                message: format!("prop '{k}' differs between isolation and situ"),
                baseline: bv.cloned().unwrap_or(Value::Null),
                in_situ: sv.cloned().unwrap_or(Value::Null),
            });
        }
    }
}

/// `portal_mismatch`: at the same path, a portal on one side only or a
/// different mount target. Nothing at or beneath a tag-changed path is
/// compared.
fn portal_check(
    in_situ: &ComponentCapture,
    b_nodes: &BTreeMap<&str, &ComponentCaptureNode>,
    retagged: &[String],
    findings: &mut Vec<ComponentFinding>,
) {
    for sn in &in_situ.nodes {
        let Some(bn) = b_nodes.get(sn.path.as_str()) else {
            continue;
        };
        if at_or_below_any(&sn.path, retagged) {
            continue;
        }
        let message = match (&bn.portal, &sn.portal) {
            (None, None) => continue,
            (Some(b), Some(s)) if b.mount_target == s.mount_target => continue,
            (Some(b), Some(s)) => format!(
                "portal at path '{}' mounts into '{}' in isolation but '{}' in situ",
                sn.path, b.mount_target, s.mount_target
            ),
            (Some(b), None) => format!(
                "path '{}' is a portal into '{}' in isolation but renders inline in situ",
                sn.path, b.mount_target
            ),
            (None, Some(s)) => format!(
                "path '{}' renders inline in isolation but is a portal into '{}' in situ",
                sn.path, s.mount_target
            ),
        };
        findings.push(ComponentFinding {
            kind: ComponentFindingKind::PortalMismatch,
            path: sn.path.clone(),
            message,
            baseline: serde_json::to_value(&bn.portal).unwrap_or(Value::Null),
            in_situ: serde_json::to_value(&sn.portal).unwrap_or(Value::Null),
        });
    }
}

fn compare_viewports(base: &ComponentCapture, in_situ: &ComponentCapture) -> ViewportComparison {
    let vp_eq = base.viewport.zip(in_situ.viewport).map(|(b, s)| b == s);
    let dpr_eq = base.dpr.zip(in_situ.dpr).map(|(b, s)| b == s);
    let matches = match (vp_eq, dpr_eq) {
        (Some(false), _) | (_, Some(false)) => Some(false),
        (Some(true), Some(true)) => Some(true),
        _ => None,
    };
    ViewportComparison {
        baseline: base.viewport,
        in_situ: in_situ.viewport,
        baseline_dpr: base.dpr,
        in_situ_dpr: in_situ.dpr,
        matches,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::assertions::{evaluate, AssertionOutcome};
    use crate::component_capture::{
        CapturePortal, CaptureRoot, CaptureTarget, CaptureTargetKind, ClipAncestor,
    };

    fn rect(x: f64, y: f64, width: f64, height: f64) -> CaptureRect {
        CaptureRect {
            x,
            y,
            width,
            height,
        }
    }

    fn node(path: &str, tag: &str, name: Option<&str>, bbox: CaptureRect) -> ComponentCaptureNode {
        ComponentCaptureNode {
            path: path.to_string(),
            tag: tag.to_string(),
            component_name: name.map(str::to_string),
            bbox: Some(bbox),
            ..Default::default()
        }
    }

    /// A date-range picker: a root div holding a label and a DatePicker
    /// input, root at `root` (page coords) inside `parent`.
    fn capture(root: CaptureRect, parent: Option<CaptureRect>) -> ComponentCapture {
        let w = root.width;
        ComponentCapture {
            schema_version: Some(1),
            target: Some(CaptureTarget {
                kind: CaptureTargetKind::Component,
                id: "date-range-picker".to_string(),
            }),
            source_sha: Some("abc123".to_string()),
            viewport: Some(CaptureViewport {
                width: 1280.0,
                height: 800.0,
            }),
            dpr: Some(1.0),
            root: Some(CaptureRoot {
                bbox: Some(root),
                parent_content_box: parent,
            }),
            nodes: vec![
                node(
                    "",
                    "div",
                    Some("DateRangePicker"),
                    rect(0.0, 0.0, w, root.height),
                ),
                node(
                    "0",
                    "label",
                    Some("DateRangePicker"),
                    rect(0.0, 0.0, 80.0, 20.0),
                ),
                node("1", "div", Some("DatePicker"), rect(0.0, 20.0, w, 20.0)),
                node(
                    "1/0",
                    "input",
                    Some("DatePicker"),
                    rect(0.0, 20.0, 120.0, 20.0),
                ),
            ],
            ..Default::default()
        }
    }

    fn assertion() -> Assertion {
        serde_json::from_value(json!({
            "type": "matches_isolated_render",
            "component": "date-range-picker",
            "baseline": "storybook/DateRangePicker--default@abc123",
        }))
        .unwrap()
    }

    const NAME: &str = "storybook/DateRangePicker--default@abc123";

    fn baseline(c: ComponentCapture) -> ComponentBaseline {
        ComponentBaseline {
            name: NAME.to_string(),
            story_id: Some("DateRangePicker--default".to_string()),
            source_sha: None,
            captured_at: None,
            capture: c,
        }
    }

    fn run_with(a: &Assertion, base: ComponentCapture, situ: ComponentCapture) -> AssertionResult {
        let baselines = HashMap::from([(NAME.to_string(), baseline(base))]);
        let captures = HashMap::from([("date-range-picker".to_string(), situ)]);
        evaluate(
            a,
            &EvalContext {
                component_baselines: Some(&baselines),
                component_captures: Some(&captures),
                ..Default::default()
            },
        )
    }

    fn run(base: ComponentCapture, situ: ComponentCapture) -> AssertionResult {
        run_with(&assertion(), base, situ)
    }

    fn report(r: &AssertionResult) -> ComponentDiffReport {
        serde_json::from_str(r.detail.as_deref().expect("detail")).expect("detail is a report")
    }

    fn kinds(r: &AssertionResult) -> Vec<ComponentFindingKind> {
        report(r).findings.iter().map(|f| f.kind).collect()
    }

    fn page() -> CaptureRect {
        rect(100.0, 300.0, 400.0, 40.0)
    }

    fn parent_of(r: CaptureRect) -> Option<CaptureRect> {
        Some(rect(r.x, r.y, r.width, 600.0))
    }

    #[test]
    fn defaults_apply_when_omitted() {
        let Assertion::MatchesIsolatedRender {
            size_tolerance_pct,
            style_props,
            compare_props,
            ..
        } = assertion()
        else {
            panic!()
        };
        assert_eq!(size_tolerance_pct, 5.0);
        assert_eq!(&*style_props, ["display", "fontSize", "color", "overflow"]);
        assert!(!compare_props);
    }

    #[test]
    fn identical_renders_pass() {
        let r = run(
            capture(page(), parent_of(page())),
            capture(page(), parent_of(page())),
        );
        assert_eq!(r.outcome, AssertionOutcome::Passed, "{:?}", r.detail);
        assert!(report(&r).findings.is_empty());
    }

    // ---- size ---------------------------------------------------------

    #[test]
    fn size_stretched_when_absolute_and_fill_ratio_both_differ() {
        // Isolation: 200 wide in a 400-wide parent (fill 0.5).
        // In situ:  1000 wide in a 1000-wide parent (fill 1.0).
        let b = rect(0.0, 0.0, 200.0, 40.0);
        let s = rect(0.0, 0.0, 1000.0, 40.0);
        let r = run(
            capture(b, Some(rect(0.0, 0.0, 400.0, 40.0))),
            capture(s, Some(rect(0.0, 0.0, 1000.0, 40.0))),
        );
        assert_eq!(r.outcome, AssertionOutcome::Failed);
        let rep = report(&r);
        let f = rep
            .findings
            .iter()
            .find(|f| f.kind == ComponentFindingKind::Size)
            .expect("size finding");
        assert_eq!(f.path, "");
        assert!(f.message.contains("stretched on width"), "{}", f.message);
        assert!(
            f.message.contains("parent content box 1000x40"),
            "{}",
            f.message
        );
        assert_eq!(f.baseline["fillRatio"]["width"], json!(0.5));
        assert_eq!(f.in_situ["fillRatio"]["width"], json!(1.0));
        assert!(!f.message.contains("height"), "{}", f.message);
    }

    /// The runner's Storybook preview is `layout: "centered"` plus a padding
    /// wrapper, so a `width:100%` component shrink-wraps in isolation (fills
    /// its 320px wrapper) and fills a 1000px page column in situ. The
    /// absolute width differs by 212% but the fill ratio is 1.0 on both
    /// sides, so it is not stretched: no `size` finding.
    #[test]
    fn centered_story_vs_full_width_page_is_not_a_size_finding() {
        let b = rect(480.0, 200.0, 320.0, 40.0);
        let s = rect(0.0, 64.0, 1000.0, 40.0);
        let r = run(
            capture(b, Some(rect(480.0, 200.0, 320.0, 40.0))),
            capture(s, Some(rect(0.0, 64.0, 1000.0, 40.0))),
        );
        assert_eq!(r.outcome, AssertionOutcome::Passed, "{:?}", r.detail);
        assert!(!kinds(&r).contains(&ComponentFindingKind::Size));
    }

    #[test]
    fn same_absolute_size_in_a_different_parent_is_not_a_size_finding() {
        let b = rect(0.0, 0.0, 200.0, 40.0);
        let r = run(
            capture(b, Some(rect(0.0, 0.0, 400.0, 40.0))),
            capture(b, Some(rect(0.0, 0.0, 1000.0, 600.0))),
        );
        assert_eq!(r.outcome, AssertionOutcome::Passed, "{:?}", r.detail);
    }

    #[test]
    fn within_tolerance_is_not_a_size_finding() {
        let b = rect(0.0, 0.0, 200.0, 40.0);
        let s = rect(0.0, 0.0, 208.0, 40.0); // +4%
        let r = run(
            capture(b, Some(rect(0.0, 0.0, 400.0, 40.0))),
            capture(s, Some(rect(0.0, 0.0, 800.0, 40.0))),
        );
        assert_eq!(r.outcome, AssertionOutcome::Passed, "{:?}", r.detail);
    }

    #[test]
    fn missing_parent_box_leaves_a_size_difference_undecided() {
        let b = rect(0.0, 0.0, 200.0, 40.0);
        let s = rect(0.0, 0.0, 1000.0, 40.0);
        let r = run(
            capture(b, None),
            capture(s, Some(rect(0.0, 0.0, 1000.0, 40.0))),
        );
        assert_input_missing(&r, "size: fill ratio not compared");
        let rep = report(&r);
        assert!(!kinds(&r).contains(&ComponentFindingKind::Size));
        assert!(
            rep.unknown_checks
                .iter()
                .any(|n| n.starts_with("size: fill ratio not compared")
                    && n.contains("baseline side")
                    && n.contains("width")),
            "{:?}",
            rep.unknown_checks
        );

        // Another finding outranks the undecided size: Failed, size noted.
        let mut base = capture(b, None);
        let mut situ = capture(s, Some(rect(0.0, 0.0, 1000.0, 40.0)));
        base.nodes[3].styles.insert("color".into(), "red".into());
        situ.nodes[3].styles.insert("color".into(), "blue".into());
        let r = run(base, situ);
        assert_eq!(r.outcome, AssertionOutcome::Failed, "{:?}", r.detail);
        assert_eq!(kinds(&r), vec![ComponentFindingKind::StyleDelta]);
        assert!(report(&r)
            .unknown_checks
            .iter()
            .any(|n| n.starts_with("size: fill ratio not compared")));

        // Absolute size within tolerance: decided without a fill ratio.
        let r = run(capture(b, None), capture(b, None));
        assert_eq!(r.outcome, AssertionOutcome::Passed, "{:?}", r.detail);
    }

    // ---- clipped ------------------------------------------------------

    #[test]
    fn clipped_by_an_in_situ_ancestor_absent_in_isolation() {
        let mut situ = capture(page(), parent_of(page()));
        // A 300-wide clipping pane at the root's origin: the 400-wide root
        // overflows it horizontally, and `hidden` cuts it off.
        situ.clip_ancestors = vec![ClipAncestor {
            descriptor: "div.scroll-pane".to_string(),
            overflow_x: Some("hidden".to_string()),
            overflow_y: Some("clip".to_string()),
            rect: Some(rect(100.0, 300.0, 300.0, 600.0)),
        }];
        let r = run(capture(page(), parent_of(page())), situ);
        assert_eq!(r.outcome, AssertionOutcome::Failed);
        let rep = report(&r);
        let clipped: Vec<_> = rep
            .findings
            .iter()
            .filter(|f| f.kind == ComponentFindingKind::Clipped)
            .collect();
        // Only the topmost clipped node of the subtree is reported.
        assert_eq!(clipped.len(), 1, "{clipped:?}");
        assert_eq!(clipped[0].path, "");
        assert!(clipped[0].message.contains("div.scroll-pane"));
        assert_eq!(clipped[0].in_situ["clipRect"]["width"], json!(300.0));
        // Node rect converted to page coordinates via the root bbox.
        assert_eq!(clipped[0].in_situ["nodeRect"]["x"], json!(100.0));
    }

    #[test]
    fn a_clip_present_in_isolation_too_is_not_a_finding() {
        let clip = |r: CaptureRect| ClipAncestor {
            descriptor: "div.frame".to_string(),
            overflow_x: Some("hidden".to_string()),
            overflow_y: Some("hidden".to_string()),
            rect: Some(rect(r.x, r.y, 300.0, 600.0)),
        };
        let mut base = capture(page(), parent_of(page()));
        base.clip_ancestors = vec![clip(page())];
        let mut situ = capture(page(), parent_of(page()));
        situ.clip_ancestors = vec![clip(page())];
        let r = run(base, situ);
        assert_eq!(r.outcome, AssertionOutcome::Passed, "{:?}", r.detail);
    }

    #[test]
    fn overflow_visible_axis_does_not_clip() {
        let mut situ = capture(page(), parent_of(page()));
        situ.clip_ancestors = vec![ClipAncestor {
            descriptor: "div.v-scroll".to_string(),
            overflow_x: Some("visible".to_string()),
            overflow_y: Some("auto".to_string()),
            rect: Some(rect(100.0, 300.0, 300.0, 600.0)),
        }];
        let r = run(capture(page(), parent_of(page())), situ);
        assert_eq!(r.outcome, AssertionOutcome::Passed, "{:?}", r.detail);
    }

    // ---- child_identity -----------------------------------------------

    #[test]
    fn fallback_input_instead_of_datepicker_is_a_child_identity_finding() {
        let mut situ = capture(page(), parent_of(page()));
        // A plain <input> rendered where the DatePicker's wrapper div was.
        situ.nodes[2] = node(
            "1",
            "input",
            Some("DateRangePicker"),
            rect(0.0, 20.0, 400.0, 20.0),
        );
        situ.nodes.truncate(3);
        let r = run(capture(page(), parent_of(page())), situ);
        assert_eq!(r.outcome, AssertionOutcome::Failed);
        let rep = report(&r);
        let ids: Vec<_> = rep
            .findings
            .iter()
            .filter(|f| f.kind == ComponentFindingKind::ChildIdentity)
            .collect();
        assert!(
            ids.iter().any(|f| f.path == "1"
                && f.message.contains("<div>")
                && f.message.contains("<input>")),
            "{ids:?}"
        );
        // Topmost only: the DatePicker's inner input beneath the retagged
        // path is not reported again as missing.
        assert_eq!(ids.len(), 1, "{ids:?}");
        assert_eq!(rep.findings.len(), 1, "{:?}", rep.findings);
        assert_eq!(ids[0].baseline["tag"], json!("div"));
    }

    #[test]
    fn a_retagged_node_yields_only_its_child_identity_finding() {
        let mut base = capture(page(), parent_of(page()));
        let mut situ = capture(page(), parent_of(page()));
        situ.nodes[2].tag = "section".to_string();
        base.nodes[2].styles.insert("color".into(), "red".into());
        situ.nodes[2].styles.insert("color".into(), "blue".into());
        situ.nodes[2].portal = Some(CapturePortal {
            mount_target: "body".to_string(),
            react_parent_path: None,
        });
        let r = run(base, situ);
        let rep = report(&r);
        assert_eq!(rep.findings.len(), 1, "{:?}", rep.findings);
        assert_eq!(rep.findings[0].kind, ComponentFindingKind::ChildIdentity);
        assert_eq!(rep.findings[0].path, "1");
    }

    #[test]
    fn the_summary_names_each_kind_once_however_findings_interleave() {
        let f = |kind| ComponentFinding {
            kind,
            path: String::new(),
            message: String::new(),
            baseline: Value::Null,
            in_situ: Value::Null,
        };
        let findings = [
            f(ComponentFindingKind::StyleDelta),
            f(ComponentFindingKind::ChildIdentity),
            f(ComponentFindingKind::StyleDelta),
        ];
        assert_eq!(
            summarize("c", "b", &findings),
            "component 'c' diverges in situ from baseline 'b': 3 finding(s) \
             (style_delta, child_identity)"
        );
    }

    #[test]
    fn nothing_beneath_a_tag_changed_path_is_reported() {
        let mut base = capture(page(), parent_of(page()));
        let mut situ = capture(page(), parent_of(page()));
        // Path "1" is a <section> in situ; beneath it the child's name,
        // style and portal all differ too, and none of it is reported.
        situ.nodes[2].tag = "section".to_string();
        situ.nodes[3].component_name = Some("Other".to_string());
        base.nodes[3].styles.insert("color".into(), "red".into());
        situ.nodes[3].styles.insert("color".into(), "blue".into());
        situ.nodes[3].portal = Some(CapturePortal {
            mount_target: "body".to_string(),
            react_parent_path: None,
        });
        situ.nodes
            .push(node("1/1", "b", None, rect(0.0, 0.0, 1.0, 1.0)));
        let r = run(base, situ);
        let rep = report(&r);
        assert_eq!(rep.findings.len(), 1, "{:?}", rep.findings);
        assert_eq!(rep.findings[0].kind, ComponentFindingKind::ChildIdentity);
        assert_eq!(rep.findings[0].path, "1");
    }

    #[test]
    fn a_different_component_name_at_the_same_path_is_a_finding() {
        let mut situ = capture(page(), parent_of(page()));
        situ.nodes[3].component_name = Some("NativeDateInput".to_string());
        let r = run(capture(page(), parent_of(page())), situ);
        assert_eq!(r.outcome, AssertionOutcome::Failed);
        let rep = report(&r);
        assert_eq!(rep.findings.len(), 1, "{:?}", rep.findings);
        let f = &rep.findings[0];
        assert_eq!(f.kind, ComponentFindingKind::ChildIdentity);
        assert_eq!(f.path, "1/0");
        assert_eq!(f.baseline["componentName"], json!("DatePicker"));
        assert_eq!(f.in_situ["componentName"], json!("NativeDateInput"));
    }

    #[test]
    fn an_extra_node_in_situ_is_a_finding() {
        let mut situ = capture(page(), parent_of(page()));
        situ.nodes.push(node(
            "2",
            "span",
            Some("ErrorBoundaryFallback"),
            rect(0.0, 40.0, 10.0, 10.0),
        ));
        let r = run(capture(page(), parent_of(page())), situ);
        assert_eq!(r.outcome, AssertionOutcome::Failed);
        let f = &report(&r).findings[0];
        assert_eq!(f.kind, ComponentFindingKind::ChildIdentity);
        assert_eq!(f.path, "2");
        assert!(f.message.contains("not in isolation"));
        assert_eq!(f.baseline, Value::Null);
    }

    #[test]
    fn redacted_or_unreadable_names_are_unknown_not_a_mismatch() {
        let mut situ = capture(page(), parent_of(page()));
        for n in &mut situ.nodes {
            n.component_name = None;
        }
        // Unreadable names alone skip only the name comparison, with a note.
        let r = run(capture(page(), parent_of(page())), situ.clone());
        assert_eq!(r.outcome, AssertionOutcome::Passed, "{:?}", r.detail);
        // When the capture itself lists the check it could not perform, that
        // check is undecided and the answer is unknown.
        situ.unknown_checks = vec!["componentName: unreadable (minified or redacted)".into()];
        let r = run(capture(page(), parent_of(page())), situ);
        assert_eq!(r.outcome, AssertionOutcome::Unknown, "{:?}", r.detail);
        let rep = report(&r);
        assert!(rep.findings.is_empty());
        assert!(
            rep.unknown_checks
                .iter()
                .any(|n| n.starts_with("child_identity:") && n.contains("4 path(s)")),
            "{:?}",
            rep.unknown_checks
        );
        assert!(rep
            .unknown_checks
            .iter()
            .any(|n| n.starts_with("in-situ capture: componentName")));
    }

    #[test]
    fn unreadable_names_still_let_geometry_decide() {
        let mut situ = capture(
            rect(0.0, 0.0, 1000.0, 40.0),
            Some(rect(0.0, 0.0, 1000.0, 40.0)),
        );
        for n in &mut situ.nodes {
            n.component_name = None;
        }
        let r = run(
            capture(
                rect(0.0, 0.0, 200.0, 40.0),
                Some(rect(0.0, 0.0, 400.0, 40.0)),
            ),
            situ,
        );
        assert_eq!(r.outcome, AssertionOutcome::Failed);
        assert_eq!(kinds(&r), vec![ComponentFindingKind::Size]);
    }

    // ---- style_delta --------------------------------------------------

    #[test]
    fn a_whitelisted_style_difference_is_a_finding_and_cascades_are_not_repeated() {
        let mut base = capture(page(), parent_of(page()));
        let mut situ = capture(page(), parent_of(page()));
        for n in &mut base.nodes {
            n.styles.insert("fontSize".into(), "14px".into());
            n.styles.insert("color".into(), "rgb(0, 0, 0)".into());
            n.styles.insert("width".into(), "400px".into());
        }
        for n in &mut situ.nodes {
            // Page CSS overrides the font size on the root; it inherits down.
            n.styles.insert("fontSize".into(), "16px".into());
            n.styles.insert("color".into(), "rgb(0, 0, 0)".into());
            // `width` is not in the default style_props.
            n.styles.insert("width".into(), "999px".into());
        }
        situ.nodes[1]
            .styles
            .insert("color".into(), "rgb(255, 0, 0)".into());
        let r = run(base, situ);
        assert_eq!(r.outcome, AssertionOutcome::Failed);
        let rep = report(&r);
        let styles: Vec<_> = rep
            .findings
            .iter()
            .filter(|f| f.kind == ComponentFindingKind::StyleDelta)
            .collect();
        assert_eq!(styles.len(), 2, "{styles:?}");
        let font = styles
            .iter()
            .find(|f| f.baseline.get("fontSize").is_some())
            .unwrap();
        assert_eq!(font.path, "");
        assert_eq!(font.in_situ["fontSize"], json!("16px"));
        let color = styles
            .iter()
            .find(|f| f.baseline.get("color").is_some())
            .unwrap();
        assert_eq!(color.path, "0");
        assert_eq!(color.in_situ["color"], json!("rgb(255, 0, 0)"));
    }

    #[test]
    fn duplicate_style_props_do_not_duplicate_findings() {
        let mut base = capture(page(), parent_of(page()));
        let mut situ = capture(page(), parent_of(page()));
        base.nodes[3].styles.insert("color".into(), "red".into());
        situ.nodes[3].styles.insert("color".into(), "blue".into());
        let a: Assertion = serde_json::from_value(json!({
            "type": "matches_isolated_render",
            "component": "date-range-picker",
            "baseline": NAME,
            "style_props": ["color", "color", "fontSize", "color"],
        }))
        .unwrap();
        let r = run_with(&a, base, situ);
        assert_eq!(kinds(&r), vec![ComponentFindingKind::StyleDelta]);
    }

    /// The root carries a color delta; its child "1" matches the baseline
    /// again; "1/0" diverges with the same values as the root. The reset in
    /// between means "1/0" is reported, not suppressed as inherited.
    #[test]
    fn a_reset_in_between_lets_a_re_divergence_report() {
        let mut base = capture(page(), parent_of(page()));
        let mut situ = capture(page(), parent_of(page()));
        for n in &mut base.nodes {
            n.styles.insert("color".into(), "black".into());
        }
        for n in &mut situ.nodes {
            n.styles.insert("color".into(), "red".into());
        }
        situ.nodes[2].styles.insert("color".into(), "black".into()); // path "1" resets
        let r = run(base, situ);
        let paths: Vec<String> = report(&r)
            .findings
            .iter()
            .filter(|f| f.kind == ComponentFindingKind::StyleDelta)
            .map(|f| f.path.clone())
            .collect();
        // "0" inherits the root's delta (suppressed); "1/0" re-diverges.
        assert_eq!(paths, vec!["".to_string(), "1/0".to_string()]);
    }

    #[test]
    fn cascade_suppression_keys_on_trimmed_values() {
        let mut base = capture(page(), parent_of(page()));
        let mut situ = capture(page(), parent_of(page()));
        for n in &mut base.nodes {
            n.styles.insert("fontSize".into(), "14px".into());
        }
        for n in &mut situ.nodes {
            n.styles.insert("fontSize".into(), "16px".into());
        }
        // Whitespace-only variation down the tree is the same delta.
        base.nodes[1]
            .styles
            .insert("fontSize".into(), " 14px ".into());
        situ.nodes[1]
            .styles
            .insert("fontSize".into(), "16px ".into());
        let r = run(base, situ);
        let rep = report(&r);
        assert_eq!(rep.findings.len(), 1, "{:?}", rep.findings);
        assert_eq!(rep.findings[0].path, "");
    }

    // ---- props_delta --------------------------------------------------

    #[test]
    fn props_are_compared_only_with_compare_props() {
        let mut base = capture(page(), parent_of(page()));
        base.props = Some(json!({"mode": "range", "locale": "en"}));
        let mut situ = capture(page(), parent_of(page()));
        situ.props = Some(json!({"mode": "single", "locale": "en"}));
        let r = run(base.clone(), situ.clone());
        assert_eq!(r.outcome, AssertionOutcome::Passed, "{:?}", r.detail);

        let a: Assertion = serde_json::from_value(json!({
            "type": "matches_isolated_render",
            "component": "date-range-picker",
            "baseline": NAME,
            "compare_props": true,
        }))
        .unwrap();
        let r = run_with(&a, base, situ);
        assert_eq!(r.outcome, AssertionOutcome::Failed);
        let rep = report(&r);
        assert_eq!(rep.findings.len(), 1);
        let f = &rep.findings[0];
        assert_eq!(f.kind, ComponentFindingKind::PropsDelta);
        assert_eq!(f.path, "props.mode");
        assert_eq!(f.baseline, json!("range"));
        assert_eq!(f.in_situ, json!("single"));
    }

    // ---- portal_mismatch ----------------------------------------------

    #[test]
    fn a_portal_mounted_elsewhere_is_a_finding() {
        let portal = |t: &str| {
            Some(CapturePortal {
                mount_target: t.to_string(),
                react_parent_path: Some("1".to_string()),
            })
        };
        let mut base = capture(page(), parent_of(page()));
        base.nodes[3].portal = portal("body>div#portal-root");
        let mut situ = capture(page(), parent_of(page()));
        situ.nodes[3].portal = portal("div.modal-body");
        let r = run(base.clone(), situ);
        assert_eq!(r.outcome, AssertionOutcome::Failed);
        let f = &report(&r).findings[0];
        assert_eq!(f.kind, ComponentFindingKind::PortalMismatch);
        assert_eq!(f.path, "1/0");
        assert_eq!(f.baseline["mountTarget"], json!("body>div#portal-root"));
        assert_eq!(f.in_situ["mountTarget"], json!("div.modal-body"));

        // A portal on one side only.
        let r = run(base, capture(page(), parent_of(page())));
        assert_eq!(kinds(&r), vec![ComponentFindingKind::PortalMismatch]);
        assert!(report(&r).findings[0].message.contains("inline in situ"));
    }

    // ---- Unknown paths ------------------------------------------------

    #[test]
    fn unregistered_baseline_is_input_missing() {
        let captures = HashMap::from([(
            "date-range-picker".to_string(),
            capture(page(), parent_of(page())),
        )]);
        let empty = HashMap::new();
        for baselines in [None, Some(&empty)] {
            let r = evaluate(
                &assertion(),
                &EvalContext {
                    component_baselines: baselines,
                    component_captures: Some(&captures),
                    ..Default::default()
                },
            );
            assert_eq!(r.outcome, AssertionOutcome::Unknown);
            assert_eq!(r.code, Some(UnknownCode::InputMissing));
        }
    }

    #[test]
    fn absent_in_situ_capture_is_input_missing() {
        let baselines = HashMap::from([(
            NAME.to_string(),
            baseline(capture(page(), parent_of(page()))),
        )]);
        let other = HashMap::from([("some-other".to_string(), capture(page(), None))]);
        for captures in [None, Some(&other)] {
            let r = evaluate(
                &assertion(),
                &EvalContext {
                    component_baselines: Some(&baselines),
                    component_captures: captures,
                    ..Default::default()
                },
            );
            assert_eq!(r.outcome, AssertionOutcome::Unknown);
            assert_eq!(r.code, Some(UnknownCode::InputMissing));
            assert!(r.detail.unwrap().contains("date-range-picker"));
        }
    }

    #[test]
    fn a_side_without_root_geometry_is_input_missing() {
        let mut situ = capture(page(), parent_of(page()));
        situ.root = None;
        let r = run(capture(page(), parent_of(page())), situ);
        assert_input_missing(
            &r,
            "size/clipped not compared: the in-situ capture has no root",
        );

        let mut base = capture(page(), parent_of(page()));
        base.root = None;
        let r = run(base.clone(), capture(page(), parent_of(page())));
        assert_input_missing(
            &r,
            "size/clipped not compared: the baseline capture has no root",
        );

        let mut situ = capture(page(), parent_of(page()));
        situ.root = None;
        let r = run(base, situ);
        assert_input_missing(
            &r,
            "size/clipped not compared: neither capture has root geometry",
        );
    }

    #[test]
    fn missing_root_geometry_still_runs_the_other_checks() {
        let mut base = capture(page(), parent_of(page()));
        base.props = Some(json!({"mode": "range"}));
        let mut situ = capture(page(), parent_of(page()));
        situ.root = None;
        situ.props = Some(json!({"mode": "single"}));
        let r = run_with(&props_assertion(), base, situ);
        assert_eq!(r.outcome, AssertionOutcome::Failed, "{:?}", r.detail);
        assert_eq!(kinds(&r), vec![ComponentFindingKind::PropsDelta]);
        assert!(
            report(&r)
                .unknown_checks
                .iter()
                .any(|n| n.starts_with("size/clipped not compared: the in-situ capture")),
            "{:?}",
            report(&r).unknown_checks
        );
    }

    #[test]
    fn a_decimal_boundary_difference_is_within_tolerance() {
        let a: Assertion = serde_json::from_value(json!({
            "type": "matches_isolated_render",
            "component": "date-range-picker",
            "baseline": NAME,
            "size_tolerance_pct": 7.0,
        }))
        .unwrap();
        // 100 -> 107 in an unchanged 100 parent: rel_diff * 100 computes as
        // 7.000000000000001 on the absolute side, and the fill ratio side too.
        assert!(rel_diff(100.0, 107.0) * 100.0 > 7.0);
        let p = Some(rect(0.0, 0.0, 100.0, 40.0));
        let r = run_with(
            &a,
            capture(rect(0.0, 0.0, 100.0, 40.0), p),
            capture(rect(0.0, 0.0, 107.0, 40.0), p),
        );
        assert_eq!(r.outcome, AssertionOutcome::Passed, "{:?}", r.detail);
        let r = run_with(
            &a,
            capture(rect(0.0, 0.0, 100.0, 40.0), p),
            capture(rect(0.0, 0.0, 108.0, 40.0), p),
        );
        assert_eq!(r.outcome, AssertionOutcome::Failed, "{:?}", r.detail);
    }

    #[test]
    fn a_scroll_note_on_a_parent_does_not_hide_an_undecided_child() {
        let mut situ = capture(page(), parent_of(page()));
        // "1/0" is positioned below the root (page y 350..370).
        situ.nodes[3].bbox = Some(rect(0.0, 50.0, 120.0, 20.0));
        situ.clip_ancestors = vec![
            // The 400-wide root extends past this 300-wide scroller: a note.
            ClipAncestor {
                descriptor: "div.scroller".to_string(),
                overflow_x: Some("auto".to_string()),
                overflow_y: Some("auto".to_string()),
                rect: Some(rect(100.0, 300.0, 300.0, 600.0)),
            },
            // The root fits this unreported ancestor; "1/0" extends below it.
            ClipAncestor {
                descriptor: "div.unknown".to_string(),
                overflow_x: None,
                overflow_y: None,
                rect: Some(rect(100.0, 300.0, 400.0, 45.0)),
            },
        ];
        let mut base = capture(page(), parent_of(page()));
        base.nodes[3].bbox = Some(rect(0.0, 50.0, 120.0, 20.0));
        let r = run(base, situ);
        assert_input_missing(&r, "clipped not decided: <input> at path '1/0'");
        assert!(report(&r)
            .unknown_checks
            .iter()
            .any(|n| n.starts_with("clipped: <div> at path ''")));
    }

    #[test]
    fn an_unreported_baseline_ancestor_makes_an_in_situ_clip_undecided() {
        let frame = |overflow: Option<&str>| ClipAncestor {
            descriptor: "div.frame".to_string(),
            overflow_x: overflow.map(str::to_string),
            overflow_y: overflow.map(str::to_string),
            rect: Some(rect(100.0, 300.0, 300.0, 600.0)),
        };
        let mut situ = capture(page(), parent_of(page()));
        situ.clip_ancestors = vec![frame(Some("hidden"))];
        // In isolation the root extends past an ancestor of unknown overflow:
        // it may have been clipped there too, so the in-situ clip is not new.
        let mut base = capture(page(), parent_of(page()));
        base.clip_ancestors = vec![frame(Some("bogus"))];
        let r = run(base, situ.clone());
        assert_input_missing(
            &r,
            "but in isolation it extends outside clip ancestor 'div.frame'",
        );
        assert!(!kinds(&r).contains(&ComponentFindingKind::Clipped));

        // A baseline root with no bbox cannot be judged either, but the
        // undecided root does not hide its child "1", which is clipped in
        // situ and definitely not in isolation: Failed, with the root noted.
        let mut base = capture(page(), parent_of(page()));
        base.nodes[0].bbox = None;
        let r = run(base, situ);
        assert_eq!(r.outcome, AssertionOutcome::Failed, "{:?}", r.detail);
        let rep = report(&r);
        let clipped: Vec<&str> = rep
            .findings
            .iter()
            .filter(|f| f.kind == ComponentFindingKind::Clipped)
            .map(|f| f.path.as_str())
            .collect();
        assert_eq!(clipped, vec!["1"]);
        assert!(
            rep.unknown_checks.iter().any(|n| n.contains("at path ''")
                && n.contains("the node has no bbox in the baseline capture")),
            "{:?}",
            rep.unknown_checks
        );
    }

    fn rectless(x: &str, y: &str) -> ClipAncestor {
        ClipAncestor {
            descriptor: "div.rectless".to_string(),
            overflow_x: Some(x.to_string()),
            overflow_y: Some(y.to_string()),
            rect: None,
        }
    }

    #[test]
    fn a_rectless_baseline_clip_ancestor_makes_an_in_situ_clip_undecided() {
        let mut situ = capture(page(), parent_of(page()));
        situ.clip_ancestors = vec![ClipAncestor {
            descriptor: "div.frame".to_string(),
            overflow_x: Some("hidden".to_string()),
            overflow_y: Some("hidden".to_string()),
            rect: Some(rect(100.0, 300.0, 300.0, 600.0)),
        }];
        let mut base = capture(page(), parent_of(page()));
        base.clip_ancestors = vec![rectless("hidden", "hidden")];
        let r = run(base, situ);
        assert_input_missing(&r, "a baseline clip ancestor has no rect");
        assert!(!kinds(&r).contains(&ComponentFindingKind::Clipped));
        // The child "1" repeats the root's reason: folded into one entry.
        let entries = report(&r)
            .unknown_checks
            .iter()
            .filter(|n| n.contains("a baseline clip ancestor has no rect"))
            .count();
        assert_eq!(entries, 1);
    }

    #[test]
    fn a_descendant_with_a_different_baseline_reason_keeps_its_entry() {
        let mut situ = capture(page(), parent_of(page()));
        situ.clip_ancestors = vec![ClipAncestor {
            descriptor: "div.frame".to_string(),
            overflow_x: Some("hidden".to_string()),
            overflow_y: Some("hidden".to_string()),
            rect: Some(rect(100.0, 300.0, 300.0, 600.0)),
        }];
        // Root: no bbox in the baseline. "1": extends past an unreported
        // baseline ancestor that the (bbox-less) root cannot be judged by.
        let mut base = capture(page(), parent_of(page()));
        base.nodes[0].bbox = None;
        base.clip_ancestors = vec![ClipAncestor {
            descriptor: "div.unknown".to_string(),
            overflow_x: None,
            overflow_y: None,
            rect: Some(rect(100.0, 300.0, 300.0, 600.0)),
        }];
        let r = run(base, situ);
        let rep = report(&r);
        assert_eq!(r.outcome, AssertionOutcome::Unknown, "{:?}", r.detail);
        assert!(rep
            .unknown_checks
            .iter()
            .any(|n| n.contains("at path ''") && n.contains("has no bbox in the baseline")));
        assert!(
            rep.unknown_checks
                .iter()
                .any(|n| n.contains("at path '1'") && n.contains("'div.unknown'")),
            "{:?}",
            rep.unknown_checks
        );
    }

    #[test]
    fn a_rectless_scroll_container_cannot_clip_and_passes() {
        let mut situ = capture(page(), parent_of(page()));
        situ.clip_ancestors = vec![rectless("auto", "scroll")];
        let r = run(capture(page(), parent_of(page())), situ);
        assert_eq!(r.outcome, AssertionOutcome::Passed, "{:?}", r.detail);
    }

    #[test]
    fn a_rectless_baseline_ancestor_matters_only_when_judging_an_in_situ_clip() {
        let mut base = capture(page(), parent_of(page()));
        base.clip_ancestors = vec![rectless("hidden", "hidden")];
        let r = run(base, capture(page(), parent_of(page())));
        assert_eq!(r.outcome, AssertionOutcome::Passed, "{:?}", r.detail);
    }

    #[test]
    fn a_captures_own_unknown_checks_make_an_otherwise_passing_result_unknown() {
        let mut situ = capture(page(), parent_of(page()));
        situ.unknown_checks = vec!["styles unreadable".into()];
        let r = run(capture(page(), parent_of(page())), situ);
        assert_input_missing(&r, "in-situ capture: styles unreadable");
    }

    #[test]
    fn null_style_props_and_compare_props_take_their_defaults() {
        let a: Assertion = serde_json::from_value(json!({
            "type": "matches_isolated_render",
            "component": "date-range-picker",
            "baseline": NAME,
            "style_props": null,
            "compare_props": null,
        }))
        .expect("explicit nulls parse");
        let Assertion::MatchesIsolatedRender {
            style_props,
            compare_props,
            ..
        } = a
        else {
            panic!()
        };
        assert_eq!(style_props, default_props());
        assert!(!compare_props);
    }

    #[test]
    fn a_clip_ancestor_without_a_rect_or_a_node_without_a_bbox_is_undecided() {
        let mut situ = capture(page(), parent_of(page()));
        situ.clip_ancestors = vec![ClipAncestor {
            descriptor: "div.frame".to_string(),
            overflow_x: Some("hidden".to_string()),
            overflow_y: Some("hidden".to_string()),
            rect: None,
        }];
        let r = run(capture(page(), parent_of(page())), situ);
        assert_input_missing(&r, "in-situ clip ancestor 0 ('div.frame') has no rect");

        let mut situ = capture(page(), parent_of(page()));
        situ.clip_ancestors = vec![ClipAncestor {
            descriptor: "div.frame".to_string(),
            overflow_x: Some("hidden".to_string()),
            overflow_y: Some("hidden".to_string()),
            rect: Some(rect(100.0, 300.0, 400.0, 600.0)),
        }];
        situ.nodes[2].bbox = None;
        let r = run(capture(page(), parent_of(page())), situ);
        assert_input_missing(&r, "1 in-situ node(s) have no bbox ('1')");
    }

    #[test]
    fn staleness_is_reported_before_missing_geometry() {
        let mut situ = capture(page(), parent_of(page()));
        situ.root = None;
        situ.source_sha = Some("def456".to_string());
        let r = run(capture(page(), parent_of(page())), situ);
        assert_eq!(r.outcome, AssertionOutcome::Unknown);
        assert_eq!(r.code, Some(UnknownCode::StaleInput));
    }

    #[test]
    fn an_invalid_size_tolerance_is_input_missing() {
        for tol in [f64::NAN, -1.0, f64::INFINITY] {
            let a = Assertion::MatchesIsolatedRender {
                component: "date-range-picker".into(),
                baseline: NAME.into(),
                size_tolerance_pct: tol,
                style_props: default_props(),
                compare_props: false,
            };
            let r = run_with(
                &a,
                capture(page(), parent_of(page())),
                capture(page(), parent_of(page())),
            );
            assert_input_missing(&r, "size not compared: invalid tolerance");

            // The other checks still run: a style delta fails.
            let mut base = capture(page(), parent_of(page()));
            let mut situ = capture(page(), parent_of(page()));
            base.nodes[3].styles.insert("color".into(), "red".into());
            situ.nodes[3].styles.insert("color".into(), "blue".into());
            let r = run_with(&a, base, situ);
            assert_eq!(r.outcome, AssertionOutcome::Failed, "{tol}: {:?}", r.detail);
            assert_eq!(kinds(&r), vec![ComponentFindingKind::StyleDelta]);
        }
    }

    fn default_props() -> Box<[String]> {
        DEFAULT_STYLE_PROPS.iter().map(|s| s.to_string()).collect()
    }

    #[test]
    fn zero_sizes_compare_without_dividing_by_zero() {
        assert_eq!(rel_diff(0.0, 0.0), 0.0);
        assert_eq!(rel_diff(0.0, 5.0), f64::INFINITY);
        assert_eq!(ratio(5.0, Some(0.0)), None);
        assert_eq!(ratio(5.0, None), None);

        // 0 -> 0 on both axes (a collapsed component on both sides): no
        // finding. The zero-sized parent leaves the fill ratio unknown.
        let z = rect(0.0, 0.0, 0.0, 0.0);
        let r = run(capture(z, Some(z)), capture(z, Some(z)));
        assert_eq!(r.outcome, AssertionOutcome::Passed, "{:?}", r.detail);
        assert!(report(&r)
            .unknown_checks
            .iter()
            .any(|n| n.starts_with("size:")));

        // 0 -> N: collapsed in isolation, rendered in situ, no fill ratio
        // on the baseline side (zero-sized parent): size is undecided.
        let s = rect(0.0, 0.0, 300.0, 40.0);
        let r = run(capture(z, Some(z)), capture(s, Some(s)));
        assert_input_missing(&r, "size: fill ratio not compared");
        assert!(!kinds(&r).contains(&ComponentFindingKind::Size));
    }

    #[test]
    fn a_baseline_clip_on_x_does_not_cover_an_in_situ_clip_on_y() {
        let mut base = capture(page(), parent_of(page()));
        // Isolation: a 300-wide frame clips the 400-wide root on x only.
        base.clip_ancestors = vec![ClipAncestor {
            descriptor: "div.x-frame".to_string(),
            overflow_x: Some("hidden".to_string()),
            overflow_y: Some("visible".to_string()),
            rect: Some(rect(100.0, 300.0, 300.0, 600.0)),
        }];
        // In situ: a 30-tall frame clips the 40-tall root on y only.
        let mut situ = capture(page(), parent_of(page()));
        situ.clip_ancestors = vec![ClipAncestor {
            descriptor: "div.y-frame".to_string(),
            overflow_x: Some("visible".to_string()),
            overflow_y: Some("hidden".to_string()),
            rect: Some(rect(100.0, 300.0, 400.0, 30.0)),
        }];
        let r = run(base.clone(), situ);
        assert_eq!(r.outcome, AssertionOutcome::Failed, "{:?}", r.detail);
        assert_eq!(kinds(&r), vec![ComponentFindingKind::Clipped]);
        assert_eq!(report(&r).findings[0].path, "");

        // The same axis on both sides is covered: no finding.
        let mut situ = capture(page(), parent_of(page()));
        situ.clip_ancestors = base.clip_ancestors.clone();
        let r = run(base, situ);
        assert_eq!(r.outcome, AssertionOutcome::Passed, "{:?}", r.detail);
    }

    #[test]
    fn a_node_without_a_bbox_under_only_scroll_containers_passes() {
        let mut situ = capture(page(), parent_of(page()));
        situ.clip_ancestors = vec![ClipAncestor {
            descriptor: "div.scroller".to_string(),
            overflow_x: Some("auto".to_string()),
            overflow_y: Some("scroll".to_string()),
            rect: Some(rect(100.0, 300.0, 400.0, 600.0)),
        }];
        situ.nodes[2].bbox = None;
        let r = run(capture(page(), parent_of(page())), situ);
        assert_eq!(r.outcome, AssertionOutcome::Passed, "{:?}", r.detail);
    }

    #[test]
    fn null_list_and_style_value_fields_parse_as_absent() {
        let c: ComponentCapture = serde_json::from_value(json!({
            "schemaVersion": 1,
            "nodes": [
                { "path": null, "tag": null, "styles": { "color": null, "display": "block" } },
                { "path": "0", "tag": "span", "styles": null }
            ],
            "clipAncestors": null,
            "unknownChecks": null,
        }))
        .expect("nulls in defaulted fields parse");
        assert!(c.clip_ancestors.is_empty());
        assert!(c.unknown_checks.is_empty());
        assert_eq!(c.nodes[0].path, "");
        assert_eq!(c.nodes[0].tag, "");
        assert_eq!(c.nodes[0].styles.len(), 1);
        assert_eq!(c.nodes[0].styles["display"], "block");
        assert!(c.nodes[1].styles.is_empty());

        let a: ClipAncestor = serde_json::from_value(json!({"descriptor": null})).unwrap();
        assert_eq!(a.descriptor, "");
        let n: ComponentCapture =
            serde_json::from_value(json!({"schemaVersion": 1, "nodes": null})).unwrap();
        assert!(n.nodes.is_empty());
    }

    #[test]
    fn differing_known_source_shas_are_stale_input() {
        let mut situ = capture(page(), parent_of(page()));
        situ.source_sha = Some("def456".to_string());
        let r = run(capture(page(), parent_of(page())), situ.clone());
        assert_eq!(r.outcome, AssertionOutcome::Unknown);
        assert_eq!(r.code, Some(UnknownCode::StaleInput));

        // The baseline's own sourceSha wins over its capture's.
        let mut b = baseline(capture(page(), parent_of(page())));
        b.source_sha = Some("def456".to_string());
        let baselines = HashMap::from([(NAME.to_string(), b)]);
        let captures = HashMap::from([("date-range-picker".to_string(), situ.clone())]);
        let r = evaluate(
            &assertion(),
            &EvalContext {
                component_baselines: Some(&baselines),
                component_captures: Some(&captures),
                ..Default::default()
            },
        );
        assert_eq!(r.outcome, AssertionOutcome::Passed, "{:?}", r.detail);

        // An unknown sha on one side is not staleness.
        situ.source_sha = None;
        let r = run(capture(page(), parent_of(page())), situ);
        assert_eq!(r.outcome, AssertionOutcome::Passed, "{:?}", r.detail);
    }

    // ---- viewport -----------------------------------------------------

    #[test]
    fn a_different_viewport_with_the_same_relative_geometry_passes() {
        let base = capture(page(), parent_of(page()));
        // Mobile viewport, DPR 2, component at a different page offset: the
        // root-relative node geometry is identical.
        let moved = rect(12.0, 900.0, 400.0, 40.0);
        let mut situ = capture(moved, parent_of(moved));
        situ.viewport = Some(CaptureViewport {
            width: 390.0,
            height: 844.0,
        });
        situ.dpr = Some(2.0);
        let r = run(base, situ);
        assert_eq!(r.outcome, AssertionOutcome::Passed, "{:?}", r.detail);
        let rep = report(&r);
        assert_eq!(rep.viewport.matches, Some(false));
        assert_eq!(rep.viewport.in_situ_dpr, Some(2.0));
    }

    #[test]
    fn a_dpr_only_mismatch_is_reported_and_does_not_fail() {
        let mut situ = capture(page(), parent_of(page()));
        situ.dpr = Some(2.0);
        let r = run(capture(page(), parent_of(page())), situ.clone());
        assert_eq!(r.outcome, AssertionOutcome::Passed, "{:?}", r.detail);
        let v = report(&r).viewport;
        assert_eq!(v.matches, Some(false));
        assert_eq!((v.baseline_dpr, v.in_situ_dpr), (Some(1.0), Some(2.0)));

        // One DPR unknown, viewports equal: unknown, not a mismatch.
        situ.dpr = None;
        let r = run(capture(page(), parent_of(page())), situ.clone());
        assert_eq!(report(&r).viewport.matches, None);
        // One DPR unknown but the known viewports differ: a mismatch.
        situ.viewport = Some(CaptureViewport {
            width: 390.0,
            height: 844.0,
        });
        let r = run(capture(page(), parent_of(page())), situ);
        assert_eq!(r.outcome, AssertionOutcome::Passed, "{:?}", r.detail);
        assert_eq!(report(&r).viewport.matches, Some(false));
        // All four known and equal.
        let r = run(
            capture(page(), parent_of(page())),
            capture(page(), parent_of(page())),
        );
        assert_eq!(report(&r).viewport.matches, Some(true));
    }

    // ---- wire ---------------------------------------------------------

    /// The contract's example, as the SDK emits it.
    const CONTRACT_EXAMPLE: &str = r#"{
      "schemaVersion": 1,
      "target": { "kind": "component", "id": "date-range-picker" },
      "snapshotId": null,
      "route": "/settings",
      "sourceSha": "abc123",
      "viewport": { "width": 1280, "height": 800 },
      "dpr": 2,
      "root": {
        "bbox": { "x": 10, "y": 20, "width": 400, "height": 40 },
        "parentContentBox": null
      },
      "nodes": [
        { "path": "", "tag": "div", "role": null, "registeredId": "date-range-picker",
          "componentName": "DateRangePicker",
          "bbox": { "x": 0, "y": 0, "width": 400, "height": 40 },
          "styles": { "display": "block", "fontSize": "14px" }, "portal": null },
        { "path": "0", "tag": "input", "role": "textbox", "registeredId": null,
          "componentName": null,
          "bbox": { "x": 0, "y": 0, "width": 120, "height": 20 },
          "styles": {},
          "portal": { "mountTarget": "body>div#portal-root", "reactParentPath": "" } }
      ],
      "clipAncestors": [
        { "descriptor": "div.scroll-pane", "overflowX": "hidden", "overflowY": "auto",
          "rect": { "x": 0, "y": 0, "width": 300, "height": 600 } }
      ],
      "state": { "open": false },
      "props": null,
      "unknownChecks": ["componentName: unreadable (minified or redacted)"],
      "someFutureField": { "tolerated": true }
    }"#;

    #[test]
    fn the_contract_example_round_trips() {
        let c: ComponentCapture = serde_json::from_str(CONTRACT_EXAMPLE).unwrap();
        assert_eq!(c.schema_version, Some(1));
        assert_eq!(
            c.target.as_ref().unwrap().kind,
            CaptureTargetKind::Component
        );
        assert_eq!(c.root_bbox(), Some(rect(10.0, 20.0, 400.0, 40.0)));
        assert_eq!(c.parent_content_box(), None);
        assert_eq!(c.nodes.len(), 2);
        assert_eq!(
            c.nodes[0].registered_id.as_deref(),
            Some("date-range-picker")
        );
        assert_eq!(c.nodes[1].component_name, None);
        assert_eq!(
            c.nodes[1].portal.as_ref().unwrap().mount_target,
            "body>div#portal-root"
        );
        // overflowX hidden clips; overflowY auto scrolls and does not.
        assert!(c.clip_ancestors[0].clips_x());
        assert!(!c.clip_ancestors[0].clips_y());
        assert_eq!(c.clip_ancestors[0].axis_y(), OverflowAxis::Scrolls);
        assert_eq!(c.props, None);

        let wire = serde_json::to_value(&c).unwrap();
        // camelCase on the wire; absent optionals omitted.
        assert!(wire.get("clipAncestors").is_some());
        assert!(wire.get("snapshotId").is_none());
        assert!(wire["root"].get("parentContentBox").is_none());
        let back: ComponentCapture = serde_json::from_value(wire).unwrap();
        assert_eq!(back, c);

        let b: ComponentBaseline = serde_json::from_value(json!({
            "name": NAME,
            "storyId": "DateRangePicker--default",
            "sourceSha": "abc123",
            "capturedAt": "2026-11-13T12:00:00Z",
            "capture": serde_json::from_str::<Value>(CONTRACT_EXAMPLE).unwrap(),
        }))
        .unwrap();
        assert_eq!(b.effective_source_sha(), Some("abc123"));
        let back: ComponentBaseline =
            serde_json::from_value(serde_json::to_value(&b).unwrap()).unwrap();
        assert_eq!(back, b);
    }

    #[test]
    fn the_assertion_round_trips_its_wire_spelling() {
        let wire = json!({"type":"matches_isolated_render","component":"date-range-picker",
            "baseline":"storybook/DateRangePicker--default@abc",
            "size_tolerance_pct":5.0,"style_props":["fontSize","color","overflow","display"],
            "compare_props":false});
        let a: Assertion = serde_json::from_value(wire.clone()).unwrap();
        assert_eq!(serde_json::to_value(&a).unwrap(), wire);
    }

    // ---- review fixes -------------------------------------------------

    fn props_assertion() -> Assertion {
        serde_json::from_value(json!({
            "type": "matches_isolated_render",
            "component": "date-range-picker",
            "baseline": NAME,
            "compare_props": true,
        }))
        .unwrap()
    }

    fn assert_input_missing(r: &AssertionResult, needle: &str) {
        assert_eq!(r.outcome, AssertionOutcome::Unknown, "{:?}", r.detail);
        assert_eq!(r.code, Some(UnknownCode::InputMissing), "{:?}", r.detail);
        let d = r.detail.as_deref().unwrap_or_default();
        assert!(d.contains(needle), "{d}");
    }

    #[test]
    fn an_unsupported_schema_version_on_either_side_is_input_missing() {
        let mut base = capture(page(), parent_of(page()));
        base.schema_version = Some(2);
        let r = run(base, capture(page(), parent_of(page())));
        assert_input_missing(
            &r,
            "schemaVersion 2 and the in-situ capture is schemaVersion 1",
        );

        let mut situ = capture(page(), parent_of(page()));
        situ.schema_version = Some(7);
        let r = run(capture(page(), parent_of(page())), situ);
        assert_input_missing(
            &r,
            "schemaVersion 1 and the in-situ capture is schemaVersion 7",
        );
        assert!(r
            .detail
            .unwrap()
            .contains("understands only schemaVersion 1"));
    }

    #[test]
    fn an_absent_schema_version_is_input_missing() {
        let mut wire: Value = serde_json::from_str(CONTRACT_EXAMPLE).unwrap();
        wire.as_object_mut().unwrap().remove("schemaVersion");
        let unversioned: ComponentCapture = serde_json::from_value(wire).unwrap();
        assert_eq!(unversioned.schema_version, None);
        // An unversioned capture serializes without the key, not as 1.
        assert!(serde_json::to_value(&unversioned)
            .unwrap()
            .get("schemaVersion")
            .is_none());

        let mut situ = capture(page(), parent_of(page()));
        situ.schema_version = None;
        let r = run(capture(page(), parent_of(page())), situ);
        assert_input_missing(&r, "the in-situ capture carries no schemaVersion");

        let mut base = capture(page(), parent_of(page()));
        base.schema_version = None;
        let r = run(base, capture(page(), parent_of(page())));
        assert_input_missing(&r, "the baseline capture carries no schemaVersion");
    }

    #[test]
    fn a_side_with_no_nodes_is_input_missing() {
        let mut base = capture(page(), parent_of(page()));
        base.nodes.clear();
        let r = run(base, capture(page(), parent_of(page())));
        assert_input_missing(
            &r,
            "per-node checks skipped: the baseline capture has no nodes",
        );

        let mut situ = capture(page(), parent_of(page()));
        situ.nodes.clear();
        let r = run(capture(page(), parent_of(page())), situ);
        assert_input_missing(
            &r,
            "per-node checks skipped: the in-situ capture has no nodes",
        );
    }

    #[test]
    fn a_side_with_no_root_node_is_input_missing() {
        let mut base = capture(page(), parent_of(page()));
        base.nodes.remove(0);
        let r = run(base, capture(page(), parent_of(page())));
        assert_input_missing(&r, "the baseline capture has no root node");

        let mut situ = capture(page(), parent_of(page()));
        situ.nodes.remove(0);
        let r = run(capture(page(), parent_of(page())), situ);
        assert_input_missing(&r, "the in-situ capture has no root node");
    }

    /// Isolation: 200 wide in a 400-wide parent; in situ: 1000 of 1000.
    fn stretched() -> (ComponentCapture, ComponentCapture) {
        (
            capture(
                rect(0.0, 0.0, 200.0, 40.0),
                Some(rect(0.0, 0.0, 400.0, 40.0)),
            ),
            capture(
                rect(0.0, 0.0, 1000.0, 40.0),
                Some(rect(0.0, 0.0, 1000.0, 40.0)),
            ),
        )
    }

    #[test]
    fn a_size_delta_with_no_node_tree_fails_with_a_note() {
        for clear_base in [true, false] {
            let (mut base, mut situ) = stretched();
            if clear_base {
                base.nodes.clear();
            } else {
                situ.nodes.clear();
            }
            let r = run(base, situ);
            assert_eq!(r.outcome, AssertionOutcome::Failed, "{:?}", r.detail);
            let rep = report(&r);
            assert_eq!(kinds(&r), vec![ComponentFindingKind::Size]);
            assert!(
                rep.unknown_checks
                    .iter()
                    .any(|n| n.starts_with("per-node checks skipped:")),
                "{:?}",
                rep.unknown_checks
            );
        }
    }

    #[test]
    fn a_size_delta_with_props_missing_fails_with_a_note() {
        let (base, mut situ) = stretched();
        situ.props = Some(json!({"mode": "range"}));
        let r = run_with(&props_assertion(), base, situ);
        assert_eq!(r.outcome, AssertionOutcome::Failed, "{:?}", r.detail);
        let rep = report(&r);
        assert_eq!(kinds(&r), vec![ComponentFindingKind::Size]);
        assert!(
            rep.unknown_checks
                .iter()
                .any(|n| n.starts_with("props not compared:")
                    && n.contains("the baseline capture has no props")),
            "{:?}",
            rep.unknown_checks
        );
    }

    #[test]
    fn an_unknown_after_comparison_still_carries_the_report() {
        let mut base = capture(page(), parent_of(page()));
        base.nodes.clear();
        let r = run(base, capture(page(), parent_of(page())));
        assert_eq!(r.outcome, AssertionOutcome::Unknown);
        let rep = report(&r);
        assert!(rep.findings.is_empty());
        assert!(
            rep.summary.contains("could not be fully compared"),
            "{}",
            rep.summary
        );
    }

    #[test]
    fn compare_props_with_props_missing_on_either_side_is_input_missing() {
        let with_props = || {
            let mut c = capture(page(), parent_of(page()));
            c.props = Some(json!({"mode": "range"}));
            c
        };
        let bare = || capture(page(), parent_of(page()));
        let a = props_assertion();
        assert_input_missing(
            &run_with(&a, bare(), with_props()),
            "props not compared: compare_props was requested but the baseline capture has no props",
        );
        assert_input_missing(
            &run_with(&a, with_props(), bare()),
            "the in-situ capture has no props",
        );
        assert_input_missing(&run_with(&a, bare(), bare()), "neither capture has props");
        // Without compare_props, absent props are irrelevant.
        assert_eq!(run(bare(), bare()).outcome, AssertionOutcome::Passed);
    }

    #[test]
    fn extending_outside_a_scroll_container_is_noted_not_clipped() {
        for (ox, oy) in [("auto", "auto"), ("scroll", "visible")] {
            let mut situ = capture(page(), parent_of(page()));
            // A 300-wide scroll container: the 400-wide root extends past it
            // horizontally, and is scrolled to, not cut off.
            situ.clip_ancestors = vec![ClipAncestor {
                descriptor: "div.scroller".to_string(),
                overflow_x: Some(ox.to_string()),
                overflow_y: Some(oy.to_string()),
                rect: Some(rect(100.0, 300.0, 300.0, 600.0)),
            }];
            let r = run(capture(page(), parent_of(page())), situ);
            assert_eq!(r.outcome, AssertionOutcome::Passed, "{ox}: {:?}", r.detail);
            let rep = report(&r);
            assert!(!kinds(&r).contains(&ComponentFindingKind::Clipped));
            let notes: Vec<_> = rep
                .unknown_checks
                .iter()
                .filter(|n| n.starts_with("clipped:"))
                .collect();
            // Topmost only: the root, not its children beneath it.
            assert_eq!(notes.len(), 1, "{ox}: {notes:?}");
            assert!(notes[0].contains("div.scroller") && notes[0].contains("on x"));
        }
    }

    fn unreported_clip(overflow: Option<&str>) -> ClipAncestor {
        ClipAncestor {
            descriptor: "div.unknown".to_string(),
            overflow_x: overflow.map(str::to_string),
            overflow_y: overflow.map(str::to_string),
            rect: Some(rect(100.0, 300.0, 300.0, 600.0)),
        }
    }

    #[test]
    fn an_unreported_overflow_axis_is_unknown_when_nothing_else_fails() {
        for overflow in [None, Some("bogus")] {
            let mut situ = capture(page(), parent_of(page()));
            situ.clip_ancestors = vec![unreported_clip(overflow)];
            let r = run(capture(page(), parent_of(page())), situ);
            assert_input_missing(&r, "clipped not decided");
            assert!(r.detail.unwrap().contains("div.unknown"), "{overflow:?}");
        }
    }

    #[test]
    fn an_unreported_overflow_axis_is_a_note_when_something_else_fails() {
        let mut situ = capture(page(), parent_of(page()));
        situ.clip_ancestors = vec![unreported_clip(None)];
        situ.nodes[3].component_name = Some("NativeDateInput".to_string());
        let r = run(capture(page(), parent_of(page())), situ);
        assert_eq!(r.outcome, AssertionOutcome::Failed, "{:?}", r.detail);
        assert_eq!(kinds(&r), vec![ComponentFindingKind::ChildIdentity]);
        assert!(report(&r)
            .unknown_checks
            .iter()
            .any(|n| n.starts_with("clipped not decided") && n.contains("div.unknown")));
    }

    #[test]
    fn non_inherited_style_deltas_are_reported_at_every_node() {
        let mut base = capture(page(), parent_of(page()));
        let mut situ = capture(page(), parent_of(page()));
        for n in &mut base.nodes {
            n.styles.insert("display".into(), "block".into());
            n.styles.insert("overflow".into(), "visible".into());
        }
        for n in &mut situ.nodes {
            n.styles.insert("display".into(), "flex".into());
            n.styles.insert("overflow".into(), "hidden".into());
        }
        let r = run(base, situ);
        let rep = report(&r);
        for prop in ["display", "overflow"] {
            let paths: Vec<&str> = rep
                .findings
                .iter()
                .filter(|f| f.baseline.get(prop).is_some())
                .map(|f| f.path.as_str())
                .collect();
            assert_eq!(paths, vec!["", "0", "1", "1/0"], "{prop}");
        }
    }

    #[test]
    fn the_inherited_set_covers_the_cascading_properties_only() {
        for p in [
            "color",
            "fontSize",
            "fontFamily",
            "font",
            "lineHeight",
            "visibility",
        ] {
            assert!(is_inherited(p), "{p}");
        }
        for p in ["display", "overflow", "width", "margin", "backgroundColor"] {
            assert!(!is_inherited(p), "{p}");
        }
    }

    #[test]
    fn a_null_size_tolerance_takes_the_default() {
        let a: Assertion = serde_json::from_value(json!({
            "type": "matches_isolated_render",
            "component": "date-range-picker",
            "baseline": NAME,
            "size_tolerance_pct": null,
        }))
        .expect("an explicit null size_tolerance_pct parses");
        let Assertion::MatchesIsolatedRender {
            size_tolerance_pct, ..
        } = a
        else {
            panic!()
        };
        assert_eq!(size_tolerance_pct, DEFAULT_SIZE_TOLERANCE_PCT);
        // A wrong type is still a parse error.
        assert!(serde_json::from_value::<Assertion>(json!({
            "type": "matches_isolated_render",
            "component": "date-range-picker",
            "baseline": NAME,
            "size_tolerance_pct": "5",
        }))
        .is_err());
    }

    #[test]
    fn the_store_it_first_hint_is_given_only_when_baselines_were_supplied() {
        let captures = HashMap::from([(
            "date-range-picker".to_string(),
            capture(page(), parent_of(page())),
        )]);
        let ctx = |b| EvalContext {
            component_baselines: b,
            component_captures: Some(&captures),
            ..Default::default()
        };
        let none = evaluate(&assertion(), &ctx(None)).detail.unwrap();
        assert!(
            none.contains("no component baselines were supplied"),
            "{none}"
        );
        assert!(!none.contains("vision/component-baseline"), "{none}");

        let empty = HashMap::new();
        let unreg = evaluate(&assertion(), &ctx(Some(&empty))).detail.unwrap();
        assert!(unreg.contains("is not registered"), "{unreg}");
        assert!(unreg.contains("vision/component-baseline"), "{unreg}");
    }
}
