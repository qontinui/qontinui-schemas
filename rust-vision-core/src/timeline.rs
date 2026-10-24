//! The interaction timeline — the Rust mirror of the UI Bridge SDK's
//! `ChangeTimeline` (`@qontinui/ui-bridge` `packages/ui-bridge/src/ai/types.ts`,
//! as landed in qontinui/ui-bridge#258; plan
//! `2026-09-20-ui-bridge-represents-the-users-path-and-the-passage-of-time`,
//! D5 and Phase 4).
//!
//! One timeline per interaction: the action, every event observed after it
//! with its offset from the action, the `settled` event, and a derived
//! [`TimelineSummary`] whose every field is an [`Observation`]. `POST
//! /ai/execute-with-diff` returns it as `ActionDiffResult.timeline` (default-on;
//! `timeline: false` is the explicit opt-out), and
//! [`crate::assertions::EvalContext::timeline`] is how the timeline assertions
//! ([`crate::Assertion::AnimationSettled`], [`crate::Assertion::SettledWithin`],
//! [`crate::Assertion::StateEnteredWithin`]) read it.
//!
//! # Why it lives in this crate
//!
//! Every summary field is the shared [`Observation`] envelope, which lives
//! here, and so do the assertions that consume the timeline. `qontinui-types`
//! deliberately does not depend on this crate (it would pull `image` into every
//! consumer of the wire types), so the timeline cannot live there without a
//! second envelope — the parallel unknown type the plan forbids.
//!
//! # Wire format
//!
//! Exactly the SDK's: structs `camelCase`, [`TimelineEvent`] internally tagged
//! on `type` with kebab-case tags and camelCase fields,
//! [`ExpectedStateOutcome`] a one-key object. A TS `T | null` field is an
//! `Option` that always serializes (`null` is a statement there); a TS `x?: T`
//! field is an `Option` skipped when `None`. Times are `f64` milliseconds: the
//! SDK rounds most offsets, but a caller-echoed window or a network tracker's
//! duration need not be integral, and a fractional value must not drop the
//! whole timeline at parse.
//!
//! Unknown keys are IGNORED, as the envelope ignores them, so a newer SDK
//! adding a field does not make an older runner drop the timeline. The
//! fixture gate `tests/timeline_wire.rs` round-trips SDK-captured timelines
//! and fails on any key this mirror would drop, so drift is a red test rather
//! than a silent loss. A re-serialized timeline therefore carries only the
//! mirrored fields — no open `serde_json::Value` slot other than the
//! envelope's own `provenance.source`.
//!
//! PRIVACY: [`TimelineEventBase::summary`] is SDK-generated prose that quotes
//! URLs, the `page-changed` event carries the full new URL, and the
//! `network-request-*` events carry `url` as origin + pathname (the SDK strips
//! the query and fragment). A concrete pathname can embed user input
//! (`/users/<email>`). This type mirrors the wire and does not redact; a
//! consumer that STORES a timeline (the journey ledger's `timeline` column)
//! stores [`ChangeTimeline::for_ledger`], the one redacted projection.
//!
//! # No thresholds
//!
//! Nothing here compares a timing against a constant. The timeline carries
//! values; the caller's `withinMs` / an assertion's `ms` is the only bound.

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::observation::Observation;

/// One CSS animation or transition, as the SDK's idle signals and the
/// timeline report it (`idle/detected-animation.ts`).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct DetectedAnimation {
    /// `animationName`, or `transition:<property>` for a CSS transition.
    pub name: String,
    /// `ElementFingerprintData.hash` of the animated element, when it could
    /// be computed.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub target_element: Option<String>,
    /// Duration (ms) when known — the elapsed time at end, or the effect's
    /// timing.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub duration_ms: Option<f64>,
    /// Whether the animation repeats forever.
    pub is_infinite: bool,
}

