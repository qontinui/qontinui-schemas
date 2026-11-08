//! Wire contract of the runner's `GET /terminal-layout` snapshot
//! ([`TerminalLayout`] and the three types inside it).
//!
//! qontinui-runner serializes these types; qontinui-web validates the relayed
//! body with the generated Python model (plan
//! `2026-11-08-save-per-tab-runner-session-lists-on-sessions-page-kept-14-days`,
//! D3 and D9). These tests pin the two wire rules the types document:
//!
//! - **null means unknown or not applicable, never a stand-in.** Every
//!   optional field serializes as an explicit `null` (the key is never left
//!   out), and a missing key reads as `None`. `order` and `zoneIndex` are
//!   unsigned, so the runner's `-1` sentinel is refused rather than carried.
//!   The plain bools are required, so a missing one is an error, never a
//!   silent `false`.
//! - **Forward compatibility.** No `deny_unknown_fields`: a key added by a
//!   newer runner deserializes and is ignored, both in serde here and in the
//!   JSON Schema that the Python and TypeScript bindings are generated from.

use std::collections::BTreeSet;

use qontinui_types::terminal::{
    TerminalLayout, TerminalLayoutPage, TerminalLayoutRunner, TerminalLayoutSession,
};
use schemars::schema_for;
use serde_json::{json, Value};

/// The optional (tri-state) keys of a session row.
const SESSION_TRI_STATE_KEYS: [&str; 9] = [
    "title",
    "spawnName",
    "claudeSessionId",
    "accountLabel",
    "sessionName",
    "zoneIndex",
    "finished",
    "finishedAt",
    "finishReason",
];

/// A full body: a registered tab holding one known, finished session and one
/// terminal whose session fields are all unknown (and whose title was
/// withheld), then a tab the webview never registered.
fn full_example() -> Value {
    json!({
        "runner": {
            "instanceName": "primary",
            "port": 9876,
            "buildId": "20261108.1",
            "deviceId": "0b6f3c1e-8f7a-4c55-9d1e-2a4b6c8d0e1f"
        },
        "labelsRegistered": true,
        "capturedAt": "2026-11-08T09:30:00Z",
        "pages": [
            {
                "pageId": "page-a",
                "label": "Coord work",
                "order": 0,
                "sessions": [
                    {
                        "terminalId": "term-1",
                        "title": "claude: layout wire type",
                        "titleWithheld": false,
                        "isAlive": true,
                        "spawnName": "plan-phase-1",
                        "claudeSessionId": "61a189b8-6516-459d-a981-9659f36b6743",
                        "accountLabel": "susanne",
                        "sessionName": "layout wire type",
                        "zoneIndex": 2,
                        "finished": true,
                        "finishedAt": "2026-11-08T09:00:00Z",
                        "finishReason": "unattended"
                    },
                    {
                        "terminalId": "term-2",
                        "title": null,
                        "titleWithheld": true,
                        "isAlive": false,
                        "spawnName": null,
                        "claudeSessionId": null,
                        "accountLabel": null,
                        "sessionName": null,
                        "zoneIndex": null,
                        "finished": null,
                        "finishedAt": null,
                        "finishReason": null
                    }
                ]
            },
            {
                "pageId": "page-b",
                "label": null,
                "order": null,
                "sessions": []
            }
        ]
    })
}

/// [`full_example`] as the Rust value the runner would build.
fn full_layout() -> TerminalLayout {
    TerminalLayout {
        runner: TerminalLayoutRunner {
            instance_name: Some("primary".to_string()),
            port: Some(9876),
            build_id: Some("20261108.1".to_string()),
            device_id: Some("0b6f3c1e-8f7a-4c55-9d1e-2a4b6c8d0e1f".to_string()),
        },
        labels_registered: true,
        captured_at: "2026-11-08T09:30:00Z".to_string(),
        pages: vec![
            TerminalLayoutPage {
                page_id: "page-a".to_string(),
                label: Some("Coord work".to_string()),
                order: Some(0),
                sessions: vec![
                    TerminalLayoutSession {
                        terminal_id: "term-1".to_string(),
                        title: Some("claude: layout wire type".to_string()),
                        title_withheld: false,
                        is_alive: true,
                        spawn_name: Some("plan-phase-1".to_string()),
                        claude_session_id: Some("61a189b8-6516-459d-a981-9659f36b6743".to_string()),
                        account_label: Some("susanne".to_string()),
                        session_name: Some("layout wire type".to_string()),
                        zone_index: Some(2),
                        finished: Some(true),
                        finished_at: Some("2026-11-08T09:00:00Z".to_string()),
                        finish_reason: Some("unattended".to_string()),
                    },
                    TerminalLayoutSession {
                        terminal_id: "term-2".to_string(),
                        title: None,
                        title_withheld: true,
                        is_alive: false,
                        spawn_name: None,
                        claude_session_id: None,
                        account_label: None,
                        session_name: None,
                        zone_index: None,
                        finished: None,
                        finished_at: None,
                        finish_reason: None,
                    },
                ],
            },
            TerminalLayoutPage {
                page_id: "page-b".to_string(),
                label: None,
                order: None,
                sessions: vec![],
            },
        ],
    }
}

