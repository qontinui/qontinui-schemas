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
    CaptureRect, CaptureViewport, ComponentBaseline, ComponentCapture, ComponentCaptureNode,
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
    /// Both viewports (and DPRs) known and equal. `None` when either is
    /// unknown.
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

    if !size_tolerance_pct.is_finite() || size_tolerance_pct < 0.0 {
        return AssertionResult::unknown(
            a,
            UnknownCode::InputMissing,
            format!(
                "size_tolerance_pct {size_tolerance_pct} is not a finite non-negative \
                 percentage, so the captures were not compared"
            ),
        );
    }

    let Some(entry) = ctx.component_baselines.and_then(|m| m.get(&*baseline)) else {
        let why = if ctx.component_baselines.is_none() {
            "no component baselines were supplied to the evaluator".to_string()
        } else {
            format!("component baseline '{baseline}' is not registered")
        };
        return AssertionResult::unknown(
            a,
            UnknownCode::InputMissing,
            format!(
                "{why}, so the in-situ render was not compared — store it with \
                 `vision/component-baseline` first"
            ),
        );
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
    let (Some(b_root), Some(s_root)) = (entry.capture.root_bbox(), in_situ.root_bbox()) else {
        let side = match (entry.capture.root_bbox(), in_situ.root_bbox()) {
            (None, None) => "neither capture",
            (None, Some(_)) => "the baseline capture",
            _ => "the in-situ capture",
        };
        return AssertionResult::unknown(
            a,
            UnknownCode::InputMissing,
            format!("{side} has root geometry, so the renders were not compared"),
        );
    };
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

    let opts = DiffOptions {
        size_tolerance_pct,
        style_props: &style_props,
        compare_props,
    };
    let (findings, unknown_checks) = diff(entry, in_situ, b_root, s_root, &opts);
    let viewport = compare_viewports(&entry.capture, in_situ);
    let summary = if findings.is_empty() {
        format!("component '{component}' renders in situ like baseline '{baseline}'")
    } else {
        let mut kinds: Vec<&str> = findings.iter().map(|f| kind_name(f.kind)).collect();
        kinds.dedup();
        format!(
            "component '{component}' diverges in situ from baseline '{baseline}': {} finding(s) ({})",
            findings.len(),
            kinds.join(", ")
        )
    };
    let failed = !findings.is_empty();
    let report = ComponentDiffReport {
        summary,
        component: component.into(),
        baseline: baseline.into(),
        findings,
        unknown_checks,
        viewport,
    };
    let detail = serde_json::to_string(&report).expect("a ComponentDiffReport always serializes");
    if failed {
        AssertionResult::fail(a, detail)
    } else {
        AssertionResult::pass_with(a, detail)
    }
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
    size_tolerance_pct: f64,
    style_props: &'a [String],
    compare_props: bool,
}

