// SPDX-License-Identifier: MIT
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
//! **One field deliberately declines `skip_serializing_if`:**
//! [`AssertionResult::confidence`] (and its analyzer-side twin
//! [`crate::Finding::confidence`]). Its `None` is a positive statement —
//! "no estimated input contributed to this verdict" — and a skipped key
//! cannot make a statement, only withhold one. So it always emits, as
//! `null`. That ADDS a key to the wire rather than removing one, so it is
//! additive **on the wire**: a consumer that does not read it is
//! unaffected, and one that does can finally tell a deduction from an
//! unstated confidence.
//!
//! **On the RUST side, adding a field is not free**, and the paragraph above
//! must not be read as saying it is. [`AssertionResult`] and
//! [`crate::Finding`] are plain `pub` structs with public fields and no
//! `#[non_exhaustive]`, so a downstream STRUCT LITERAL gains a required
//! field at re-pin and stops compiling. Callers going through
//! `Finding::new` / the `AssertionResult` constructors are unaffected. Given
//! the re-pin model above that surfaces as an honest build break rather than
//! silently, which is why it is acceptable — not because it cannot happen.
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
use crate::observation::{Observation, ObservationState, UnknownCode};
use crate::timeline::{ChangeTimeline, ExpectedStateOutcome, TimelineEvent, TimelineSource};

pub use crate::analyzers::image::NaturalRegion;

mod image;

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
    /// `[above, below]` — asserts `elements[0]` paints on top of `elements[1]`.
    ///
    /// The counterpart to [`crate::analyzers::layout`]'s `occlusion` finding:
    /// the analyzer REPORTS whatever stacking trouble it happens to trip over,
    /// while this variant lets an author DECLARE the expectation and get a
    /// pass/fail. "The dropdown must render over the panel" is a requirement,
    /// not an observation.
    ///
    /// `require_overlap` defaults **true**: "which of these is on top?" is not
    /// a question two disjoint elements answer, so asserting it over a pair
    /// that never touches is almost always an authoring mistake. Set it
    /// `false` for the rare case where the ordering matters independently of
    /// present geometry (an element about to be animated into place).
    ElementAbove {
        /// `[above, below]` — asserts elements[0] paints on top of elements[1].
        elements: [String; 2],
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
    /// Element's text fits within the element's content box, in BOTH axes.
    ///
    /// Horizontal: `scroll_width_px > bbox.w` means the laid-out text is
    /// wider than the box showing it, i.e. it is ellipsised or clipped.
    /// Vertical: `bbox.h` shorter than ~1.5x the font size.
    ///
    /// The horizontal arm needs the producer to populate `scroll_width_px`;
    /// without it that arm reports UNKNOWN rather than passing, because a
    /// silent pass on the un-measured axis is what let truncated labels
    /// through before.
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
    ///
    /// Still ΔE_76, deliberately, although [`delta_e_2000`] now exists: the
    /// two scales differ, and moving this variant to ΔE_2000 would silently
    /// re-mean every `delta_e_max` a caller has already written. A ΔE_2000
    /// colour check is a new variant, not a changed one —
    /// [`Assertion::ImagePaletteWithin`] is the first.
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
    /// No CSS animation or transition is still running when the
    /// interaction's timeline ended. Evaluated against
    /// [`EvalContext::timeline`]: an `animation-started` with no matching
    /// `animation-ended` after it, or a non-empty
    /// `summary.animationsUnsettledAtEnd`, fails; neither passes; no timeline
    /// is unknown.
    ///
    /// `region` scopes the claim. The timeline names an animation's target
    /// by element fingerprint, not by geometry, so with a region set a
    /// still-running animation cannot be placed inside or outside it: that
    /// is unknown, while "nothing is running anywhere" still passes.
    ///
    /// (The variant once took a `settle_frames` count for a multi-frame
    /// pixel comparison no caller ever supplied; the key is ignored if sent.)
    AnimationSettled {
        #[serde(default)]
        region: Option<Region>,
    },
    /// The UI settled within `ms` of the action: the timeline's
    /// `summary.settleMs` is at most `ms`. A recording that covered `ms` and
    /// never settled fails; one that stopped short of `ms` without settling
    /// is unknown. `ms` is the caller's bound — this crate ships no default.
    SettledWithin { ms: u64 },
    /// The declared UI state `state` became active within `ms` of the
    /// action. Read from the timeline's `summary.expectedStates` entry for
    /// `state` (request it with `expectStates` + `withinMs`), else from its
    /// `state-entered` events. A state that was already active throughout
    /// FAILS: the action did not enter it. `ms` is the caller's bound.
    StateEnteredWithin { state: String, ms: u64 },
    /// Element passes WCAG contrast at the named level (AA 4.5:1 or AAA 7.0:1).
    ContrastMeetsWcag {
        element: String,
        #[serde(default = "default_wcag_aa")]
        level: WcagLevel,
    },

    // ---- L1 image assertions (plan 2026-10-13-image-generation-and-visual-
    // judgment, Phase 15) ------------------------------------------------
    //
    // The first evaluators that read [`EvalContext::frame`]. Each one that
    // does sets [`AssertionResult::confidence`] (see that field). Thresholds
    // are the CALLER's — the visual system or the slot spec — and an
    // undeclared one answers `Unknown{input_missing}`, never a pass.
    /// At least `min_coverage` (0..=1) of the image's pixels, as read from
    /// the frame, lie within `max_delta_e` (CIEDE2000) of some colour in the
    /// declared `palette` (a slot spec's `thresholds.min_palette_coverage` /
    /// `max_palette_delta_e`).
    ///
    /// `palette` is a boxed slice rather than a `Vec` only to keep
    /// [`Assertion`] (and so every `AssertionResult`) under clippy's
    /// large-error threshold; the wire is the same JSON array.
    ImagePaletteWithin {
        element: String,
        palette: Box<[Rgb]>,
        #[serde(default)]
        max_delta_e: Option<f64>,
        #[serde(default)]
        min_coverage: Option<f64>,
    },
    /// No two images on the page have pHashes within `max_hamming` bits of
    /// each other.
    ///
    /// **Unknown on most real captures today, by design.** An image the
    /// producer reports `visible: false` without a `visibility_reason` — every
    /// off-screen image of a viewport-only frame, from every producer that
    /// does not yet send the reason — cannot be told from one hidden under a
    /// transparent overlay, so this answers Unknown rather than pass over it.
    /// Phase 26 must not promote this check to a gate before the producers
    /// (the style-gate `normalize.ts` and the ccfg converter) send
    /// `visibility_reason`. Perceptual, so a re-encoded or rescaled copy is caught; a
    /// distinct variant from Phase 8's `no_duplicate_images`, which compares
    /// `src_digest`. Flat fills are excluded (see
    /// [`Assertion::NoPlaceholderImages`]).
    NoNearDuplicateImages {
        #[serde(default)]
        max_hamming: Option<u32>,
    },
    /// No image (or the named one) renders a placeholder: the slot's
    /// placeholder marker, a low-entropy fill, or the crossed-box pattern.
    ///
    /// **Unknown on most real captures today, by design** — page-wide, for
    /// the same reason as [`Assertion::NoNearDuplicateImages`]: an image
    /// reported `visible: false` with no `visibility_reason` is unreadable,
    /// not left out. Phase 26 must not promote this check to a gate before
    /// the producers send `visibility_reason`.
    NoPlaceholderImages {
        #[serde(default)]
        element: Option<String>,
    },
    /// The whole `focal` region (natural-frame fractions — a slot spec's
    /// `focal.region`, passed verbatim) survives object-fit/object-position.
    /// Reads the snapshot's media facts, not the frame.
    ImageFocalVisible {
        element: String,
        focal: NaturalRegion,
    },
    /// The image's edge and stroke pixels reach `min_ratio` WCAG contrast
    /// against what surrounds them on this frame — a transparent icon that
    /// vanishes on one theme's background fails on that theme's frame.
    TransparentImageLegible {
        element: String,
        #[serde(default)]
        min_ratio: Option<f64>,
    },
    /// `cta_id` is the most salient element on the frame (spectral-residual
    /// saliency, mean over its box) among the interactive and media elements
    /// that neither contain it nor sit inside it.
    SalienceOrder { cta_id: String },
    /// The image's sharpness at its rendered scale — variance of the
    /// Laplacian over its frame crop — is at least `min_sharpness`.
    ImageSharpnessAtLeast {
        element: String,
        #[serde(default)]
        min_sharpness: Option<f64>,
    },
    /// The image's compression blockiness (block-edge step ÷ other step,
    /// ~1.0 clean) is at most `max_blockiness`.
    ImageBlockinessWithin {
        element: String,
        #[serde(default)]
        max_blockiness: Option<f64>,
    },
}

fn default_wcag_aa() -> WcagLevel {
    WcagLevel::Aa
}

/// Default for [`Assertion::ElementAbove`]'s `require_overlap`. A bare `bool`
/// field defaults to `false` under `#[serde(default)]`, which is the wrong
/// safety direction here — see the variant doc.
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

/// Which of the THREE things happened to an assertion.
///
/// `passed: bool` alone cannot say them: it collapses "evaluated, and the
/// expectation was violated" together with "could not be evaluated at all".
/// Those are different facts and call for different action — the first is a
/// bug in the page, the second is a gap in the snapshot (or in the caller's
/// inputs), and only the first should send anyone looking at the UI.
///
/// The bit is kept alongside as the GATE, and every consumer that reads it
/// keeps its current behaviour: [`AssertionOutcome::Unknown`] is
/// `passed = false`. That is deliberate and is the safe direction — a
/// vacuous pass over an un-evaluated assertion is a silent false negative
/// for exactly the bug the assertion was written to catch.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AssertionOutcome {
    /// The assertion was evaluated against real inputs and HELD.
    Passed,
    /// The assertion was evaluated against real inputs and was VIOLATED.
    /// A verdict was reached; the page is wrong.
    Failed,
    /// The assertion could NOT be evaluated — an input it needs was absent
    /// (no snapshot, an operand missing from the snapshot, a measurement
    /// field the producer does not populate, no registered baseline, no OCR
    /// blocks). Nothing is known about whether the expectation holds.
    ///
    /// This is NOT a weaker failure. Reading it as one sends a reader
    /// looking for a layout bug that no evidence points at; reading it as a
    /// pass hides one that no evidence would have ruled out.
    Unknown,
}

impl AssertionOutcome {
    /// The gate bit. Only [`Self::Passed`] is green — see the type doc for
    /// why `Unknown` deliberately is not.
    pub fn passed(self) -> bool {
        matches!(self, Self::Passed)
    }
    /// True when no verdict was reached. Callers that want to report
    /// coverage ("3 passed, 1 failed, 2 not evaluated") read this rather
    /// than subtracting counts.
    pub fn is_unknown(self) -> bool {
        matches!(self, Self::Unknown)
    }
}

