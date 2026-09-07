//! Phase 6 vision analyzers — declarative geometry + appearance queries
//! over a captured frame + [`crate::ElementSnapshot`].
//!
//! Five analyzers ([`Analyzer::Layout`], [`Analyzer::Typography`],
//! [`Analyzer::Color`], [`Analyzer::Dynamic`], [`Analyzer::Elements`])
//! map roughly to the Python qontinui library's
//! `vision/environment/analyzers/*.py` modules — same surface, pure Rust,
//! no Python interpreter on the runtime path.
//!
//! Each analyzer's `run()` returns an [`AnalyzerResult`]: the findings it
//! produced, the [`SnapshotCoverage`] it had to work with, and an
//! [`AnalyzerVerdict`] saying whether its preconditions were met at all. A
//! [`Finding`] is structured: kind, severity, optional region, detail —
//! plus the provenance every observation carries, [`Finding::analyzer`]
//! (which analyzer produced it) and [`Finding::confidence`] (whether it
//! rests on an estimated input, or is an exact deduction).
//! Callers can choose to format them as text (the runner's
//! `/vision/analyze` endpoint serializes them as JSON, the `/visual-audit`
//! skill formats them as markdown).
//!
//! **An empty finding list is not self-describing, and the verdict is what
//! makes it so.** `findings: []` under [`AnalyzerVerdict::Checked`] means
//! the page is clean; the same empty list under
//! [`AnalyzerVerdict::Blocked`] means the input was too impoverished to
//! check anything. Those were byte-identical before the verdict existed.
//!
//! For more targeted "did exactly X hold?" checks, prefer [`crate::assertions`].

use serde::{Deserialize, Serialize};

use crate::coverage::SnapshotCoverage;
use crate::element_snapshot::ElementSnapshot;
use crate::frame::{Frame, Region};

pub mod color;
pub mod dynamic;
pub mod elements;
pub mod layout;
pub mod typography;

/// The five canonical analyzers. Wire-name (snake_case) matches the
/// `/vision/analyze` request's `analyzer` field.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Analyzer {
    Layout,
    Typography,
    Color,
    Dynamic,
    Elements,
}

impl Analyzer {
    pub fn name(self) -> &'static str {
        match self {
            Self::Layout => "layout",
            Self::Typography => "typography",
            Self::Color => "color",
            Self::Dynamic => "dynamic",
            Self::Elements => "elements",
        }
    }
}

/// Severity matches CSS-Lint / WCAG conventions: `Info` is observational,
/// `Warning` is "probably a bug, look at it", `Critical` is "definitely
/// broken UX".
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Severity {
    Info,
    Warning,
    Critical,
}

/// One observation produced by an analyzer.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Finding {
    /// Short machine-readable kind. e.g., `"overlap"`, `"misalignment"`,
    /// `"contrast"`. Stable across analyzer versions; downstream consumers
    /// can match on it.
    pub kind: String,
    pub severity: Severity,
    /// Which analyzer produced this observation.
    ///
    /// Stamped by [`run`] — the dispatcher — for every finding the run
    /// produced, rather than at each `Finding::new` site inside the five
    /// analyzers. Attribution that each analyzer had to remember is
    /// attribution a sixth analyzer will forget, and the moment a caller
    /// merges five analyzers' findings into one list (which is exactly what
    /// a five-analyzer audit does) an unattributed finding is unreadable.
    ///
    /// `None` means the finding was produced OUTSIDE the dispatcher — a
    /// direct `layout::run(..)` call, or a value built by hand or read from
    /// a payload written before this field existed. That is genuinely
    /// UNKNOWN attribution, not a claim that no analyzer produced it, which
    /// is why it is omitted from the wire rather than written as `null`.
    /// Contrast [`Self::confidence`], whose `None` is a positive statement
    /// and is therefore always on the wire.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub analyzer: Option<Analyzer>,
    /// Pixel-space region where the finding manifests, when one is
    /// meaningful (e.g., the overlapping intersection bbox).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub region: Option<Region>,
    /// One-line human-readable description.
    pub detail: String,
    /// IDs of elements involved (for analyses that compare 2+ elements).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub elements: Vec<String>,
    /// How far this observation rests on an ESTIMATED input, when it rests
    /// on one at all.
    ///
    /// **`None` is a positive statement, not a gap: no estimated input
    /// contributed to this finding — it is a deduction, not an estimate.**
    /// An `overlap` finding derived from two bounding boxes, a
    /// `frame_delta` counted pixel-for-pixel, a `low_contrast` computed from
    /// colours the snapshot DECLARED: each is exact, and a number attached
    /// to it would manufacture precision that was never measured. Saying so
    /// is the whole point of the field. A silently ambiguous `None` —
    /// "either exact or nobody said" — would recreate the collapsed
    /// distinction this crate's verdict types ([`AnalyzerVerdict`],
    /// [`crate::AssertionOutcome`]) exist to prevent.
    ///
    /// It is therefore **always serialized**, as `"confidence": null`, and
    /// never skipped: a reader must be able to tell "this producer states
    /// the finding is a deduction" from "this producer says nothing about
    /// confidence", and an omitted key cannot say the first. A payload with
    /// no key at all is the second, and comes only from a producer older
    /// than this field — every `Finding` this crate emits states one.
    ///
    /// `Some(c)` is in `0.0..=1.0` and its MEANING is the producing site's
    /// to document; there is no crate-wide scale. Today exactly one site
    /// sets it: [`color`]'s pixel-sampled contrast arm (see
    /// [`color::sample_dominant_two_scored`]).
    #[serde(default)]
    pub confidence: Option<f64>,
}

