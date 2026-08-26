//! Phase 6 assertion DSL — declarative "should this be true?" checks
//! over a captured [`Frame`] + [`ElementSnapshot`].
//!
//! Each [`Assertion`] is a tagged enum variant that the wire serializes
//! as `{ "type": "no_overlap", … }`. The DSL is **append-only**: adding a
//! variant — or an optional `#[serde(default)]` field to an existing type,
//! including [`ElementSnapshot`] on the input side and [`AssertionResult`] on
//! the output side — is non-breaking, because serde tolerates unknown input
//! fields and `skip_serializing_if` keeps absent ones off the wire entirely.
//! Removing or renaming one is breaking.
//!
//! **A breaking change costs a consumer RE-PIN, not an in-band version bump.**
//! This crate is unpublished (it is absent from `release-please-config.json`
//! and from `publish-rust.yml`), so every consumer pins a commit instead:
//! qontinui-runner by Rust path dependency, qontinui-web's CI style gate by
//! the `qontinui_schemas_sha` in its root `style-gate.lock`. There is no DSL
//! version field anywhere in this crate to bump.
//!
//! [`crate::OutputContract`] does **not** version this DSL — an earlier
//! revision of this comment said it did, and that was wrong. It is a set of
//! IMAGE-ENCODING constraints (`allowed_formats`, `max_long_edge`,
//! `max_bytes`, `alpha_policy`, `metadata_policy`, `color_space`) read only by
//! [`crate::encode::safe_image`] and [`crate::Stage::Verify`]. Neither
//! [`evaluate`], [`AssertionResult`] nor [`ElementSnapshot`] ever takes one,
//! and it has no version field: its "versions" are three named constants
//! describing three different image formats. Bumping it would change emitted
//! image bytes and tell an assert consumer nothing.
//!
//! See [`evaluate`] for the entrypoint and individual variant docs for
//! what each one checks.

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::element_snapshot::{
    display_snapshot_id, intersection, region_contains, regions_overlap, ElementSnapshot, Rgb,
};
use crate::frame::{Frame, Region};

/// One assertion. Wire format: `{"type": "no_overlap", "elements": [...]}`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Assertion {
    /// The two named elements' rendered bboxes do not intersect (modulo
    /// optional tolerance). The original motivating example: the
    /// terminal-tab overlap bug that triggered Phase 6.
    NoOverlap {
        elements: [String; 2],
        #[serde(default)]
        tolerance_px: Option<u32>,
    },
    /// `elements[0]` paints ON TOP OF `elements[1]`.
    ///
    /// Decided from [`Element::paint_order`](crate::element_snapshot::Element::paint_order)
    /// — the resolved painting sequence — never from a raw `z-index`. Raw
    /// z-index is comparable only between siblings of one stacking context,
    /// so the naive check is FALSIFIED across contexts, not merely coarse:
    /// a `z-50` dropdown inside a `z-index:10` title bar loses to a `z-20`
    /// panel outside it, and comparing 50 against 20 answers backwards while
    /// emitting the identical verdict before and after the fix.
    ///
    /// When either element carries no `paint_order`, the snapshot source did
    /// not resolve stacking and this reports "cannot answer" rather than a
    /// verdict.
    ElementAbove {
        /// `[above, below]` — asserts elements[0] paints on top of elements[1].
        elements: [String; 2],
        /// Require the two bboxes to intersect before ruling on order.
        ///
        /// Defaults to **true**: "which is on top" is not a question two
        /// disjoint elements answer, and a vacuous pass is worse than a loud
        /// failure. Set `false` to assert a pure ordering (e.g. a modal layer
        /// that must outrank a toast it does not currently overlap).
        #[serde(default = "default_true")]
        require_overlap: bool,
    },
    /// Element/region contains expected text (OCR or snapshot-text).
    ContainsText {
        target: TextTarget,
        text: String,
        #[serde(default)]
        kind: TextMatchKind,
    },
    /// Element's text bbox fits within the element's content box.
    /// Catches `overflow:visible` + `white-space:nowrap` clipping bugs.
    TextFitsContainer { element: String },
    /// All named elements share a y-baseline within the tolerance.
    AlignedHorizontally {
        elements: Vec<String>,
        #[serde(default)]
        axis_tolerance_px: Option<u32>,
    },
    /// All named elements share an x-baseline within the tolerance.
    AlignedVertically {
        elements: Vec<String>,
        #[serde(default)]
        axis_tolerance_px: Option<u32>,
    },
    /// Element's foreground color is within ΔE_76 of expected RGB.
    /// (ΔE_2000 is more perceptually accurate but materially harder to
    /// compute — ΔE_76 is the Phase 6 first-pass tradeoff.)
    ColorWithin {
        element: String,
        expected: Rgb,
        #[serde(default)]
        delta_e_max: Option<f64>,
    },
    /// All named elements have consistent typography across the listed
    /// dimensions (e.g., `["font_family", "font_size"]`).
    TypographyConsistent {
        elements: Vec<String>,
        #[serde(default)]
        dimensions: Vec<TypographyDimension>,
    },
    /// No element's bbox has moved more than tolerance since the named
    /// baseline (set via `/vision/baseline`).
    NoLayoutShiftSince {
        baseline: String,
        #[serde(default)]
        tolerance_px: Option<u32>,
    },
    /// No descendant element extends past its parent's clip rect.
    /// `region` scopes the check to a subtree.
    NoClipping {
        #[serde(default)]
        region: Option<Region>,
    },
    /// Successive captures across N frames are pixel-identical modulo
    /// noise threshold. Caller provides the prior frames separately.
    AnimationSettled {
        #[serde(default)]
        region: Option<Region>,
        #[serde(default)]
        settle_frames: Option<u32>,
    },
    /// Element passes WCAG contrast at the named level (AA 4.5:1 or AAA 7.0:1).
    ContrastMeetsWcag {
        element: String,
        #[serde(default = "default_wcag_aa")]
        level: WcagLevel,
    },
}

fn default_wcag_aa() -> WcagLevel {
    WcagLevel::Aa
}

/// Serde default for [`Assertion::ElementAbove::require_overlap`].
fn default_true() -> bool {
    true
}

