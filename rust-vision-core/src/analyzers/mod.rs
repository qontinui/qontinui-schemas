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
//! [`Finding`] is structured: kind, severity, optional region, detail.
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
    /// Pixel-space region where the finding manifests, when one is
    /// meaningful (e.g., the overlapping intersection bbox).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub region: Option<Region>,
    /// One-line human-readable description.
    pub detail: String,
    /// IDs of elements involved (for analyses that compare 2+ elements).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub elements: Vec<String>,
}

impl Finding {
    pub fn new(kind: impl Into<String>, severity: Severity, detail: impl Into<String>) -> Self {
        Self {
            kind: kind.into(),
            severity,
            region: None,
            detail: detail.into(),
            elements: Vec::new(),
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

    match analyzer {
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
    }
}