impl Finding {
    pub fn new(kind: impl Into<String>, severity: Severity, detail: impl Into<String>) -> Self {
        Self {
            kind: kind.into(),
            severity,
            analyzer: None,
            region: None,
            detail: detail.into(),
            elements: Vec::new(),
            confidence: None,
        }
    }

    pub fn with_region(mut self, r: Region) -> Self {
        self.region = Some(r);
        self
    }

    pub fn with_elements(mut self, ids: impl IntoIterator<Item = String>) -> Self {
        self.elements = ids.into_iter().collect();
        self
    }

    /// Record that this observation rests on an estimated input, at `c`.
    ///
    /// Call this ONLY where the input really was estimated — see
    /// [`Self::confidence`]. Leaving it uncalled is the statement that the
    /// finding is a deduction, and that statement reaches the wire.
    ///
    /// The `debug_assert` here is belt-and-braces rather than a guard, and
    /// that differs from its twin on [`crate::AssertionResult`]: the only
    /// caller is `color`'s sampled arm, whose value is a ratio of integer
    /// pixel counts over a divisor `clamp_to_frame` guarantees is `>= 1`, so
    /// it is finite and in range by construction. The assertion on the
    /// assertion side guards a value composed OUTSIDE this crate.
    pub fn with_confidence(mut self, c: f64) -> Self {
        debug_assert!(
            (0.0..=1.0).contains(&c),
            "confidence {c} is outside the documented 0.0..=1.0"
        );
        self.confidence = Some(c);
        self
    }
}