/// A source of timeline events. A source NOT listed in
/// [`ChangeTimeline::observed_sources`] was not looked at, so the absence of
/// its events means "unknown", never "none".
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "kebab-case")]
pub enum TimelineSource {
    /// Registry snapshot polling: elements-*, page-changed, settled.
    SnapshotPoll,
    /// The idle detector gated `settled`.
    IdleDetector,
    /// PerformanceObserver `layout-shift`.
    LayoutShift,
    /// PerformanceObserver `longtask`.
    LongTask,
    /// DOM animation / transition events.
    AnimationEvents,
    /// `document.getAnimations()` at the end of the recording.
    GetAnimations,
    /// A NetworkRequestTracker.
    Network,
    /// The declared-state registry.
    States,
}

/// Why an animation-ended event fired.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum AnimationEndReason {
    /// The animation ran to its end.
    Finished,
    /// The animation was cancelled.
    Cancelled,
}

/// Fields every [`TimelineEvent`] carries.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct TimelineEventBase {
    /// Time offset from action start (ms).
    pub offset_ms: f64,
    /// Human-readable summary (SDK prose — see the module's PRIVACY note).
    pub summary: String,
}

/// A single event in the interaction timeline, discriminated on `type`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(
    tag = "type",
    rename_all = "kebab-case",
    rename_all_fields = "camelCase"
)]
pub enum TimelineEvent {
    /// The action itself — always the first event, at offset 0.
    Action {
        #[serde(flatten)]
        base: TimelineEventBase,
    },
    /// Registered elements appeared between two polls.
    ElementsAppeared {
        #[serde(flatten)]
        base: TimelineEventBase,
        /// Element IDs involved.
        element_ids: Vec<String>,
        /// Number of elements affected.
        count: u64,
    },
    /// Registered elements disappeared between two polls.
    ElementsDisappeared {
        #[serde(flatten)]
        base: TimelineEventBase,
        /// Element IDs involved.
        element_ids: Vec<String>,
        /// Number of elements affected.
        count: u64,
    },
    /// Registered elements were modified between two polls.
    ElementsModified {
        #[serde(flatten)]
        base: TimelineEventBase,
        /// Element IDs involved.
        element_ids: Vec<String>,
        /// Number of elements affected.
        count: u64,
    },
    /// The page URL changed.
    PageChanged {
        #[serde(flatten)]
        base: TimelineEventBase,
        /// The new URL, or `null` when the diff did not carry one.
        url: Option<String>,
    },
    /// The UI was stable for `settleMinStable` (and the idle detector, when
    /// present, reported idle).
    Settled {
        #[serde(flatten)]
        base: TimelineEventBase,
    },
    /// One `layout-shift` PerformanceObserver entry.
    LayoutShift {
        #[serde(flatten)]
        base: TimelineEventBase,
        /// The shift score (Layout Instability API `value`).
        value: f64,
        /// The browser's `hadRecentInput` flag for this shift.
        had_recent_input: bool,
        /// `ElementFingerprintData.hash` of each shifted element still in the
        /// document.
        shifted_fingerprints: Vec<String>,
        /// Shift sources whose node was gone (or not an element) when the
        /// entry was read.
        unattributed_source_count: u64,
        /// How many shifted elements `shiftedFingerprints` named. NOT sent by
        /// the SDK: [`ChangeTimeline::for_ledger`] sets it when it empties
        /// `shiftedFingerprints`, so a stored timeline keeps the count and
        /// not the hashes. Absent on the wire from the SDK.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        shifted_element_count: Option<u64>,
    },
    /// A CSS animation or transition started.
    AnimationStarted {
        #[serde(flatten)]
        base: TimelineEventBase,
        /// The animation.
        animation: DetectedAnimation,
    },
    /// A CSS animation or transition ended or was cancelled.
    AnimationEnded {
        #[serde(flatten)]
        base: TimelineEventBase,
        /// The animation.
        animation: DetectedAnimation,
        /// Finished or cancelled.
        end_reason: AnimationEndReason,
    },
    /// One `longtask` PerformanceObserver entry.
    LongTask {
        #[serde(flatten)]
        base: TimelineEventBase,
        /// Task duration (ms).
        duration_ms: f64,
    },
    /// A tracked network request started at or after action start.
    NetworkRequestStarted {
        #[serde(flatten)]
        base: TimelineEventBase,
        /// The tracker's request id, or `null` when the source supplies none.
        request_id: Option<String>,
        /// HTTP method.
        method: String,
        /// Origin + pathname — never the query or fragment.
        url: String,
        /// The request URL carried a query string (not reported).
        has_query: bool,
    },
    /// A tracked network request (started at or after action start) finished.
    NetworkRequestFinished {
        #[serde(flatten)]
        base: TimelineEventBase,
        /// The tracker's request id, or `null` when the source supplies none.
        request_id: Option<String>,
        /// HTTP method.
        method: String,
        /// Origin + pathname — never the query or fragment.
        url: String,
        /// The request URL carried a query string (not reported).
        has_query: bool,
        /// HTTP status, or `null` when no response arrived.
        status_code: Option<u16>,
        /// Request duration as the tracker measured it, or `null` without a
        /// response.
        duration_ms: Option<f64>,
        /// The request threw (no HTTP response). An HTTP error status is NOT
        /// `errored` — read `statusCode`.
        errored: bool,
    },
    /// A declared UI state became active.
    StateEntered {
        #[serde(flatten)]
        base: TimelineEventBase,
        /// The declared state id.
        state_id: String,
    },
    /// A declared UI state became inactive.
    StateExited {
        #[serde(flatten)]
        base: TimelineEventBase,
        /// The declared state id.
        state_id: String,
    },
}