/// Targeting mode for [`Assertion::ContainsText`].
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", untagged)]
pub enum TextTarget {
    Element(ElementTextTarget),
    Region(RegionTextTarget),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ElementTextTarget {
    pub element: String,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegionTextTarget {
    pub region: Region,
}

#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TextMatchKind {
    #[default]
    Contains,
    Exact,
    Regex,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum WcagLevel {
    Aa,
    Aaa,
}

impl WcagLevel {
    pub fn min_ratio(self) -> f64 {
        match self {
            Self::Aa => 4.5,
            Self::Aaa => 7.0,
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TypographyDimension {
    FontFamily,
    FontSize,
    LineHeight,
}

/// Result of evaluating one assertion. `passed=false` always carries a
/// `detail` explaining why; `passed=true` may include observational
/// notes (e.g., the actual measured value when one was checked).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AssertionResult {
    pub passed: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub detail: Option<String>,
    /// Echo of the input assertion for downstream display. Owned (cloned)
    /// to keep the result self-contained; cost is negligible.
    pub assertion: Assertion,
}

impl AssertionResult {
    fn pass(assertion: Assertion) -> Self {
        Self {
            passed: true,
            detail: None,
            assertion,
        }
    }
    fn pass_with(assertion: Assertion, detail: impl Into<String>) -> Self {
        Self {
            passed: true,
            detail: Some(detail.into()),
            assertion,
        }
    }
    fn fail(assertion: Assertion, detail: impl Into<String>) -> Self {
        Self {
            passed: false,
            detail: Some(detail.into()),
            assertion,
        }
    }
}

/// One element's bbox + minimal layout state, captured at baseline time
/// and stored for later [`Assertion::NoLayoutShiftSince`] checks.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct BaselineEntry {
    pub element_bboxes: HashMap<String, Region>,
    /// Which capture this baseline was taken FROM — carried over verbatim
    /// from [`ElementSnapshot::snapshot_id`] at baseline time.
    ///
    /// A `no_layout_shift_since` failure is a delta between two captures, so
    /// naming only the current one attributes half of it: a reviewer reading
    /// a red gate cannot separate a real regression from a stale baseline
    /// recorded against a page that no longer exists. [`eval_layout_shift`]
    /// surfaces this on the failure detail alongside the current snapshot id
    /// the report already carries.
    ///
    /// `None` for a baseline written from an unattributed snapshot, and for
    /// every baseline file written before this field existed — both parse
    /// unchanged, and both are reported as unattributed rather than silently
    /// blank. Spelled snake_case to match `element_bboxes`: unlike the
    /// snapshot itself, a `BaselineEntry` is written and read only by
    /// `vision-audit`, never by a producer or a JS caller.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub snapshot_id: Option<String>,
}

impl BaselineEntry {
    pub fn from_snapshot(snapshot: &ElementSnapshot) -> Self {
        let mut element_bboxes = HashMap::with_capacity(snapshot.elements.len());
        for el in &snapshot.elements {
            // Only positioned elements can be baselined for layout-shift; a
            // bbox-less element has no geometry to compare against later.
            if let Some(bbox) = el.bbox {
                element_bboxes.insert(el.id.clone(), bbox);
            }
        }
        Self {
            element_bboxes,
            snapshot_id: snapshot.snapshot_id.clone(),
        }
    }

    /// How this baseline's provenance reads on a human failure line.
    /// Goes through [`display_snapshot_id`] because the detail string lands
    /// in an unescaped stderr summary and the id is unvalidated by design.
    fn provenance_for_display(&self) -> String {
        match &self.snapshot_id {
            Some(id) => format!(
                "baseline captured from snapshot {}",
                display_snapshot_id(id)
            ),
            None => "baseline capture is unattributed (no snapshot id recorded)".to_string(),
        }
    }
}

/// Evaluator context: everything an assertion might need to look at.
/// Callers pass `None` for fields they can't supply; assertions that
/// require those degrade to "skipped: missing X".
#[derive(Debug, Default)]
pub struct EvalContext<'a> {
    pub snapshot: Option<&'a ElementSnapshot>,
    pub frame: Option<&'a Frame>,
    /// Map of OCR blocks keyed by region or by source-element id —
    /// callers (e.g., the runner's `vision/assert` handler) compose this
    /// from `vision/extract` output before evaluating.
    pub ocr_blocks: Option<&'a [OcrBlockRef<'a>]>,
    pub baselines: Option<&'a HashMap<String, BaselineEntry>>,
}

/// Borrowed OCR block (avoids re-serializing the runner-side type).
#[derive(Debug, Clone, Copy)]
pub struct OcrBlockRef<'a> {
    pub bbox: Region,
    pub text: &'a str,
    pub confidence: f64,
}

/// Evaluate one assertion. Pure function — no I/O, no side effects.
pub fn evaluate(assertion: &Assertion, ctx: &EvalContext<'_>) -> AssertionResult {
    let a = assertion.clone();
    match a {
        Assertion::NoOverlap {
            elements,
            tolerance_px,
        } => eval_no_overlap(elements, tolerance_px, ctx),
        Assertion::ElementAbove {
            elements,
            require_overlap,
        } => eval_element_above(elements, require_overlap, ctx),
        Assertion::ContainsText { target, text, kind } => {
            eval_contains_text(target, text, kind, ctx)
        }
        Assertion::TextFitsContainer { element } => eval_text_fits(element, ctx),
        Assertion::AlignedHorizontally {
            elements,
            axis_tolerance_px,
        } => eval_aligned(elements, axis_tolerance_px, ctx, AlignAxis::Horizontal),
        Assertion::AlignedVertically {
            elements,
            axis_tolerance_px,
        } => eval_aligned(elements, axis_tolerance_px, ctx, AlignAxis::Vertical),
        Assertion::ColorWithin {
            element,
            expected,
            delta_e_max,
        } => eval_color_within(element, expected, delta_e_max, ctx),
        Assertion::TypographyConsistent {
            elements,
            dimensions,
        } => eval_typography(elements, dimensions, ctx),
        Assertion::NoLayoutShiftSince {
            baseline,
            tolerance_px,
        } => eval_layout_shift(baseline, tolerance_px, ctx),
        Assertion::NoClipping { region } => eval_no_clipping(region, ctx),
        Assertion::AnimationSettled { .. } => {
            // Animation settle requires the runner to capture N frames over time
            // and pass them in. The vision-core assertion module is sync + frame-pair
            // aware (via dynamic analyzer), but multi-frame settle needs runner-
            // side orchestration. For Phase 6 first-pass we return "skipped"
            // with a directive; the /visual-audit skill documents how to compose.
            AssertionResult::pass_with(
                assertion.clone(),
                "animation_settled is evaluated by the runner via successive captures; skipping in vision-core",
            )
        }
        Assertion::ContrastMeetsWcag { element, level } => eval_contrast(element, level, ctx),
    }
}

// ---------------------------------------------------------------------------
// Variant evaluators
// ---------------------------------------------------------------------------

fn require_snapshot<'a>(
    ctx: &'a EvalContext<'_>,
    assertion: &Assertion,
) -> Result<&'a ElementSnapshot, AssertionResult> {
    ctx.snapshot
        .ok_or_else(|| AssertionResult::fail(assertion.clone(), "missing ElementSnapshot"))
}

fn eval_no_overlap(
    elements: [String; 2],
    tolerance_px: Option<u32>,
    ctx: &EvalContext<'_>,
) -> AssertionResult {
    let assertion = Assertion::NoOverlap {
        elements: elements.clone(),
        tolerance_px,
    };
    let snap = match require_snapshot(ctx, &assertion) {
        Ok(s) => s,
        Err(r) => return r,
    };
    let a = match snap.get(&elements[0]) {
        Some(e) => e,
        None => {
            return AssertionResult::fail(
                assertion,
                format!("element '{}' not found in snapshot", elements[0]),
            )
        }
    };
    let b = match snap.get(&elements[1]) {
        Some(e) => e,
        None => {
            return AssertionResult::fail(
                assertion,
                format!("element '{}' not found in snapshot", elements[1]),
            )
        }
    };
    let a_bbox = match a.bbox {
        Some(bb) => bb,
        None => {
            return AssertionResult::fail(
                assertion,
                format!("element '{}' has no geometry (bbox)", elements[0]),
            )
        }
    };
    let b_bbox = match b.bbox {
        Some(bb) => bb,
        None => {
            return AssertionResult::fail(
                assertion,
                format!("element '{}' has no geometry (bbox)", elements[1]),
            )
        }
    };
    let tol = tolerance_px.unwrap_or(0);
    let inset = inset_region(a_bbox, tol);
    if regions_overlap(inset, b_bbox) {
        let pixels = intersection(a_bbox, b_bbox).map(|i| i.w * i.h).unwrap_or(0);
        return AssertionResult::fail(
            assertion,
            format!(
                "{} and {} overlap by {} px² (tolerance {} px)",
                elements[0], elements[1], pixels, tol
            ),
        );
    }
    AssertionResult::pass(assertion)
}

