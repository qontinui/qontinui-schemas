//! The next-action contract — one envelope for every operator-facing refusal.
//!
//! Plan `2026-09-20-the-published-product-works-without-knowing-a-development-environment-exists`,
//! Phase D1.
//!
//! ## What this fixes
//!
//! A refusal that says only what went wrong leaves the reader to guess what to
//! do next, and a free-text "try again" cannot be counted, tested or rendered
//! as a button. [`Refusal`] makes the next step a typed field:
//!
//! - [`Refusal::next_action`] is **not** an `Option`. No refusal can be
//!   constructed without saying what the reader does next — even when the
//!   answer is "nothing" ([`NextActionKind::NoneTerminal`]) or "this is our
//!   bug" ([`NextActionKind::ReportDefect`]). Those two exist so that the
//!   honest answers are STATED rather than rendered as an empty string.
//! - [`Refusal::glossary_terms`] cites vocabulary by typed id
//!   ([`GlossaryTerm`]), so a producer cannot cite a term the glossary does
//!   not define.
//! - [`Refusal::render`] is the human sentence: a pure, table-driven
//!   projection of the fields, in the shape of the merge verdict's own
//!   `next_action` table. Every consumer renders the same sentence for the same
//!   refusal.
//!
//! ## Forward compatibility — a newer producer's refusal always decodes
//!
//! Producers and readers ship on different schedules, so a reader will meet
//! codes, kinds, sources, glossary ids and fields newer than itself. The
//! envelope must survive that with its `next_action` intact; a refusal that
//! fails to parse is a refusal nobody sees. So the Rust decoder is lenient in
//! exactly these places, and says what it did not recognise rather than
//! dropping it silently:
//!
//! | Unrecognised on the wire | Decodes as |
//! |---|---|
//! | `code` | [`RefusalCode::Unknown`], raw string kept in [`Refusal::unrecognised_code`] |
//! | `next_action.kind` | [`NextActionKind::Unrecognised`] (target and delay kept) |
//! | `source` | [`RefusalSource::Unrecognised`] |
//! | a `glossary_terms` id | dropped from `glossary_terms`, kept in [`Refusal::unrecognised_glossary_terms`] |
//! | any other field | ignored (no `deny_unknown_fields` on the wire types or their schemas) |
//!
//! `unrecognised_code` is what keeps "the CAUSE is unknown" (a producer sent
//! `code: "unknown"`) distinct from "this READER does not know the cause" (it
//! sent a code newer than the reader). The `Unrecognised` variants and the two
//! `unrecognised_*` fields are reader-side: producers never construct them,
//! and they are excluded from [`NextActionKind::ALL`] / [`RefusalSource::ALL`].
//!
//! **Generated TS and Python readers.** The bindings are generated from this
//! module's JSON Schema, so enum fields are closed string unions there. A TS
//! reader is unaffected at run time (types are erased), but must keep a
//! `default` arm when switching on `code`, `next_action.kind` or `source`. A
//! Python reader validating with the generated pydantic models will REJECT an
//! unrecognised enum value; until a consumer needs strict models, read
//! refusals from newer producers with `model_validate(..., strict=False)` on a
//! copy whose unknown enum values have been mapped to `"unknown"` /
//! `"unrecognised"`, or validate only the fields you render. Unknown extra
//! fields are tolerated in both languages (the schemas no longer forbid them).
//!
//! ## Relation to the runner's error envelope
//!
//! [`NextActionKind`] is a strict superset of the runner's closed
//! `RecoveryHint` enum, so the runner's envelope can carry a [`Refusal`] in
//! place of its own hint without losing a case. The mapping is documented
//! here only — this crate does not depend on the runner, so there is no
//! `From<RecoveryHint>`:
//!
//! | `RecoveryHint`     | [`NextActionKind`]                                      |
//! |--------------------|---------------------------------------------------------|
//! | `RetryAfterMs(ms)` | `retry_later` + `retry_after_s` ([`NextAction::retry_after_ms`]) |
//! | `WaitForRecovery`  | `retry_later` (no delay known)                          |
//! | `FixRequest`       | `fix_request`                                           |
//! | `Unrecoverable`    | `none_terminal`                                         |
//! | `Resnapshot`       | `resnapshot`                                            |
//! | `ScrollIntoView`   | `scroll_into_view`                                      |
//! | `WaitForEnabled`   | `wait_for_enabled`                                      |
//! | `BroadenSelector`  | `broaden_selector`                                      |
//!
//! When the runner adopts this envelope it should map from its error CODE,
//! not only from the hint, because several codes carry a hint that
//! understates them: `InvalidParam`, `MissingParam` and `InvalidRequest` are
//! `fix_request`, and `InternalError` is `report_defect` (not
//! `none_terminal` — "our bug" is not "nothing can be done").
//!
//! ## Wire format
//!
//! Field names and enum values are `snake_case`. Optional fields are omitted
//! when absent (`skip_serializing_if`), so absence and `null` round-trip
//! faithfully. `glossary_terms` is always emitted (an empty list, never
//! `null`) and is required in the schema; a reader tolerates its absence.
//! `observed_at` is an ISO 8601 string, per this crate's convention.
//!
//! ## Unknown is first-class
//!
//! [`RefusalCode::Unknown`] is the arm for a refusal whose cause is not in the
//! enumerated set: it renders "The request was refused for a reason this
//! version does not recognise" and carries the raw reason in
//! [`Refusal::detail`] — never a guessed code.