impl TimelineEvent {
    /// The fields every event carries.
    pub fn base(&self) -> &TimelineEventBase {
        match self {
            Self::Action { base }
            | Self::ElementsAppeared { base, .. }
            | Self::ElementsDisappeared { base, .. }
            | Self::ElementsModified { base, .. }
            | Self::PageChanged { base, .. }
            | Self::Settled { base }
            | Self::LayoutShift { base, .. }
            | Self::AnimationStarted { base, .. }
            | Self::AnimationEnded { base, .. }
            | Self::LongTask { base, .. }
            | Self::NetworkRequestStarted { base, .. }
            | Self::NetworkRequestFinished { base, .. }
            | Self::StateEntered { base, .. }
            | Self::StateExited { base, .. } => base,
        }
    }

    /// Offset from action start (ms).
    pub fn offset_ms(&self) -> f64 {
        self.base().offset_ms
    }

    /// The wire `type` tag.
    pub fn type_tag(&self) -> &'static str {
        match self {
            Self::Action { .. } => "action",
            Self::ElementsAppeared { .. } => "elements-appeared",
            Self::ElementsDisappeared { .. } => "elements-disappeared",
            Self::ElementsModified { .. } => "elements-modified",
            Self::PageChanged { .. } => "page-changed",
            Self::Settled { .. } => "settled",
            Self::LayoutShift { .. } => "layout-shift",
            Self::AnimationStarted { .. } => "animation-started",
            Self::AnimationEnded { .. } => "animation-ended",
            Self::LongTask { .. } => "long-task",
            Self::NetworkRequestStarted { .. } => "network-request-started",
            Self::NetworkRequestFinished { .. } => "network-request-finished",
            Self::StateEntered { .. } => "state-entered",
            Self::StateExited { .. } => "state-exited",
        }
    }
}

/// How an expected state fared inside the caller's `withinMs` window. Exactly
/// one key on the wire.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub enum ExpectedStateOutcome {
    /// The state became active at this offset (<= `withinMs`).
    EnteredAtMs(f64),
    /// It did not become active within the window (the window is echoed, so
    /// a negative carries its own bound).
    NotEnteredWithinMs(f64),
    /// It was already active when the action ran and stayed active for the
    /// whole window, so it never "entered".
    ActiveThroughoutWithinMs(f64),
}

/// One `expectStates` entry and its answer.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct ExpectedStateResult {
    /// The declared state id the caller asked about.
    pub state_id: String,
    /// The answer — `unknown{input_missing}` for an undeclared state or no
    /// state registry.
    pub outcome: Observation<ExpectedStateOutcome>,
}