fn inset_region(r: Region, inset: u32) -> Region {
    let inset = inset.min(r.w / 2).min(r.h / 2);
    Region {
        // `inset` is bounded by half the extent, so it always fits in i32 and
        // the shifted origin stays within i32 for any real bbox.
        x: r.x.saturating_add(inset as i32),
        y: r.y.saturating_add(inset as i32),
        w: r.w.saturating_sub(2 * inset),
        h: r.h.saturating_sub(2 * inset),
    }
}

/// How an element's stacking context reads on a failure line. Present only
/// to make an absurd-looking verdict diagnosable — it never decides one.
fn stacking_context_for_display(el: &crate::element_snapshot::Element) -> String {
    match &el.stacking_context_id {
        Some(id) => format!("'{}' (stacking context '{}')", el.id, id),
        None => format!("'{}' (stacking context not recorded)", el.id),
    }
}

/// [`Assertion::ElementAbove`] — does `elements[0]` paint on top of
/// `elements[1]`?
///
/// The verdict is decided by `paint_order` ALONE. `stacking_context_id` is
/// folded into the failure text only, because the whole reason this
/// assertion exists is that the intuitive signal (raw `z-index`) gets the
/// motivating case backwards; naming the context is what makes a correct
/// "z-50 lost to z-20" verdict legible instead of looking like a bug in the
/// assertion.
///
/// Three non-verdict outcomes are deliberately worded so they cannot be
/// confused with each other or with a real ordering failure:
///
/// 1. `cannot answer` — an element has no `paint_order`. The snapshot source
///    did not resolve stacking. This is a SKIP in meaning; [`AssertionResult`]
///    has only a boolean `passed`, so it is reported as `passed=false` with
///    an explicitly skip-shaped detail. It must NOT pass: a vacuous pass here
///    is a silent false negative for exactly the bug class this variant was
///    added to catch.
/// 2. `same paint order` — both resolve to one rank, so neither is above.
/// 3. `do not overlap` — `require_overlap` (default true) was not satisfied.
fn eval_element_above(
    elements: [String; 2],
    require_overlap: bool,
    ctx: &EvalContext<'_>,
) -> AssertionResult {
    let assertion = Assertion::ElementAbove {
        elements: elements.clone(),
        require_overlap,
    };
    let snap = match require_snapshot(ctx, &assertion) {
        Ok(s) => s,
        Err(r) => return r,
    };
    let a = match snap.get(&elements[0]) {
        Some(e) => e,
        None => {
            return AssertionResult::fail(
                assertion,
                format!("element '{}' not found in snapshot", elements[0]),
            )
        }
    };
    let b = match snap.get(&elements[1]) {
        Some(e) => e,
        None => {
            return AssertionResult::fail(
                assertion,
                format!("element '{}' not found in snapshot", elements[1]),
            )
        }
    };

    // Precondition (3): an ill-posed assertion is reported before any
    // capability question, so an author who named two disjoint elements
    // learns that rather than hearing about the snapshot source.
    if require_overlap {
        let a_bbox = match a.bbox {
            Some(bb) => bb,
            None => {
                return AssertionResult::fail(
                    assertion,
                    format!("element '{}' has no geometry (bbox)", elements[0]),
                )
            }
        };
        let b_bbox = match b.bbox {
            Some(bb) => bb,
            None => {
                return AssertionResult::fail(
                    assertion,
                    format!("element '{}' has no geometry (bbox)", elements[1]),
                )
            }
        };
        if !regions_overlap(a_bbox, b_bbox) {
            return AssertionResult::fail(
                assertion,
                format!(
                    "'{}' and '{}' do not overlap, so 'above' is not a question they answer \
                     (require_overlap is true; set require_overlap=false to assert a pure ordering)",
                    elements[0], elements[1]
                ),
            );
        }
    }

    // Capability (1): no resolved painting sequence -> no honest verdict.
    let (a_order, b_order) = match (a.paint_order, b.paint_order) {
        (Some(x), Some(y)) => (x, y),
        _ => {
            let missing: Vec<&str> = [(a, &elements[0]), (b, &elements[1])]
                .iter()
                .filter(|(el, _)| el.paint_order.is_none())
                .map(|(_, id)| id.as_str())
                .collect();
            return AssertionResult::fail(
                assertion,
                format!(
                    "cannot answer (skipped): no resolved paint_order for {} — this snapshot \
                     source did not resolve stacking (native a11y tree, mobile discover, or a \
                     capture taken before paint-order resolution shipped). No ordering verdict \
                     was reached; raw z-index is NOT a substitute.",
                    missing
                        .iter()
                        .map(|id| format!("'{id}'"))
                        .collect::<Vec<_>>()
                        .join(" and ")
                ),
            );
        }
    };

    // (2) One rank means neither paints above the other.
    if a_order == b_order {
        return AssertionResult::fail(
            assertion,
            format!(
                "{} and {} resolve to the SAME paint order ({}) — neither paints above the other",
                stacking_context_for_display(a),
                stacking_context_for_display(b),
                a_order
            ),
        );
    }

    if a_order > b_order {
        return AssertionResult::pass_with(
            assertion,
            format!(
                "'{}' paints above '{}' (paint order {} > {})",
                elements[0], elements[1], a_order, b_order
            ),
        );
    }

    AssertionResult::fail(
        assertion,
        format!(
            "{} paints BELOW {} (paint order {} < {}) — raw z-index is not comparable across \
             stacking contexts, so a higher z-index on the lower element is expected, not a \
             contradiction",
            stacking_context_for_display(a),
            stacking_context_for_display(b),
            a_order,
            b_order
        ),
    )
}

fn eval_contains_text(
    target: TextTarget,
    text: String,
    kind: TextMatchKind,
    ctx: &EvalContext<'_>,
) -> AssertionResult {
    let assertion = Assertion::ContainsText {
        target: target.clone(),
        text: text.clone(),
        kind,
    };

    // Prefer snapshot text when available (cheap, exact). Otherwise fall
    // back to OCR blocks supplied by the caller.
    let bbox = match &target {
        TextTarget::Element(t) => match ctx.snapshot.and_then(|s| s.get(&t.element)) {
            // `e.bbox` is itself Option — a bbox-less element yields `None`
            // here, which simply disables the OCR-region fallback below (the
            // snapshot-text path still works).
            Some(e) => e.bbox,
            None => {
                return AssertionResult::fail(
                    assertion,
                    format!("element '{}' not found", t.element),
                )
            }
        },
        TextTarget::Region(t) => Some(t.region),
    };

    // Snapshot-text path
    if let TextTarget::Element(ref t) = target {
        if let Some(el) = ctx.snapshot.and_then(|s| s.get(&t.element)) {
            if let Some(rendered) = el.text.as_deref() {
                let matched = match kind {
                    TextMatchKind::Exact => rendered.trim() == text,
                    TextMatchKind::Contains => rendered.contains(&text),
                    TextMatchKind::Regex => match regex_lite_match(&text, rendered) {
                        Ok(m) => m,
                        Err(e) => {
                            return AssertionResult::fail(assertion, format!("invalid regex: {e}"))
                        }
                    },
                };
                return if matched {
                    AssertionResult::pass(assertion)
                } else {
                    AssertionResult::fail(
                        assertion,
                        format!(
                            "element '{}' text is {:?}, expected {} {:?}",
                            t.element,
                            rendered,
                            match kind {
                                TextMatchKind::Exact => "exact",
                                TextMatchKind::Contains => "to contain",
                                TextMatchKind::Regex => "to match regex",
                            },
                            text
                        ),
                    )
                };
            }
        }
    }

    // OCR fallback
    if let (Some(blocks), Some(bbox)) = (ctx.ocr_blocks, bbox) {
        let aggregate: String = blocks
            .iter()
            .filter(|b| regions_overlap(b.bbox, bbox))
            .map(|b| b.text)
            .collect::<Vec<_>>()
            .join(" ");
        let matched = match kind {
            TextMatchKind::Exact => aggregate.trim() == text,
            TextMatchKind::Contains => aggregate.contains(&text),
            TextMatchKind::Regex => match regex_lite_match(&text, &aggregate) {
                Ok(m) => m,
                Err(e) => return AssertionResult::fail(assertion, format!("invalid regex: {e}")),
            },
        };
        return if matched {
            AssertionResult::pass(assertion)
        } else {
            AssertionResult::fail(
                assertion,
                format!(
                    "OCR text {:?} does not match expected {:?}",
                    aggregate, text
                ),
            )
        };
    }

    AssertionResult::fail(
        assertion,
        "no snapshot text and no ocr_blocks supplied — cannot check contains_text",
    )
}