use std::fmt;

use schemars::JsonSchema;
use serde::{Deserialize, Deserializer, Serialize};

use crate::glossary::GlossaryTerm;

/// What went wrong, as a stable machine-readable code.
///
/// Extend by adding a variant (and its row in [`RefusalCode::headline`] and
/// [`RefusalCode::from_wire`]); the tables are exhaustive `match`es, so a new
/// code without a sentence is a compile error.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum RefusalCode {
    /// No workspace folder could be resolved for an operation that needs one.
    WorkspaceRootUnresolved,
    /// A checkout the operation builds against is not present next to the one
    /// being worked on.
    SiblingCheckoutAbsent,
    /// The address of a service the operation must call could not be resolved
    /// from configuration.
    EndpointUnresolved,
    /// A glossary id was asked for that this version's glossary does not
    /// define.
    GlossaryTermUnknown,
    /// The cause is not in the enumerated set. The raw reason belongs in
    /// [`Refusal::detail`]. Also what a reader decodes a code it does not
    /// know into (see [`Refusal::unrecognised_code`]).
    #[serde(other)]
    Unknown,
}

impl RefusalCode {
    /// Every code, in declaration order. Kept total by
    /// `every_code_is_listed_in_all` below.
    pub const ALL: &'static [RefusalCode] = &[
        RefusalCode::WorkspaceRootUnresolved,
        RefusalCode::SiblingCheckoutAbsent,
        RefusalCode::EndpointUnresolved,
        RefusalCode::GlossaryTermUnknown,
        RefusalCode::Unknown,
    ];

    /// The wire value (`snake_case`).
    pub fn as_str(self) -> &'static str {
        match self {
            RefusalCode::WorkspaceRootUnresolved => "workspace_root_unresolved",
            RefusalCode::SiblingCheckoutAbsent => "sibling_checkout_absent",
            RefusalCode::EndpointUnresolved => "endpoint_unresolved",
            RefusalCode::GlossaryTermUnknown => "glossary_term_unknown",
            RefusalCode::Unknown => "unknown",
        }
    }

    /// The code for a wire value, or `None` when this version does not know
    /// it.
    pub fn from_wire(s: &str) -> Option<Self> {
        Self::ALL.iter().copied().find(|c| c.as_str() == s)
    }

    /// The first clause of [`Refusal::render`]: what happened, in the
    /// reader's terms.
    pub fn headline(self) -> &'static str {
        match self {
            RefusalCode::WorkspaceRootUnresolved => {
                "No workspace folder could be found for this operation"
            }
            RefusalCode::SiblingCheckoutAbsent => {
                "Source code this operation depends on is not available on this machine"
            }
            RefusalCode::EndpointUnresolved => {
                "The address of a service this operation needs is not configured"
            }
            RefusalCode::GlossaryTermUnknown => "That term is not in this version's glossary",
            RefusalCode::Unknown => {
                "The request was refused for a reason this version does not recognise"
            }
        }
    }
}

/// What the reader does next. A closed set: every refusal names exactly one.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum NextActionKind {
    /// Try the same thing again later; [`NextAction::retry_after_s`] says when,
    /// when it is known.
    RetryLater,
    /// Run the command named in [`NextAction::target`].
    RunCommand,
    /// Open the page named in [`NextAction::target`].
    OpenPage,
    /// Sign in (at [`NextAction::target`] when given).
    SignIn,
    /// Pair this device with an account (at [`NextAction::target`] when given).
    PairDevice,
    /// Set the setting named in [`NextAction::target`].
    SetSetting,
    /// Wait for the gate named in [`NextAction::target`] to clear; the work
    /// resumes on its own.
    WaitForGate,
    /// Nothing the reader does will change the outcome. Stated, not implied.
    NoneTerminal,
    /// The product is at fault; report it (at [`NextAction::target`] when
    /// given).
    ReportDefect,
    /// The request itself is malformed: correct it and send it again. Sending
    /// it unchanged fails the same way.
    FixRequest,
    /// Take a fresh snapshot of the page; element references may have changed.
    Resnapshot,
    /// Scroll or navigate so the element is visible, then retry.
    ScrollIntoView,
    /// The element exists but is disabled; wait for it to become enabled.
    WaitForEnabled,
    /// Use a different or broader selector.
    BroadenSelector,
    /// READER-SIDE ONLY: the producer named a kind newer than this reader.
    /// Never constructed by a producer; not in [`NextActionKind::ALL`].
    #[serde(other)]
    Unrecognised,
}