fn object_keys(value: &Value) -> BTreeSet<&str> {
    value
        .as_object()
        .expect("a JSON object")
        .keys()
        .map(String::as_str)
        .collect()
}

#[test]
fn full_example_round_trips() {
    let layout: TerminalLayout = serde_json::from_value(full_example()).unwrap();
    assert_eq!(layout, full_layout());
    // Value equality compares every object's full key set, so this also pins
    // each camelCase wire name and that no key is left out.
    assert_eq!(serde_json::to_value(&layout).unwrap(), full_example());

    let json1 = serde_json::to_string(&layout).unwrap();
    let back: TerminalLayout = serde_json::from_str(&json1).unwrap();
    assert_eq!(json1, serde_json::to_string(&back).unwrap());
}

#[test]
fn unknown_values_serialize_as_explicit_null() {
    let value = serde_json::to_value(full_layout()).unwrap();

    let unknown_session = &value["pages"][0]["sessions"][1];
    for key in SESSION_TRI_STATE_KEYS {
        assert_eq!(
            unknown_session.get(key),
            Some(&Value::Null),
            "session `{key}` must be serialized as an explicit null"
        );
    }

    let unregistered_page = &value["pages"][1];
    for key in ["label", "order"] {
        assert_eq!(
            unregistered_page.get(key),
            Some(&Value::Null),
            "page `{key}` must be serialized as an explicit null"
        );
    }

    let unknown_runner = TerminalLayoutRunner {
        instance_name: None,
        port: None,
        build_id: None,
        device_id: None,
    };
    assert_eq!(
        serde_json::to_value(unknown_runner).unwrap(),
        json!({"instanceName": null, "port": null, "buildId": null, "deviceId": null})
    );
}

#[test]
fn missing_optional_keys_read_as_unknown() {
    let session: TerminalLayoutSession = serde_json::from_value(json!({
        "terminalId": "term-3",
        "titleWithheld": false,
        "isAlive": true
    }))
    .unwrap();
    assert_eq!(
        session,
        TerminalLayoutSession {
            terminal_id: "term-3".to_string(),
            title: None,
            title_withheld: false,
            is_alive: true,
            spawn_name: None,
            claude_session_id: None,
            account_label: None,
            session_name: None,
            zone_index: None,
            finished: None,
            finished_at: None,
            finish_reason: None,
        }
    );

    let page: TerminalLayoutPage =
        serde_json::from_value(json!({"pageId": "page-c", "sessions": []})).unwrap();
    assert_eq!(page.label, None);
    assert_eq!(page.order, None);

    let runner: TerminalLayoutRunner = serde_json::from_value(json!({})).unwrap();
    assert_eq!(
        runner,
        TerminalLayoutRunner {
            instance_name: None,
            port: None,
            build_id: None,
            device_id: None,
        }
    );
}

#[test]
fn a_missing_required_key_is_an_error_not_a_default() {
    for key in ["terminalId", "titleWithheld", "isAlive"] {
        let mut session = full_example()["pages"][0]["sessions"][0].clone();
        session.as_object_mut().unwrap().remove(key);
        assert!(
            serde_json::from_value::<TerminalLayoutSession>(session).is_err(),
            "a session without `{key}` must be refused, not defaulted"
        );
    }
    for key in ["pageId", "sessions"] {
        let mut page = full_example()["pages"][0].clone();
        page.as_object_mut().unwrap().remove(key);
        assert!(
            serde_json::from_value::<TerminalLayoutPage>(page).is_err(),
            "a page without `{key}` must be refused, not defaulted"
        );
    }
    for key in ["runner", "labelsRegistered", "capturedAt", "pages"] {
        let mut layout = full_example();
        layout.as_object_mut().unwrap().remove(key);
        assert!(
            serde_json::from_value::<TerminalLayout>(layout).is_err(),
            "a layout without `{key}` must be refused, not defaulted"
        );
    }
}