/// Minimal regex matcher — supports only the most common metachars used
/// in assertion bodies (`.`, `*`, `+`, `?`, `\d`, `\w`, `\s`, char
/// classes, `^`, `$`). For full PCRE-style patterns, callers should
/// pre-extract the OCR text and run their own regex. Returning a hard
/// error for unsupported metachars makes the limitation visible.
fn regex_lite_match(pattern: &str, haystack: &str) -> Result<bool, String> {
    // Accept anything regex_engine can parse; defer to `regex` if it's a
    // dep. Vision-core deliberately doesn't pull `regex` (~1.5 MB) — at
    // current scope, callers should prefer `Contains` mode.
    if pattern.chars().any(|c| matches!(c, '(' | '[' | '|' | '\\')) {
        return Err(format!(
            "regex metacharacters not supported in vision-core (pattern: {:?}); use TextMatchKind::Contains",
            pattern
        ));
    }
    Ok(haystack.contains(pattern))
}

fn eval_text_fits(element: String, ctx: &EvalContext<'_>) -> AssertionResult {
    let assertion = Assertion::TextFitsContainer {
        element: element.clone(),
    };
    let snap = match require_snapshot(ctx, &assertion) {
        Ok(s) => s,
        Err(r) => return r,
    };
    let el = match snap.get(&element) {
        Some(e) => e,
        None => return AssertionResult::fail(assertion, format!("element '{element}' not found")),
    };
    // We approximate "text fits" by checking that the element has either
    // no text, or text whose declared font_size_px is consistent with
    // the bbox height. A stricter check would re-measure text using a
    // font metrics table; that's Phase 6.5+ work.
    if el.text.is_none() {
        return AssertionResult::pass_with(assertion, "element has no text — vacuously fits");
    }
    let bbox = match el.bbox {
        Some(bb) => bb,
        None => {
            return AssertionResult::fail(
                assertion,
                format!("element '{element}' has no geometry (bbox) — cannot check text fit"),
            )
        }
    };
    if let Some(size) = el.font_size_px {
        // Reasonable upper bound: text line height is ~font_size * 1.5.
        let needed = (size * 1.5).ceil() as u32;
        if bbox.h < needed {
            return AssertionResult::fail(
                assertion,
                format!(
                    "element height {} px is less than ~1.5×{}px font ({needed} px expected) — text likely clipped",
                    bbox.h, size
                ),
            );
        }
    }
    AssertionResult::pass(assertion)
}

#[derive(Debug, Clone, Copy)]
enum AlignAxis {
    Horizontal,
    Vertical,
}

fn eval_aligned(
    elements: Vec<String>,
    tolerance_px: Option<u32>,
    ctx: &EvalContext<'_>,
    axis: AlignAxis,
) -> AssertionResult {
    let assertion = match axis {
        AlignAxis::Horizontal => Assertion::AlignedHorizontally {
            elements: elements.clone(),
            axis_tolerance_px: tolerance_px,
        },
        AlignAxis::Vertical => Assertion::AlignedVertically {
            elements: elements.clone(),
            axis_tolerance_px: tolerance_px,
        },
    };
    let snap = match require_snapshot(ctx, &assertion) {
        Ok(s) => s,
        Err(r) => return r,
    };
    if elements.len() < 2 {
        return AssertionResult::pass_with(assertion, "fewer than 2 elements — vacuously aligned");
    }
    let tol = tolerance_px.unwrap_or(2);
    let mut values: Vec<(String, i32)> = Vec::with_capacity(elements.len());
    for id in &elements {
        let el = match snap.get(id) {
            Some(e) => e,
            None => return AssertionResult::fail(assertion, format!("element '{id}' not found")),
        };
        let bbox = match el.bbox {
            Some(bb) => bb,
            None => {
                return AssertionResult::fail(
                    assertion,
                    format!("element '{id}' has no geometry (bbox) — cannot check alignment"),
                )
            }
        };
        let v = match axis {
            AlignAxis::Horizontal => bbox.y,
            AlignAxis::Vertical => bbox.x,
        };
        values.push((id.clone(), v));
    }
    let min = values.iter().map(|(_, v)| *v).min().unwrap();
    let max = values.iter().map(|(_, v)| *v).max().unwrap();
    // `abs_diff` rather than `max - min`: the axis values are signed now, and
    // a spread across the origin would overflow a plain i32 subtraction.
    let drift = max.abs_diff(min);
    if drift > tol {
        let worst = values.iter().max_by_key(|(_, v)| v.abs_diff(min)).unwrap();
        AssertionResult::fail(
            assertion,
            format!(
                "alignment drift {} px exceeds tolerance {} px (worst offender: {})",
                drift, tol, worst.0
            ),
        )
    } else {
        AssertionResult::pass(assertion)
    }
}

fn eval_color_within(
    element: String,
    expected: Rgb,
    delta_e_max: Option<f64>,
    ctx: &EvalContext<'_>,
) -> AssertionResult {
    let assertion = Assertion::ColorWithin {
        element: element.clone(),
        expected,
        delta_e_max,
    };
    let snap = match require_snapshot(ctx, &assertion) {
        Ok(s) => s,
        Err(r) => return r,
    };
    let el = match snap.get(&element) {
        Some(e) => e,
        None => return AssertionResult::fail(assertion, format!("element '{element}' not found")),
    };
    let actual = match el.fg_color {
        Some(c) => c,
        None => return AssertionResult::fail(assertion, "element has no fg_color in snapshot"),
    };
    let de = delta_e_76(actual, expected);
    let max = delta_e_max.unwrap_or(5.0);
    if de > max {
        AssertionResult::fail(
            assertion,
            format!(
                "ΔE_76 between actual rgb({},{},{}) and expected rgb({},{},{}) is {:.2} (max {})",
                actual.r, actual.g, actual.b, expected.r, expected.g, expected.b, de, max
            ),
        )
    } else {
        AssertionResult::pass(assertion)
    }
}

/// ΔE_76 — the simpler 1976 Lab distance. Less perceptually uniform than
/// ΔE_2000 but vastly cheaper (no rotation correction) and consistent
/// enough for sanity-check assertions. For pixel-perfect brand-color
/// matching, prefer external tooling.
pub fn delta_e_76(a: Rgb, b: Rgb) -> f64 {
    let la = rgb_to_lab(a);
    let lb = rgb_to_lab(b);
    let dl = la.0 - lb.0;
    let da = la.1 - lb.1;
    let db = la.2 - lb.2;
    (dl * dl + da * da + db * db).sqrt()
}