/// Which of the THREE things happened to an analyzer run.
///
/// `findings: Vec<Finding>` alone cannot say them: an empty list collapses
/// "ran over real inputs and found nothing wrong" together with "the input
/// carried nothing this analyzer could measure". Those are different facts
/// and call for different action — the first is a clean page, the second is
/// a gap in the snapshot (or in the producer that projected it), and only
/// the first is a pass.
///
/// The measured case: a snapshot whose elements carry `text` and
/// `interactable` but no `bbox` produced output byte-identical to a healthy
/// page's, because every geometric check filtered the element list down to
/// nothing and reported no problems with the emptiness it was left holding.
///
/// A gate bit is kept alongside as [`Self::conclusive`], and it is derived
/// from the verdict rather than from list-emptiness, so a caller that reads
/// only the bit gets `false` on [`Self::Blocked`] automatically.
///
/// # Why `Degraded` is green
///
/// [`Self::Degraded`] deliberately does NOT move the gate bit. It is not a
/// softer `Blocked`: it says a NAMED dimension was unmeasurable while the
/// analyzer's own preconditions were met, so the findings it did produce are
/// real findings about a real page. Making it non-green would also make it
/// useless — the commonest degradation is absent stacking order, and a
/// healthy projected snapshot routinely carries none at all (the projector
/// emits `z_index` only for a computed `zIndex` that parses as an integer,
/// and `auto` does not), so a `Degraded` gate would fire on essentially
/// every real snapshot and be muted within a week.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "state", rename_all = "snake_case")]
pub enum AnalyzerVerdict {
    /// The analyzer's preconditions were met and it ran over real inputs.
    /// An empty finding list here genuinely means "nothing wrong found".
    Checked,
    /// The analyzer ran and its findings stand, but a named dimension could
    /// not be measured, so some class of defect was not ruled out.
    ///
    /// Informational, and green by design — see the type doc.
    Degraded {
        /// What could not be measured, and what that costs. Written for a
        /// human reading a report, e.g. "2 intersecting pair(s) carry no
        /// usable stacking order, so occlusion is UNKNOWN".
        reason: String,
    },
    /// The analyzer's preconditions were NOT met. **No finding list from
    /// this run is a statement about the page** — including an empty one,
    /// which is exactly what a blocked analyzer usually produces.
    ///
    /// This is not a failure of the page and must not be read as one; it is
    /// a refusal to answer. Reading it as a pass hides every defect the
    /// analyzer exists to catch; reading it as a failure sends a reader
    /// looking for a bug that no evidence points at.
    Blocked {
        /// Which precondition failed, in terms of the coverage that was
        /// measured, e.g. "no element carries a bbox (0/7)".
        reason: String,
    },
}

impl AnalyzerVerdict {
    /// The gate bit. [`Self::Blocked`] is the only non-green verdict — see
    /// the type doc for why [`Self::Degraded`] deliberately is not.
    ///
    /// Named for what it asserts: the analyzer reached a conclusion, so its
    /// finding list — empty or not — describes the page.
    pub fn conclusive(&self) -> bool {
        !matches!(self, Self::Blocked { .. })
    }

    /// True when the analyzer refused to answer. Callers reporting coverage
    /// ("3 checked, 1 blocked") read this rather than inspecting the tag.
    pub fn is_blocked(&self) -> bool {
        matches!(self, Self::Blocked { .. })
    }

    /// True when the analyzer answered but left a named dimension
    /// unmeasured.
    pub fn is_degraded(&self) -> bool {
        matches!(self, Self::Degraded { .. })
    }

    /// The explanation carried by a non-`Checked` verdict, `None` for
    /// [`Self::Checked`] (which has nothing to explain).
    pub fn reason(&self) -> Option<&str> {
        match self {
            Self::Checked => None,
            Self::Degraded { reason } | Self::Blocked { reason } => Some(reason),
        }
    }
}

/// Result of running one analyzer: the findings, the evidence about the
/// input they were derived from, and the verdict that says how to read them.
///
/// `conclusive` and `verdict` are two views of one answer and are always
/// consistent: `conclusive == verdict.conclusive()`. `conclusive` is the
/// flat gate bit a consumer can fold without matching on an enum (the
/// pattern [`crate::AssertionResult`]'s `passed` established in this crate);
/// `verdict` is the finer answer and carries the reason.
///
/// # Findings are carried even when `Blocked`
///
/// A blocked analyzer's findings are RETAINED rather than discarded, for two
/// reasons. Diagnostically they are the useful part — the dispatcher's
/// `skipped` finding and `elements`' `empty_snapshot` are exactly the
/// messages a reader needs, and a verdict that swallowed them would be less
/// informative than what it replaced. And a partial block is real: `layout`
/// blocks when nothing is `interactable`, which empties the overlap and
/// occlusion passes while leaving its zero-area and alignment passes over
/// bbox-bearing elements perfectly meaningful.
///
/// What `Blocked` withdraws is not the findings but the CLAIM they add up to
/// a verdict on the page. Consumers must gate on `conclusive`, never on
/// `findings.is_empty()`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[serde(from = "AnalyzerResultWire")]
pub struct AnalyzerResult {
    /// The gate bit, derived from `verdict`. See [`AnalyzerVerdict::conclusive`].
    pub conclusive: bool,
    /// The three-way verdict. See [`AnalyzerVerdict`].
    pub verdict: AnalyzerVerdict,
    /// What the snapshot carried, as counted by [`SnapshotCoverage::of`].
    ///
    /// `None` when there was no snapshot to measure — `dynamic` takes two
    /// frames and no snapshot at all, and a dispatcher call missing the
    /// snapshot it needed has nothing to count. `None` is therefore "not
    /// applicable / not observed", never "all zeroes": an all-zero coverage
    /// is itself a measurement and is spelled as one.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub coverage: Option<SnapshotCoverage>,
    /// What the analyzer observed. Read this only through `verdict` — see
    /// the type doc.
    pub findings: Vec<Finding>,
}