impl NextActionKind {
    /// Every kind a producer may emit, in declaration order (excludes
    /// [`NextActionKind::Unrecognised`]). Kept total by
    /// `every_kind_is_listed_in_all` below.
    pub const ALL: &'static [NextActionKind] = &[
        NextActionKind::RetryLater,
        NextActionKind::RunCommand,
        NextActionKind::OpenPage,
        NextActionKind::SignIn,
        NextActionKind::PairDevice,
        NextActionKind::SetSetting,
        NextActionKind::WaitForGate,
        NextActionKind::NoneTerminal,
        NextActionKind::ReportDefect,
        NextActionKind::FixRequest,
        NextActionKind::Resnapshot,
        NextActionKind::ScrollIntoView,
        NextActionKind::WaitForEnabled,
        NextActionKind::BroadenSelector,
    ];

    /// The wire value (`snake_case`).
    pub fn as_str(self) -> &'static str {
        match self {
            NextActionKind::RetryLater => "retry_later",
            NextActionKind::RunCommand => "run_command",
            NextActionKind::OpenPage => "open_page",
            NextActionKind::SignIn => "sign_in",
            NextActionKind::PairDevice => "pair_device",
            NextActionKind::SetSetting => "set_setting",
            NextActionKind::WaitForGate => "wait_for_gate",
            NextActionKind::NoneTerminal => "none_terminal",
            NextActionKind::ReportDefect => "report_defect",
            NextActionKind::FixRequest => "fix_request",
            NextActionKind::Resnapshot => "resnapshot",
            NextActionKind::ScrollIntoView => "scroll_into_view",
            NextActionKind::WaitForEnabled => "wait_for_enabled",
            NextActionKind::BroadenSelector => "broaden_selector",
            NextActionKind::Unrecognised => "unrecognised",
        }
    }
}

/// A delay beyond this is not rendered as a count: "try again in 3 years"
/// would be a number, not advice.
const RETRY_RENDER_CEILING_S: u32 = 2 * 24 * 60 * 60;

/// Whole-unit humanised delay: seconds under two minutes, minutes under two
/// hours, hours up to the ceiling (always rounded UP, so the reader never
/// retries early).
fn humanise_delay(s: u32) -> String {
    let plural = |n: u32, unit: &str| {
        if n == 1 {
            format!("1 {unit}")
        } else {
            format!("{n} {unit}s")
        }
    };
    if s < 120 {
        plural(s, "second")
    } else if s < 2 * 60 * 60 {
        plural(s.div_ceil(60), "minute")
    } else {
        plural(s.div_ceil(60 * 60), "hour")
    }
}

/// A caller-supplied target, made safe to quote in one line of prose: control
/// characters and line breaks become spaces, runs of whitespace collapse, and
/// double quotes become single quotes so the surrounding quotes stay
/// unambiguous. `None` when nothing printable is left.
fn quotable(target: Option<&str>) -> Option<String> {
    let cleaned: String = target?
        .chars()
        .map(|c| match c {
            '"' => '\'',
            c if c.is_control() => ' ',
            c => c,
        })
        .collect();
    let collapsed = cleaned.split_whitespace().collect::<Vec<_>>().join(" ");
    (!collapsed.is_empty()).then_some(collapsed)
}

/// The typed next step of a [`Refusal`].
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct NextAction {
    pub kind: NextActionKind,
    /// What the action applies to: the command to run, the page to open, the
    /// setting to set, the gate to wait for. Its meaning is fixed by `kind`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub target: Option<String>,
    /// For [`NextActionKind::RetryLater`]: the earliest useful retry, in whole
    /// seconds. Absent when no delay is known.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub retry_after_s: Option<u32>,
}

impl NextAction {
    /// A next action of `kind` with no target and no delay.
    pub fn new(kind: NextActionKind) -> Self {
        Self {
            kind,
            target: None,
            retry_after_s: None,
        }
    }

