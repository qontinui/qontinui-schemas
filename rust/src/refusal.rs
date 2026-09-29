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
//!   ([`GlossaryTerm`]), so a refusal cannot cite a term the glossary does not
//!   define.
//! - [`Refusal::render`] is the human sentence: a pure, table-driven
//!   projection of the fields, in the shape of the merge verdict's own
//!   `next_action` table. Every consumer renders the same sentence for the same
//!   refusal.
//!
//! ## Relation to the runner's `RecoveryHint`
//!
//! [`NextActionKind`] is a strict superset of the runner's closed
//! `RecoveryHint` enum, so the runner's error envelope can carry a
//! [`Refusal`] in place of its own hint without losing a case:
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
//! ## Wire format
//!
//! Field names and enum values are `snake_case`. Optional fields are omitted
//! when absent (`skip_serializing_if`), so absence and `null` round-trip
//! faithfully. `observed_at` is an ISO 8601 string, per this crate's
//! convention.
//!
//! ## Unknown is first-class
//!
//! [`RefusalCode::Unknown`] is the arm for a refusal whose cause is not in the
//! enumerated set: it renders "The cause is not one this version recognises"
//! and carries the raw reason in [`Refusal::detail`] — never a guessed code.
//! It is also what an older reader decodes a code it has never seen into
//! (`#[serde(other)]`), so a newer producer's refusal degrades to an honest
//! UNKNOWN with its `next_action` intact instead of failing to parse.

use std::fmt;

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::glossary::GlossaryTerm;

/// What went wrong, as a stable machine-readable code.
///
/// Extend by adding a variant (and its row in [`RefusalCode::headline`]); the
/// render table is an exhaustive `match`, so a new code without a sentence is
/// a compile error.
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
    /// [`Refusal::detail`]. Also the decode target for a code this version
    /// does not know.
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

    /// The first clause of [`Refusal::render`]: what happened, in the
    /// reader's terms.
    pub fn headline(self) -> &'static str {
        match self {
            RefusalCode::WorkspaceRootUnresolved => {
                "No workspace folder could be found for this operation"
            }
            RefusalCode::SiblingCheckoutAbsent => {
                "A repository this operation builds against is not checked out beside this one"
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
}

impl NextActionKind {
    /// Every kind, in declaration order. Kept total by
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
        }
    }
}