impl AnalyzerResult {
    /// Assemble a result, deriving the gate bit from the verdict so the two
    /// views cannot disagree. Every constructor below routes through here.
    pub fn new(
        verdict: AnalyzerVerdict,
        coverage: Option<SnapshotCoverage>,
        findings: Vec<Finding>,
    ) -> Self {
        Self {
            conclusive: verdict.conclusive(),
            verdict,
            coverage,
            findings,
        }
    }

    /// Preconditions met; the findings describe the page.
    pub fn checked(coverage: Option<SnapshotCoverage>, findings: Vec<Finding>) -> Self {
        Self::new(AnalyzerVerdict::Checked, coverage, findings)
    }

    /// Ran, findings stand, one named dimension unmeasured. Green.
    pub fn degraded(
        reason: impl Into<String>,
        coverage: Option<SnapshotCoverage>,
        findings: Vec<Finding>,
    ) -> Self {
        Self::new(
            AnalyzerVerdict::Degraded {
                reason: reason.into(),
            },
            coverage,
            findings,
        )
    }

    /// Preconditions not met. `findings` are diagnostic only — see the type
    /// doc on why they are carried rather than dropped.
    pub fn blocked(
        reason: impl Into<String>,
        coverage: Option<SnapshotCoverage>,
        findings: Vec<Finding>,
    ) -> Self {
        Self::new(
            AnalyzerVerdict::Blocked {
                reason: reason.into(),
            },
            coverage,
            findings,
        )
    }
}

/// Deserialization shim enforcing `conclusive == verdict.conclusive()`.
///
/// The bit is DERIVED, so a payload claiming otherwise is claiming something
/// the type does not permit. Recomputing rather than trusting is what keeps
/// the invariant true of every value in the program, including one that came
/// off a wire someone else wrote — a hand-edited `"conclusive": true` beside
/// a `blocked` verdict is exactly the vacuous pass this whole type exists to
/// make unrepresentable.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct AnalyzerResultWire {
    verdict: AnalyzerVerdict,
    #[serde(default)]
    coverage: Option<SnapshotCoverage>,
    #[serde(default)]
    findings: Vec<Finding>,
}

impl From<AnalyzerResultWire> for AnalyzerResult {
    fn from(w: AnalyzerResultWire) -> Self {
        Self::new(w.verdict, w.coverage, w.findings)
    }
}

/// Input bundle. Not every analyzer needs both a frame and a snapshot —
/// `layout` works with snapshot-only, `dynamic` works with frame(s)
/// only — but the run() dispatcher accepts both so callers don't have to
/// pre-route.
#[derive(Debug, Clone)]
pub struct AnalyzeInput<'a> {
    pub frame: Option<&'a Frame>,
    pub snapshot: Option<&'a ElementSnapshot>,
    /// Optional second frame (for `dynamic` analyzer: diff vs this).
    pub prior_frame: Option<&'a Frame>,
}

