//! Wire gate: SDK-captured interaction timelines round-trip through
//! [`ChangeTimeline`] and the timeline assertions answer over them.
//!
//! `tests/fixtures/ui_bridge_timeline/<name>.json` is SDK output, never
//! hand-typed JSON — a map of scenario name → `ActionDiffResult.timeline`,
//! captured by `capture/timeline-fixture.capture.test.ts`; `<name>.provenance.json`
//! beside it records the SDK commit, the capture command and the fixture's
//! sha256. The directory is enumerated, so a fixture captured from a later SDK
//! is gated without editing this file. Add one when the SDK's timeline wire
//! changes; keep the old ones while deployed apps still emit that wire.
//!
//! The round-trip is the drift detector: [`ChangeTimeline`] ignores unknown
//! keys at parse (so a newer SDK cannot make an older runner drop a timeline),
//! which means a field the mirror lacks would vanish silently on
//! re-serialization. Comparing the re-serialized value with the captured one
//! turns that into a red test. Numbers are compared numerically, because the
//! mirror reads every millisecond as `f64` (`85` and `85.0` are one value).

use std::collections::BTreeMap;
use std::path::PathBuf;

use qontinui_vision_core::{
    edge_timing, evaluate_assertion, Assertion, AssertionOutcome, ChangeTimeline, EdgeTimingSample,
    EvalContext, ExpectedStateOutcome, TimelineEvent, UnknownCode, LEDGER_SOURCE_KEYS,
};
use serde_json::Value;
use sha2::{Digest, Sha256};

const SIDECAR_SUFFIX: &str = ".provenance.json";

fn fixture_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/ui_bridge_timeline")
}

/// Every fixture (name → bytes), each checked against its sidecar's sha256.
fn fixtures() -> BTreeMap<String, Vec<u8>> {
    let mut out = BTreeMap::new();
    for entry in std::fs::read_dir(fixture_dir()).expect("fixture dir") {
        let path = entry.unwrap().path();
        let name = path.file_name().unwrap().to_string_lossy().to_string();
        if !name.ends_with(".json") || name.ends_with(SIDECAR_SUFFIX) {
            continue;
        }
        let bytes = std::fs::read(&path).unwrap();
        let sidecar = path.with_file_name(format!(
            "{}{SIDECAR_SUFFIX}",
            name.trim_end_matches(".json")
        ));
        let prov: Value = serde_json::from_slice(
            &std::fs::read(&sidecar)
                .unwrap_or_else(|e| panic!("{name} has no sidecar {}: {e}", sidecar.display())),
        )
        .unwrap();
        for key in [
            "fixture",
            "sha256",
            "sdk_commit",
            "capture_command",
            "captured_at",
        ] {
            assert!(
                prov[key].as_str().is_some_and(|s| !s.is_empty()),
                "{} lacks {key}",
                sidecar.display()
            );
        }
        assert_eq!(prov["fixture"], Value::from(name.clone()));
        let digest = format!("{:x}", Sha256::digest(&bytes));
        assert_eq!(
            prov["sha256"],
            Value::from(digest),
            "{name} changed after capture"
        );
        out.insert(name, bytes);
    }
    assert!(
        !out.is_empty(),
        "the timeline fixture directory must never be emptied"
    );
    out
}

/// `value` with every number replaced by its f64, so `85` == `85.0`.
fn numeric(value: &Value) -> Value {
    match value {
        Value::Number(n) => Value::from(n.as_f64().expect("finite")),
        Value::Array(a) => Value::Array(a.iter().map(numeric).collect()),
        Value::Object(o) => Value::Object(o.iter().map(|(k, v)| (k.clone(), numeric(v))).collect()),
        other => other.clone(),
    }
}

/// Every scenario of every fixture, parsed as the runner parses it.
fn timelines() -> Vec<(String, Value, ChangeTimeline)> {
    let mut out = Vec::new();
    for (name, bytes) in fixtures() {
        let scenarios: BTreeMap<String, Value> = serde_json::from_slice(&bytes).unwrap();
        for (scenario, raw) in scenarios {
            let t: ChangeTimeline = serde_json::from_value(raw.clone())
                .unwrap_or_else(|e| panic!("{name}:{scenario} does not parse: {e}"));
            out.push((format!("{name}:{scenario}"), raw, t));
        }
    }
    out
}

