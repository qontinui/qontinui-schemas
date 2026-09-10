//! Wire-compatibility gate: recorded, GENUINE UI Bridge SDK snapshots must
//! still deserialize into [`UIBridgeSnapshot`].
//!
//! ## What this stands in for
//!
//! qontinui-runner parses live SDK snapshots with a plain strict serde parse
//! into this crate's type, with no adapter in between:
//!
//! - `crates/spec-check/src/fetch.rs` `wrap_snapshot` —
//!   `serde_json::from_value(raw.clone())` into `UIBridgeSnapshot`;
//! - `src-tauri/src/spec_api/spec_check.rs` `parse_supplied_snapshot` —
//!   `serde_json::from_value::<UIBridgeSnapshot>(value)`.
//!
//! [`parses_as_the_runner_does`] is that exact call. A change to
//! `rust/src/ui_bridge.rs` can compile in every consumer (which is all the
//! `consumer-gate` workflow proves) and still make every live snapshot fail to
//! parse at runtime: qontinui-schemas#164 retyped `customActions` from names to
//! structs while the SDK still emits names. This test is what turns that into a
//! red PR instead of a runtime outage.
//!
//! ## The fixtures
//!
//! `tests/fixtures/ui_bridge_sdk/<name>.json` is SDK output, never hand-typed
//! JSON; `<name>.provenance.json` beside it records the SDK version/commit, the
//! capture command, the date, any trim, and the fixture's sha256. The directory
//! is enumerated, so a new fixture is gated without editing this file.
//!
//! **Fixtures are append-only.** A wire-changing SDK release ADDS a fixture
//! captured from that release; the old ones stay, because deployed apps keep
//! emitting the old wire until they upgrade. Modifying, deleting or renaming an
//! existing fixture or sidecar is refused by the `consumer-gate` job in
//! `.github/workflows/runner-consumer-check.yml` unless the PR carries the
//! `wire-fixture:retire` label — retiring a fixture means "we no longer accept
//! that SDK's wire", which is a decision, not a fix.

use qontinui_types::ui_bridge::UIBridgeSnapshot;
use serde_json::Value;
use sha2::{Digest, Sha256};
use std::collections::BTreeMap;
use std::path::PathBuf;

/// The directory must never be quietly emptied: two fixtures exist today (the
/// SDK's `origin/main` and the live runner on the published SDK it locks).
const MIN_FIXTURES: usize = 2;

const SIDECAR_SUFFIX: &str = ".provenance.json";

/// Keys every sidecar must carry as non-empty strings.
const REQUIRED_PROVENANCE_KEYS: &[&str] = &[
    "fixture",
    "sha256",
    "sdk_package",
    "sdk_version",
    "sdk_commit",
    "source",
    "capture_command",
    "captured_at",
    "trim",
];

struct Fixture {
    name: String,
    bytes: Vec<u8>,
    provenance: Value,
}

fn fixture_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/ui_bridge_sdk")
}

/// Load every fixture with its sidecar. Panics on an orphan in either
/// direction, so a fixture can never go ungated for want of a sidecar and a
/// sidecar can never vouch for a fixture that is gone.
fn load_fixtures() -> Vec<Fixture> {
    let dir = fixture_dir();
    let mut fixtures: BTreeMap<String, Vec<u8>> = BTreeMap::new();
    let mut sidecars: BTreeMap<String, Value> = BTreeMap::new();

    for entry in std::fs::read_dir(&dir).unwrap_or_else(|e| panic!("read {}: {e}", dir.display())) {
        let path = entry.expect("dir entry").path();
        if !path.is_file() {
            continue;
        }
        let file_name = path
            .file_name()
            .and_then(|n| n.to_str())
            .expect("utf-8 file name")
            .to_string();
        if let Some(stem) = file_name.strip_suffix(SIDECAR_SUFFIX) {
            let raw = std::fs::read(&path).expect("read sidecar");
            let value: Value = serde_json::from_slice(&raw)
                .unwrap_or_else(|e| panic!("{file_name} is not valid JSON: {e}"));
            sidecars.insert(format!("{stem}.json"), value);
        } else if file_name.ends_with(".json") {
            fixtures.insert(file_name, std::fs::read(&path).expect("read fixture"));
        }
    }

    let without_sidecar: Vec<&String> = fixtures
        .keys()
        .filter(|n| !sidecars.contains_key(*n))
        .collect();
    let without_fixture: Vec<&String> = sidecars
        .keys()
        .filter(|n| !fixtures.contains_key(*n))
        .collect();
    assert!(
        without_sidecar.is_empty() && without_fixture.is_empty(),
        "fixture/sidecar mismatch in {}: fixtures with no {SIDECAR_SUFFIX}: {without_sidecar:?}; \
         sidecars with no fixture: {without_fixture:?}",
        dir.display()
    );

    fixtures
        .into_iter()
        .map(|(name, bytes)| {
            let provenance = sidecars.remove(&name).expect("checked above");
            Fixture {
                name,
                bytes,
                provenance,
            }
        })
        .collect()
}