    /// `retry_later`, optionally after `retry_after_s` seconds.
    pub fn retry_later(retry_after_s: Option<u32>) -> Self {
        Self {
            retry_after_s,
            ..Self::new(NextActionKind::RetryLater)
        }
    }

    /// `retry_later` from a millisecond delay (the runner's
    /// `RecoveryHint::RetryAfterMs`). Rounded UP to whole seconds so a
    /// sub-second delay never renders as "retry in 0 seconds", and saturated
    /// at `u32::MAX`.
    pub fn retry_after_ms(ms: u64) -> Self {
        let secs = ms.div_ceil(1000);
        Self::retry_later(Some(u32::try_from(secs).unwrap_or(u32::MAX)))
    }

    /// Builder: set [`NextAction::target`].
    pub fn with_target(mut self, target: impl Into<String>) -> Self {
        self.target = Some(target.into());
        self
    }

    /// The second clause of [`Refusal::render`]: what to do, as an imperative
    /// sentence without its closing full stop. Never empty. A target is
    /// quoted in plain double quotes (never markup) after [`quotable`]
    /// cleaning.
    pub fn render(&self) -> String {
        const UNNAMED: &str = "(it did not name one, which is itself a defect worth reporting)";
        let target = quotable(self.target.as_deref());
        match (self.kind, target) {
            (NextActionKind::RetryLater, _) => match self.retry_after_s {
                Some(0) => "Try again now".to_string(),
                Some(s) if s > RETRY_RENDER_CEILING_S => {
                    "Try again later; the suggested wait is more than two days".to_string()
                }
                Some(s) => format!("Try again in {}", humanise_delay(s)),
                None => "Try again later".to_string(),
            },
            (NextActionKind::RunCommand, Some(t)) => format!("Run the command \"{t}\""),
            (NextActionKind::RunCommand, None) => {
                format!("Run the command this refusal refers to {UNNAMED}")
            }
            (NextActionKind::OpenPage, Some(t)) => format!("Open \"{t}\""),
            (NextActionKind::OpenPage, None) => {
                format!("Open the page this refusal refers to {UNNAMED}")
            }
            (NextActionKind::SignIn, Some(t)) => format!("Sign in at \"{t}\", then try again"),
            (NextActionKind::SignIn, None) => "Sign in, then try again".to_string(),
            (NextActionKind::PairDevice, Some(t)) => {
                format!("Pair this device with your account at \"{t}\", then try again")
            }
            (NextActionKind::PairDevice, None) => {
                "Pair this device with your account, then try again".to_string()
            }
            (NextActionKind::SetSetting, Some(t)) => {
                format!("Set the \"{t}\" setting, then try again")
            }
            (NextActionKind::SetSetting, None) => {
                format!("Set the setting this refusal refers to {UNNAMED}")
            }
            (NextActionKind::WaitForGate, Some(t)) => {
                format!("Wait for gate \"{t}\" to clear; the work resumes on its own")
            }
            (NextActionKind::WaitForGate, None) => {
                "Wait for the blocking gate to clear; the work resumes on its own".to_string()
            }
            (NextActionKind::NoneTerminal, _) => {
                "Nothing you can do will change this outcome".to_string()
            }
            (NextActionKind::ReportDefect, Some(t)) => {
                format!("This is a defect in the product; report it at \"{t}\"")
            }
            (NextActionKind::ReportDefect, None) => {
                "This is a defect in the product; please report it".to_string()
            }
            (NextActionKind::FixRequest, Some(t)) => format!(
                "Correct \"{t}\" in the request and send it again; sending it unchanged fails the same way"
            ),
            (NextActionKind::FixRequest, None) => {
                "Correct the request and send it again; sending it unchanged fails the same way"
                    .to_string()
            }
            (NextActionKind::Resnapshot, _) => {
                "Take a fresh snapshot of the page, then retry".to_string()
            }
            (NextActionKind::ScrollIntoView, _) => {
                "Scroll or navigate until the element is visible, then retry".to_string()
            }
            (NextActionKind::WaitForEnabled, _) => {
                "Wait for the element to become enabled, then retry".to_string()
            }
            (NextActionKind::BroadenSelector, _) => {
                "Use a different or broader selector".to_string()
            }
            (NextActionKind::Unrecognised, _) => {
                "This version cannot show the suggested next step; update the application to see it"
                    .to_string()
            }
        }
    }
}

/// The component that produced a [`Refusal`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum RefusalSource {
    /// The desktop runner.
    Runner,
    /// The coordination service.
    Coord,
    /// The web application's backend.
    WebBackend,
    /// The web application's frontend.
    WebFrontend,
    /// READER-SIDE ONLY: a source newer than this reader. Never constructed
    /// by a producer; not in [`RefusalSource::ALL`].
    #[serde(other)]
    Unrecognised,
}