fn scenario(name: &str) -> ChangeTimeline {
    timelines()
        .into_iter()
        .find(|(id, _, _)| id.ends_with(&format!(":{name}")))
        .unwrap_or_else(|| panic!("no scenario {name}"))
        .2
}

#[test]
fn every_captured_timeline_round_trips_without_losing_a_key() {
    let all = timelines();
    assert!(
        all.len() >= 4,
        "expected the four captured scenarios, got {}",
        all.len()
    );
    for (id, raw, t) in all {
        let back = serde_json::to_value(&t).unwrap();
        assert_eq!(
            numeric(&back),
            numeric(&raw),
            "{id}: the Rust mirror does not reproduce the SDK's timeline — a field was \
             dropped, renamed or reshaped"
        );
    }
}

#[test]
fn the_captures_cover_every_event_type_and_outcome_arm() {
    let mut types = std::collections::BTreeSet::new();
    let mut arms = std::collections::BTreeSet::new();
    for (_, _, t) in timelines() {
        types.extend(t.events.iter().map(TimelineEvent::type_tag));
        for e in &t.summary.expected_states {
            arms.insert(match e.outcome.value() {
                Some(ExpectedStateOutcome::EnteredAtMs(_)) => "entered",
                Some(ExpectedStateOutcome::NotEnteredWithinMs(_)) => "not_entered",
                Some(ExpectedStateOutcome::ActiveThroughoutWithinMs(_)) => "active_throughout",
                None => "not_measured",
            });
        }
    }
    let all_types = [
        "action",
        "elements-appeared",
        "elements-disappeared",
        "elements-modified",
        "page-changed",
        "settled",
        "layout-shift",
        "animation-started",
        "animation-ended",
        "long-task",
        "network-request-started",
        "network-request-finished",
        "state-entered",
        "state-exited",
    ];
    for ty in all_types {
        assert!(
            types.contains(ty),
            "no captured timeline carries a `{ty}` event"
        );
    }
    assert_eq!(arms.len(), 4, "outcome arms seen: {arms:?}");
}

fn eval(t: &ChangeTimeline, a: Assertion) -> qontinui_vision_core::AssertionResult {
    evaluate_assertion(
        &a,
        &EvalContext {
            timeline: Some(t),
            ..Default::default()
        },
    )
}

#[test]
fn assertions_answer_over_the_sdk_shape() {
    let rich = scenario("rich");
    // `pulse` started and never ended.
    let res = eval(&rich, Assertion::AnimationSettled { region: None });
    assert_eq!(res.outcome, AssertionOutcome::Failed, "{res:?}");
    assert!(res.detail.unwrap().contains("pulse"));

    let state = |s: &str, ms| Assertion::StateEnteredWithin {
        state: s.into(),
        ms,
    };
    assert_eq!(
        eval(&rich, state("saved-toast", 400)).outcome,
        AssertionOutcome::Passed
    );
    assert_eq!(
        eval(&rich, state("error-banner", 400)).outcome,
        AssertionOutcome::Failed
    );
    assert_eq!(
        eval(&rich, state("editor", 400)).outcome,
        AssertionOutcome::Failed
    );
    let res = eval(&rich, state("undeclared-state", 400));
    assert_eq!(res.outcome, AssertionOutcome::Unknown);
    assert_eq!(res.code, Some(UnknownCode::InputMissing));
    let settle = rich
        .summary
        .settle_ms
        .value()
        .copied()
        .expect("rich settled");
    assert_eq!(
        eval(&rich, Assertion::SettledWithin { ms: settle as u64 }).outcome,
        AssertionOutcome::Passed
    );

    // Never settled within a 120ms settle timeout: a bound inside the
    // recording is a measured negative, a bound past it is unknown.
    let never = scenario("neverSettles");
    assert_eq!(
        eval(&never, Assertion::SettledWithin { ms: 100 }).outcome,
        AssertionOutcome::Failed
    );
    let res = eval(&never, Assertion::SettledWithin { ms: 5000 });
    assert_eq!(res.outcome, AssertionOutcome::Unknown);
    assert_eq!(res.code, Some(UnknownCode::NoEventBeforeTimeout));

    // jsdom has no document.getAnimations, so "nothing seen running" is a
    // degraded reading, not a pass.
    let res = eval(
        &scenario("staticPage"),
        Assertion::AnimationSettled { region: None },
    );
    assert_eq!(res.outcome, AssertionOutcome::Unknown, "{res:?}");
}