/// Result of evaluating one assertion. A non-`Passed` outcome always carries
/// a `detail` explaining why; a pass may include observational notes (e.g.,
/// the actual measured value when one was checked).
///
/// `passed` and `outcome` are two views of one verdict and are always
/// consistent: `passed == outcome.passed()`. `passed` is retained as the
/// wire-stable gate bit existing consumers read (`vision-audit`'s exit code);
/// `outcome` is the finer answer, and the runner's assert route rolls up
/// `outcome` (failed > unknown > passed), never `passed`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[serde(try_from = "AssertionResultWire")]
pub struct AssertionResult {
    pub passed: bool,
    /// The three-way verdict. See [`AssertionOutcome`].
    pub outcome: AssertionOutcome,
    /// WHY the assertion could not be evaluated, typed. Present iff
    /// `outcome` is [`AssertionOutcome::Unknown`] — every unknown carries
    /// one, and no other outcome does, so the key's presence on the wire is
    /// itself the statement "not evaluated" and its value says why. The
    /// deserializer refuses a payload that breaks either half.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub code: Option<UnknownCode>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub detail: Option<String>,
    /// Echo of the input assertion for downstream display. Owned (cloned)
    /// to keep the result self-contained; cost is negligible.
    pub assertion: Assertion,
    /// How far this verdict rests on an ESTIMATED input, when it rests on
    /// one at all.
    ///
    /// **`None` is a positive statement, not a gap: no estimated input
    /// contributed to this verdict** — with one enumerated exception, named
    /// below, which this doc states rather than leaves for a reader to
    /// discover. Every other operand is exact — bboxes, declared
    /// `fg_color`/`bg_color`, snapshot text, a recorded baseline — so the
    /// result is a deduction and a number attached to it would manufacture
    /// precision nobody measured. That covers the OCR path's own no-op case
    /// too: when not one OCR block overlapped the target, nothing was read,
    /// so nothing estimated reached the verdict.
    ///
    /// It is therefore **always serialized**, as `"confidence": null`, and
    /// never skipped. A reader must be able to tell "this producer states
    /// the verdict is a deduction" from "this producer says nothing about
    /// confidence", and an omitted key cannot say the first — it says only
    /// the second, and that is what a payload written before this field
    /// existed means.
    ///
    /// `Some(c)` is in `0.0..=1.0`. Two families of evaluator set it: the L1
    /// image assertions that read [`EvalContext::frame`] (below), and
    /// [`Assertion::ContainsText`]'s OCR fallback, which reports the LOWEST
    /// [`OcrBlockRef::confidence`] among the blocks whose text it actually
    /// read. Lowest, not mean: the aggregate string is only as trustworthy
    /// as its worst-read fragment, and averaging would let one crisp block
    /// launder a garbled one.
    ///
    /// **One case yields a `Some(c)` that is not any block's reported
    /// value.** When blocks contributed and not one carried a FINITE
    /// confidence, the value is floored to `Some(0.0)`. An OCR reading
    /// decided the verdict, so `None` would be a false claim of exactness —
    /// and a `NaN` would reach the wire as `null` and make that same false
    /// claim, since `serde_json` writes a non-finite float as null. `0.0` is
    /// the conservative representable answer: a consumer thresholding on
    /// this discards a verdict it might have kept, and never keeps one it
    /// should have discarded.
    ///
    /// Note what that costs. "The engine reported `0.0`" and "no usable
    /// confidence was reported at all" are now ONE representation and cannot
    /// be told apart. That is deliberate and bounded — no consumer behaves
    /// differently on the two, and the alternatives are worse — but it IS a
    /// collapse, and this field exists so collapses get stated rather than
    /// absorbed.
    ///
    /// Note what does NOT set it, and that the two reasons are different.
    ///
    /// **The frame readers.** The L1 image assertions
    /// ([`Assertion::ImagePaletteWithin`], [`Assertion::NoNearDuplicateImages`],
    /// [`Assertion::NoPlaceholderImages`], [`Assertion::TransparentImageLegible`],
    /// [`Assertion::SalienceOrder`], [`Assertion::ImageSharpnessAtLeast`],
    /// [`Assertion::ImageBlockinessWithin`]) are the ONLY evaluators that read
    /// [`EvalContext::frame`], and a pixel read from a composited frame is an
    /// estimate of the image under it. Each sets this field from
    /// [`crate::analyzers::image::Crop::confidence`] — the share of the
    /// image's box that was inside the frame and not painted over — scaled
    /// where its own doc says so (the ring of surround pixels for
    /// `transparent_image_legible`; the saliency map's peakedness and the
    /// winning margin for `salience_order`). A verdict that rests on the
    /// slot's placeholder MARKER alone, and [`Assertion::ImageFocalVisible`]
    /// (media facts, no pixels), are deductions and state `None`.
    ///
    /// Every OTHER evaluator is exact, and the reason is mechanical rather
    /// than a hand-maintained list: none of them reads
    /// [`EvalContext::frame`], so none samples a pixel. They compare declared
    /// snapshot values ([`Assertion::ContrastMeetsWcag`],
    /// [`Assertion::ColorWithin`], [`Assertion::TypographyConsistent`]),
    /// recorded baselines ([`Assertion::NoLayoutShiftSince`]) and bboxes.
    /// `ContrastMeetsWcag` in particular returns `Unknown` when a colour is
    /// absent rather than falling back to sampling.
    ///
    /// The test to apply when adding an evaluator, so the next author
    /// classifies their own site without re-reading the crate: **does the
    /// verdict rest on a quantity the snapshot did not carry, which this
    /// crate stood in for?** A THRESHOLD over an exactly-measured quantity is
    /// not that — `dynamic`'s per-channel tolerance of 8, `typography`'s
    /// more-than-three-families drift, `elements`' 24 px target are policy
    /// choices over exact inputs and correctly state `None`.
    ///
    /// [`Assertion::TextFitsContainer`]'s **vertical** arm IS that, and is
    /// the one place in this evaluator where `None` is a KNOWN GAP in this
    /// field's coverage rather than a claim of exactness. It thresholds
    /// `bbox.h` against a hard-coded `font_size_px * 1.5` line-height guess
    /// and its own failure detail says "text likely clipped". It states
    /// `None` today because `1.5` is an unmeasured constant: deriving a
    /// number from it would manufacture exactly the precision this field
    /// exists to avoid. Its horizontal arm (`scroll_width_px` against the
    /// box) is a real measurement and is genuinely exact. Closing the
    /// vertical gap needs a measured line height on the wire, not a
    /// coefficient invented here.
    #[serde(default)]
    pub confidence: Option<f64>,
}

/// Deserialization shim for results written BEFORE `outcome` existed.
///
/// Such a payload carries `passed` only. `true` maps to
/// [`AssertionOutcome::Passed`] exactly; `false` maps to
/// [`AssertionOutcome::Failed`], which is not a guess — it is precisely what
/// that bit meant when the payload was written, since every un-evaluatable
/// case was reported as a failure back then. The information the third state
/// adds is simply not in the old wire form, and this shim does not invent
/// it.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct AssertionResultWire {
    passed: bool,
    outcome: Option<AssertionOutcome>,
    #[serde(default)]
    code: Option<UnknownCode>,
    #[serde(default)]
    detail: Option<String>,
    assertion: Assertion,
    /// Absent in every payload written before the field existed, and that
    /// absence is carried through as `None` — see [`AssertionResult::confidence`].
    #[serde(default)]
    confidence: Option<f64>,
}

impl TryFrom<AssertionResultWire> for AssertionResult {
    type Error = String;

    fn try_from(w: AssertionResultWire) -> Result<Self, Self::Error> {
        let outcome = w.outcome.unwrap_or(if w.passed {
            AssertionOutcome::Passed
        } else {
            AssertionOutcome::Failed
        });
        // `code` is present iff the outcome is `unknown`. Unlike `outcome`
        // itself there is no legacy form to repair toward: an `unknown`
        // written without a code does not say why, and that is the shape
        // this field exists to make unrepresentable.
        match (outcome, w.code) {
            (AssertionOutcome::Unknown, None) => {
                return Err("an `unknown` assertion result must carry a `code`".to_string())
            }
            (AssertionOutcome::Passed | AssertionOutcome::Failed, Some(code)) => {
                return Err(format!(
                    "`code` ({code}) is legal only on an `unknown` assertion result"
                ))
            }
            _ => {}
        }
        Ok(Self {
            // The outcome is authoritative: a payload whose two fields
            // disagree is repaired toward the richer one rather than
            // carried forward inconsistent.
            passed: outcome.passed(),
            outcome,
            code: w.code,
            detail: w.detail,
            assertion: w.assertion,
            confidence: w.confidence,
        })
    }
}

impl AssertionResult {
    fn of(assertion: Assertion, outcome: AssertionOutcome, detail: Option<String>) -> Self {
        Self {
            passed: outcome.passed(),
            outcome,
            code: None,
            detail,
            assertion,
            confidence: None,
        }
    }

    /// Record that this verdict rests on an estimated input, at `c`.
    ///
    /// Call this ONLY where the input really was estimated — see
    /// [`Self::confidence`]. Leaving it uncalled is the statement that the
    /// verdict is a deduction, and that statement reaches the wire.
    fn with_confidence(mut self, c: f64) -> Self {
        debug_assert!(
            (0.0..=1.0).contains(&c),
            "confidence {c} is outside 0.0..=1.0 — an OcrBlockRef built on a \
             percentage scale, or a share that overflowed"
        );
        self.confidence = Some(c);
        self
    }
    fn pass(assertion: Assertion) -> Self {
        Self::of(assertion, AssertionOutcome::Passed, None)
    }
    fn pass_with(assertion: Assertion, detail: impl Into<String>) -> Self {
        Self::of(assertion, AssertionOutcome::Passed, Some(detail.into()))
    }
    fn fail(assertion: Assertion, detail: impl Into<String>) -> Self {
        Self::of(assertion, AssertionOutcome::Failed, Some(detail.into()))
    }
    /// The assertion could not be evaluated. `detail` must say WHICH input
    /// was missing — an `unknown` whose detail does not name the gap is
    /// indistinguishable from a failure to the reader it exists to inform.
    ///
    /// `code` is required, and is the machine-readable half of the same
    /// statement: an agent branches on it without parsing `detail`.
    fn unknown(assertion: Assertion, code: UnknownCode, detail: impl Into<String>) -> Self {
        Self {
            code: Some(code),
            ..Self::of(assertion, AssertionOutcome::Unknown, Some(detail.into()))
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
    /// **Only the L1 image assertions read this** (the evaluators in
    /// `assertions/image.rs`), which keeps [`AssertionResult::confidence`]'s
    /// enumeration checkable by grep rather than by hand: every evaluator that
    /// samples a pixel lives in that one file, and each states a confidence.
    /// Any other evaluator that starts reading it is producing an ESTIMATE
    /// and owes one too — see that field.
    pub frame: Option<&'a Frame>,
    /// Map of OCR blocks keyed by region or by source-element id —
    /// callers (e.g., the runner's `vision/assert` handler) compose this
    /// from `vision/extract` output before evaluating.
    pub ocr_blocks: Option<&'a [OcrBlockRef<'a>]>,
    pub baselines: Option<&'a HashMap<String, BaselineEntry>>,
    /// The interaction timeline the timeline assertions
    /// ([`Assertion::AnimationSettled`], [`Assertion::SettledWithin`],
    /// [`Assertion::StateEnteredWithin`]) are evaluated against. `None` makes
    /// each of them `unknown{input_missing}`, naming where a timeline comes
    /// from ([`NO_TIMELINE`]).
    pub timeline: Option<&'a ChangeTimeline>,
}

/// Borrowed OCR block (avoids re-serializing the runner-side type).
#[derive(Debug, Clone, Copy)]
pub struct OcrBlockRef<'a> {
    pub bbox: Region,
    pub text: &'a str,
    /// The OCR engine's confidence for this block, in `0.0..=1.0`.
    ///
    /// **Callers normalise.** This crate passes the value through to
    /// [`AssertionResult::confidence`] unchanged, so an engine reporting a
    /// percentage must divide by 100 before building this struct — otherwise
    /// that field's documented range is a lie, and every consumer
    /// calibrating a threshold on it is calibrating against the wrong scale.
    /// The composing caller is out of this repo (the runner assembles these
    /// from `vision/extract` output), which is why the contract is stated
    /// here, on the surface it reads.
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
        Assertion::AnimationSettled { region } => eval_animation_settled(region, ctx),
        Assertion::SettledWithin { ms } => eval_settled_within(ms, ctx),
        Assertion::StateEnteredWithin { state, ms } => eval_state_entered_within(state, ms, ctx),
        Assertion::ContrastMeetsWcag { element, level } => eval_contrast(element, level, ctx),
        Assertion::ImagePaletteWithin { .. }
        | Assertion::NoNearDuplicateImages { .. }
        | Assertion::NoPlaceholderImages { .. }
        | Assertion::ImageFocalVisible { .. }
        | Assertion::TransparentImageLegible { .. }
        | Assertion::SalienceOrder { .. }
        | Assertion::ImageSharpnessAtLeast { .. }
        | Assertion::ImageBlockinessWithin { .. } => image::evaluate(a, ctx),
    }
}

// ---------------------------------------------------------------------------
// Variant evaluators
// ---------------------------------------------------------------------------

/// Detail for an operand the assertion named that the snapshot does not
/// contain. Always an [`AssertionOutcome::Unknown`]: an absent operand means
/// the check never ran, and the two live causes point at different people —
/// a wrong id is the author's, an unregistered surface is the producer's.
///
/// `consequence` completes "..., so {consequence}." and should name what was
/// NOT done, in the past tense.
fn missing_operand(id: &str, consequence: &str) -> String {
    format!(
        "element '{id}' is not in the snapshot, so {consequence}. Either the id is wrong, or \
         the surface is not registered with the producer."
    )
}