impl RefusalSource {
    /// Every source a producer may emit (excludes
    /// [`RefusalSource::Unrecognised`]).
    pub const ALL: &'static [RefusalSource] = &[
        RefusalSource::Runner,
        RefusalSource::Coord,
        RefusalSource::WebBackend,
        RefusalSource::WebFrontend,
    ];
}

/// One operator-facing refusal: what went wrong, and what to do next.
///
/// Construct with [`Refusal::new`] and the `with_*` builders. `next_action` is
/// deliberately not optional — see the module docs. Decoding is lenient toward
/// newer producers (module docs, "Forward compatibility").
#[derive(Debug, Clone, PartialEq, Eq, Serialize, JsonSchema)]
pub struct Refusal {
    pub code: RefusalCode,
    /// Narrows `code` to the specific case (the setting that was empty, the
    /// service whose address was missing). Rendered after the headline.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub discriminator: Option<String>,
    pub next_action: NextAction,
    /// Glossary terms a reader may need to act on this refusal. Typed, so a
    /// producer cannot cite a term the glossary does not define. Always
    /// present on the wire (an empty list, never null).
    pub glossary_terms: Vec<GlossaryTerm>,
    /// Raw diagnostic text (the underlying error, the unrecognised reason).
    /// Shown beside the rendered sentence, never inside it.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub detail: Option<String>,
    /// When the refusal was observed (ISO 8601).
    pub observed_at: String,
    pub source: RefusalSource,
    /// READER-SIDE: the raw `code` when this reader did not recognise it
    /// (`code` is then [`RefusalCode::Unknown`]). Absent when the producer
    /// itself said `unknown`. Never set by a producer.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub unrecognised_code: Option<String>,
    /// READER-SIDE: glossary ids this reader's glossary does not define,
    /// removed from `glossary_terms`. Never set by a producer.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub unrecognised_glossary_terms: Vec<String>,
}

/// The lenient decode shape: every closed vocabulary arrives as a string and
/// is classified in [`Refusal::deserialize`].
#[derive(Deserialize)]
struct RefusalWire {
    code: String,
    #[serde(default)]
    discriminator: Option<String>,
    next_action: NextAction,
    #[serde(default)]
    glossary_terms: Option<Vec<String>>,
    #[serde(default)]
    detail: Option<String>,
    observed_at: String,
    source: RefusalSource,
    #[serde(default)]
    unrecognised_code: Option<String>,
    #[serde(default)]
    unrecognised_glossary_terms: Vec<String>,
}

impl<'de> Deserialize<'de> for Refusal {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let w = RefusalWire::deserialize(deserializer)?;
        let (code, mut unrecognised_code) = match RefusalCode::from_wire(&w.code) {
            Some(c) => (c, None),
            None => (RefusalCode::Unknown, Some(w.code)),
        };
        // A relayed refusal that was already classified keeps its record.
        if unrecognised_code.is_none() && code == RefusalCode::Unknown {
            unrecognised_code = w.unrecognised_code;
        }
        let mut glossary_terms = Vec::new();
        let mut unrecognised_glossary_terms = w.unrecognised_glossary_terms;
        for id in w.glossary_terms.unwrap_or_default() {
            match GlossaryTerm::from_id(&id) {
                Some(t) => glossary_terms.push(t),
                None => unrecognised_glossary_terms.push(id),
            }
        }
        Ok(Refusal {
            code,
            discriminator: w.discriminator,
            next_action: w.next_action,
            glossary_terms,
            detail: w.detail,
            observed_at: w.observed_at,
            source: w.source,
            unrecognised_code,
            unrecognised_glossary_terms,
        })
    }
}

impl Refusal {
    /// A refusal with no discriminator, no glossary terms and no detail.
    pub fn new(
        code: RefusalCode,
        next_action: NextAction,
        source: RefusalSource,
        observed_at: impl Into<String>,
    ) -> Self {
        Self {
            code,
            discriminator: None,
            next_action,
            glossary_terms: Vec::new(),
            detail: None,
            observed_at: observed_at.into(),
            source,
            unrecognised_code: None,
            unrecognised_glossary_terms: Vec::new(),
        }
    }

    /// Builder: set [`Refusal::discriminator`].
    pub fn with_discriminator(mut self, discriminator: impl Into<String>) -> Self {
        self.discriminator = Some(discriminator.into());
        self
    }

    /// Builder: set [`Refusal::detail`].
    pub fn with_detail(mut self, detail: impl Into<String>) -> Self {
        self.detail = Some(detail.into());
        self
    }