/// Values derived from a [`ChangeTimeline`]. EVERY field is an
/// [`Observation`]: `measured` with the value, `absent` when the producer
/// looked over the whole window and the thing did not occur, or `unknown`
/// with a closed `UnknownCode` when it could not look. No field is a verdict
/// and no threshold is applied.
///
/// Each field's `provenance.source` carries `window` and `observedSources`;
/// a field derived from an event type the recorder capped carries
/// `provenance.coverage.unmeasured[{ dimension: "timeline-events:<type>", … }]`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct TimelineSummary {
    /// Offset of the first RESPONSE event (elements-*, page-changed,
    /// layout-shift, animation-*, state-*). `unknown{no_event_before_timeout}`
    /// when nothing responded in the window.
    pub first_response_ms: Observation<f64>,
    /// Offset of the `settled` event. `unknown{no_event_before_timeout}` when
    /// the UI did not settle within `settleTimeout`.
    pub settle_ms: Observation<f64>,
    /// Sum of `layout-shift` values in the window.
    /// `unknown{performance_observer_unsupported}` where the entry type is
    /// missing.
    pub layout_shift_total: Observation<f64>,
    /// Number of `layout-shift` events. Same unknown rule.
    pub layout_shift_count: Observation<u64>,
    /// Offset of the LAST layout shift minus `firstResponseMs`. `absent` when
    /// no shift occurred.
    pub layout_shift_after_first_response_ms: Observation<f64>,
    /// Animations/transitions running when recording ended. Degraded
    /// (`coverage.unmeasured`) where `document.getAnimations` is missing.
    pub animations_unsettled_at_end: Observation<Vec<DetectedAnimation>>,
    /// Sum of `long-task` durations (ms).
    /// `unknown{performance_observer_unsupported}` where `longtask` is
    /// missing.
    pub long_task_total_ms: Observation<f64>,
    /// One entry per requested `expectStates` id, in request order; `[]` when
    /// none were requested.
    pub expected_states: Vec<ExpectedStateResult>,
}

impl TimelineSummary {
    /// Every scalar summary Observation's coverage-unmeasured dimension names,
    /// so a reader can ask "did the recorder drop events of type X?" without
    /// knowing which field derives from X.
    pub fn unmeasured_dimensions(&self) -> impl Iterator<Item = &str> {
        let provs = [
            self.first_response_ms.provenance(),
            self.settle_ms.provenance(),
            self.layout_shift_total.provenance(),
            self.layout_shift_count.provenance(),
            self.layout_shift_after_first_response_ms.provenance(),
            self.animations_unsettled_at_end.provenance(),
            self.long_task_total_ms.provenance(),
        ];
        provs
            .into_iter()
            .chain(self.expected_states.iter().map(|e| e.outcome.provenance()))
            .flat_map(|p| p.coverage.unmeasured.iter().map(|u| u.dimension.as_str()))
    }
}

/// The interaction timeline: action → every observed event → settled, each
/// with its offset from the action, plus a derived [`TimelineSummary`].
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct ChangeTimeline {
    /// Timestamped events, ordered by `offsetMs` (ties keep arrival order).
    pub events: Vec<TimelineEvent>,
    /// How long the timeline was recorded, from action start (ms): the settle
    /// window, extended by any `withinMs` window. Not the settle time.
    pub recorded_ms: f64,
    /// Whether the UI reached a stable state within `settleTimeout`.
    pub settled: bool,
    /// The sources this recording could observe.
    pub observed_sources: Vec<TimelineSource>,
    /// Values derived from `events`.
    pub summary: TimelineSummary,
}

/// The `provenance.source` keys the SDK's timeline summary writes, every one
/// structural (a window, a source list, an event-type tag, a support probe,
/// offsets) — the only ones [`ChangeTimeline::for_ledger`] keeps.
pub const LEDGER_SOURCE_KEYS: [&str; 8] = [
    "eventType",
    "firstResponseAtMs",
    "lastShiftAtMs",
    "observedSources",
    "supportProbe",
    "supportedEntryTypes",
    "window",
    "withinMs",
];