/// The journey ledger stores `for_ledger()`. Over every SDK-captured shape it
/// must answer every assertion and every timing metric exactly as the
/// original does, while carrying none of the SDK prose or URL paths.
#[test]
fn the_ledger_projection_keeps_every_answer_and_drops_user_text() {
    let assertions = || {
        [
            Assertion::AnimationSettled { region: None },
            Assertion::SettledWithin { ms: 100 },
            Assertion::SettledWithin { ms: 5000 },
            Assertion::StateEnteredWithin {
                state: "saved-toast".into(),
                ms: 400,
            },
            Assertion::StateEnteredWithin {
                state: "error-banner".into(),
                ms: 400,
            },
            Assertion::StateEnteredWithin {
                state: "undeclared-state".into(),
                ms: 400,
            },
        ]
    };
    for (name, _, original) in timelines() {
        let stored = original.for_ledger();
        assert_eq!(
            stored.for_ledger(),
            stored,
            "{name}: for_ledger is idempotent"
        );

        // Nothing user-derived survives.
        let text = serde_json::to_string(&stored).unwrap();
        for leak in ["/fixture/", "/api/", "?", "URL changed", "answered"] {
            assert!(
                !text.contains(leak),
                "{name}: stored timeline carries {leak:?}: {text}"
            );
        }
        for ev in &stored.events {
            assert_eq!(ev.base().summary, ev.type_tag(), "{name}");
            match ev {
                TimelineEvent::PageChanged { url, .. } => assert_eq!(url, &None),
                TimelineEvent::NetworkRequestStarted { url, .. }
                | TimelineEvent::NetworkRequestFinished { url, .. } => {
                    assert_eq!(url.as_deref(), Some("http://localhost:3000"), "{name}")
                }
                _ => {}
            }
        }
        let v = serde_json::to_value(&stored.summary).unwrap();
        for (field, o) in v.as_object().unwrap() {
            if let Some(src) = o.get("provenance").and_then(|p| p["source"].as_object()) {
                for k in src.keys() {
                    assert!(
                        LEDGER_SOURCE_KEYS.contains(&k.as_str()),
                        "{name}.{field}: {k}"
                    );
                }
            }
        }

        // Every answer is unchanged.
        assert_eq!(stored.events.len(), original.events.len());
        for a in assertions() {
            let (x, y) = (eval(&original, a.clone()), eval(&stored, a.clone()));
            assert_eq!((x.outcome, x.code), (y.outcome, y.code), "{name}: {a:?}");
        }
        let sample = |t: &ChangeTimeline| EdgeTimingSample {
            observation_id: "e".into(),
            observed_at: chrono::DateTime::UNIX_EPOCH,
            app_version: None,
            timeline: Some(t.clone()),
        };
        let (x, y) = (
            edge_timing(&[sample(&original)]),
            edge_timing(&[sample(&stored)]),
        );
        assert_eq!(x.first_response_ms, y.first_response_ms, "{name}");
        assert_eq!(x.settle_ms, y.settle_ms, "{name}");
        assert_eq!(x.layout_shift_total, y.layout_shift_total, "{name}");
        assert_eq!(x.layout_shift_count, y.layout_shift_count, "{name}");
        assert_eq!(x.long_task_total_ms, y.long_task_total_ms, "{name}");
        assert_eq!(
            x.layout_shift_after_first_response_ms, y.layout_shift_after_first_response_ms,
            "{name}"
        );
    }
    // The fixture really did carry what is dropped (the probe is not vacuous).
    let rich = serde_json::to_string(&scenario("rich")).unwrap();
    assert!(rich.contains("/api/save") && rich.contains("/fixture/saved"));
}

/// Every element id, shifted-element fingerprint and animation target the
/// SDK captured, as strings.
fn element_identities(t: &ChangeTimeline) -> Vec<String> {
    let mut out = Vec::new();
    for ev in &t.events {
        match ev {
            TimelineEvent::ElementsAppeared { element_ids, .. }
            | TimelineEvent::ElementsDisappeared { element_ids, .. }
            | TimelineEvent::ElementsModified { element_ids, .. } => {
                out.extend(element_ids.iter().cloned())
            }
            TimelineEvent::LayoutShift {
                shifted_fingerprints,
                ..
            } => out.extend(shifted_fingerprints.iter().cloned()),
            TimelineEvent::AnimationStarted { animation, .. }
            | TimelineEvent::AnimationEnded { animation, .. } => {
                out.extend(animation.target_element.iter().cloned())
            }
            _ => {}
        }
    }
    if let Some(list) = t.summary.animations_unsettled_at_end.value() {
        out.extend(list.iter().filter_map(|a| a.target_element.clone()));
    }
    out.retain(|s| !s.is_empty());
    out
}