/// The runner keeps `-1` as its own "unzoned or unknown" sentinel. On the wire
/// that is `null`; a `-1` that leaks through is refused, by serde here and by
/// the schema's `minimum: 0` in the generated models.
#[test]
fn a_negative_sentinel_is_refused() {
    let mut session = full_example()["pages"][0]["sessions"][0].clone();
    session["zoneIndex"] = json!(-1);
    assert!(
        serde_json::from_value::<TerminalLayoutSession>(session).is_err(),
        "zoneIndex -1 must be refused, not carried"
    );

    let mut page = full_example()["pages"][0].clone();
    page["order"] = json!(-1);
    assert!(
        serde_json::from_value::<TerminalLayoutPage>(page).is_err(),
        "order -1 must be refused, not carried"
    );

    let session_schema = serde_json::to_value(schema_for!(TerminalLayoutSession)).unwrap();
    let page_schema = serde_json::to_value(schema_for!(TerminalLayoutPage)).unwrap();
    for (name, schema, key) in [
        ("TerminalLayoutSession", &session_schema, "zoneIndex"),
        ("TerminalLayoutPage", &page_schema, "order"),
    ] {
        assert_eq!(
            schema["properties"][key].get("minimum"),
            Some(&json!(0)),
            "{name}.{key} must carry minimum 0 so the generated models refuse -1 too"
        );
    }
}

#[test]
fn keys_from_a_newer_runner_are_accepted_and_ignored() {
    let mut body = full_example();
    body["schemaVersion"] = json!(2);
    body["runner"]["gitSha"] = json!("0123abcd");
    body["pages"][0]["color"] = json!("blue");
    body["pages"][0]["sessions"][0]["coordStatus"] = json!({"value": "finished", "read": "ok"});

    let layout: TerminalLayout =
        serde_json::from_value(body).expect("an unknown key must never refuse a capture");
    assert_eq!(layout, full_layout());
}

/// The JSON Schema of each layout type, as qontinui-runner's `export_schemas`
/// emits it for the binding generators.
fn layout_schemas() -> [(&'static str, Value); 4] {
    [
        (
            "TerminalLayout",
            serde_json::to_value(schema_for!(TerminalLayout)).unwrap(),
        ),
        (
            "TerminalLayoutRunner",
            serde_json::to_value(schema_for!(TerminalLayoutRunner)).unwrap(),
        ),
        (
            "TerminalLayoutPage",
            serde_json::to_value(schema_for!(TerminalLayoutPage)).unwrap(),
        ),
        (
            "TerminalLayoutSession",
            serde_json::to_value(schema_for!(TerminalLayoutSession)).unwrap(),
        ),
    ]
}

/// Fails on any `additionalProperties: false` (or `unevaluatedProperties:
/// false`) anywhere in `node`, nested `$defs` included.
fn assert_open_everywhere(node: &Value, path: &str) {
    match node {
        Value::Object(map) => {
            for closing_key in ["additionalProperties", "unevaluatedProperties"] {
                assert_ne!(
                    map.get(closing_key),
                    Some(&Value::Bool(false)),
                    "{path} closes the object with `{closing_key}: false`; the generated \
                     models would then refuse a key added by a newer runner"
                );
            }
            for (key, child) in map {
                assert_open_everywhere(child, &format!("{path}/{key}"));
            }
        }
        Value::Array(items) => {
            for (index, child) in items.iter().enumerate() {
                assert_open_everywhere(child, &format!("{path}/{index}"));
            }
        }
        _ => {}
    }
}

#[test]
fn json_schema_leaves_every_layout_object_open() {
    for (name, schema) in layout_schemas() {
        assert_open_everywhere(&schema, name);
    }
}

#[test]
fn json_schema_requires_only_the_non_optional_fields() {
    let expected_required: [(&str, &[&str]); 4] = [
        (
            "TerminalLayout",
            &["capturedAt", "labelsRegistered", "pages", "runner"],
        ),
        ("TerminalLayoutRunner", &[]),
        ("TerminalLayoutPage", &["pageId", "sessions"]),
        (
            "TerminalLayoutSession",
            &["isAlive", "terminalId", "titleWithheld"],
        ),
    ];
    for ((name, schema), (expected_name, required)) in
        layout_schemas().iter().zip(expected_required)
    {
        assert_eq!(*name, expected_name);
        let actual: BTreeSet<&str> = schema
            .get("required")
            .and_then(Value::as_array)
            .map(|keys| keys.iter().filter_map(Value::as_str).collect())
            .unwrap_or_default();
        let expected: BTreeSet<&str> = required.iter().copied().collect();
        assert_eq!(actual, expected, "{name}: required keys");

        // Every other property is a tri-state field: nullable, defaulting to
        // null, so a generated model reads a missing key as unknown.
        let properties = &schema["properties"];
        for key in object_keys(properties) {
            if expected.contains(key) {
                continue;
            }
            let property = &properties[key];
            assert_eq!(
                property.get("default"),
                Some(&Value::Null),
                "{name}.{key} must default to null"
            );
            let nullable = property
                .get("type")
                .and_then(Value::as_array)
                .is_some_and(|types| types.contains(&json!("null")));
            assert!(nullable, "{name}.{key} must admit null: {property}");
        }
    }
}