fn retain_ledger_source<T>(o: &mut Observation<T>) {
    if let Some(source) = o.source_mut() {
        source.retain(|k, _| LEDGER_SOURCE_KEYS.contains(&k.as_str()));
    }
}

/// `scheme://host[:port]` of a URL, or `None` when it has no `scheme://`
/// authority — the network events keep their origin, never their path.
fn url_origin(url: &str) -> Option<String> {
    let (scheme, rest) = url.split_once("://")?;
    let authority = rest.split(['/', '?', '#']).next().unwrap_or("");
    // A userinfo part (`user:pass@host`) is credentials, never kept.
    let host = authority.rsplit_once('@').map_or(authority, |(_, h)| h);
    if scheme.is_empty() || host.is_empty() {
        return None;
    }
    Some(format!("{scheme}://{host}"))
}

impl ChangeTimeline {
    /// Whether this recording looked at `source`.
    pub fn observed(&self, source: TimelineSource) -> bool {
        self.observed_sources.contains(&source)
    }

    /// The projection a consumer STORES — the journey ledger's `timeline`
    /// column holds exactly this, and nothing else. It keeps every offset,
    /// count, score, declared state id, animation name and every summary
    /// [`Observation`] (status, value, coverage, unknown code and detail), so
    /// every timeline assertion and [`crate::edge_timing`] answer the same
    /// over it as over the original. It drops what can carry user input:
    ///
    /// - every `elements-*` event's `elementIds` becomes `[]` (its `count`
    ///   stays): an SDK auto-registered id is a slug of the element's text;
    /// - every `layout-shift` `shiftedFingerprints` becomes `[]`, with the
    ///   number it held in `shiftedElementCount`, and every animation's
    ///   `targetElement` (events and `summary.animationsUnsettledAtEnd`)
    ///   becomes absent: an element fingerprint hashes the element's
    ///   accessible name, which is brute-forceable;
    /// - every event's `summary` prose becomes its `type` tag;
    /// - `page-changed` `url` becomes `null` (a URL path or query can embed
    ///   user input; the journey node already names the page);
    /// - `network-request-*` `url` becomes its origin (`scheme://host`), or
    ///   `""` when it has none — never a pathname;
    /// - each summary field's `provenance.source` keeps only
    ///   [`LEDGER_SOURCE_KEYS`].
    ///
    /// Idempotent. Serializing the result through this typed mirror carries
    /// no key it does not model, so a stored row has no open value slot.
    pub fn for_ledger(&self) -> ChangeTimeline {
        let mut t = self.clone();
        for ev in &mut t.events {
            let tag = ev.type_tag();
            match ev {
                TimelineEvent::PageChanged { url, .. } => *url = None,
                TimelineEvent::NetworkRequestStarted { url, .. }
                | TimelineEvent::NetworkRequestFinished { url, .. } => {
                    *url = url_origin(url).unwrap_or_default();
                }
                TimelineEvent::ElementsAppeared { element_ids, .. }
                | TimelineEvent::ElementsDisappeared { element_ids, .. }
                | TimelineEvent::ElementsModified { element_ids, .. } => element_ids.clear(),
                TimelineEvent::LayoutShift {
                    shifted_fingerprints,
                    shifted_element_count,
                    ..
                } => {
                    shifted_element_count.get_or_insert(shifted_fingerprints.len() as u64);
                    shifted_fingerprints.clear();
                }
                TimelineEvent::AnimationStarted { animation, .. }
                | TimelineEvent::AnimationEnded { animation, .. } => {
                    animation.target_element = None;
                }
                _ => {}
            }
            let base = match ev {
                TimelineEvent::Action { base }
                | TimelineEvent::ElementsAppeared { base, .. }
                | TimelineEvent::ElementsDisappeared { base, .. }
                | TimelineEvent::ElementsModified { base, .. }
                | TimelineEvent::PageChanged { base, .. }
                | TimelineEvent::Settled { base }
                | TimelineEvent::LayoutShift { base, .. }
                | TimelineEvent::AnimationStarted { base, .. }
                | TimelineEvent::AnimationEnded { base, .. }
                | TimelineEvent::LongTask { base, .. }
                | TimelineEvent::NetworkRequestStarted { base, .. }
                | TimelineEvent::NetworkRequestFinished { base, .. }
                | TimelineEvent::StateEntered { base, .. }
                | TimelineEvent::StateExited { base, .. } => base,
            };
            base.summary = tag.to_string();
        }
        let s = &mut t.summary;
        retain_ledger_source(&mut s.first_response_ms);
        retain_ledger_source(&mut s.settle_ms);
        retain_ledger_source(&mut s.layout_shift_total);
        retain_ledger_source(&mut s.layout_shift_count);
        retain_ledger_source(&mut s.layout_shift_after_first_response_ms);
        retain_ledger_source(&mut s.animations_unsettled_at_end);
        s.animations_unsettled_at_end = s.animations_unsettled_at_end.clone().map(|list| {
            list.into_iter()
                .map(|a| DetectedAnimation {
                    target_element: None,
                    ..a
                })
                .collect()
        });
        retain_ledger_source(&mut s.long_task_total_ms);
        for e in &mut s.expected_states {
            retain_ledger_source(&mut e.outcome);
        }
        t
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn event_tags_and_fields_match_the_sdk_spelling() {
        let ev: TimelineEvent = serde_json::from_value(json!({
            "offsetMs": 85, "type": "state-entered", "summary": "State entered: x", "stateId": "x"
        }))
        .unwrap();
        assert_eq!(ev.type_tag(), "state-entered");
        assert_eq!(ev.offset_ms(), 85.0);
        let v = serde_json::to_value(&ev).unwrap();
        assert_eq!(v["stateId"], json!("x"));
        assert_eq!(v["type"], json!("state-entered"));

        let ended: TimelineEvent = serde_json::from_value(json!({
            "offsetMs": 30, "type": "animation-ended", "summary": "s",
            "animation": { "name": "slide-in", "durationMs": 200, "isInfinite": false },
            "endReason": "cancelled"
        }))
        .unwrap();
        let v = serde_json::to_value(&ended).unwrap();
        assert_eq!(v["endReason"], json!("cancelled"));
        assert!(v["animation"].get("targetElement").is_none());
    }

    #[test]
    fn expected_state_outcome_is_a_one_key_object() {
        for (o, key) in [
            (ExpectedStateOutcome::EnteredAtMs(85.0), "enteredAtMs"),
            (
                ExpectedStateOutcome::NotEnteredWithinMs(2000.0),
                "notEnteredWithinMs",
            ),
            (
                ExpectedStateOutcome::ActiveThroughoutWithinMs(80.0),
                "activeThroughoutWithinMs",
            ),
        ] {
            let v = serde_json::to_value(o).unwrap();
            let obj = v.as_object().unwrap();
            assert_eq!(obj.len(), 1);
            assert!(obj.contains_key(key), "{v}");
            assert_eq!(
                serde_json::from_value::<ExpectedStateOutcome>(v).unwrap(),
                o
            );
        }
    }

    #[test]
    fn url_origin_keeps_only_scheme_and_host() {
        assert_eq!(
            url_origin("http://localhost:3000/api/users/a@b.c?x=1").as_deref(),
            Some("http://localhost:3000")
        );
        assert_eq!(
            url_origin("https://u:secret@api.example.com/x").as_deref(),
            Some("https://api.example.com")
        );
        assert_eq!(url_origin("/relative/path"), None);
        assert_eq!(url_origin("https://"), None);
    }

    #[test]
    fn an_unknown_event_type_is_refused() {
        let r = serde_json::from_value::<TimelineEvent>(json!({
            "offsetMs": 0, "type": "something-new", "summary": "s"
        }));
        assert!(r.is_err(), "the event union is closed");
    }
}