/// An SDK auto-registered element id is a slug of the element's text
/// (`react/useAutoRegister.ts`), and an element fingerprint hashes its
/// accessible name (`core/element-fingerprint.ts`) — both can carry what a
/// user typed or what the page said about them. The stored projection keeps
/// the counts and none of the identities.
#[test]
fn the_ledger_projection_keeps_element_counts_and_drops_element_identities() {
    // Every identity in the SDK captures is gone from the stored form.
    let mut seen = 0;
    for (name, _, original) in timelines() {
        let ids = element_identities(&original);
        seen += ids.len();
        let stored = original.for_ledger();
        assert!(element_identities(&stored).is_empty(), "{name}");
        let text = serde_json::to_string(&stored).unwrap();
        for id in &ids {
            // As a JSON string value: a bare substring would also match a
            // declared state id such as `saved-toast` for the id `save`.
            let quoted = serde_json::to_string(id).unwrap();
            assert!(
                !text.contains(&quoted),
                "{name}: stored timeline carries {quoted}"
            );
        }
        // Counts survive.
        for (o, s) in original.events.iter().zip(&stored.events) {
            match (o, s) {
                (
                    TimelineEvent::ElementsAppeared { count: a, .. },
                    TimelineEvent::ElementsAppeared { count: b, .. },
                )
                | (
                    TimelineEvent::ElementsDisappeared { count: a, .. },
                    TimelineEvent::ElementsDisappeared { count: b, .. },
                )
                | (
                    TimelineEvent::ElementsModified { count: a, .. },
                    TimelineEvent::ElementsModified { count: b, .. },
                ) => assert_eq!(a, b, "{name}"),
                (
                    TimelineEvent::LayoutShift {
                        shifted_fingerprints,
                        ..
                    },
                    TimelineEvent::LayoutShift {
                        shifted_element_count,
                        ..
                    },
                ) => assert_eq!(
                    *shifted_element_count,
                    Some(shifted_fingerprints.len() as u64),
                    "{name}"
                ),
                _ => {}
            }
        }
    }
    assert!(
        seen > 0,
        "the captures carry element identities (probe is not vacuous)"
    );

    // A fixture element registered under a semantic, text-derived id, as the
    // SDK's auto-registration names one, plus that element's fingerprint as
    // a shift source and an animation target.
    let mut v: Value = serde_json::to_value(scenario("rich")).unwrap();
    let events = v["events"].as_array_mut().unwrap();
    events.push(serde_json::json!({
        "type": "elements-appeared", "offsetMs": 900, "summary": "1 element(s) appeared",
        "elementIds": ["button-send-invoice-to-jane-doe-example-com"], "count": 1
    }));
    events.push(serde_json::json!({
        "type": "layout-shift", "offsetMs": 910, "summary": "s", "value": 0.05,
        "hadRecentInput": false, "shiftedFingerprints": ["fp-jane-doe-invoice"],
        "unattributedSourceCount": 0
    }));
    events.push(serde_json::json!({
        "type": "animation-started", "offsetMs": 920, "summary": "s",
        "animation": { "name": "toast-in", "targetElement": "fp-jane-doe-toast", "isInfinite": false }
    }));
    let t: ChangeTimeline = serde_json::from_value(v).unwrap();
    assert!(serde_json::to_string(&t).unwrap().contains("jane-doe"));
    let stored = t.for_ledger();
    let text = serde_json::to_string(&stored).unwrap();
    assert!(!text.contains("jane"), "{text}");
    assert_eq!(stored.for_ledger(), stored, "idempotent");
    match &stored.events[stored.events.len() - 2] {
        TimelineEvent::LayoutShift {
            shifted_element_count,
            ..
        } => assert_eq!(*shifted_element_count, Some(1)),
        other => panic!("{other:?}"),
    }
    // And the answers over it are the original's.
    for a in [
        Assertion::AnimationSettled { region: None },
        Assertion::SettledWithin { ms: 5000 },
        Assertion::StateEnteredWithin {
            state: "saved-toast".into(),
            ms: 400,
        },
    ] {
        let (x, y) = (eval(&t, a.clone()), eval(&stored, a.clone()));
        assert_eq!((x.outcome, x.code), (y.outcome, y.code), "{a:?}");
    }
}