fn as_json(fixture: &Fixture) -> Value {
    serde_json::from_slice(&fixture.bytes)
        .unwrap_or_else(|e| panic!("{} is not valid JSON: {e}", fixture.name))
}

fn array_len(value: &Value, key: &str) -> usize {
    value.get(key).and_then(Value::as_array).map_or(0, Vec::len)
}

/// The parse both runner call sites perform, verbatim.
fn parses_as_the_runner_does(value: Value) -> Result<UIBridgeSnapshot, serde_json::Error> {
    serde_json::from_value::<UIBridgeSnapshot>(value)
}

#[test]
fn fixture_directory_is_populated_and_every_fixture_has_a_sidecar() {
    let fixtures = load_fixtures();
    assert!(
        fixtures.len() >= MIN_FIXTURES,
        "only {} SDK wire fixture(s) under {} (floor {MIN_FIXTURES}); fixtures are \
         append-only and must not be removed to make a change pass",
        fixtures.len(),
        fixture_dir().display()
    );
}

#[test]
fn sidecars_record_complete_provenance_and_the_fixture_digest() {
    for fixture in load_fixtures() {
        let p = &fixture.provenance;
        for key in REQUIRED_PROVENANCE_KEYS {
            let ok = p
                .get(*key)
                .and_then(Value::as_str)
                .is_some_and(|s| !s.trim().is_empty());
            assert!(
                ok,
                "{}: sidecar is missing a non-empty `{key}`",
                fixture.name
            );
        }
        assert_eq!(
            p["fixture"].as_str(),
            Some(fixture.name.as_str()),
            "{}: sidecar names a different fixture",
            fixture.name
        );
        let actual = hex::encode(Sha256::digest(&fixture.bytes));
        assert_eq!(
            p["sha256"].as_str(),
            Some(actual.as_str()),
            "{}: bytes do not match the sha256 recorded at capture. A captured fixture \
             is never edited; capture a NEW fixture from the SDK instead.",
            fixture.name
        );
    }
}

/// Anti-vacuity on the fixtures themselves: each must keep exercising the
/// planes a typed change can break, so none can be "fixed" by deleting the
/// field that fails to parse.
#[test]
fn fixtures_exercise_custom_actions_components_and_undo_redo() {
    for fixture in load_fixtures() {
        let raw = as_json(&fixture);
        let elements = raw["elements"]
            .as_array()
            .map(Vec::as_slice)
            .unwrap_or_default();
        assert!(
            elements.iter().any(|e| array_len(e, "customActions") > 0),
            "{}: no element carries a non-empty customActions array",
            fixture.name
        );
        let components = raw["components"]
            .as_array()
            .map(Vec::as_slice)
            .unwrap_or_default();
        assert!(
            components.iter().any(|c| array_len(c, "actions") > 0),
            "{}: no component carries actions",
            fixture.name
        );
        assert!(
            raw.get("undoRedo").is_some_and(Value::is_object),
            "{}: no undoRedo object",
            fixture.name
        );
    }
}

#[test]
fn recorded_sdk_snapshots_strict_parse_into_ui_bridge_snapshot() {
    let mut failures = Vec::new();
    for fixture in load_fixtures() {
        let raw = as_json(&fixture);
        let snapshot = match parses_as_the_runner_does(raw.clone()) {
            Ok(s) => s,
            Err(e) => {
                failures.push(format!("{}: {e}", fixture.name));
                continue;
            }
        };

        // Nothing may be silently dropped on the way in.
        assert_eq!(
            snapshot.elements.len(),
            array_len(&raw, "elements"),
            "{}: element count changed across the parse",
            fixture.name
        );
        assert_eq!(
            snapshot.components.len(),
            array_len(&raw, "components"),
            "{}: component count changed across the parse",
            fixture.name
        );
        for (parsed, wire) in snapshot
            .elements
            .iter()
            .zip(raw["elements"].as_array().into_iter().flatten())
        {
            assert_eq!(
                parsed.custom_actions.as_ref().map_or(0, Vec::len),
                array_len(wire, "customActions"),
                "{}: element {} lost custom actions across the parse",
                fixture.name,
                parsed.id
            );
        }
    }
    assert!(
        failures.is_empty(),
        "recorded UI Bridge SDK snapshots no longer parse into UIBridgeSnapshot — every \
         consumer's live parse would fail the same way:\n  {}",
        failures.join("\n  ")
    );
}