    /// Builder: set [`Refusal::glossary_terms`].
    pub fn with_glossary_terms(mut self, terms: impl IntoIterator<Item = GlossaryTerm>) -> Self {
        self.glossary_terms = terms.into_iter().collect();
        self
    }

    /// The human sentence: `<headline>[ (<discriminator>)]. <next action>.`
    ///
    /// Pure and table-driven — a projection of `code`, `discriminator` and
    /// `next_action` only. `detail` (raw diagnostic text), `glossary_terms`
    /// (rendered by the consumer as links or tooltips) and the reader-side
    /// `unrecognised_*` fields are deliberately not folded in. Never empty:
    /// every [`RefusalCode`] has a headline and every [`NextActionKind`] a
    /// sentence.
    pub fn render(&self) -> String {
        let mut out = String::from(self.code.headline());
        if let Some(d) = quotable(self.discriminator.as_deref()) {
            out.push_str(" (");
            out.push_str(&d);
            out.push(')');
        }
        out.push_str(". ");
        out.push_str(&self.next_action.render());
        out.push('.');
        out
    }
}

impl fmt::Display for Refusal {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.render())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const AT: &str = "2026-09-29T00:00:00Z";

    /// Compile-time totality: adding a variant without listing it in `ALL`
    /// fails here, because the match below must name it and the count check
    /// then fails.
    #[test]
    fn every_code_is_listed_in_all() {
        let count = |c: RefusalCode| match c {
            RefusalCode::WorkspaceRootUnresolved
            | RefusalCode::SiblingCheckoutAbsent
            | RefusalCode::EndpointUnresolved
            | RefusalCode::GlossaryTermUnknown
            | RefusalCode::Unknown => 1,
        };
        assert_eq!(RefusalCode::ALL.iter().map(|c| count(*c)).sum::<usize>(), 5);
        for c in RefusalCode::ALL {
            assert_eq!(serde_json::to_value(c).unwrap(), c.as_str());
            assert_eq!(RefusalCode::from_wire(c.as_str()), Some(*c));
        }
    }

    #[test]
    fn every_kind_is_listed_in_all() {
        let count = |k: NextActionKind| match k {
            NextActionKind::RetryLater
            | NextActionKind::RunCommand
            | NextActionKind::OpenPage
            | NextActionKind::SignIn
            | NextActionKind::PairDevice
            | NextActionKind::SetSetting
            | NextActionKind::WaitForGate
            | NextActionKind::NoneTerminal
            | NextActionKind::ReportDefect
            | NextActionKind::FixRequest
            | NextActionKind::Resnapshot
            | NextActionKind::ScrollIntoView
            | NextActionKind::WaitForEnabled
            | NextActionKind::BroadenSelector => 1,
            // Reader-side only: deliberately NOT in ALL.
            NextActionKind::Unrecognised => 0,
        };
        assert_eq!(
            NextActionKind::ALL.iter().map(|k| count(*k)).sum::<usize>(),
            14
        );
        assert!(!NextActionKind::ALL.contains(&NextActionKind::Unrecognised));
        assert!(!RefusalSource::ALL.contains(&RefusalSource::Unrecognised));
        for k in NextActionKind::ALL {
            assert_eq!(serde_json::to_value(k).unwrap(), k.as_str());
        }
    }

    #[test]
    fn serde_round_trip_full_and_minimal() {
        let full = Refusal::new(
            RefusalCode::EndpointUnresolved,
            NextAction::new(NextActionKind::SetSetting).with_target("backend_url"),
            RefusalSource::Runner,
            AT,
        )
        .with_discriminator("backend")
        .with_detail("no value in settings or environment")
        .with_glossary_terms([GlossaryTerm::Device, GlossaryTerm::Tenant]);
        let json = serde_json::to_value(&full).unwrap();
        assert_eq!(
            json,
            serde_json::json!({
                "code": "endpoint_unresolved",
                "discriminator": "backend",
                "next_action": {"kind": "set_setting", "target": "backend_url"},
                "glossary_terms": ["device", "tenant"],
                "detail": "no value in settings or environment",
                "observed_at": AT,
                "source": "runner"
            })
        );
        let back: Refusal = serde_json::from_value(json).unwrap();
        assert_eq!(back, full);

        let minimal = Refusal::new(
            RefusalCode::Unknown,
            NextAction::retry_later(None),
            RefusalSource::Coord,
            AT,
        );
        let json = serde_json::to_value(&minimal).unwrap();
        assert_eq!(
            json,
            serde_json::json!({
                "code": "unknown",
                "next_action": {"kind": "retry_later"},
                "glossary_terms": [],
                "observed_at": AT,
                "source": "coord"
            })
        );
        assert_eq!(serde_json::from_value::<Refusal>(json).unwrap(), minimal);

        // Absent or null `glossary_terms` decodes as empty.
        for terms in [None, Some(serde_json::Value::Null)] {
            let mut v = serde_json::json!({
                "code": "unknown",
                "next_action": {"kind": "none_terminal"},
                "observed_at": AT,
                "source": "web_backend"
            });
            if let Some(t) = terms {
                v["glossary_terms"] = t;
            }
            let r: Refusal = serde_json::from_value(v).unwrap();
            assert!(r.glossary_terms.is_empty());
        }
    }

    #[test]
    fn glossary_terms_is_required_and_non_null_in_the_schema() {
        let schema = serde_json::to_value(schemars::schema_for!(Refusal)).unwrap();
        let required: Vec<&str> = schema["required"]
            .as_array()
            .unwrap()
            .iter()
            .map(|v| v.as_str().unwrap())
            .collect();
        assert!(required.contains(&"glossary_terms"), "{required:?}");
        assert!(required.contains(&"next_action"), "{required:?}");
        assert_eq!(schema["properties"]["glossary_terms"]["type"], "array");
        // No `additionalProperties: false`: readers tolerate newer fields.
        assert!(schema.get("additionalProperties").is_none(), "{schema}");
        let na = &schema["$defs"]["NextAction"];
        assert!(na.get("additionalProperties").is_none(), "{na}");
    }

    #[test]
    fn next_action_is_required_on_the_wire() {
        let missing = serde_json::json!({
            "code": "unknown",
            "observed_at": AT,
            "source": "runner"
        });
        let err = serde_json::from_value::<Refusal>(missing).unwrap_err();
        assert!(err.to_string().contains("next_action"), "{err}");
    }

    /// A refusal from a newer producer — new code, kind, source, glossary id
    /// and field all at once — decodes, keeps its target, and records what
    /// was not recognised.
    #[test]
    fn a_newer_producers_refusal_decodes_with_what_was_unrecognised_recorded() {
        let newer = serde_json::json!({
            "code": "some_code_from_a_newer_producer",
            "next_action": {"kind": "do_something_new", "target": "t", "hint": 1},
            "glossary_terms": ["gate", "a_term_from_a_newer_glossary"],
            "observed_at": AT,
            "source": "a_new_component",
            "a_new_field": {"x": 1}
        });
        let r: Refusal = serde_json::from_value(newer).unwrap();
        assert_eq!(r.code, RefusalCode::Unknown);
        assert_eq!(
            r.unrecognised_code.as_deref(),
            Some("some_code_from_a_newer_producer")
        );
        assert_eq!(r.next_action.kind, NextActionKind::Unrecognised);
        assert_eq!(r.next_action.target.as_deref(), Some("t"));
        assert_eq!(r.source, RefusalSource::Unrecognised);
        assert_eq!(r.glossary_terms, vec![GlossaryTerm::Gate]);
        assert_eq!(
            r.unrecognised_glossary_terms,
            vec!["a_term_from_a_newer_glossary".to_string()]
        );
        assert!(
            r.render().contains("update the application"),
            "{}",
            r.render()
        );

        // Re-emitting keeps the record, and decoding THAT again is stable.
        let again: Refusal = serde_json::from_value(serde_json::to_value(&r).unwrap()).unwrap();
        assert_eq!(again, r);
    }

    /// "The producer says the cause is unknown" and "this reader does not
    /// know the producer's code" stay distinguishable.
    #[test]
    fn cause_unknown_is_not_collapsed_into_reader_unknown() {
        let cause_unknown: Refusal = serde_json::from_value(serde_json::json!({
            "code": "unknown",
            "next_action": {"kind": "report_defect"},
            "glossary_terms": [],
            "observed_at": AT,
            "source": "coord"
        }))
        .unwrap();
        assert_eq!(cause_unknown.code, RefusalCode::Unknown);
        assert_eq!(cause_unknown.unrecognised_code, None);
    }

    #[test]
    fn retry_after_ms_rounds_up_and_saturates() {
        assert_eq!(NextAction::retry_after_ms(0).retry_after_s, Some(0));
        assert_eq!(NextAction::retry_after_ms(1).retry_after_s, Some(1));
        assert_eq!(NextAction::retry_after_ms(500).retry_after_s, Some(1));
        assert_eq!(NextAction::retry_after_ms(1000).retry_after_s, Some(1));
        assert_eq!(NextAction::retry_after_ms(1001).retry_after_s, Some(2));
        assert_eq!(
            NextAction::retry_after_ms(u64::MAX).retry_after_s,
            Some(u32::MAX)
        );
        assert_eq!(
            NextAction::retry_after_ms(500).kind,
            NextActionKind::RetryLater
        );
    }

    #[test]
    fn retry_delays_are_humanised_and_capped() {
        let r = |s| NextAction::retry_later(Some(s)).render();
        assert_eq!(r(0), "Try again now");
        assert_eq!(r(1), "Try again in 1 second");
        assert_eq!(r(119), "Try again in 119 seconds");
        assert_eq!(r(120), "Try again in 2 minutes");
        assert_eq!(r(121), "Try again in 3 minutes");
        assert_eq!(r(7200), "Try again in 2 hours");
        assert_eq!(r(RETRY_RENDER_CEILING_S), "Try again in 48 hours");
        assert_eq!(
            r(RETRY_RENDER_CEILING_S + 1),
            "Try again later; the suggested wait is more than two days"
        );
        assert_eq!(r(u32::MAX), r(RETRY_RENDER_CEILING_S + 1));
    }

    #[test]
    fn targets_are_quoted_as_one_clean_line() {
        let na =
            NextAction::new(NextActionKind::RunCommand).with_target("echo \"hi\"\n  && `rm`\tx");
        assert_eq!(na.render(), "Run the command \"echo 'hi' && `rm` x\"");
        let r = Refusal::new(RefusalCode::Unknown, na, RefusalSource::Runner, AT)
            .with_discriminator("a\nb");
        assert!(r.render().contains("(a b)"), "{}", r.render());
        assert_eq!(r.render().lines().count(), 1);
    }

    #[test]
    fn render_is_the_table_projection() {
        let r = Refusal::new(
            RefusalCode::WorkspaceRootUnresolved,
            NextAction::new(NextActionKind::SetSetting).with_target("workspace_root"),
            RefusalSource::Runner,
            AT,
        )
        .with_detail("raw detail is not rendered");
        assert_eq!(
            r.render(),
            "No workspace folder could be found for this operation. \
             Set the \"workspace_root\" setting, then try again."
        );
        assert_eq!(r.to_string(), r.render());

        let r = Refusal::new(
            RefusalCode::EndpointUnresolved,
            NextAction::retry_after_ms(1500),
            RefusalSource::WebFrontend,
            AT,
        )
        .with_discriminator("backend");
        assert_eq!(
            r.render(),
            "The address of a service this operation needs is not configured (backend). \
             Try again in 2 seconds."
        );

        // A blank discriminator or target renders as absent, not as "()" / "\"\"".
        let r = Refusal::new(
            RefusalCode::Unknown,
            NextAction::new(NextActionKind::RunCommand).with_target("  "),
            RefusalSource::Runner,
            AT,
        )
        .with_discriminator(" ");
        assert!(!r.render().contains("()"), "{}", r.render());
        assert!(!r.render().contains("\"\""), "{}", r.render());
    }

    /// Exhaustive over codes × kinds (including the reader-side
    /// `Unrecognised`) × target/delay/discriminator shapes: never empty,
    /// always one headline and one sentence on one line, and no blank
    /// placeholder leaks through. (The fleet-noun half of this property reads
    /// the repo-root vocabulary, so it is a repo test:
    /// `tests/refusal_render.rs`.)
    #[test]
    fn render_is_never_empty_for_any_shape() {
        let targets = [None, Some(""), Some("x"), Some("a\n\"b\"")];
        let delays = [None, Some(0), Some(1), Some(90), Some(u32::MAX)];
        let discriminators = [None, Some(""), Some("d")];
        let kinds: Vec<NextActionKind> = NextActionKind::ALL
            .iter()
            .copied()
            .chain([NextActionKind::Unrecognised])
            .collect();
        let mut n = 0usize;
        for &code in RefusalCode::ALL {
            for &kind in &kinds {
                for target in targets {
                    for retry_after_s in delays {
                        for disc in discriminators {
                            let mut na = NextAction::new(kind);
                            na.target = target.map(str::to_string);
                            na.retry_after_s = retry_after_s;
                            let mut r = Refusal::new(code, na, RefusalSource::Runner, AT);
                            r.discriminator = disc.map(str::to_string);
                            let s = r.render();
                            assert!(!s.trim().is_empty());
                            assert!(s.ends_with('.'), "{s}");
                            assert!(s.starts_with(code.headline()), "{s}");
                            assert_eq!(s.lines().count(), 1, "{s}");
                            assert!(!r.next_action.render().trim().is_empty());
                            assert!(!s.contains("()") && !s.contains("\"\""), "{s}");
                            n += 1;
                        }
                    }
                }
            }
        }
        assert_eq!(n, RefusalCode::ALL.len() * kinds.len() * 4 * 5 * 3);
    }
}