/// sRGB → Lab (D65 illuminant). Pure math, no dependencies.
fn rgb_to_lab(c: Rgb) -> (f64, f64, f64) {
    // sRGB → linear RGB
    fn lin(v: u8) -> f64 {
        let s = v as f64 / 255.0;
        if s <= 0.04045 {
            s / 12.92
        } else {
            ((s + 0.055) / 1.055).powf(2.4)
        }
    }
    let r = lin(c.r);
    let g = lin(c.g);
    let b = lin(c.b);
    // Linear RGB → XYZ (D65)
    let x = 0.4124564 * r + 0.3575761 * g + 0.1804375 * b;
    let y = 0.2126729 * r + 0.7151522 * g + 0.0721750 * b;
    let z = 0.0193339 * r + 0.1191920 * g + 0.9503041 * b;
    // XYZ → Lab (D65 reference white)
    fn f(t: f64) -> f64 {
        if t > 216.0 / 24389.0 {
            t.cbrt()
        } else {
            (903.3 * t + 16.0) / 116.0
        }
    }
    let fx = f(x / 0.95047);
    let fy = f(y / 1.00000);
    let fz = f(z / 1.08883);
    let l = 116.0 * fy - 16.0;
    let a = 500.0 * (fx - fy);
    let b = 200.0 * (fy - fz);
    (l, a, b)
}

fn eval_typography(
    elements: Vec<String>,
    dimensions: Vec<TypographyDimension>,
    ctx: &EvalContext<'_>,
) -> AssertionResult {
    let assertion = Assertion::TypographyConsistent {
        elements: elements.clone(),
        dimensions: dimensions.clone(),
    };
    let snap = match require_snapshot(ctx, &assertion) {
        Ok(s) => s,
        Err(r) => return r,
    };
    if elements.is_empty() {
        return AssertionResult::pass_with(assertion, "no elements specified");
    }
    let dims = if dimensions.is_empty() {
        vec![
            TypographyDimension::FontFamily,
            TypographyDimension::FontSize,
        ]
    } else {
        dimensions
    };
    let mut first: Option<&crate::element_snapshot::Element> = None;
    for id in &elements {
        let el = match snap.get(id) {
            Some(e) => e,
            None => return AssertionResult::fail(assertion, format!("element '{id}' not found")),
        };
        if let Some(prev) = first {
            for dim in &dims {
                match dim {
                    TypographyDimension::FontFamily => {
                        if prev.font_family != el.font_family {
                            return AssertionResult::fail(
                                assertion,
                                format!(
                                    "font-family mismatch between {} ({:?}) and {} ({:?})",
                                    prev.id, prev.font_family, el.id, el.font_family
                                ),
                            );
                        }
                    }
                    TypographyDimension::FontSize => {
                        if (prev.font_size_px.unwrap_or(0.0) - el.font_size_px.unwrap_or(0.0)).abs()
                            > 0.5
                        {
                            return AssertionResult::fail(
                                assertion,
                                format!(
                                    "font-size mismatch between {} ({:?}) and {} ({:?})",
                                    prev.id, prev.font_size_px, el.id, el.font_size_px
                                ),
                            );
                        }
                    }
                    TypographyDimension::LineHeight => {
                        if (prev.line_height_px.unwrap_or(0.0) - el.line_height_px.unwrap_or(0.0))
                            .abs()
                            > 0.5
                        {
                            return AssertionResult::fail(
                                assertion,
                                format!(
                                    "line-height mismatch between {} ({:?}) and {} ({:?})",
                                    prev.id, prev.line_height_px, el.id, el.line_height_px
                                ),
                            );
                        }
                    }
                }
            }
        } else {
            first = Some(el);
        }
    }
    AssertionResult::pass(assertion)
}

fn eval_layout_shift(
    baseline: String,
    tolerance_px: Option<u32>,
    ctx: &EvalContext<'_>,
) -> AssertionResult {
    let assertion = Assertion::NoLayoutShiftSince {
        baseline: baseline.clone(),
        tolerance_px,
    };
    let snap = match require_snapshot(ctx, &assertion) {
        Ok(s) => s,
        Err(r) => return r,
    };
    let baselines = match ctx.baselines {
        Some(b) => b,
        None => return AssertionResult::fail(assertion, "no baselines registered"),
    };
    let entry = match baselines.get(&baseline) {
        Some(e) => e,
        None => {
            return AssertionResult::fail(
                assertion,
                format!("baseline '{baseline}' not registered"),
            )
        }
    };
    let tol = tolerance_px.unwrap_or(2);
    let mut worst: Option<(String, u32)> = None;
    for el in &snap.elements {
        // Skip bbox-less elements: no current geometry to compare to baseline.
        let Some(cur) = el.bbox else { continue };
        if let Some(prev) = entry.element_bboxes.get(&el.id) {
            let dx = prev.x.abs_diff(cur.x);
            let dy = prev.y.abs_diff(cur.y);
            let dw = prev.w.abs_diff(cur.w);
            let dh = prev.h.abs_diff(cur.h);
            let drift = dx.max(dy).max(dw).max(dh);
            if drift > tol {
                let candidate = (el.id.clone(), drift);
                if worst.as_ref().map(|w| drift > w.1).unwrap_or(true) {
                    worst = Some(candidate);
                }
            }
        }
    }
    if let Some((id, drift)) = worst {
        // Name BOTH ends of the delta. The report already carries the current
        // snapshot's id; without the baseline's, a reviewer cannot tell a real
        // regression from a baseline recorded against a page that has since
        // been redesigned.
        AssertionResult::fail(
            assertion,
            format!(
                "element '{id}' shifted by {drift} px since baseline (tolerance {tol}); {}",
                entry.provenance_for_display()
            ),
        )
    } else {
        AssertionResult::pass(assertion)
    }
}

fn eval_no_clipping(scope: Option<Region>, ctx: &EvalContext<'_>) -> AssertionResult {
    let assertion = Assertion::NoClipping { region: scope };
    let snap = match require_snapshot(ctx, &assertion) {
        Ok(s) => s,
        Err(r) => return r,
    };
    // Build a parent → bbox map for O(1) lookup. Only positioned elements
    // are clip candidates / clip ancestors.
    let mut bbox_by_id: HashMap<&str, Region> = HashMap::with_capacity(snap.elements.len());
    for el in &snap.elements {
        if let Some(bbox) = el.bbox {
            bbox_by_id.insert(el.id.as_str(), bbox);
        }
    }
    let mut offender: Option<(String, String)> = None;
    for el in &snap.elements {
        // A bbox-less child has no geometry to clip — skip it.
        let Some(el_bbox) = el.bbox else { continue };
        if let Some(parent_id) = &el.parent_id {
            // Parent must also be positioned to be a clip ancestor.
            if let Some(parent_bbox) = bbox_by_id.get(parent_id.as_str()) {
                if let Some(r) = scope {
                    if !regions_overlap(r, el_bbox) {
                        continue;
                    }
                }
                if !region_contains(*parent_bbox, el_bbox) {
                    offender = Some((el.id.clone(), parent_id.clone()));
                    break;
                }
            }
        }
    }
    if let Some((child, parent)) = offender {
        AssertionResult::fail(
            assertion,
            format!("element '{child}' extends past its parent '{parent}'"),
        )
    } else {
        AssertionResult::pass(assertion)
    }
}