fn require_snapshot<'a>(
    ctx: &'a EvalContext<'_>,
    assertion: &Assertion,
) -> Result<&'a ElementSnapshot, AssertionResult> {
    ctx.snapshot.ok_or_else(|| {
        AssertionResult::unknown(
            assertion.clone(),
            UnknownCode::InputMissing,
            "no ElementSnapshot supplied, so this assertion was never evaluated",
        )
    })
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
            return AssertionResult::unknown(
                assertion,
                UnknownCode::InputMissing,
                missing_operand(&elements[0], "the pair was never compared"),
            )
        }
    };
    let b = match snap.get(&elements[1]) {
        Some(e) => e,
        None => {
            return AssertionResult::unknown(
                assertion,
                UnknownCode::InputMissing,
                missing_operand(&elements[1], "the pair was never compared"),
            )
        }
    };
    let a_bbox = match a.bbox {
        Some(bb) => bb,
        None => {
            return AssertionResult::unknown(
                assertion,
                UnknownCode::InputMissing,
                format!(
                    "element '{}' carries no geometry (bbox), so overlap was never measured",
                    elements[0]
                ),
            )
        }
    };
    let b_bbox = match b.bbox {
        Some(bb) => bb,
        None => {
            return AssertionResult::unknown(
                assertion,
                UnknownCode::InputMissing,
                format!(
                    "element '{}' carries no geometry (bbox), so overlap was never measured",
                    elements[1]
                ),
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

/// Evaluate [`Assertion::ElementAbove`].
///
/// **Check order is load-bearing** — element lookup, then (when required)
/// geometry, then ordering evidence. Producers do not populate `z_index` yet,
/// so an evidence-first order would answer every authoring mistake with
/// "missing z_index" and hide the mistake itself: an author who named two
/// disjoint elements must hear about THAT.
///
/// **Verdict rule**, derived from the semantics `analyzers::layout` already
/// established for the same three fields, in this precedence:
///
/// 1. [`crate::element_snapshot::Element::occluded_by`] — the producer hit-tested the live tree and
///    attributed the occlusion. That beats anything inferable here, exactly as
///    the analyzer's `attributed` pass beats its derived pass. No inference
///    needed, so it is consulted first and answers in both directions: it can
///    confirm the assertion *or* refute it.
/// 2. [`crate::element_snapshot::Element::z_index`] — the producer's RESOLVED painting order. Higher
///    paints on top; equal means neither does (document order decides, and a
///    snapshot does not carry it reliably — the analyzer declines to guess
///    there too, and so do we).
/// 3. Neither available → the assertion cannot be answered. See below.
///
/// **On the ancestry exemption.** `analyzers::layout` skips ancestor/descendant
/// pairs, because a descendant painting over its ancestor's box is what nesting
/// IS and reporting it would put a false `occlusion` on every nested layout.
/// That exemption is NOT mirrored here, deliberately: it exists to suppress
/// *unsolicited* findings over pairs nobody asked about, and an assertion has
/// no such noise problem — the author chose both ids. Worse, mirroring it would
/// make the single most common real question unanswerable, since a dropdown
/// asserted to paint over its own title bar is precisely a descendant/ancestor
/// pair. So a descendant over its ancestor is answered on the evidence, like
/// any other pair.
///
/// **On "cannot answer".** [`AssertionResult`] carries only `passed: bool`, so
/// there is no third state to return. This reports `passed = false` with a
/// skip-shaped detail rather than the vacuous-pass idiom `AnimationSettled`
/// uses: a pass with no evidence behind it is a silent false negative for
/// exactly the stacking bug this variant exists to catch. A tri-state
/// `AssertionResult` would be the better answer and remains a possible
/// follow-up.
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
    let (above_id, below_id) = (&elements[0], &elements[1]);

    // 1. Element lookup.
    let above = match snap.get(above_id) {
        Some(e) => e,
        None => {
            return AssertionResult::unknown(
                assertion,
                UnknownCode::InputMissing,
                missing_operand(above_id, "no ordering verdict was reached"),
            )
        }
    };
    let below = match snap.get(below_id) {
        Some(e) => e,
        None => {
            return AssertionResult::unknown(
                assertion,
                UnknownCode::InputMissing,
                missing_operand(below_id, "no ordering verdict was reached"),
            )
        }
    };

    // 2. Geometry, only when the caller requires the pair to actually overlap.
    if require_overlap {
        let (above_bbox, below_bbox) = match (above.bbox, below.bbox) {
            (Some(a), Some(b)) => (a, b),
            _ => {
                let which = match (above.bbox.is_none(), below.bbox.is_none()) {
                    (true, true) => format!("both '{above_id}' and '{below_id}'"),
                    (true, false) => format!("'{above_id}'"),
                    _ => format!("'{below_id}'"),
                };
                return AssertionResult::unknown(
                    assertion,
                    UnknownCode::InputMissing,
                    format!(
                        "element_above[{above_id}, {below_id}]: no geometry (bbox) on {which}, \
                         so the required overlap could not be checked. Set \
                         `require_overlap: false` to assert stacking order without it."
                    ),
                );
            }
        };
        if !regions_overlap(above_bbox, below_bbox) {
            return AssertionResult::fail(
                assertion,
                format!(
                    "element_above[{above_id}, {below_id}]: the two elements do not overlap, so \
                     which one paints on top is not a question they answer. Set \
                     `require_overlap: false` if the ordering matters anyway."
                ),
            );
        }
    }

    // 3a. Producer attribution — strongest evidence, answers both directions.
    if below.occluded_by.as_deref() == Some(above_id.as_str()) {
        return AssertionResult::pass_with(
            assertion,
            format!(
                "element_above[{above_id}, {below_id}]: confirmed by the producer's own \
                 attribution (`occluded_by`)"
            ),
        );
    }
    if above.occluded_by.as_deref() == Some(below_id.as_str()) {
        return AssertionResult::fail(
            assertion,
            format!(
                "element_above[{above_id}, {below_id}]: the producer attributed the occlusion the \
                 OTHER WAY — '{above_id}' is occluded_by '{below_id}', so '{below_id}' paints on \
                 top."
            ),
        );
    }

    // 3b. Resolved stacking order.
    let (Some(az), Some(bz)) = (above.z_index, below.z_index) else {
        let which = match (above.z_index.is_none(), below.z_index.is_none()) {
            (true, true) => format!("both '{above_id}' and '{below_id}'"),
            (true, false) => format!("'{above_id}'"),
            _ => format!("'{below_id}'"),
        };
        return AssertionResult::unknown(
            assertion,
            UnknownCode::InputMissing,
            format!(
                "element_above[{above_id}, {below_id}]: CANNOT ANSWER — no ordering verdict was \
                 reached. Neither element carries an `occluded_by` attribution, and `z_index` is \
                 missing on {which}. Reported as `unknown`, not as a failure: nothing was \
                 measured, so nothing is known — and not as a pass either, because a vacuous \
                 pass would silently hide the very stacking bug this assertion exists to catch. \
                 Note `z_index` is the producer's RESOLVED, GLOBAL paint rank: a raw CSS \
                 `z-index` value is NOT a substitute and must not be projected in its place."
            ),
        );
    };
    if az == bz {
        return AssertionResult::fail(
            assertion,
            format!(
                "element_above[{above_id}, {below_id}]: EQUAL resolved stacking order (z_index \
                 {az} on both), so neither paints above the other. At equal z the painting order \
                 falls to document order, which a snapshot does not carry reliably."
            ),
        );
    }
    if az < bz {
        return AssertionResult::fail(
            assertion,
            format!(
                "element_above[{above_id}, {below_id}]: '{above_id}' paints BELOW '{below_id}' \
                 (resolved z_index {az} < {bz})."
            ),
        );
    }
    AssertionResult::pass_with(
        assertion,
        format!("element_above[{above_id}, {below_id}]: resolved z_index {az} > {bz}"),
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
                return AssertionResult::unknown(
                    assertion,
                    UnknownCode::InputMissing,
                    missing_operand(&t.element, "its text was never read"),
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
        let contributing: Vec<&OcrBlockRef<'_>> = blocks
            .iter()
            .filter(|b| regions_overlap(b.bbox, bbox))
            .collect();
        let aggregate: String = contributing
            .iter()
            .map(|b| b.text)
            .collect::<Vec<_>>()
            .join(" ");
        // The only ESTIMATED input in this whole evaluator: OCR text.
        //
        // Non-finite block confidences are FILTERED rather than folded. One
        // malformed block must not erase a reading every other block agrees
        // on — `f64::min`'s non-NaN-wins behaviour would give that for free,
        // but only until every contributor is non-finite, and then the fold
        // yields `NaN`. That must never reach the wire: `serde_json` writes a
        // non-finite float as `null`, byte-identical to the `None` that
        // states "this was a deduction". A `debug_assert` does not stop it
        // either, since the runner consumes this crate in release.
        //
        // So the three cases are spelled out, because each is a different
        // fact. See [`AssertionResult::confidence`].
        let finite = contributing
            .iter()
            .map(|b| b.confidence)
            .filter(|c| c.is_finite())
            .reduce(f64::min);
        let confidence = match (finite, contributing.is_empty()) {
            // At least one usable reading: report the WORST of them, not the
            // mean. The aggregate string is only as trustworthy as its
            // worst-read fragment, and averaging would let one crisp block
            // launder a garbled one.
            (Some(c), _) => Some(c),
            // No contributing block at all: no OCR text was read, so nothing
            // estimated reached the verdict, and `None` is accurate.
            (None, true) => None,
            // Blocks contributed and not one carried a usable confidence. An
            // OCR reading DID decide this verdict, so `None` would be a false
            // claim of exactness. Floor it: estimated, and trusted not at all.
            (None, false) => Some(0.0),
        };
        let qualify = |r: AssertionResult| match confidence {
            Some(c) => r.with_confidence(c),
            None => r,
        };
        let matched = match kind {
            TextMatchKind::Exact => aggregate.trim() == text,
            TextMatchKind::Contains => aggregate.contains(&text),
            TextMatchKind::Regex => match regex_lite_match(&text, &aggregate) {
                Ok(m) => m,
                // An unsupported pattern is an authoring error, not an OCR
                // reading: no estimated input decided it, so it carries no
                // confidence.
                Err(e) => return AssertionResult::fail(assertion, format!("invalid regex: {e}")),
            },
        };
        return if matched {
            qualify(AssertionResult::pass(assertion))
        } else {
            qualify(AssertionResult::fail(
                assertion,
                format!(
                    "OCR text {:?} does not match expected {:?}",
                    aggregate, text
                ),
            ))
        };
    }

    AssertionResult::unknown(
        assertion,
        UnknownCode::InputMissing,
        "no snapshot text and no ocr_blocks supplied, so the text was never read",
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
        None => {
            return AssertionResult::unknown(
                assertion,
                UnknownCode::InputMissing,
                missing_operand(&element, "text fit was never checked"),
            )
        }
    };
    // Two axes, and they fail differently. HORIZONTAL truncation is a
    // measurement (`scroll_width_px` vs the box) and is the common case —
    // a `truncate` / `text-overflow: ellipsis` label whose name is cut to
    // a fragment. VERTICAL is an estimate from font size against the box
    // height. Only heights were checked before, so every ellipsised label
    // in the app passed this assertion.
    if el.text.is_none() {
        return AssertionResult::pass_with(assertion, "element has no text — vacuously fits");
    }
    let bbox = match el.bbox {
        Some(bb) => bb,
        None => {
            return AssertionResult::unknown(
                assertion,
                UnknownCode::InputMissing,
                format!(
                    "element '{element}' carries no geometry (bbox), so text fit was never checked"
                ),
            )
        }
    };
    // --- Horizontal: a measurement, so it is the stronger signal. ---
    if let Some(sw) = el.scroll_width_px {
        if sw > bbox.w {
            let how = match el.text_overflow.as_deref() {
                Some("ellipsis") => "ellipsised",
                Some("clip") => "clipped",
                _ => "truncated",
            };
            let shown = el.text.as_deref().unwrap_or_default().trim();
            return AssertionResult::fail(
                assertion,
                format!(
                    "text is {how}: content is {sw}px wide but the box is only {}px \
                     ({}px hidden). Full text: {shown:?}",
                    bbox.w,
                    sw - bbox.w
                ),
            );
        }
    }

    // --- Vertical: an estimate from the declared font size. ---
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

    // Both arms are clear — but say which ones actually RAN. An element
    // with no `scroll_width_px` was never checked for truncation, and
    // reporting that as an unqualified pass is what let ellipsised labels
    // through.
    if el.scroll_width_px.is_none() {
        return AssertionResult::pass_with(
            assertion,
            "vertical fit OK; horizontal fit UNKNOWN — the snapshot carries no \
             `scroll_width_px` for this element, so truncation was not checked",
        );
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
            None => {
                return AssertionResult::unknown(
                    assertion,
                    UnknownCode::InputMissing,
                    missing_operand(id, "alignment was never measured"),
                )
            }
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
        None => {
            return AssertionResult::unknown(
                assertion,
                UnknownCode::InputMissing,
                missing_operand(&element, "its color was never read"),
            )
        }
    };
    let actual = match el.fg_color {
        Some(c) => c,
        None => {
            return AssertionResult::unknown(
                assertion,
                UnknownCode::InputMissing,
                "the snapshot carries no `fg_color` for this element, so no color was compared",
            )
        }
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

/// CIEDE2000 (Sharma, Wu & Dalal 2005) between two sRGB colours, through
/// CIELAB D65. The perceptual distance [`delta_e_76`] approximates: ΔE_2000
/// corrects the hue rotation in the blues and the compressed chroma scale, so
/// a just-noticeable difference is ~1 everywhere in the gamut.
pub fn delta_e_2000(a: Rgb, b: Rgb) -> f64 {
    delta_e_2000_lab(rgb_to_lab(a), rgb_to_lab(b))
}

/// [`delta_e_2000`] over CIELAB triples `(L*, a*, b*)` (kL = kC = kH = 1).
pub fn delta_e_2000_lab(lab1: (f64, f64, f64), lab2: (f64, f64, f64)) -> f64 {
    let (l1, a1, b1) = lab1;
    let (l2, a2, b2) = lab2;
    let pow7 = |v: f64| v.powi(7);
    let c_bar = (a1.hypot(b1) + a2.hypot(b2)) / 2.0;
    let g = 0.5 * (1.0 - (pow7(c_bar) / (pow7(c_bar) + pow7(25.0))).sqrt());
    let a1p = (1.0 + g) * a1;
    let a2p = (1.0 + g) * a2;
    let c1p = a1p.hypot(b1);
    let c2p = a2p.hypot(b2);
    let hue = |b: f64, ap: f64| {
        if b == 0.0 && ap == 0.0 {
            0.0
        } else {
            b.atan2(ap).to_degrees().rem_euclid(360.0)
        }
    };
    let h1p = hue(b1, a1p);
    let h2p = hue(b2, a2p);
    let dl = l2 - l1;
    let dc = c2p - c1p;
    let dh_deg = if c1p * c2p == 0.0 {
        0.0
    } else {
        let d = h2p - h1p;
        if d.abs() <= 180.0 {
            d
        } else if d > 180.0 {
            d - 360.0
        } else {
            d + 360.0
        }
    };
    let dh = 2.0 * (c1p * c2p).sqrt() * (dh_deg / 2.0).to_radians().sin();
    let l_bar = (l1 + l2) / 2.0;
    let c_bar_p = (c1p + c2p) / 2.0;
    let h_bar = if c1p * c2p == 0.0 {
        h1p + h2p
    } else if (h1p - h2p).abs() <= 180.0 {
        (h1p + h2p) / 2.0
    } else if h1p + h2p < 360.0 {
        (h1p + h2p + 360.0) / 2.0
    } else {
        (h1p + h2p - 360.0) / 2.0
    };
    let cosd = |d: f64| d.to_radians().cos();
    let t =
        1.0 - 0.17 * cosd(h_bar - 30.0) + 0.24 * cosd(2.0 * h_bar) + 0.32 * cosd(3.0 * h_bar + 6.0)
            - 0.20 * cosd(4.0 * h_bar - 63.0);
    let d_theta = 30.0 * (-((h_bar - 275.0) / 25.0).powi(2)).exp();
    let r_c = 2.0 * (pow7(c_bar_p) / (pow7(c_bar_p) + pow7(25.0))).sqrt();
    let s_l = 1.0 + 0.015 * (l_bar - 50.0).powi(2) / (20.0 + (l_bar - 50.0).powi(2)).sqrt();
    let s_c = 1.0 + 0.045 * c_bar_p;
    let s_h = 1.0 + 0.015 * c_bar_p * t;
    let r_t = -(2.0 * d_theta).to_radians().sin() * r_c;
    let (tl, tc, th) = (dl / s_l, dc / s_c, dh / s_h);
    (tl * tl + tc * tc + th * th + r_t * tc * th)
        .max(0.0)
        .sqrt()
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
            None => {
                return AssertionResult::unknown(
                    assertion,
                    UnknownCode::InputMissing,
                    missing_operand(id, "typography was never compared"),
                )
            }
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
        None => {
            return AssertionResult::unknown(
                assertion,
                UnknownCode::InputMissing,
                "no baselines were supplied to the evaluator, so no shift was measured",
            )
        }
    };
    let entry = match baselines.get(&baseline) {
        Some(e) => e,
        None => {
            return AssertionResult::unknown(
                assertion,
                UnknownCode::InputMissing,
                format!(
                    "baseline '{baseline}' is not registered, so no shift was measured — \
                     register it with `vision/baseline` first"
                ),
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
        None => {
            return AssertionResult::unknown(
                assertion,
                UnknownCode::InputMissing,
                missing_operand(&element, "contrast was never computed"),
            )
        }
    };
    let (fg, bg) = match (el.fg_color, el.bg_color) {
        (Some(f), Some(b)) => (f, b),
        _ => {
            let which = match (el.fg_color.is_none(), el.bg_color.is_none()) {
                (true, true) => "`fg_color` and `bg_color`",
                (true, false) => "`fg_color`",
                _ => "`bg_color`",
            };
            return AssertionResult::unknown(
                assertion,
                UnknownCode::InputMissing,
                format!(
                    "the snapshot carries no {which} for this element, so contrast was never \
                     computed"
                ),
            );
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

// ---------------------------------------------------------------------------
// Timeline assertions — evaluated against `EvalContext::timeline`
// ---------------------------------------------------------------------------
//
// No constant here bounds a timing: the only bound is the caller's `ms`.

/// Where an interaction timeline comes from — the detail every timeline
/// assertion's no-timeline `unknown{input_missing}` carries, so the reader
/// learns which route would let it answer.
pub const NO_TIMELINE: &str = "no interaction timeline was supplied, so nothing was measured. \
     A timeline is recorded by the UI Bridge SDK's `POST /ai/execute-with-diff` \
     (`ActionDiffResult.timeline`, on unless the request sends `timeline: false`) and reaches \
     this evaluator through the runner's `POST /vision/assert` `timeline_ref` (a journey \
     edge-observation id), or `EvalContext::timeline` directly";

fn require_timeline<'a>(
    ctx: &'a EvalContext<'_>,
    assertion: &Assertion,
) -> Result<&'a ChangeTimeline, AssertionResult> {
    ctx.timeline.ok_or_else(|| {
        AssertionResult::unknown(assertion.clone(), UnknownCode::InputMissing, NO_TIMELINE)
    })
}

/// The status word and, for an unknown, its typed reason and detail.
fn not_measured<T>(o: &Observation<T>) -> (&'static str, Option<(UnknownCode, &str)>) {
    match o.state() {
        ObservationState::Measured(_) => ("measured", None),
        ObservationState::Absent => ("absent", None),
        ObservationState::Unknown(u) => ("unknown", Some((u.code, u.detail.as_str()))),
    }
}

fn eval_animation_settled(region: Option<Region>, ctx: &EvalContext<'_>) -> AssertionResult {
    let assertion = Assertion::AnimationSettled { region };
    let t = match require_timeline(ctx, &assertion) {
        Ok(t) => t,
        Err(r) => return r,
    };

    // Replay the animation events: a start with no later matching end is
    // still running. Matched on (name, target), else on name alone (an end
    // whose target could not be fingerprinted). Only meaningful when the
    // recording listened for them.
    let mut running: Vec<&crate::timeline::DetectedAnimation> = Vec::new();
    if t.observed(TimelineSource::AnimationEvents) {
        for ev in &t.events {
            match ev {
                TimelineEvent::AnimationStarted { animation, .. } => running.push(animation),
                TimelineEvent::AnimationEnded { animation, .. } => {
                    let at = running
                        .iter()
                        .position(|r| {
                            r.name == animation.name && r.target_element == animation.target_element
                        })
                        .or_else(|| running.iter().position(|r| r.name == animation.name));
                    if let Some(i) = at {
                        running.remove(i);
                    }
                }
                _ => {}
            }
        }
    }
    let summary = &t.summary.animations_unsettled_at_end;
    let mut names: std::collections::BTreeSet<&str> =
        running.iter().map(|a| a.name.as_str()).collect();
    if let Some(list) = summary.value() {
        names.extend(list.iter().map(|a| a.name.as_str()));
    }

    if !names.is_empty() {
        let names = names.into_iter().collect::<Vec<_>>().join(", ");
        // A running verdict rests on an `animation-ended` that never came. If
        // the recorder dropped `animation-ended` events at its cap, that end
        // may have been dropped: unknown, not a fail. (A dropped
        // `animation-started`, or no `getAnimations`, can only HIDE a running
        // animation, so neither can make this verdict wrong.)
        if t.summary
            .unmeasured_dimensions()
            .any(|d| d == "timeline-events:animation-ended")
        {
            return AssertionResult::unknown(
                assertion,
                UnknownCode::InputMissing,
                format!(
                    "animation(s) look still running when the {}ms recording ended ({names}), \
                     but the recorder dropped `animation-ended` events at its cap, so their end \
                     may have been lost",
                    t.recorded_ms
                ),
            );
        }
        if region.is_some() {
            return AssertionResult::unknown(
                assertion,
                UnknownCode::InputMissing,
                format!(
                    "animation(s) still running when the {}ms recording ended ({names}), but the \
                     timeline names an animation's target by element fingerprint, not geometry, \
                     so whether any of them is inside the region cannot be decided",
                    t.recorded_ms
                ),
            );
        }
        return AssertionResult::fail(
            assertion,
            format!(
                "animation(s) still running when the {}ms recording ended: {names}",
                t.recorded_ms
            ),
        );
    }

    if let (status, Some((code, detail))) = not_measured(summary) {
        return AssertionResult::unknown(
            assertion,
            code,
            format!(
                "no animation was seen running, but summary.animationsUnsettledAtEnd is \
                 {status} ({detail}), so one running unseen cannot be ruled out"
            ),
        );
    }
    let mut gaps: Vec<&str> = summary
        .provenance()
        .coverage
        .unmeasured
        .iter()
        .map(|u| u.dimension.as_str())
        .collect();
    gaps.extend(
        t.summary
            .unmeasured_dimensions()
            .filter(|d| d.starts_with("timeline-events:animation-")),
    );
    if !gaps.is_empty() {
        gaps.sort_unstable();
        gaps.dedup();
        return AssertionResult::unknown(
            assertion,
            UnknownCode::InputMissing,
            format!(
                "no animation was seen running, but {} went unmeasured, so one running unseen \
                 cannot be ruled out",
                gaps.join(", ")
            ),
        );
    }
    AssertionResult::pass_with(
        assertion,
        format!(
            "no animation running when the {}ms recording ended",
            t.recorded_ms
        ),
    )
}

fn eval_settled_within(ms: u64, ctx: &EvalContext<'_>) -> AssertionResult {
    let assertion = Assertion::SettledWithin { ms };
    let t = match require_timeline(ctx, &assertion) {
        Ok(t) => t,
        Err(r) => return r,
    };
    let bound = ms as f64;
    let settle = &t.summary.settle_ms;
    if let Some(&at) = settle.value() {
        return if at <= bound {
            AssertionResult::pass_with(assertion, format!("settled at {at}ms (bound {ms}ms)"))
        } else {
            AssertionResult::fail(assertion, format!("settled at {at}ms, later than {ms}ms"))
        };
    }
    let (status, unknown) = not_measured(settle);
    // A settle that did not happen is a measured negative over the window the
    // recording covered — but only for the "it did not occur" readings, never
    // for an unknown with another cause.
    let did_not_occur = match unknown {
        None => true,
        Some((code, _)) => code == UnknownCode::NoEventBeforeTimeout,
    };
    if did_not_occur && !t.settled && t.recorded_ms >= bound {
        if t.summary
            .unmeasured_dimensions()
            .any(|d| d == "timeline-events:settled")
        {
            return AssertionResult::unknown(
                assertion,
                UnknownCode::InputMissing,
                format!(
                    "no `settled` event within {ms}ms was recorded, but the recorder dropped \
                     `settled` events at its cap, so one may have been lost"
                ),
            );
        }
        return AssertionResult::fail(
            assertion,
            format!(
                "did not settle within {ms}ms: the recording covered {}ms and no `settled` \
                 event occurred",
                t.recorded_ms
            ),
        );
    }
    let (code, detail) = unknown.unwrap_or((UnknownCode::NoEventBeforeTimeout, "no settle event"));
    AssertionResult::unknown(
        assertion,
        code,
        format!(
            "summary.settleMs is {status} ({detail}) and the recording covered {}ms of the {ms}ms \
             asked about, so whether the UI settled within {ms}ms is not known",
            t.recorded_ms
        ),
    )
}

fn eval_state_entered_within(state: String, ms: u64, ctx: &EvalContext<'_>) -> AssertionResult {
    let assertion = Assertion::StateEnteredWithin {
        state: state.clone(),
        ms,
    };
    let t = match require_timeline(ctx, &assertion) {
        Ok(t) => t,
        Err(r) => return r,
    };
    let bound = ms as f64;

    // 1. The SDK's own answer, when the caller asked for this state. A
    // DEGRADED answer (its coverage names an unmeasured dimension) still
    // passes on a measured entry inside the bound, but never fails: whatever
    // went unmeasured could hold the entry that would make it pass.
    if let Some(entry) = t
        .summary
        .expected_states
        .iter()
        .find(|e| e.state_id == state)
    {
        let gaps: Vec<&str> = entry
            .outcome
            .provenance()
            .coverage
            .unmeasured
            .iter()
            .map(|u| u.dimension.as_str())
            .collect();
        let would_fail = match entry.outcome.state() {
            ObservationState::Measured(ExpectedStateOutcome::EnteredAtMs(at)) => *at > bound,
            ObservationState::Measured(
                ExpectedStateOutcome::NotEnteredWithinMs(w)
                | ExpectedStateOutcome::ActiveThroughoutWithinMs(w),
            ) => *w >= bound,
            _ => false,
        };
        if would_fail && !gaps.is_empty() {
            return AssertionResult::unknown(
                assertion,
                UnknownCode::InputMissing,
                format!(
                    "summary.expectedStates['{state}'] says it did not enter within {ms}ms, but \
                     it is degraded ({}), so an entry may have gone unmeasured",
                    gaps.join(", ")
                ),
            );
        }
        match entry.outcome.state() {
            ObservationState::Measured(ExpectedStateOutcome::EnteredAtMs(at)) => {
                return if *at <= bound {
                    AssertionResult::pass_with(
                        assertion,
                        format!("'{state}' entered at {at}ms (bound {ms}ms)"),
                    )
                } else {
                    AssertionResult::fail(
                        assertion,
                        format!("'{state}' entered at {at}ms, later than {ms}ms"),
                    )
                };
            }
            ObservationState::Measured(ExpectedStateOutcome::NotEnteredWithinMs(w))
                if *w >= bound =>
            {
                return AssertionResult::fail(
                    assertion,
                    format!("'{state}' did not enter within the {w}ms window, which covers {ms}ms"),
                );
            }
            ObservationState::Measured(ExpectedStateOutcome::ActiveThroughoutWithinMs(w))
                if *w >= bound =>
            {
                return AssertionResult::fail(
                    assertion,
                    format!(
                        "'{state}' was already active when the action ran and stayed active for \
                         the whole {w}ms window, so the action did not enter it"
                    ),
                );
            }
            ObservationState::Unknown(u) => {
                return AssertionResult::unknown(
                    assertion,
                    u.code,
                    format!("summary.expectedStates['{state}'] is unknown: {}", u.detail),
                );
            }
            // A window shorter than `ms`, or `absent`: the events may still
            // settle it.
            ObservationState::Measured(_) | ObservationState::Absent => {}
        }
    }

    // 2. The state events.
    if !t.observed(TimelineSource::States) {
        return AssertionResult::unknown(
            assertion,
            UnknownCode::InputMissing,
            format!(
                "the timeline did not observe the declared-state registry (no `states` source) \
                 and answers nothing for '{state}' within {ms}ms; record it with \
                 `expectStates: [\"{state}\"]` and `withinMs`"
            ),
        );
    }
    let mut named = false;
    for ev in &t.events {
        match ev {
            TimelineEvent::StateEntered { base, state_id } if *state_id == state => {
                named = true;
                if base.offset_ms <= bound {
                    return AssertionResult::pass_with(
                        assertion,
                        format!("'{state}' entered at {}ms (bound {ms}ms)", base.offset_ms),
                    );
                }
            }
            TimelineEvent::StateExited { state_id, .. } if *state_id == state => named = true,
            _ => {}
        }
    }
    let dropped = t
        .summary
        .unmeasured_dimensions()
        .any(|d| d == "timeline-events:state-entered");
    if dropped {
        return AssertionResult::unknown(
            assertion,
            UnknownCode::InputMissing,
            format!(
                "no `state-entered` for '{state}' within {ms}ms was recorded, but the recorder \
                 dropped `state-entered` events at its cap, so one may have been lost"
            ),
        );
    }
    if t.recorded_ms < bound {
        return AssertionResult::unknown(
            assertion,
            UnknownCode::NoEventBeforeTimeout,
            format!(
                "'{state}' had not entered when the recording ended at {}ms, short of the {ms}ms \
                 asked about",
                t.recorded_ms
            ),
        );
    }
    if !named {
        return AssertionResult::unknown(
            assertion,
            UnknownCode::InputMissing,
            format!(
                "no event names '{state}' and the timeline carries no expectedStates entry for \
                 it, so a declared state that never entered cannot be told from an undeclared \
                 id; record it with `expectStates: [\"{state}\"]` and `withinMs`"
            ),
        );
    }
    AssertionResult::fail(
        assertion,
        format!(
            "'{state}' did not enter within {ms}ms (the recording covered {}ms)",
            t.recorded_ms
        ),
    )
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
            ..Default::default()
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

    // -----------------------------------------------------------------
    // element_above
    // -----------------------------------------------------------------

    fn above(elements: [&str; 2], require_overlap: bool) -> Assertion {
        Assertion::ElementAbove {
            elements: [elements[0].into(), elements[1].into()],
            require_overlap,
        }
    }

    fn eval_on(snap: &ElementSnapshot, assertion: &Assertion) -> AssertionResult {
        let ctx = EvalContext {
            snapshot: Some(snap),
            ..Default::default()
        };
        evaluate(assertion, &ctx)
    }

    /// The reported bug shape: the prompts panel (Tailwind `z-20`) vs the
    /// title bar's dropdown (`z-50`), where the dropdown is a DESCENDANT of
    /// the `z-10` title bar.
    ///
    /// A raw CSS comparison reads 50 > 20 and concludes the dropdown wins.
    /// That is exactly backwards: the `z-10` title bar establishes a stacking
    /// context, so the dropdown's `z-50` is resolved WITHIN it and its whole
    /// subtree paints between the title bar and the next sibling context —
    /// below the panel's z-20. `Element::z_index` is documented as the
    /// RESOLVED order, so the fixture carries what a layout engine actually
    /// computes: dropdown just above its own title bar, still under the panel.
    fn prompts_panel_stacking() -> ElementSnapshot {
        snap_of(vec![
            Element {
                id: "title-bar".into(),
                bbox: Some(Region {
                    x: 0,
                    y: 0,
                    w: 1000,
                    h: 48,
                }),
                z_index: Some(10),
                ..Default::default()
            },
            Element {
                // CSS `z-50`, resolved inside the title bar's context.
                id: "title-bar-dropdown".into(),
                bbox: Some(Region {
                    x: 700,
                    y: 40,
                    w: 240,
                    h: 300,
                }),
                parent_id: Some("title-bar".into()),
                z_index: Some(11),
                ..Default::default()
            },
            Element {
                // CSS `z-20`, a sibling stacking context of the title bar's.
                id: "prompts-panel".into(),
                bbox: Some(Region {
                    x: 600,
                    y: 0,
                    w: 400,
                    h: 800,
                }),
                z_index: Some(20),
                ..Default::default()
            },
        ])
    }

    #[test]
    fn element_above_uses_resolved_stacking_not_raw_css_z_index() {
        let snap = prompts_panel_stacking();

        // What the bug report assumed (50 > 20) — and it is false.
        let res = eval_on(&snap, &above(["title-bar-dropdown", "prompts-panel"], true));
        assert!(
            !res.passed,
            "dropdown must NOT be reported above the panel: {:?}",
            res.detail
        );
        let detail = res.detail.unwrap();
        assert!(detail.contains("paints BELOW"), "{detail}");

        // What actually renders.
        let res = eval_on(&snap, &above(["prompts-panel", "title-bar-dropdown"], true));
        assert!(res.passed, "{:?}", res.detail);
    }

    #[test]
    fn element_above_answers_a_descendant_over_its_own_ancestor() {
        // `analyzers::layout` exempts ancestor/descendant pairs from its
        // unsolicited occlusion sweep. A DECLARED assertion does not inherit
        // that exemption — the author named both ids, and this pair shape is
        // the commonest real question there is.
        let snap = prompts_panel_stacking();
        let res = eval_on(&snap, &above(["title-bar-dropdown", "title-bar"], true));
        assert!(res.passed, "{:?}", res.detail);
    }

    #[test]
    fn element_above_prefers_occluded_by_over_z_index() {
        // z_index says "panel on top"; the producer hit-tested and says the
        // dropdown is. The attribution wins in both directions.
        let snap = snap_of(vec![
            Element {
                id: "dropdown".into(),
                bbox: Some(Region {
                    x: 0,
                    y: 0,
                    w: 100,
                    h: 100,
                }),
                z_index: Some(1),
                ..Default::default()
            },
            Element {
                id: "panel".into(),
                bbox: Some(Region {
                    x: 50,
                    y: 50,
                    w: 100,
                    h: 100,
                }),
                z_index: Some(99),
                occluded_by: Some("dropdown".into()),
                ..Default::default()
            },
        ]);

        let res = eval_on(&snap, &above(["dropdown", "panel"], true));
        assert!(res.passed, "{:?}", res.detail);
        assert!(res.detail.unwrap().contains("occluded_by"));

        let res = eval_on(&snap, &above(["panel", "dropdown"], true));
        assert!(!res.passed, "{:?}", res.detail);
        assert!(res.detail.unwrap().contains("OTHER WAY"));
    }

    #[test]
    fn element_above_without_evidence_cannot_answer_and_does_not_pass() {
        let snap = snap_of(vec![
            el("a", 0, 0, 100, 100),
            Element {
                id: "b".into(),
                bbox: Some(Region {
                    x: 50,
                    y: 50,
                    w: 100,
                    h: 100,
                }),
                z_index: Some(3),
                ..Default::default()
            },
        ]);
        let res = eval_on(&snap, &above(["a", "b"], true));
        assert!(
            !res.passed,
            "a vacuous pass here hides the bug being hunted"
        );
        let detail = res.detail.unwrap();
        assert!(detail.contains("CANNOT ANSWER"), "{detail}");
        assert!(detail.contains("'a'"), "must name the element: {detail}");
        assert!(!detail.contains("'b'"), "must not blame 'b': {detail}");
        assert!(detail.contains("raw CSS"), "{detail}");
    }

    #[test]
    fn element_above_equal_z_index_means_neither_is_above() {
        let snap = snap_of(vec![
            Element {
                id: "a".into(),
                bbox: Some(Region {
                    x: 0,
                    y: 0,
                    w: 100,
                    h: 100,
                }),
                z_index: Some(7),
                ..Default::default()
            },
            Element {
                id: "b".into(),
                bbox: Some(Region {
                    x: 50,
                    y: 50,
                    w: 100,
                    h: 100,
                }),
                z_index: Some(7),
                ..Default::default()
            },
        ]);
        let res = eval_on(&snap, &above(["a", "b"], true));
        assert!(!res.passed);
        assert!(res
            .detail
            .unwrap()
            .contains("EQUAL resolved stacking order"));
    }

    #[test]
    fn element_above_disjoint_fails_on_overlap_before_evidence() {
        // Neither element carries z_index. The author still must hear about
        // the disjoint pair they named, not about the missing evidence —
        // producers do not populate z_index yet, so an evidence-first order
        // would mask every authoring mistake behind the same message.
        let snap = snap_of(vec![el("a", 0, 0, 50, 50), el("b", 500, 500, 50, 50)]);
        let res = eval_on(&snap, &above(["a", "b"], true));
        assert!(!res.passed);
        // Failed, not Unknown: the geometry WAS measured and the declared
        // overlap requirement was violated. Contrast with the missing-bbox
        // arm above, where there was nothing to measure.
        assert_eq!(res.outcome, AssertionOutcome::Failed);
        let detail = res.detail.unwrap();
        assert!(detail.contains("do not overlap"), "{detail}");
        assert!(detail.contains("require_overlap"), "{detail}");
        assert!(!detail.contains("CANNOT ANSWER"), "{detail}");
    }

    #[test]
    fn element_above_disjoint_is_answered_when_overlap_not_required() {
        let snap = snap_of(vec![
            Element {
                id: "a".into(),
                bbox: Some(Region {
                    x: 0,
                    y: 0,
                    w: 50,
                    h: 50,
                }),
                z_index: Some(5),
                ..Default::default()
            },
            Element {
                id: "b".into(),
                bbox: Some(Region {
                    x: 500,
                    y: 500,
                    w: 50,
                    h: 50,
                }),
                z_index: Some(1),
                ..Default::default()
            },
        ]);
        assert!(eval_on(&snap, &above(["a", "b"], false)).passed);
        assert!(!eval_on(&snap, &above(["a", "b"], true)).passed);
    }

    #[test]
    fn element_above_wire_form_defaults_require_overlap_to_true() {
        let a: Assertion =
            serde_json::from_str(r#"{"type":"element_above","elements":["x","y"]}"#).unwrap();
        match &a {
            Assertion::ElementAbove {
                elements,
                require_overlap,
            } => {
                assert_eq!(elements, &["x".to_string(), "y".to_string()]);
                assert!(*require_overlap, "absent require_overlap must default true");
            }
            other => panic!("wrong variant: {other:?}"),
        }
        // And the tag round-trips as snake_case.
        let json = serde_json::to_string(&a).unwrap();
        assert!(json.contains(r#""type":"element_above""#), "{json}");
    }

    #[test]
    fn element_above_four_outcomes_are_pairwise_distinct() {
        // 1. cannot answer
        let cannot = snap_of(vec![el("a", 0, 0, 100, 100), el("b", 50, 50, 100, 100)]);
        // 2. equal z_index
        let z = |id: &str, x: i32, zi: i32| Element {
            id: id.into(),
            bbox: Some(Region {
                x,
                y: 0,
                w: 100,
                h: 100,
            }),
            z_index: Some(zi),
            ..Default::default()
        };
        let equal = snap_of(vec![z("a", 0, 4), z("b", 50, 4)]);
        // 3. no overlap while require_overlap
        let disjoint = snap_of(vec![z("a", 0, 9), z("b", 900, 1)]);
        // 4. the real ordering failure
        let inverted = snap_of(vec![z("a", 0, 1), z("b", 50, 9)]);

        let details: Vec<String> = [&cannot, &equal, &disjoint, &inverted]
            .iter()
            .map(|s| {
                let res = eval_on(s, &above(["a", "b"], true));
                assert!(!res.passed);
                res.detail.expect("every failure carries a detail")
            })
            .collect();

        for i in 0..details.len() {
            for j in (i + 1)..details.len() {
                assert_ne!(
                    details[i], details[j],
                    "outcomes {i} and {j} are indistinguishable"
                );
            }
        }
    }

    // -----------------------------------------------------------------
    // Tri-state outcome
    //
    // The bit and the outcome are two views of one verdict, so the tests
    // that matter are the ones pinning that they never disagree and that
    // the two RED kinds stay apart. `element_above` motivated the third
    // state, but the property is crate-wide.
    // -----------------------------------------------------------------

    /// Every result this crate can produce, over a snapshot deliberately
    /// missing most of what the evaluators want. The invariant is not about
    /// any one verdict: it is that `passed` is exactly `outcome == Passed`,
    /// never an independently-set bool that can drift from it.
    #[test]
    fn passed_bit_is_exactly_the_passed_outcome() {
        let snap = snap_of(vec![
            el("a", 0, 0, 100, 100),
            el("b", 50, 50, 100, 100),
            Element {
                id: "c".into(),
                bbox: Some(Region {
                    x: 0,
                    y: 0,
                    w: 10,
                    h: 10,
                }),
                text: Some("hello".into()),
                fg_color: Some(Rgb::new(0, 0, 0)),
                bg_color: Some(Rgb::new(255, 255, 255)),
                font_size_px: Some(4.0),
                z_index: Some(1),
                ..Default::default()
            },
        ]);
        let all = vec![
            Assertion::NoOverlap {
                elements: ["a".into(), "b".into()],
                tolerance_px: None,
            },
            Assertion::NoOverlap {
                elements: ["a".into(), "ghost".into()],
                tolerance_px: None,
            },
            above(["a", "b"], true),
            above(["c", "a"], false),
            Assertion::ContainsText {
                target: TextTarget::Element(ElementTextTarget {
                    element: "c".into(),
                }),
                text: "hello".into(),
                kind: TextMatchKind::Contains,
            },
            Assertion::ContainsText {
                target: TextTarget::Element(ElementTextTarget {
                    element: "a".into(),
                }),
                text: "hello".into(),
                kind: TextMatchKind::Contains,
            },
            Assertion::TextFitsContainer {
                element: "c".into(),
            },
            Assertion::TextFitsContainer {
                element: "ghost".into(),
            },
            Assertion::AlignedHorizontally {
                elements: vec!["a".into(), "ghost".into()],
                axis_tolerance_px: None,
            },
            Assertion::AlignedVertically {
                elements: vec!["a".into(), "b".into()],
                axis_tolerance_px: Some(1000),
            },
            Assertion::ColorWithin {
                element: "a".into(),
                expected: Rgb::new(0, 0, 0),
                delta_e_max: None,
            },
            Assertion::TypographyConsistent {
                elements: vec!["c".into(), "ghost".into()],
                dimensions: vec![TypographyDimension::FontSize],
            },
            Assertion::NoLayoutShiftSince {
                baseline: "never-registered".into(),
                tolerance_px: None,
            },
            Assertion::NoClipping { region: None },
            Assertion::AnimationSettled { region: None },
            Assertion::SettledWithin { ms: 500 },
            Assertion::StateEnteredWithin {
                state: "saved-toast".into(),
                ms: 2000,
            },
            Assertion::ContrastMeetsWcag {
                element: "a".into(),
                level: WcagLevel::Aa,
            },
        ];

        let mut seen_pass = false;
        let mut seen_fail = false;
        let mut seen_unknown = false;
        for a in &all {
            let res = eval_on(&snap, a);
            assert_eq!(
                res.passed,
                res.outcome == AssertionOutcome::Passed,
                "bit and outcome disagree for {a:?}: {res:?}"
            );
            match res.outcome {
                AssertionOutcome::Passed => {
                    seen_pass = true;
                    assert_eq!(res.code, None, "a pass carries no unknown code: {a:?}");
                }
                AssertionOutcome::Failed => {
                    seen_fail = true;
                    assert!(res.detail.is_some(), "a failure must say why: {a:?}");
                    assert_eq!(res.code, None, "a failure carries no unknown code: {a:?}");
                }
                AssertionOutcome::Unknown => {
                    seen_unknown = true;
                    assert!(
                        res.detail.is_some(),
                        "an unknown whose detail does not name the missing input is \
                         indistinguishable from a failure: {a:?}"
                    );
                    assert!(
                        res.code.is_some(),
                        "an unknown must carry a typed code: {a:?}"
                    );
                }
            }
        }
        // A corpus that exercises only one arm would satisfy the invariant
        // vacuously.
        assert!(seen_pass && seen_fail && seen_unknown, "corpus is too thin");
    }

    /// The distinction the third state exists for: `element_above` with no
    /// evidence is NOT the same result as `element_above` with evidence that
    /// refutes it. Both are red; only one of them says the page is wrong.
    #[test]
    fn cannot_answer_is_unknown_while_a_real_inversion_is_failed() {
        let z = |id: &str, zi: Option<i32>| Element {
            id: id.into(),
            bbox: Some(Region {
                x: 0,
                y: 0,
                w: 100,
                h: 100,
            }),
            z_index: zi,
            ..Default::default()
        };

        // No z_index anywhere, no occluded_by: nothing was measured.
        let blind = snap_of(vec![z("a", None), z("b", None)]);
        let res = eval_on(&blind, &above(["a", "b"], true));
        assert_eq!(res.outcome, AssertionOutcome::Unknown, "{:?}", res.detail);
        assert!(!res.passed, "an un-evaluated assertion must not be green");
        assert!(res.detail.as_deref().unwrap().contains("CANNOT ANSWER"));

        // Measured, and the ordering is the other way round.
        let inverted = snap_of(vec![z("a", Some(1)), z("b", Some(9))]);
        let res = eval_on(&inverted, &above(["a", "b"], true));
        assert_eq!(res.outcome, AssertionOutcome::Failed, "{:?}", res.detail);
        assert!(res.detail.as_deref().unwrap().contains("paints BELOW"));

        // And the producer's own attribution refuting it is likewise a real
        // failure, not an absence of evidence.
        let mut attributed = snap_of(vec![z("a", None), z("b", None)]);
        attributed.elements[0].occluded_by = Some("b".into());
        let res = eval_on(&attributed, &above(["a", "b"], true));
        assert_eq!(res.outcome, AssertionOutcome::Failed, "{:?}", res.detail);
    }

    /// The CANNOT ANSWER detail is the only place a producer is warned off
    /// projecting a raw CSS `z-index` into `z_index`. Losing that sentence
    /// would cost nothing visible and invert verdicts silently, so it is
    /// pinned.
    #[test]
    fn cannot_answer_names_the_global_paint_rank_contract() {
        let snap = snap_of(vec![el("a", 0, 0, 100, 100), el("b", 50, 50, 100, 100)]);
        let detail = eval_on(&snap, &above(["a", "b"], true))
            .detail
            .expect("cannot-answer carries a detail");
        assert!(detail.contains("raw CSS"), "{detail}");
        assert!(detail.contains("GLOBAL"), "{detail}");
    }

    // -- Timeline assertions -------------------------------------------------

    fn prov(unmeasured: serde_json::Value) -> serde_json::Value {
        serde_json::json!({
            "producer": { "id": "sdk/timeline/events", "version": "test" },
            "observedAt": "2026-10-20T00:00:00Z",
            "evaluatedAt": "2026-10-20T00:00:00Z",
            "coverage": { "considered": 1, "measured": 1, "unmeasured": unmeasured },
            "confidence": null,
            "cache": null,
            "source": null
        })
    }
    fn measured(v: serde_json::Value) -> serde_json::Value {
        serde_json::json!({ "status": "measured", "value": v, "provenance": prov(serde_json::json!([])) })
    }
    fn degraded(v: serde_json::Value, dimension: &str) -> serde_json::Value {
        serde_json::json!({
            "status": "measured",
            "value": v,
            "provenance": prov(serde_json::json!([
                { "dimension": dimension, "count": 0, "code": "input_missing" }
            ]))
        })
    }
    fn unknown(code: &str) -> serde_json::Value {
        serde_json::json!({
            "status": "unknown",
            "unknown": { "code": code, "detail": "test" },
            "provenance": prov(serde_json::json!([]))
        })
    }
    fn anim(ty: &str, at: u64, name: &str) -> serde_json::Value {
        let mut v = serde_json::json!({
            "type": ty, "offsetMs": at, "summary": "s",
            "animation": { "name": name, "targetElement": format!("fp-{name}"), "isInfinite": false }
        });
        if ty == "animation-ended" {
            v["endReason"] = serde_json::json!("finished");
        }
        v
    }
    fn state_ev(ty: &str, at: u64, id: &str) -> serde_json::Value {
        serde_json::json!({ "type": ty, "offsetMs": at, "summary": "s", "stateId": id })
    }

    /// A timeline with every summary field measured over full coverage, the
    /// given extra events after the action, and `edit` applied to its JSON.
    fn timeline(
        events: Vec<serde_json::Value>,
        edit: impl FnOnce(&mut serde_json::Value),
    ) -> ChangeTimeline {
        let mut all = vec![serde_json::json!({ "type": "action", "offsetMs": 0, "summary": "a" })];
        all.extend(events);
        let mut v = serde_json::json!({
            "events": all,
            "recordedMs": 1000,
            "settled": true,
            "observedSources": [
                "snapshot-poll", "layout-shift", "long-task", "animation-events",
                "get-animations", "states"
            ],
            "summary": {
                "firstResponseMs": measured(serde_json::json!(10)),
                "settleMs": measured(serde_json::json!(300)),
                "layoutShiftTotal": measured(serde_json::json!(0)),
                "layoutShiftCount": measured(serde_json::json!(0)),
                "layoutShiftAfterFirstResponseMs": measured(serde_json::json!(0)),
                "animationsUnsettledAtEnd": measured(serde_json::json!([])),
                "longTaskTotalMs": measured(serde_json::json!(0)),
                "expectedStates": []
            }
        });
        edit(&mut v);
        serde_json::from_value(v).expect("test timeline parses")
    }

    fn eval_timeline(t: &ChangeTimeline, a: &Assertion) -> AssertionResult {
        let res = evaluate(
            a,
            &EvalContext {
                timeline: Some(t),
                ..Default::default()
            },
        );
        assert_eq!(res.passed, res.outcome.passed());
        assert_eq!(res.code.is_some(), res.outcome.is_unknown(), "{res:?}");
        res
    }

    fn settled_anim() -> Assertion {
        Assertion::AnimationSettled { region: None }
    }

    /// With no timeline every timeline assertion is unknown and names the
    /// route that supplies one — the old `animation_settled` "needs frames"
    /// unknown pointed at a multi-frame path that does not exist.
    #[test]
    fn timeline_assertions_without_a_timeline_are_unknown_and_name_the_route() {
        let snap = snap_of(vec![el("a", 0, 0, 10, 10)]);
        for a in [
            settled_anim(),
            Assertion::SettledWithin { ms: 500 },
            Assertion::StateEnteredWithin {
                state: "saved-toast".into(),
                ms: 2000,
            },
        ] {
            let res = eval_on(&snap, &a);
            assert_eq!(res.outcome, AssertionOutcome::Unknown, "{a:?}");
            assert!(!res.passed, "a gate that never ran must not report green");
            assert_eq!(res.code, Some(UnknownCode::InputMissing));
            let detail = res.detail.as_deref().unwrap();
            assert!(detail.contains("POST /ai/execute-with-diff"), "{detail}");
            assert!(detail.contains("timeline_ref"), "{detail}");
        }
    }

    #[test]
    fn timeline_assertions_parse_from_the_wire() {
        let a: Assertion = serde_json::from_value(serde_json::json!({
            "type": "settled_within", "ms": 500
        }))
        .unwrap();
        assert!(matches!(a, Assertion::SettledWithin { ms: 500 }));
        let a: Assertion = serde_json::from_value(serde_json::json!({
            "type": "state_entered_within", "state": "saved-toast", "ms": 2000
        }))
        .unwrap();
        assert!(matches!(a, Assertion::StateEnteredWithin { ms: 2000, .. }));
        // A suite written for the retired frame-count dial still parses.
        let a: Assertion = serde_json::from_value(serde_json::json!({
            "type": "animation_settled", "settle_frames": 3
        }))
        .unwrap();
        assert!(matches!(a, Assertion::AnimationSettled { region: None }));
    }

    #[test]
    fn animation_settled_fails_when_the_last_animation_event_is_a_start() {
        let t = timeline(vec![anim("animation-started", 20, "pulse")], |_| {});
        let res = eval_timeline(&t, &settled_anim());
        assert_eq!(res.outcome, AssertionOutcome::Failed, "{res:?}");
        assert!(res.detail.unwrap().contains("pulse"));
    }

    /// Review probe t1: the recorder dropped `animation-ended` events at its
    /// cap, so a start with no recorded end is not evidence of a running
    /// animation.
    #[test]
    fn animation_settled_is_unknown_when_animation_ended_events_were_dropped() {
        let t = timeline(vec![anim("animation-started", 10, "fade")], |v| {
            v["summary"]["animationsUnsettledAtEnd"] = degraded(
                serde_json::json!([{ "name": "fade", "isInfinite": false }]),
                "timeline-events:animation-ended",
            );
        });
        let res = eval_timeline(&t, &settled_anim());
        assert_eq!(res.outcome, AssertionOutcome::Unknown, "{res:?}");
        assert_eq!(res.code, Some(UnknownCode::InputMissing));
        assert!(res.detail.unwrap().contains("animation-ended"));
        // A gap elsewhere (no getAnimations) cannot make a recorded
        // start-without-end wrong: still a fail.
        let t = timeline(vec![anim("animation-started", 10, "fade")], |v| {
            v["summary"]["animationsUnsettledAtEnd"] = degraded(
                serde_json::json!([{ "name": "fade", "isInfinite": false }]),
                "animations-running-before-action(count-unknowable)",
            );
        });
        assert_eq!(
            eval_timeline(&t, &settled_anim()).outcome,
            AssertionOutcome::Failed
        );
    }

    /// DEFENSIVE: the UI Bridge SDK caps events per type but emits a single
    /// `settled`, so it never reports `timeline-events:settled` as dropped.
    /// The rule this pins guards a non-SDK producer (or a future SDK cap)
    /// that could, and is harmless otherwise.
    #[test]
    fn settled_within_never_fails_over_dropped_settled_events() {
        let t = timeline(vec![], |v| {
            v["summary"]["settleMs"] = serde_json::json!({
                "status": "unknown",
                "unknown": { "code": "no_event_before_timeout", "detail": "t" },
                "provenance": prov(serde_json::json!([
                    { "dimension": "timeline-events:settled", "count": 1, "code": "input_missing" }
                ]))
            });
            v["settled"] = serde_json::json!(false);
            v["recordedMs"] = serde_json::json!(8000);
        });
        let res = eval_timeline(&t, &Assertion::SettledWithin { ms: 500 });
        assert_eq!(res.outcome, AssertionOutcome::Unknown, "{res:?}");
        assert_eq!(res.code, Some(UnknownCode::InputMissing));
    }

    #[test]
    fn state_entered_within_never_fails_on_a_degraded_expected_state() {
        for outcome in [
            serde_json::json!({ "notEnteredWithinMs": 2000 }),
            serde_json::json!({ "activeThroughoutWithinMs": 2000 }),
            serde_json::json!({ "enteredAtMs": 1500 }),
        ] {
            let t = with_expected(degraded(outcome.clone(), "timeline-events:state-entered"));
            let res = eval_timeline(&t, &entered_within(1000));
            assert_eq!(res.outcome, AssertionOutcome::Unknown, "{outcome}: {res:?}");
            // Not degraded: the same answer is a fail.
            let t = with_expected(measured(outcome.clone()));
            assert_eq!(
                eval_timeline(&t, &entered_within(1000)).outcome,
                AssertionOutcome::Failed,
                "{outcome}"
            );
        }
        // A degraded answer inside the bound still passes.
        let t = with_expected(degraded(serde_json::json!({ "enteredAtMs": 85 }), "x"));
        assert_eq!(
            eval_timeline(&t, &entered_within(1000)).outcome,
            AssertionOutcome::Passed
        );
    }

    #[test]
    fn animation_settled_passes_when_every_start_ended() {
        let t = timeline(
            vec![
                anim("animation-started", 20, "slide-in"),
                anim("animation-ended", 220, "slide-in"),
            ],
            |_| {},
        );
        let res = eval_timeline(&t, &settled_anim());
        assert_eq!(res.outcome, AssertionOutcome::Passed, "{res:?}");
        // And with no animation at all.
        let res = eval_timeline(&timeline(vec![], |_| {}), &settled_anim());
        assert_eq!(res.outcome, AssertionOutcome::Passed, "{res:?}");
    }

    #[test]
    fn animation_settled_fails_on_a_running_animation_only_get_animations_saw() {
        let t = timeline(vec![], |v| {
            v["summary"]["animationsUnsettledAtEnd"] = measured(serde_json::json!([
                { "name": "spinner", "isInfinite": true }
            ]));
        });
        let res = eval_timeline(&t, &settled_anim());
        assert_eq!(res.outcome, AssertionOutcome::Failed, "{res:?}");
        assert!(res.detail.unwrap().contains("spinner"));
    }

    #[test]
    fn animation_settled_is_unknown_when_the_summary_could_not_look() {
        let t = timeline(vec![], |v| {
            v["summary"]["animationsUnsettledAtEnd"] = unknown("input_missing");
        });
        let res = eval_timeline(&t, &settled_anim());
        assert_eq!(res.outcome, AssertionOutcome::Unknown, "{res:?}");
        assert_eq!(res.code, Some(UnknownCode::InputMissing));

        // Degraded coverage (no getAnimations): an animation running since
        // before the action cannot be ruled out, so "none seen" is no pass.
        let t = timeline(
            vec![
                anim("animation-started", 20, "slide-in"),
                anim("animation-ended", 220, "slide-in"),
            ],
            |v| {
                v["summary"]["animationsUnsettledAtEnd"] = degraded(
                    serde_json::json!([]),
                    "animations-running-before-action(count-unknowable)",
                );
            },
        );
        let res = eval_timeline(&t, &settled_anim());
        assert_eq!(res.outcome, AssertionOutcome::Unknown, "{res:?}");
        assert!(res
            .detail
            .unwrap()
            .contains("animations-running-before-action"));
    }

    #[test]
    fn animation_settled_with_a_region_cannot_place_a_running_animation() {
        let region = Some(Region {
            x: 0,
            y: 0,
            w: 10,
            h: 10,
        });
        let running = timeline(vec![anim("animation-started", 20, "pulse")], |_| {});
        let res = eval_timeline(&running, &Assertion::AnimationSettled { region });
        assert_eq!(res.outcome, AssertionOutcome::Unknown, "{res:?}");
        let quiet = timeline(vec![], |_| {});
        let res = eval_timeline(&quiet, &Assertion::AnimationSettled { region });
        assert_eq!(res.outcome, AssertionOutcome::Passed, "{res:?}");
    }

    #[test]
    fn settled_within_reads_the_settle_observation() {
        let t = timeline(vec![], |_| {});
        let pass = eval_timeline(&t, &Assertion::SettledWithin { ms: 300 });
        assert_eq!(pass.outcome, AssertionOutcome::Passed, "{pass:?}");
        let fail = eval_timeline(&t, &Assertion::SettledWithin { ms: 299 });
        assert_eq!(fail.outcome, AssertionOutcome::Failed, "{fail:?}");
    }

    #[test]
    fn settled_within_a_never_settled_recording_fails_only_over_its_window() {
        let never = |recorded: u64| {
            timeline(vec![], move |v| {
                v["summary"]["settleMs"] = unknown("no_event_before_timeout");
                v["settled"] = serde_json::json!(false);
                v["recordedMs"] = serde_json::json!(recorded);
            })
        };
        // The recording covered the bound and nothing settled: a measured negative.
        let res = eval_timeline(&never(8000), &Assertion::SettledWithin { ms: 500 });
        assert_eq!(res.outcome, AssertionOutcome::Failed, "{res:?}");
        // It stopped short of the bound: unknown.
        let res = eval_timeline(&never(120), &Assertion::SettledWithin { ms: 500 });
        assert_eq!(res.outcome, AssertionOutcome::Unknown, "{res:?}");
        assert_eq!(res.code, Some(UnknownCode::NoEventBeforeTimeout));
        // An unknown with another cause is never turned into a verdict.
        let other = timeline(vec![], |v| {
            v["summary"]["settleMs"] = unknown("producer_failed");
            v["settled"] = serde_json::json!(false);
            v["recordedMs"] = serde_json::json!(8000);
        });
        let res = eval_timeline(&other, &Assertion::SettledWithin { ms: 500 });
        assert_eq!(res.outcome, AssertionOutcome::Unknown, "{res:?}");
        assert_eq!(res.code, Some(UnknownCode::ProducerFailed));
    }

    fn with_expected(outcome: serde_json::Value) -> ChangeTimeline {
        timeline(vec![], move |v| {
            v["summary"]["expectedStates"] =
                serde_json::json!([{ "stateId": "saved-toast", "outcome": outcome }]);
        })
    }

    fn entered_within(ms: u64) -> Assertion {
        Assertion::StateEnteredWithin {
            state: "saved-toast".into(),
            ms,
        }
    }

    #[test]
    fn state_entered_within_reads_the_expected_state_observation() {
        let t = with_expected(measured(serde_json::json!({ "enteredAtMs": 85 })));
        assert_eq!(
            eval_timeline(&t, &entered_within(400)).outcome,
            AssertionOutcome::Passed
        );
        assert_eq!(
            eval_timeline(&t, &entered_within(50)).outcome,
            AssertionOutcome::Failed
        );

        let never = with_expected(measured(serde_json::json!({ "notEnteredWithinMs": 2000 })));
        let res = eval_timeline(&never, &entered_within(2000));
        assert_eq!(res.outcome, AssertionOutcome::Failed, "{res:?}");
        assert!(res.detail.unwrap().contains("2000ms window"));

        let already = with_expected(measured(
            serde_json::json!({ "activeThroughoutWithinMs": 400 }),
        ));
        let res = eval_timeline(&already, &entered_within(400));
        assert_eq!(res.outcome, AssertionOutcome::Failed, "{res:?}");
        assert!(res.detail.unwrap().contains("already active"));

        let undeclared = with_expected(unknown("input_missing"));
        let res = eval_timeline(&undeclared, &entered_within(400));
        assert_eq!(res.outcome, AssertionOutcome::Unknown, "{res:?}");
        assert_eq!(res.code, Some(UnknownCode::InputMissing));
    }

    #[test]
    fn state_entered_within_falls_back_to_the_state_events() {
        let t = timeline(vec![state_ev("state-entered", 120, "saved-toast")], |_| {});
        assert_eq!(
            eval_timeline(&t, &entered_within(200)).outcome,
            AssertionOutcome::Passed
        );
        // Entered, but late: the id is known, the window covered, a fail.
        let res = eval_timeline(&t, &entered_within(100));
        assert_eq!(res.outcome, AssertionOutcome::Failed, "{res:?}");

        // A window the recording did not cover is unknown.
        let res = eval_timeline(&t, &entered_within(5000));
        assert_eq!(
            res.outcome,
            AssertionOutcome::Passed,
            "an entry inside any bound passes"
        );
        let short = timeline(vec![state_ev("state-exited", 50, "saved-toast")], |_| {});
        let res = eval_timeline(&short, &entered_within(5000));
        assert_eq!(res.outcome, AssertionOutcome::Unknown, "{res:?}");
        assert_eq!(res.code, Some(UnknownCode::NoEventBeforeTimeout));

        // No event names the id: a typo and a never-entered state look alike.
        let silent = timeline(vec![], |_| {});
        let res = eval_timeline(&silent, &entered_within(500));
        assert_eq!(res.outcome, AssertionOutcome::Unknown, "{res:?}");
        assert!(res.detail.unwrap().contains("expectStates"));

        // No state registry observed at all.
        let blind = timeline(vec![], |v| {
            v["observedSources"] = serde_json::json!(["snapshot-poll"]);
        });
        let res = eval_timeline(&blind, &entered_within(500));
        assert_eq!(res.outcome, AssertionOutcome::Unknown, "{res:?}");
        assert!(res.detail.unwrap().contains("`states` source"));
    }

    /// An operand the producer never registered is a gap in the snapshot,
    /// not a defect in the page. Reported as a failure it sends a reader
    /// hunting a layout bug that no evidence points at.
    #[test]
    fn a_missing_operand_is_unknown_not_failed() {
        let snap = snap_of(vec![el("a", 0, 0, 100, 100)]);
        let cases: Vec<Assertion> = vec![
            Assertion::NoOverlap {
                elements: ["a".into(), "unregistered".into()],
                tolerance_px: None,
            },
            above(["a", "unregistered"], false),
            Assertion::TextFitsContainer {
                element: "unregistered".into(),
            },
            Assertion::AlignedHorizontally {
                elements: vec!["a".into(), "unregistered".into()],
                axis_tolerance_px: None,
            },
            Assertion::ColorWithin {
                element: "unregistered".into(),
                expected: Rgb::new(0, 0, 0),
                delta_e_max: None,
            },
            Assertion::ContrastMeetsWcag {
                element: "unregistered".into(),
                level: WcagLevel::Aa,
            },
        ];
        for a in &cases {
            let res = eval_on(&snap, a);
            assert_eq!(
                res.outcome,
                AssertionOutcome::Unknown,
                "{a:?} -> {:?}",
                res.detail
            );
            assert!(
                res.detail
                    .as_deref()
                    .unwrap()
                    .contains("not in the snapshot"),
                "the detail must name the gap: {:?}",
                res.detail
            );
            assert_eq!(res.code, Some(UnknownCode::InputMissing), "{a:?}");
        }
    }

    /// A caller that supplies no snapshot at all learns that nothing ran,
    /// rather than that everything failed.
    #[test]
    fn no_snapshot_makes_every_assertion_unknown() {
        let res = evaluate(
            &Assertion::NoClipping { region: None },
            &EvalContext::default(),
        );
        assert_eq!(res.outcome, AssertionOutcome::Unknown);
        assert!(!res.passed);
        assert_eq!(res.code, Some(UnknownCode::InputMissing));
    }

    /// `code` is present iff the outcome is `unknown`, on the wire as well
    /// as in memory: a pass or failure never serializes the key.
    #[test]
    fn the_code_key_is_on_the_wire_iff_unknown() {
        let snap = snap_of(vec![el("a", 0, 0, 100, 100), el("b", 50, 50, 100, 100)]);
        let unknown = serde_json::to_value(eval_on(&snap, &above(["a", "b"], true))).unwrap();
        assert_eq!(unknown["code"], serde_json::json!("input_missing"));
        let failed = serde_json::to_value(eval_on(
            &snap,
            &Assertion::NoOverlap {
                elements: ["a".into(), "b".into()],
                tolerance_px: None,
            },
        ))
        .unwrap();
        assert_eq!(failed["outcome"], serde_json::json!("failed"));
        assert!(
            !failed.as_object().unwrap().contains_key("code"),
            "{failed}"
        );
    }

    /// An `unknown` that does not say why is refused at the deserialization
    /// boundary, and so is a code on an outcome that reached a verdict.
    #[test]
    fn an_unknown_without_a_code_or_a_verdict_with_one_fails_to_deserialize() {
        let no_code = serde_json::json!({
            "passed": false,
            "outcome": "unknown",
            "detail": "no snapshot",
            "assertion": { "type": "no_clipping" }
        });
        assert!(serde_json::from_value::<AssertionResult>(no_code).is_err());

        let code_on_failed = serde_json::json!({
            "passed": false,
            "outcome": "failed",
            "code": "input_missing",
            "assertion": { "type": "no_clipping" }
        });
        assert!(serde_json::from_value::<AssertionResult>(code_on_failed).is_err());

        let ok = serde_json::json!({
            "passed": false,
            "outcome": "unknown",
            "code": "stale_input",
            "assertion": { "type": "no_clipping" }
        });
        let res: AssertionResult = serde_json::from_value(ok).unwrap();
        assert_eq!(res.code, Some(UnknownCode::StaleInput));
    }

    /// Wire shape. `passed` stays where every existing consumer reads it,
    /// and `unknown` is false there — the gate does not change.
    #[test]
    fn unknown_serializes_as_not_passed() {
        let snap = snap_of(vec![el("a", 0, 0, 100, 100), el("b", 50, 50, 100, 100)]);
        let v = serde_json::to_value(eval_on(&snap, &above(["a", "b"], true))).unwrap();
        assert_eq!(v["passed"], serde_json::json!(false));
        assert_eq!(v["outcome"], serde_json::json!("unknown"));
    }

    /// A result written before `outcome` existed still parses. Its `false`
    /// carried no third state, so it reads back as `Failed` — which is
    /// exactly what that bit meant when it was written, not a guess.
    #[test]
    fn a_result_without_an_outcome_field_parses_from_the_bit() {
        let legacy = serde_json::json!({
            "passed": false,
            "detail": "a and b overlap by 4 px\u{00b2}",
            "assertion": { "type": "no_overlap", "elements": ["a", "b"] }
        });
        let res: AssertionResult = serde_json::from_value(legacy).unwrap();
        assert_eq!(res.outcome, AssertionOutcome::Failed);
        assert!(!res.passed);

        let legacy_pass = serde_json::json!({
            "passed": true,
            "assertion": { "type": "no_clipping" }
        });
        let res: AssertionResult = serde_json::from_value(legacy_pass).unwrap();
        assert_eq!(res.outcome, AssertionOutcome::Passed);
        assert!(res.passed);
    }

    /// A payload whose two fields disagree is repaired toward the richer
    /// one rather than carried forward inconsistent — otherwise the
    /// invariant `passed == outcome.passed()` would hold everywhere except
    /// across a deserialization boundary, which is where it matters most.
    #[test]
    fn a_disagreeing_payload_is_repaired_toward_the_outcome() {
        let contradictory = serde_json::json!({
            "passed": true,
            "outcome": "unknown",
            "code": "input_missing",
            "assertion": { "type": "no_clipping" }
        });
        let res: AssertionResult = serde_json::from_value(contradictory).unwrap();
        assert_eq!(res.outcome, AssertionOutcome::Unknown);
        assert!(!res.passed, "the bit must follow the outcome");
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

    // --- text_fits_container: the horizontal arm (added with
    // `scroll_width_px`). Before it, an ellipsised label passed cleanly,
    // which is how a truncated session name cleared this assertion.

    #[test]
    fn text_fits_fails_when_text_is_horizontally_truncated() {
        let mut e = el("label", 0, 0, 80, 20);
        e.text = Some("qontinui-web-frontend".into());
        e.font_size_px = Some(10.0);
        e.scroll_width_px = Some(160);
        e.text_overflow = Some("ellipsis".into());
        let snap = snap_of(vec![e]);
        let ctx = EvalContext {
            snapshot: Some(&snap),
            ..Default::default()
        };
        let res = evaluate(
            &Assertion::TextFitsContainer {
                element: "label".into(),
            },
            &ctx,
        );
        assert!(!res.passed, "160px of text in an 80px box must fail");
        let d = res.detail.unwrap_or_default();
        assert!(d.contains("ellipsised"), "detail should name HOW: {d}");
        assert!(d.contains("80px hidden"), "detail should quantify: {d}");
    }

    #[test]
    fn text_fits_passes_when_content_is_narrower_than_the_box() {
        let mut e = el("label", 0, 0, 200, 20);
        e.text = Some("short".into());
        e.font_size_px = Some(10.0);
        e.scroll_width_px = Some(40);
        let snap = snap_of(vec![e]);
        let ctx = EvalContext {
            snapshot: Some(&snap),
            ..Default::default()
        };
        let res = evaluate(
            &Assertion::TextFitsContainer {
                element: "label".into(),
            },
            &ctx,
        );
        assert!(res.passed);
    }

    #[test]
    fn text_fits_reports_horizontal_unknown_when_scroll_width_is_absent() {
        // The un-measured axis must not read as a clean pass — that silent
        // pass is the defect this arm exists to close.
        let mut e = el("label", 0, 0, 80, 20);
        e.text = Some("qontinui-web-frontend".into());
        e.font_size_px = Some(10.0);
        let snap = snap_of(vec![e]);
        let ctx = EvalContext {
            snapshot: Some(&snap),
            ..Default::default()
        };
        let res = evaluate(
            &Assertion::TextFitsContainer {
                element: "label".into(),
            },
            &ctx,
        );
        assert!(res.passed, "vertical fit is genuinely OK here");
        let d = res.detail.unwrap_or_default();
        assert!(
            d.contains("UNKNOWN"),
            "an unchecked axis must say so, not pass silently: {d}"
        );
    }

    #[test]
    fn text_fits_still_fails_on_the_vertical_axis() {
        let mut e = el("label", 0, 0, 200, 6);
        e.text = Some("tall text".into());
        e.font_size_px = Some(16.0);
        e.scroll_width_px = Some(40);
        let snap = snap_of(vec![e]);
        let ctx = EvalContext {
            snapshot: Some(&snap),
            ..Default::default()
        };
        let res = evaluate(
            &Assertion::TextFitsContainer {
                element: "label".into(),
            },
            &ctx,
        );
        assert!(!res.passed, "6px box for a 16px font must still fail");
    }
}
