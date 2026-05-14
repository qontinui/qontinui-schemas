//! Phase 6 vision analyzers — declarative geometry + appearance queries
//! over a captured frame + [`crate::ElementSnapshot`].
//!
//! Five analyzers ([`Analyzer::Layout`], [`Analyzer::Typography`],
//! [`Analyzer::Color`], [`Analyzer::Dynamic`], [`Analyzer::Elements`])
//! map roughly to the Python qontinui library's
//! `vision/environment/analyzers/*.py` modules — same surface, pure Rust,
//! no Python interpreter on the runtime path.
//!
//! Each analyzer's `run()` returns a [`Vec<Finding>`]. A [`Finding`] is
//! structured: kind, severity, optional region, detail. Callers can choose
//! to format them as text (the runner's `/vision/analyze` endpoint
//! serializes them as JSON, the `/visual-audit` skill formats them as
//! markdown).
//!
//! For more targeted "did exactly X hold?" checks, prefer [`crate::assertions`].

use serde::{Deserialize, Serialize};

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
/// `input`. Returns `Vec<Finding>`; an analyzer with nothing to report
/// returns empty. Errors (missing required input) become a single
/// `Severity::Warning` finding with `kind == "skipped"`.
pub fn run(analyzer: Analyzer, input: &AnalyzeInput<'_>) -> Vec<Finding> {
    match analyzer {
        Analyzer::Layout => match input.snapshot {
            Some(s) => layout::run(s),
            None => vec![Finding::new(
                "skipped",
                Severity::Warning,
                "layout analyzer requires an ElementSnapshot",
            )],
        },
        Analyzer::Typography => match input.snapshot {
            Some(s) => typography::run(s),
            None => vec![Finding::new(
                "skipped",
                Severity::Warning,
                "typography analyzer requires an ElementSnapshot",
            )],
        },
        Analyzer::Color => match (input.frame, input.snapshot) {
            (Some(f), Some(s)) => color::run(f, s),
            _ => vec![Finding::new(
                "skipped",
                Severity::Warning,
                "color analyzer requires both Frame and ElementSnapshot",
            )],
        },
        Analyzer::Dynamic => match (input.prior_frame, input.frame) {
            (Some(prior), Some(cur)) => dynamic::run(prior, cur),
            _ => vec![Finding::new(
                "skipped",
                Severity::Warning,
                "dynamic analyzer requires both a prior_frame and a current frame",
            )],
        },
        Analyzer::Elements => match input.snapshot {
            Some(s) => elements::run(s),
            None => vec![Finding::new(
                "skipped",
                Severity::Warning,
                "elements analyzer requires an ElementSnapshot",
            )],
        },
    }
}