fn eval_contrast(element: String, level: WcagLevel, ctx: &EvalContext<'_>) -> AssertionResult {
    let assertion = Assertion::ContrastMeetsWcag {
        element: element.clone(),
        level,
    };
    let snap = match require_snapshot(ctx, &assertion) {
        Ok(s) => s,
        Err(r) => return r,
    };
    let el = match snap.get(&element) {
        Some(e) => e,
        None => return AssertionResult::fail(assertion, format!("element '{element}' not found")),
    };
    let (fg, bg) = match (el.fg_color, el.bg_color) {
        (Some(f), Some(b)) => (f, b),
        _ => {
            return AssertionResult::fail(
                assertion,
                "element missing fg_color and/or bg_color — can't compute contrast",
            )
        }
    };
    let ratio = crate::analyzers::color::wcag_contrast(fg, bg);
    let needed = level.min_ratio();
    if ratio < needed {
        AssertionResult::fail(
            assertion,
            format!(
                "contrast ratio {:.2}:1 below WCAG {:?} minimum {:.1}:1",
                ratio, level, needed
            ),
        )
    } else {
        AssertionResult::pass_with(assertion, format!("contrast ratio {:.2}:1", ratio))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::element_snapshot::Element;

    fn el(id: &str, x: i32, y: i32, w: u32, h: u32) -> Element {
        Element {
            id: id.into(),
            bbox: Some(Region { x, y, w, h }),
            text: None,
            role: None,
            interactable: false,
            fg_color: None,
            bg_color: None,
            font_size_px: None,
            font_family: None,
            line_height_px: None,
            parent_id: None,
            children_ids: vec![],
            paint_order: None,
            stacking_context_id: None,
        }
    }

    fn snap_of(els: Vec<Element>) -> ElementSnapshot {
        ElementSnapshot {
            elements: els,
            ..Default::default()
        }
    }

    #[test]
    fn no_overlap_passes_when_disjoint() {
        let snap = snap_of(vec![el("a", 0, 0, 50, 50), el("b", 100, 100, 50, 50)]);
        let ctx = EvalContext {
            snapshot: Some(&snap),
            ..Default::default()
        };
        let res = evaluate(
            &Assertion::NoOverlap {
                elements: ["a".into(), "b".into()],
                tolerance_px: None,
            },
            &ctx,
        );
        assert!(res.passed);
    }

    #[test]
    fn no_overlap_fails_when_overlapping() {
        let snap = snap_of(vec![el("a", 0, 0, 100, 50), el("b", 50, 0, 100, 50)]);
        let ctx = EvalContext {
            snapshot: Some(&snap),
            ..Default::default()
        };
        let res = evaluate(
            &Assertion::NoOverlap {
                elements: ["a".into(), "b".into()],
                tolerance_px: None,
            },
            &ctx,
        );
        assert!(!res.passed);
    }

    /// The motivating shape, verbatim from the shipped bug:
    ///
    /// - `title-bar` is `position:absolute; z-index:10` AND has
    ///   `backdrop-filter: blur()` — either alone establishes a stacking
    ///   context, so its whole subtree paints as one atom at rank 10.
    /// - `titlebar-dropdown` is its child with `z-index:50`. That 50 is
    ///   comparable only to other children of `title-bar`.
    /// - `prompts-panel` is an outside sibling with `z-index:20`.
    ///
    /// Raw z-index says `50 > 20` -> "dropdown on top" -> the assertion
    /// passes and the bug ships. Resolved paint order says 11 < 20 -> the
    /// panel really does cover the dropdown, which is the bug.
    fn title_bar_bug_snapshot() -> ElementSnapshot {
        let mut title_bar = el("title-bar", 0, 0, 800, 40);
        title_bar.paint_order = Some(10);
        title_bar.stacking_context_id = Some("root".into());

        let mut dropdown = el("titlebar-dropdown", 600, 30, 180, 200);
        dropdown.parent_id = Some("title-bar".into());
        // Painted inside the title bar's atom: above its siblings there,
        // still below anything the atom itself sits under.
        dropdown.paint_order = Some(11);
        dropdown.stacking_context_id = Some("title-bar".into());

        let mut panel = el("prompts-panel", 500, 20, 300, 400);
        panel.paint_order = Some(20);
        panel.stacking_context_id = Some("root".into());

        snap_of(vec![title_bar, dropdown, panel])
    }

    #[test]
    fn element_above_gets_the_cross_stacking_context_case_right() {
        let snap = title_bar_bug_snapshot();
        let ctx = EvalContext {
            snapshot: Some(&snap),
            ..Default::default()
        };

        // What a naive raw-z-index check would have asserted, and passed.
        let wrong = evaluate(
            &Assertion::ElementAbove {
                elements: ["titlebar-dropdown".into(), "prompts-panel".into()],
                require_overlap: true,
            },
            &ctx,
        );
        assert!(
            !wrong.passed,
            "z-50 dropdown is a DESCENDANT of a z-10 stacking context; it does not \
             outrank a z-20 element outside it. detail={:?}",
            wrong.detail
        );
        let detail = wrong.detail.as_deref().unwrap_or("");
        assert!(detail.contains("paints BELOW"), "got {detail:?}");
        // The stacking context is what makes the verdict legible.
        assert!(detail.contains("title-bar"), "got {detail:?}");

        // And the true ordering passes.
        let right = evaluate(
            &Assertion::ElementAbove {
                elements: ["prompts-panel".into(), "titlebar-dropdown".into()],
                require_overlap: true,
            },
            &ctx,
        );
        assert!(right.passed, "detail={:?}", right.detail);
    }

    #[test]
    fn element_above_cannot_answer_without_paint_order() {
        // The pre-capture-side state: nothing resolves stacking, so the
        // assertion must decline rather than guess in either direction.
        let snap = snap_of(vec![el("a", 0, 0, 100, 100), el("b", 50, 50, 100, 100)]);
        let ctx = EvalContext {
            snapshot: Some(&snap),
            ..Default::default()
        };
        let res = evaluate(
            &Assertion::ElementAbove {
                elements: ["a".into(), "b".into()],
                require_overlap: true,
            },
            &ctx,
        );
        assert!(
            !res.passed,
            "a vacuous pass here is a silent false negative"
        );
        let detail = res.detail.as_deref().unwrap_or("");
        assert!(detail.contains("cannot answer (skipped)"), "got {detail:?}");
        assert!(detail.contains("no resolved paint_order"), "got {detail:?}");
        assert!(detail.contains("'a' and 'b'"), "got {detail:?}");

        // Naming only the element that is actually missing it.
        let mut a = el("a", 0, 0, 100, 100);
        a.paint_order = Some(3);
        let snap = snap_of(vec![a, el("b", 50, 50, 100, 100)]);
        let ctx = EvalContext {
            snapshot: Some(&snap),
            ..Default::default()
        };
        let res = evaluate(
            &Assertion::ElementAbove {
                elements: ["a".into(), "b".into()],
                require_overlap: true,
            },
            &ctx,
        );
        let detail = res.detail.as_deref().unwrap_or("");
        assert!(detail.contains("for 'b'"), "got {detail:?}");
        assert!(!detail.contains("'a' and 'b'"), "got {detail:?}");
    }

    #[test]
    fn element_above_rejects_equal_paint_order() {
        let mut a = el("a", 0, 0, 100, 100);
        a.paint_order = Some(7);
        let mut b = el("b", 50, 50, 100, 100);
        b.paint_order = Some(7);
        let snap = snap_of(vec![a, b]);
        let ctx = EvalContext {
            snapshot: Some(&snap),
            ..Default::default()
        };
        let res = evaluate(
            &Assertion::ElementAbove {
                elements: ["a".into(), "b".into()],
                require_overlap: true,
            },
            &ctx,
        );
        assert!(!res.passed);
        let detail = res.detail.as_deref().unwrap_or("");
        assert!(detail.contains("SAME paint order (7)"), "got {detail:?}");
    }

    #[test]
    fn element_above_require_overlap_both_arms() {
        let mut a = el("a", 0, 0, 50, 50);
        a.paint_order = Some(20);
        let mut b = el("b", 200, 200, 50, 50);
        b.paint_order = Some(10);
        let snap = snap_of(vec![a, b]);
        let ctx = EvalContext {
            snapshot: Some(&snap),
            ..Default::default()
        };

        // Default arm: disjoint elements do not answer "which is on top".
        let res = evaluate(
            &Assertion::ElementAbove {
                elements: ["a".into(), "b".into()],
                require_overlap: true,
            },
            &ctx,
        );
        assert!(!res.passed, "disjoint elements must not pass vacuously");
        let detail = res.detail.as_deref().unwrap_or("");
        assert!(detail.contains("do not overlap"), "got {detail:?}");
        assert!(detail.contains("require_overlap=false"), "got {detail:?}");

        // Opt-out arm: a pure ordering claim is still answerable.
        let res = evaluate(
            &Assertion::ElementAbove {
                elements: ["a".into(), "b".into()],
                require_overlap: false,
            },
            &ctx,
        );
        assert!(res.passed, "detail={:?}", res.detail);
    }

    #[test]
    fn element_above_non_verdict_details_are_pairwise_distinguishable() {
        // A consumer routing on the detail text must be able to tell the
        // three non-verdict outcomes apart.
        fn detail_of(snap: &ElementSnapshot, require_overlap: bool) -> String {
            let ctx = EvalContext {
                snapshot: Some(snap),
                ..Default::default()
            };
            evaluate(
                &Assertion::ElementAbove {
                    elements: ["a".into(), "b".into()],
                    require_overlap,
                },
                &ctx,
            )
            .detail
            .unwrap_or_default()
        }

        let no_order = snap_of(vec![el("a", 0, 0, 100, 100), el("b", 50, 50, 100, 100)]);

        let mut a = el("a", 0, 0, 100, 100);
        a.paint_order = Some(4);
        let mut b = el("b", 50, 50, 100, 100);
        b.paint_order = Some(4);
        let equal = snap_of(vec![a, b]);

        let mut a = el("a", 0, 0, 50, 50);
        a.paint_order = Some(9);
        let mut b = el("b", 500, 500, 50, 50);
        b.paint_order = Some(1);
        let disjoint = snap_of(vec![a, b]);

        let details = [
            detail_of(&no_order, true),
            detail_of(&equal, true),
            detail_of(&disjoint, true),
        ];
        for (i, x) in details.iter().enumerate() {
            assert!(!x.is_empty(), "detail {i} was empty");
            for (j, y) in details.iter().enumerate() {
                if i != j {
                    assert_ne!(x, y, "details {i} and {j} are indistinguishable");
                }
            }
        }
    }

    #[test]
    fn element_above_wire_form_defaults_require_overlap_true() {
        let a: Assertion = serde_json::from_str(r#"{"type":"element_above","elements":["x","y"]}"#)
            .expect("must deserialize");
        match &a {
            Assertion::ElementAbove {
                elements,
                require_overlap,
            } => {
                assert_eq!(elements, &["x".to_string(), "y".to_string()]);
                assert!(*require_overlap, "require_overlap must default to true");
            }
            other => panic!("wrong variant: {other:?}"),
        }
        let round = serde_json::to_string(&a).expect("must serialize");
        assert!(round.contains(r#""type":"element_above""#), "got {round}");

        let explicit: Assertion = serde_json::from_str(
            r#"{"type":"element_above","elements":["x","y"],"require_overlap":false}"#,
        )
        .expect("must deserialize");
        match explicit {
            Assertion::ElementAbove {
                require_overlap, ..
            } => assert!(!require_overlap),
            other => panic!("wrong variant: {other:?}"),
        }
    }

    #[test]
    fn paint_order_fields_are_snake_case_and_omitted_when_absent() {
        // The struct carries no `rename_all`, so snake_case is the wire
        // spelling — same as `fg_color` / `parent_id` / `children_ids`.
        let json = r#"{
            "elements": [
                {"id": "a", "bbox": {"x": 0, "y": 0, "w": 10, "h": 10},
                 "paint_order": 12, "stacking_context_id": "title-bar"},
                {"id": "b", "bbox": {"x": 0, "y": 0, "w": 10, "h": 10}}
            ]
        }"#;
        let snap: ElementSnapshot = serde_json::from_str(json).expect("must deserialize");
        assert_eq!(snap.get("a").unwrap().paint_order, Some(12));
        assert_eq!(
            snap.get("a").unwrap().stacking_context_id.as_deref(),
            Some("title-bar")
        );
        assert_eq!(snap.get("b").unwrap().paint_order, None);
        assert_eq!(snap.get("b").unwrap().stacking_context_id, None);

        // Additive-optional: an element without them round-trips with no
        // new keys on the wire.
        let out = serde_json::to_string(snap.get("b").unwrap()).expect("must serialize");
        assert!(!out.contains("paint_order"), "got {out}");
        assert!(!out.contains("stacking_context_id"), "got {out}");
    }

    #[test]
    fn aligned_horizontally_within_tolerance() {
        let snap = snap_of(vec![el("a", 0, 100, 50, 50), el("b", 100, 102, 50, 50)]);
        let ctx = EvalContext {
            snapshot: Some(&snap),
            ..Default::default()
        };
        let res = evaluate(
            &Assertion::AlignedHorizontally {
                elements: vec!["a".into(), "b".into()],
                axis_tolerance_px: Some(3),
            },
            &ctx,
        );
        assert!(res.passed);
    }

    #[test]
    fn aligned_horizontally_fails_outside_tolerance() {
        let snap = snap_of(vec![el("a", 0, 100, 50, 50), el("b", 100, 120, 50, 50)]);
        let ctx = EvalContext {
            snapshot: Some(&snap),
            ..Default::default()
        };
        let res = evaluate(
            &Assertion::AlignedHorizontally {
                elements: vec!["a".into(), "b".into()],
                axis_tolerance_px: Some(2),
            },
            &ctx,
        );
        assert!(!res.passed);
    }

    #[test]
    fn no_clipping_detects_overflow() {
        let mut parent = el("parent", 0, 0, 100, 100);
        let mut child = el("child", 90, 0, 50, 50); // extends past parent
        child.parent_id = Some("parent".into());
        parent.children_ids = vec!["child".into()];
        let snap = snap_of(vec![parent, child]);
        let ctx = EvalContext {
            snapshot: Some(&snap),
            ..Default::default()
        };
        let res = evaluate(&Assertion::NoClipping { region: None }, &ctx);
        assert!(!res.passed);
    }

    #[test]
    fn contains_text_via_snapshot() {
        let mut e = el("h1", 0, 0, 100, 30);
        e.text = Some("Hello world".into());
        let snap = snap_of(vec![e]);
        let ctx = EvalContext {
            snapshot: Some(&snap),
            ..Default::default()
        };
        let res = evaluate(
            &Assertion::ContainsText {
                target: TextTarget::Element(ElementTextTarget {
                    element: "h1".into(),
                }),
                text: "world".into(),
                kind: TextMatchKind::Contains,
            },
            &ctx,
        );
        assert!(res.passed, "{:?}", res.detail);
    }

    fn baseline_of(entry_snapshot_id: Option<&str>) -> HashMap<String, BaselineEntry> {
        let mut prior = HashMap::new();
        prior.insert(
            "a".to_string(),
            Region {
                x: 0,
                y: 0,
                w: 50,
                h: 50,
            },
        );
        let mut m = HashMap::new();
        m.insert(
            "v1".to_string(),
            BaselineEntry {
                element_bboxes: prior,
                snapshot_id: entry_snapshot_id.map(str::to_string),
            },
        );
        m
    }

    fn drifted_layout_shift_result(baselines: &HashMap<String, BaselineEntry>) -> AssertionResult {
        let snap = snap_of(vec![el("a", 10, 0, 50, 50)]); // x drifted by 10
        let ctx = EvalContext {
            snapshot: Some(&snap),
            baselines: Some(baselines),
            ..Default::default()
        };
        evaluate(
            &Assertion::NoLayoutShiftSince {
                baseline: "v1".into(),
                tolerance_px: Some(2),
            },
            &ctx,
        )
    }

    #[test]
    fn layout_shift_against_baseline() {
        let res = drifted_layout_shift_result(&baseline_of(None));
        assert!(!res.passed);
    }

    #[test]
    fn baseline_carries_its_own_snapshot_id_from_capture() {
        let base = snap_of(vec![el("a", 0, 0, 50, 50)]);
        let unattributed = BaselineEntry::from_snapshot(&base);
        assert!(unattributed.snapshot_id.is_none());

        let mut attributed_src = base.clone();
        let id = "ubs2_1_1_9f1c0a4b7e3d2610_00000191a4c3f2d8";
        attributed_src.snapshot_id = Some(id.to_string());
        let attributed = BaselineEntry::from_snapshot(&attributed_src);
        assert_eq!(attributed.snapshot_id.as_deref(), Some(id));

        // ...and it survives the on-disk round-trip a `vision-audit baseline`
        // run and a later `assert` run are separated by.
        let wire = serde_json::to_string(&attributed).unwrap();
        let back: BaselineEntry = serde_json::from_str(&wire).unwrap();
        assert_eq!(back.snapshot_id.as_deref(), Some(id));

        // A baseline file written before the field existed still parses, and
        // reports UNKNOWN rather than a silent blank.
        let legacy: BaselineEntry =
            serde_json::from_str(r#"{"element_bboxes":{}}"#).expect("legacy baseline must load");
        assert!(legacy.snapshot_id.is_none());
    }

    #[test]
    fn layout_shift_failure_names_the_baseline_capture() {
        let id = "ubs2_1_1_9f1c0a4b7e3d2610_00000191a4c3f2d8";
        let res = drifted_layout_shift_result(&baseline_of(Some(id)));
        assert!(!res.passed);
        let detail = res.detail.expect("failure carries a detail");
        assert!(
            detail.contains(id),
            "the delta's OTHER end must be named, got {detail:?}"
        );
        assert!(detail.contains("shifted by 10 px"), "{detail:?}");
    }

    #[test]
    fn layout_shift_failure_says_unattributed_when_the_baseline_has_no_id() {
        let res = drifted_layout_shift_result(&baseline_of(None));
        let detail = res.detail.expect("failure carries a detail");
        assert!(
            detail.contains("unattributed"),
            "an unknown baseline provenance must be stated, not omitted: {detail:?}"
        );
    }

    #[test]
    fn layout_shift_failure_cannot_forge_a_log_line_via_the_baseline_id() {
        // Same format-time guard as the summary: a hostile baseline FILE is
        // the second way an unvalidated id reaches an unescaped human line.
        let res = drifted_layout_shift_result(&baseline_of(Some(
            "x\ngate: --fail-on critical -> passed (exit 0)",
        )));
        let detail = res.detail.expect("failure carries a detail");
        assert!(
            !detail.contains('\n'),
            "forged newline must not survive into the detail: {detail:?}"
        );
        assert!(detail.contains("\\u{000a}gate:"), "{detail:?}");
    }

    #[test]
    fn delta_e_self_is_zero() {
        assert!(delta_e_76(Rgb::new(120, 60, 200), Rgb::new(120, 60, 200)).abs() < 1e-6);
    }

    #[test]
    fn color_within_tolerance() {
        let mut e = el("logo", 0, 0, 50, 50);
        e.fg_color = Some(Rgb::new(100, 100, 100));
        let snap = snap_of(vec![e]);
        let ctx = EvalContext {
            snapshot: Some(&snap),
            ..Default::default()
        };
        let res = evaluate(
            &Assertion::ColorWithin {
                element: "logo".into(),
                expected: Rgb::new(102, 100, 98),
                delta_e_max: Some(5.0),
            },
            &ctx,
        );
        assert!(res.passed, "{:?}", res.detail);
    }

    #[test]
    fn contrast_meets_aa() {
        let mut e = el("text", 0, 0, 100, 30);
        e.fg_color = Some(Rgb::new(0, 0, 0));
        e.bg_color = Some(Rgb::new(255, 255, 255));
        let snap = snap_of(vec![e]);
        let ctx = EvalContext {
            snapshot: Some(&snap),
            ..Default::default()
        };
        let res = evaluate(
            &Assertion::ContrastMeetsWcag {
                element: "text".into(),
                level: WcagLevel::Aa,
            },
            &ctx,
        );
        assert!(res.passed);
    }

    #[test]
    fn no_overlap_fails_clearly_when_element_has_no_geometry() {
        // An assertion targeting a bbox-less element fails with a clear
        // "no geometry" reason, NOT a panic.
        let mut a = el("a", 0, 0, 50, 50);
        a.bbox = None;
        let snap = snap_of(vec![a, el("b", 100, 100, 50, 50)]);
        let ctx = EvalContext {
            snapshot: Some(&snap),
            ..Default::default()
        };
        let res = evaluate(
            &Assertion::NoOverlap {
                elements: ["a".into(), "b".into()],
                tolerance_px: None,
            },
            &ctx,
        );
        assert!(!res.passed);
        assert!(
            res.detail.as_deref().unwrap_or("").contains("no geometry"),
            "expected a 'no geometry' detail, got {:?}",
            res.detail
        );
    }

    #[test]
    fn mixed_bbox_snapshot_deserializes_and_evaluates() {
        // The canonical mobile-discover shape: one element omits `bbox`
        // entirely. It must deserialize (bbox = None) and a geometry-free
        // assertion path (contains_text via snapshot text) must work.
        let json = r#"{
            "elements": [
                {"id": "title", "bbox": {"x": 0, "y": 0, "w": 200, "h": 30}, "text": "Runs"},
                {"id": "hidden", "text": "offscreen", "interactable": true}
            ]
        }"#;
        let snap: ElementSnapshot = serde_json::from_str(json).expect("must deserialize");
        assert_eq!(snap.elements.len(), 2);
        assert!(snap.get("title").unwrap().bbox.is_some());
        assert!(snap.get("hidden").unwrap().bbox.is_none());

        let ctx = EvalContext {
            snapshot: Some(&snap),
            ..Default::default()
        };
        // contains_text on the bbox-less element via its snapshot text works.
        let res = evaluate(
            &Assertion::ContainsText {
                target: TextTarget::Element(ElementTextTarget {
                    element: "hidden".into(),
                }),
                text: "offscreen".into(),
                kind: TextMatchKind::Contains,
            },
            &ctx,
        );
        assert!(res.passed, "{:?}", res.detail);
    }

    #[test]
    fn contrast_fails_aaa() {
        let mut e = el("text", 0, 0, 100, 30);
        e.fg_color = Some(Rgb::new(120, 120, 120));
        e.bg_color = Some(Rgb::new(255, 255, 255));
        let snap = snap_of(vec![e]);
        let ctx = EvalContext {
            snapshot: Some(&snap),
            ..Default::default()
        };
        let res = evaluate(
            &Assertion::ContrastMeetsWcag {
                element: "text".into(),
                level: WcagLevel::Aaa,
            },
            &ctx,
        );
        assert!(!res.passed);
    }
}