/// The typed next step of a [`Refusal`].
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[schemars(deny_unknown_fields)]
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
    /// sentence without its closing full stop. Never empty.
    pub fn render(&self) -> String {
        let target = self
            .target
            .as_deref()
            .map(str::trim)
            .filter(|t| !t.is_empty());
        match (self.kind, target) {
            (NextActionKind::RetryLater, _) => match self.retry_after_s {
                Some(0) => "Try again now".to_string(),
                Some(1) => "Try again in 1 second".to_string(),
                Some(s) => format!("Try again in {s} seconds"),
                None => "Try again later".to_string(),
            },
            (NextActionKind::RunCommand, Some(t)) => format!("Run `{t}`"),
            (NextActionKind::RunCommand, None) => {
                "Run the command this refusal refers to (it did not name one, which is itself a defect worth reporting)".to_string()
            }
            (NextActionKind::OpenPage, Some(t)) => format!("Open {t}"),
            (NextActionKind::OpenPage, None) => {
                "Open the page this refusal refers to (it did not name one, which is itself a defect worth reporting)".to_string()
            }
            (NextActionKind::SignIn, Some(t)) => format!("Sign in at {t}, then try again"),
            (NextActionKind::SignIn, None) => "Sign in, then try again".to_string(),
            (NextActionKind::PairDevice, Some(t)) => {
                format!("Pair this device with your account at {t}, then try again")
            }
            (NextActionKind::PairDevice, None) => {
                "Pair this device with your account, then try again".to_string()
            }
            (NextActionKind::SetSetting, Some(t)) => {
                format!("Set the `{t}` setting, then try again")
            }
            (NextActionKind::SetSetting, None) => {
                "Set the setting this refusal refers to (it did not name one, which is itself a defect worth reporting)".to_string()
            }
            (NextActionKind::WaitForGate, Some(t)) => {
                format!("Wait for gate {t} to clear; the work resumes on its own")
            }
            (NextActionKind::WaitForGate, None) => {
                "Wait for the blocking gate to clear; the work resumes on its own".to_string()
            }
            (NextActionKind::NoneTerminal, _) => {
                "Nothing you can do will change this outcome".to_string()
            }
            (NextActionKind::ReportDefect, Some(t)) => {
                format!("This is a defect in the product; report it at {t}")
            }
            (NextActionKind::ReportDefect, None) => {
                "This is a defect in the product; please report it".to_string()
            }
            (NextActionKind::FixRequest, Some(t)) => format!(
                "Correct `{t}` in the request and send it again; sending it unchanged fails the same way"
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
}

/// One operator-facing refusal: what went wrong, and what to do next.
///
/// Construct with [`Refusal::new`] and the `with_*` builders. `next_action` is
/// deliberately not optional — see the module docs.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[schemars(deny_unknown_fields)]
pub struct Refusal {
    pub code: RefusalCode,
    /// Narrows `code` to the specific case (the setting that was empty, the
    /// service whose address was missing). Rendered after the headline.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub discriminator: Option<String>,
    pub next_action: NextAction,
    /// Glossary terms a reader may need to act on this refusal. Typed, so a
    /// refusal cannot cite a term the glossary does not define.
    #[serde(default)]
    pub glossary_terms: Vec<GlossaryTerm>,
    /// Raw diagnostic text (the underlying error, the unrecognised reason).
    /// Shown beside the rendered sentence, never inside it.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub detail: Option<String>,
    /// When the refusal was observed (ISO 8601).
    pub observed_at: String,
    pub source: RefusalSource,
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
    /// `next_action` only. `detail` (raw diagnostic text) and
    /// `glossary_terms` (rendered by the consumer as links or tooltips) are
    /// deliberately not folded in. Never empty: every [`RefusalCode`] has a
    /// headline and every [`NextActionKind`] a sentence.
    pub fn render(&self) -> String {
        let mut out = String::from(self.code.headline());
        if let Some(d) = self
            .discriminator
            .as_deref()
            .map(str::trim)
            .filter(|d| !d.is_empty())
        {
            out.push_str(" (");
            out.push_str(d);
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
        };
        assert_eq!(
            NextActionKind::ALL.iter().map(|k| count(*k)).sum::<usize>(),
            14
        );
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

        // Absent `glossary_terms` decodes as empty.
        let no_terms = serde_json::json!({
            "code": "unknown",
            "next_action": {"kind": "none_terminal"},
            "observed_at": AT,
            "source": "web_backend"
        });
        let r: Refusal = serde_json::from_value(no_terms).unwrap();
        assert!(r.glossary_terms.is_empty());
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

    #[test]
    fn an_unrecognised_code_decodes_as_unknown_with_its_next_action_intact() {
        let newer = serde_json::json!({
            "code": "some_code_from_a_newer_producer",
            "next_action": {"kind": "sign_in"},
            "observed_at": AT,
            "source": "coord"
        });
        let r: Refusal = serde_json::from_value(newer).unwrap();
        assert_eq!(r.code, RefusalCode::Unknown);
        assert_eq!(r.next_action.kind, NextActionKind::SignIn);
    }

    #[test]
    fn an_unrecognised_next_action_kind_is_a_parse_error_not_a_guess() {
        let bad = serde_json::json!({"kind": "do_something_else"});
        assert!(serde_json::from_value::<NextAction>(bad).is_err());
    }

    #[test]
    fn an_unknown_glossary_id_is_a_parse_error() {
        let bad = serde_json::json!({
            "code": "unknown",
            "next_action": {"kind": "none_terminal"},
            "glossary_terms": ["not_a_term"],
            "observed_at": AT,
            "source": "runner"
        });
        assert!(serde_json::from_value::<Refusal>(bad).is_err());
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
             Set the `workspace_root` setting, then try again."
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

        // A blank discriminator or target renders as absent, not as "()" / "``".
        let r = Refusal::new(
            RefusalCode::Unknown,
            NextAction::new(NextActionKind::RunCommand).with_target("  "),
            RefusalSource::Runner,
            AT,
        )
        .with_discriminator(" ");
        assert!(!r.render().contains("()"), "{}", r.render());
        assert!(!r.render().contains("``"), "{}", r.render());
    }

    /// Exhaustive over codes × kinds × target/delay/discriminator shapes:
    /// never empty, always one headline and one sentence, and no blank
    /// placeholder leaks through. (The fleet-noun half of this property reads
    /// the repo-root vocabulary, so it is a repo test:
    /// `tests/refusal_render.rs`.)
    #[test]
    fn render_is_never_empty_for_any_shape() {
        let targets = [None, Some(""), Some("x")];
        let delays = [None, Some(0), Some(1), Some(90)];
        let discriminators = [None, Some(""), Some("d")];
        let mut n = 0usize;
        for &code in RefusalCode::ALL {
            for &kind in NextActionKind::ALL {
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
                            assert!(!r.next_action.render().trim().is_empty());
                            assert!(!s.contains("()") && !s.contains("``"), "{s}");
                            n += 1;
                        }
                    }
                }
            }
        }
        assert_eq!(
            n,
            RefusalCode::ALL.len() * NextActionKind::ALL.len() * 3 * 4 * 3
        );
    }
}