fn diff(
    entry: &ComponentBaseline,
    in_situ: &ComponentCapture,
    b_root: CaptureRect,
    s_root: CaptureRect,
    opts: &DiffOptions<'_>,
) -> (Vec<ComponentFinding>, Vec<String>) {
    let base = &entry.capture;
    let mut findings = Vec::new();
    let mut notes = Vec::new();

    size_check(
        base,
        in_situ,
        b_root,
        s_root,
        opts.size_tolerance_pct,
        &mut findings,
        &mut notes,
    );
    clip_check(base, in_situ, b_root, s_root, &mut findings);
    let b_nodes = by_path(base);
    let s_nodes = by_path(in_situ);
    identity_check(base, in_situ, &b_nodes, &s_nodes, &mut findings, &mut notes);
    style_check(
        in_situ,
        &b_nodes,
        opts.style_props,
        &mut findings,
        &mut notes,
    );
    if opts.compare_props {
        props_check(base, in_situ, &mut findings, &mut notes);
    }
    portal_check(in_situ, &b_nodes, &mut findings);

    for n in &base.unknown_checks {
        notes.push(format!("baseline capture: {n}"));
    }
    for n in &in_situ.unknown_checks {
        notes.push(format!("in-situ capture: {n}"));
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

fn below_any(path: &str, reported: &[String]) -> bool {
    reported.iter().any(|r| is_descendant(path, r))
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

fn ratio(num: f64, den: Option<f64>) -> Option<f64> {
    den.filter(|d| *d > 0.0).map(|d| num / d)
}

/// `size`: raised only when, on one axis, BOTH the absolute root size AND the
/// root-to-parent-content-box fill ratio differ beyond tolerance. A component
/// that fills its parent on both sides (a fluid component shrink-wrapped in a
/// centered story and filling a page column) or keeps its absolute size on
/// both sides is not stretched. Without a parent content box on either side
/// the fill ratio is unknown, so the absolute size alone decides and a note
/// says so.
fn size_check(
    base: &ComponentCapture,
    in_situ: &ComponentCapture,
    b: CaptureRect,
    s: CaptureRect,
    tol_pct: f64,
    findings: &mut Vec<ComponentFinding>,
    notes: &mut Vec<String>,
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
    for (axis, b_abs, s_abs, b_par, s_par) in axes {
        let abs_pct = rel_diff(b_abs, s_abs) * 100.0;
        let b_fill = ratio(b_abs, b_par);
        let s_fill = ratio(s_abs, s_par);
        let fill_pct = match (b_fill, s_fill) {
            (Some(bf), Some(sf)) => Some(rel_diff(bf, sf) * 100.0),
            _ => {
                fill_unknown.push(axis);
                None
            }
        };
        let abs_differs = abs_pct > tol_pct;
        let fill_differs = fill_pct.is_none_or(|p| p > tol_pct);
        if abs_differs && fill_differs {
            let verb = if s_abs > b_abs {
                "stretched"
            } else {
                "constrained"
            };
            let fill = match (b_fill, s_fill) {
                (Some(bf), Some(sf)) => {
                    format!(", fill ratio {bf:.2} -> {sf:.2} of the parent content box")
                }
                _ => ", fill ratio unknown (absolute size only)".to_string(),
            };
            reasons.push(format!("{verb} on {axis}: {b_abs}px -> {s_abs}px{fill}"));
        }
    }
    if !fill_unknown.is_empty() {
        let side = match (bp.is_some(), sp.is_some()) {
            (false, false) => "either side",
            (false, true) => "the baseline side",
            (true, false) => "the in-situ side",
            _ => "one side (zero-sized)",
        };
        notes.push(format!(
            "size: no usable parentContentBox on {side} for {}; the fill ratio was not \
             compared and absolute size alone decided",
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

/// `clipped`: a node extends past a clip ancestor in situ, while in isolation
/// no clip ancestor clipped the node at that path. Only the topmost clipped
/// node of a subtree is reported.
fn clip_check(
    base: &ComponentCapture,
    in_situ: &ComponentCapture,
    b_root: CaptureRect,
    s_root: CaptureRect,
    findings: &mut Vec<ComponentFinding>,
) {
    let b_nodes = by_path(base);
    let mut reported: Vec<String> = Vec::new();
    for node in &in_situ.nodes {
        if below_any(&node.path, &reported) {
            continue;
        }
        let Some(rel) = node.bbox.filter(CaptureRect::is_finite) else {
            continue;
        };
        let page = rel.offset_by(&s_root);
        let Some((i, clip_rect)) = clipping_ancestor(in_situ, &page) else {
            continue;
        };
        let baseline_clipped = b_nodes
            .get(node.path.as_str())
            .and_then(|bn| bn.bbox.filter(CaptureRect::is_finite))
            .and_then(|r| clipping_ancestor(base, &r.offset_by(&b_root)))
            .is_some();
        if baseline_clipped {
            continue;
        }
        let anc = &in_situ.clip_ancestors[i];
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
}

fn node_identity_json(n: &ComponentCaptureNode) -> Value {
    json!({"tag": n.tag, "componentName": n.component_name, "registeredId": n.registered_id})
}

/// `child_identity`: tag or component name differs at the same path, or a
/// node exists on one side only. A null name on either side is UNREADABLE:
/// the name comparison is skipped and noted, never reported as a mismatch.
fn identity_check(
    base: &ComponentCapture,
    in_situ: &ComponentCapture,
    b_nodes: &BTreeMap<&str, &ComponentCaptureNode>,
    s_nodes: &BTreeMap<&str, &ComponentCaptureNode>,
    findings: &mut Vec<ComponentFinding>,
    notes: &mut Vec<String>,
) {
    let mut unreadable = 0usize;
    for bn in &base.nodes {
        let Some(sn) = s_nodes.get(bn.path.as_str()) else {
            continue;
        };
        if !bn.tag.eq_ignore_ascii_case(&sn.tag) {
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
            if other.contains_key(n.path.as_str()) || below_any(&n.path, &reported) {
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
}

/// `style_delta`: a `style_props` key whose value differs at the same path.
/// A difference already reported on an ancestor with the same two values
/// (an inherited property cascading down) is not repeated.
fn style_check(
    in_situ: &ComponentCapture,
    b_nodes: &BTreeMap<&str, &ComponentCaptureNode>,
    style_props: &[String],
    findings: &mut Vec<ComponentFinding>,
    notes: &mut Vec<String>,
) {
    let mut reported: HashMap<(&str, &str, &str), Vec<String>> = HashMap::new();
    let mut one_sided = 0usize;
    for sn in &in_situ.nodes {
        let Some(bn) = b_nodes.get(sn.path.as_str()) else {
            continue;
        };
        for prop in style_props {
            match (bn.styles.get(prop), sn.styles.get(prop)) {
                (Some(bv), Some(sv)) if bv.trim() != sv.trim() => {
                    let key = (prop.as_str(), bv.as_str(), sv.as_str());
                    let seen = reported.entry(key).or_default();
                    if below_any(&sn.path, seen) {
                        continue;
                    }
                    seen.push(sn.path.clone());
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
    notes: &mut Vec<String>,
) {
    let (Some(bp), Some(sp)) = (&base.props, &in_situ.props) else {
        notes
            .push("props_delta: props not captured on at least one side; not compared".to_string());
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
/// different mount target.
fn portal_check(
    in_situ: &ComponentCapture,
    b_nodes: &BTreeMap<&str, &ComponentCaptureNode>,
    findings: &mut Vec<ComponentFinding>,
) {
    for sn in &in_situ.nodes {
        let Some(bn) = b_nodes.get(sn.path.as_str()) else {
            continue;
        };
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
    let matches = match (base.viewport, in_situ.viewport) {
        (Some(b), Some(s)) => {
            let dpr_ok = match (base.dpr, in_situ.dpr) {
                (Some(bd), Some(sd)) => bd == sd,
                (None, None) => true,
                _ => false,
            };
            Some(b == s && dpr_ok)
        }
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
            schema_version: 1,
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
    fn missing_parent_box_compares_absolute_size_only_and_notes_it() {
        let b = rect(0.0, 0.0, 200.0, 40.0);
        let s = rect(0.0, 0.0, 1000.0, 40.0);
        let r = run(
            capture(b, None),
            capture(s, Some(rect(0.0, 0.0, 1000.0, 40.0))),
        );
        assert_eq!(r.outcome, AssertionOutcome::Failed);
        let rep = report(&r);
        assert!(rep
            .findings
            .iter()
            .any(|f| f.kind == ComponentFindingKind::Size));
        assert!(
            rep.unknown_checks
                .iter()
                .any(|n| n.starts_with("size:") && n.contains("baseline side")),
            "{:?}",
            rep.unknown_checks
        );
    }

    // ---- clipped ------------------------------------------------------

    #[test]
    fn clipped_by_an_in_situ_ancestor_absent_in_isolation() {
        let mut situ = capture(page(), parent_of(page()));
        // A 300-wide scroll pane at the root's origin: the 400-wide root
        // overflows it horizontally.
        situ.clip_ancestors = vec![ClipAncestor {
            descriptor: "div.scroll-pane".to_string(),
            overflow_x: Some("hidden".to_string()),
            overflow_y: Some("auto".to_string()),
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
        assert!(
            ids.iter()
                .any(|f| f.path == "1/0" && f.message.contains("missing in situ")),
            "{ids:?}"
        );
        assert_eq!(
            ids.iter().find(|f| f.path == "1").unwrap().baseline["tag"],
            json!("div")
        );
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
        situ.unknown_checks = vec!["componentName: unreadable (minified or redacted)".into()];
        let r = run(capture(page(), parent_of(page())), situ);
        assert_eq!(r.outcome, AssertionOutcome::Passed, "{:?}", r.detail);
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
        assert_eq!(r.outcome, AssertionOutcome::Unknown);
        assert_eq!(r.code, Some(UnknownCode::InputMissing));
        assert!(r.detail.unwrap().contains("in-situ"));
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
        assert_eq!(c.schema_version, 1);
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
        assert!(c.clip_ancestors[0].clips_x() && c.clip_ancestors[0].clips_y());
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
}