/// Dispatcher. Calls the named analyzer with whatever it needs from
/// `input` and returns its [`AnalyzerResult`].
///
/// A missing required input is not a finding-shaped detail here — it is the
/// canonical [`AnalyzerVerdict::Blocked`] case, and it keeps its long-standing
/// `Severity::Warning` finding with `kind == "skipped"` so nothing a consumer
/// already displays disappears. One vocabulary: a `skipped` finding IS a
/// blocked verdict, rather than a second, parallel mechanism for saying the
/// same thing.
///
/// # Attribution happens here, once
///
/// Every finding this function returns leaves it carrying
/// [`Finding::analyzer`], stamped in one place at the end. That includes the
/// `skipped` findings of a [`AnalyzerVerdict::Blocked`] run: **a refusal to
/// answer is still an observation, and it needs its provenance more than a
/// clean result does** — it is the one a reader is most likely to be holding
/// out of context, merged into a five-analyzer report, wondering which
/// analyzer declined and why.
///
/// Stamping here rather than at each `Finding::new` site is deliberate: a
/// duty spread over five call sites is a duty a sixth analyzer will forget,
/// and the failure is silent.
pub fn run(analyzer: Analyzer, input: &AnalyzeInput<'_>) -> AnalyzerResult {
    /// The blocked-on-missing-input result, built once so all five arms
    /// spell it identically.
    fn missing_input(detail: &str, snapshot: Option<&ElementSnapshot>) -> AnalyzerResult {
        AnalyzerResult::blocked(
            detail.to_string(),
            snapshot.map(SnapshotCoverage::of),
            vec![Finding::new("skipped", Severity::Warning, detail)],
        )
    }

    let mut result = match analyzer {
        Analyzer::Layout => match input.snapshot {
            Some(s) => layout::run(s),
            None => missing_input("layout analyzer requires an ElementSnapshot", None),
        },
        Analyzer::Typography => match input.snapshot {
            Some(s) => typography::run(s),
            None => missing_input("typography analyzer requires an ElementSnapshot", None),
        },
        Analyzer::Color => match (input.frame, input.snapshot) {
            (Some(f), Some(s)) => color::run(f, s),
            (_, snapshot) => missing_input(
                "color analyzer requires both Frame and ElementSnapshot",
                snapshot,
            ),
        },
        Analyzer::Dynamic => match (input.prior_frame, input.frame) {
            (Some(prior), Some(cur)) => dynamic::run(prior, cur),
            _ => missing_input(
                "dynamic analyzer requires both a prior_frame and a current frame",
                None,
            ),
        },
        Analyzer::Elements => match input.snapshot {
            Some(s) => elements::run(s),
            None => missing_input("elements analyzer requires an ElementSnapshot", None),
        },
    };

    stamp_attribution(&mut result, analyzer);
    result
}

/// The single attribution site.
///
/// `get_or_insert` rather than assignment so that a finding which already
/// names its producer (a future analyzer that folds another's output) keeps
/// the finer attribution instead of being relabelled by the outer dispatch.
///
/// Extracted from [`run`] rather than inlined there so the test that pins
/// that property exercises THIS code and not a copy of it. A test carrying
/// its own `get_or_insert` would stay green through a "simplification" of
/// the real one to a plain assignment, which is the exact regression it
/// exists to catch.
fn stamp_attribution(result: &mut AnalyzerResult, analyzer: Analyzer) {
    for f in &mut result.findings {
        f.analyzer.get_or_insert(analyzer);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The dispatcher stamps attribution, but it must not RELABEL a finding
    /// that already names a finer producer — a future analyzer that folds
    /// another's output would otherwise have its inner attribution
    /// overwritten by the outer dispatch.
    ///
    /// Nothing in the crate does that today, which is exactly why this test
    /// exists: without it, `get_or_insert` and a plain assignment are
    /// observationally identical, and a later "simplification" to
    /// assignment would pass CI while deleting the property the field's doc
    /// comment promises.
    #[test]
    fn the_dispatcher_does_not_relabel_an_already_attributed_finding() {
        let mut result = AnalyzerResult::checked(
            None,
            vec![
                Finding {
                    analyzer: Some(Analyzer::Typography),
                    ..Finding::new("inner", Severity::Info, "folded from another analyzer")
                },
                Finding::new(
                    "outer",
                    Severity::Info,
                    "produced by the dispatched analyzer",
                ),
            ],
        );

        // The production stamp itself, not a copy of it — that is the whole
        // point of `stamp_attribution` being a named function.
        stamp_attribution(&mut result, Analyzer::Layout);

        assert_eq!(
            result.findings[0].analyzer,
            Some(Analyzer::Typography),
            "an existing attribution was overwritten by the outer dispatch"
        );
        assert_eq!(result.findings[1].analyzer, Some(Analyzer::Layout));
    }
}
