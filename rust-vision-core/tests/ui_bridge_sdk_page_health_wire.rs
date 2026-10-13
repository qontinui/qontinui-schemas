//! Wire-compatibility gate: recorded, GENUINE UI Bridge SDK page-health
//! answers must deserialize into `Observation<PageHealthValue>` and lose
//! nothing on the way in.
//!
//! ## What this stands in for
//!
//! The SDK (`@qontinui/ui-bridge` >= 0.30.0, producer `sdk/page-health`)
//! answers `/control/page-health` in the Observation envelope, typed in TS by
//! `packages/ui-bridge/src/observation/observation.ts` and
//! `packages/ui-bridge/src/server/page-health.ts`. The Rust canon is
//! [`Observation`] + [`PageHealthValue`] in this crate. Nothing else holds the
//! TS twin to the Rust canon: renaming `observedAt`, dropping a provenance
//! key, or respelling a severity on either side compiles everywhere and
//! breaks every strict parse at runtime. These fixtures turn that into a red
//! PR (plan
//! `2026-09-20-ui-bridge-observations-distinguish-cannot-see-from-not-present-and-carry-provenance`,
//! Phase 3).
//!
//! ## The fixtures
//!
//! `rust/tests/fixtures/ui_bridge_sdk/page_health/<name>.json` is SDK output,
//! never hand-typed JSON, captured by the harness in `capture/` beside them;
//! `<name>.provenance.json` records the SDK version/commit, the capture
//! command, the date, any trim, and the fixture's sha256 (the same sidecar
//! convention as `rust/tests/ui_bridge_sdk_wire.rs`). They live in a
//! SUBDIRECTORY on purpose: that test parses every `.json` directly inside
//! `ui_bridge_sdk/` as a `UIBridgeSnapshot` and skips subdirectories. The
//! directory is enumerated, so a new fixture is gated without editing this
//! file. Like the snapshot fixtures, these are append-only: a wire-changing
//! SDK release ADDS a capture; the old ones stay.

use chrono::DateTime;
use qontinui_vision_core::{Observation, ObservationStatus, PageHealthValue, UnknownCode};
use serde_json::Value;
use sha2::{Digest, Sha256};
use std::collections::BTreeMap;
use std::path::PathBuf;

/// The pair the plan requires: a populated page and an empty registry.
const MIN_FIXTURES: usize = 2;

const POPULATED: &str = "sdk_0_30_0_page_health_populated.json";
const EMPTY_REGISTRY: &str = "sdk_0_30_0_page_health_empty_registry.json";

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

/// Envelope keys whose values are RFC3339 instants. The SDK writes
/// millisecond precision (`…00.000Z`) and chrono re-serializes without it
/// (`…00Z`): the same instant, different bytes, so the loss check compares
/// these as instants.
const TIMESTAMP_KEYS: &[&str] = &["observedAt", "evaluatedAt", "storedAt"];

struct Fixture {
    name: String,
    bytes: Vec<u8>,
    provenance: Value,
}

fn fixture_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../rust/tests/fixtures/ui_bridge_sdk/page_health")
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

fn fixture(name: &str) -> Fixture {
    load_fixtures()
        .into_iter()
        .find(|f| f.name == name)
        .unwrap_or_else(|| panic!("{name} is missing from {}", fixture_dir().display()))
}

fn as_json(fixture: &Fixture) -> Value {
    serde_json::from_slice(&fixture.bytes)
        .unwrap_or_else(|e| panic!("{} is not valid JSON: {e}", fixture.name))
}

/// The strict parse a consumer (the runner forwarding the SDK's page-health)
/// performs.
fn parse(value: Value) -> Result<Observation<PageHealthValue>, serde_json::Error> {
    serde_json::from_value::<Observation<PageHealthValue>>(value)
}

fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}

/// Rewrite every RFC3339 instant under a [`TIMESTAMP_KEYS`] key to its
/// nanosecond count, so two spellings of one instant compare equal and two
/// different instants do not.
fn normalize_instants(value: &mut Value) {
    match value {
        Value::Object(map) => {
            for (key, v) in map.iter_mut() {
                if TIMESTAMP_KEYS.contains(&key.as_str()) {
                    if let Some(s) = v.as_str() {
                        let at = DateTime::parse_from_rfc3339(s)
                            .unwrap_or_else(|e| panic!("`{key}` = {s:?} is not RFC3339: {e}"));
                        *v = Value::from(at.timestamp_nanos_opt().expect("instant in range"));
                        continue;
                    }
                }
                normalize_instants(v);
            }
        }
        Value::Array(items) => items.iter_mut().for_each(normalize_instants),
        _ => {}
    }
}

#[test]
fn fixture_directory_is_populated_and_every_fixture_has_a_sidecar() {
    let fixtures = load_fixtures();
    assert!(
        fixtures.len() >= MIN_FIXTURES,
        "only {} SDK page-health fixture(s) under {} (floor {MIN_FIXTURES}); fixtures are \
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
        let actual = hex(&Sha256::digest(&fixture.bytes));
        assert_eq!(
            p["sha256"].as_str(),
            Some(actual.as_str()),
            "{}: bytes do not match the sha256 recorded at capture. A captured fixture \
             is never edited; capture a NEW fixture from the SDK instead.",
            fixture.name
        );
    }
}

/// Every recorded answer parses, and the parse keeps everything the wire
/// carried. The second half matters because the envelope and its nested
/// types IGNORE unrecognised keys (deliberately, for forward compatibility),
/// so an SDK-side rename of an OPTIONAL-looking key would otherwise parse
/// clean and silently drop it. Re-serializing and comparing with the wire
/// catches that: anything the Rust type does not model comes back missing.
#[test]
fn recorded_sdk_page_health_strict_parses_into_observation_and_loses_nothing() {
    let mut failures = Vec::new();
    for fixture in load_fixtures() {
        let raw = as_json(&fixture);
        match parse(raw.clone()) {
            Ok(observation) => {
                let mut wire = raw;
                let mut reparsed = serde_json::to_value(&observation).expect("serialize");
                normalize_instants(&mut wire);
                normalize_instants(&mut reparsed);
                if wire != reparsed {
                    failures.push(format!(
                        "{}: the parse dropped or altered wire content.\n    wire:     {wire}\n    reparsed: {reparsed}",
                        fixture.name
                    ));
                }
            }
            Err(e) => failures.push(format!("{}: {e}", fixture.name)),
        }
    }
    assert!(
        failures.is_empty(),
        "recorded UI Bridge SDK page-health answers no longer parse into \
         Observation<PageHealthValue> intact:\n  {}",
        failures.join("\n  ")
    );
}

#[test]
fn populated_page_is_measured_with_a_report() {
    let f = fixture(POPULATED);
    let obs = parse(as_json(&f)).expect("populated fixture parses");
    assert_eq!(obs.status(), ObservationStatus::Measured);
    let value = obs
        .value()
        .expect("a measured observation carries its value");
    assert!(
        !value.findings.is_empty(),
        "a measured report carries findings"
    );
    assert!(
        value
            .findings
            .iter()
            .any(|finding| !finding.data.is_empty()),
        "anti-vacuity: at least one finding must carry per-check `data`"
    );
    assert_eq!(value.heatmap.len(), 20, "the SDK's heatmap is 20 rows");
    assert!(
        value.element_count > 0,
        "the populated capture had elements"
    );
    let worst = value.findings.iter().map(|finding| finding.severity).max();
    assert_eq!(
        Some(value.summary),
        worst,
        "summary is the worst finding severity"
    );
    assert_eq!(obs.provenance().producer.id, "sdk/page-health");
    assert!(
        obs.provenance().observed_at.is_some(),
        "the populated capture analyzed a timestamped snapshot"
    );
}

#[test]
fn empty_registry_is_unknown_producer_not_run_with_no_value() {
    let f = fixture(EMPTY_REGISTRY);
    let raw = as_json(&f);
    assert!(
        raw.get("value").is_none(),
        "{EMPTY_REGISTRY}: an unknown carries no `value` key"
    );
    let obs = parse(raw).expect("empty-registry fixture parses");
    assert_eq!(obs.status(), ObservationStatus::Unknown);
    assert!(obs.value().is_none());
    let info = obs.unknown_info().expect("an unknown carries its info");
    assert_eq!(info.code, UnknownCode::ProducerNotRun);
    assert!(!info.detail.trim().is_empty());
    assert_eq!(obs.provenance().producer.id, "sdk/page-health");
}

/// The acceptance criterion, made permanent: a mutated SDK envelope must make
/// the parse RED. Each mutation is applied to an in-memory copy of a genuine
/// fixture and must be refused by the strict parse itself. (A rename of a key
/// serde would merely ignore is the loss check's job, in the test above.)
#[test]
fn mutating_the_sdk_envelope_turns_the_parse_red() {
    type Mutation = (&'static str, fn(&mut Value));
    let mutations: &[Mutation] = &[
        ("rename provenance.observedAt", |v| {
            let p = v["provenance"].as_object_mut().unwrap();
            let x = p.remove("observedAt").unwrap();
            p.insert("observed_at".into(), x);
        }),
        ("rename provenance.evaluatedAt", |v| {
            let p = v["provenance"].as_object_mut().unwrap();
            let x = p.remove("evaluatedAt").unwrap();
            p.insert("evaluated_at".into(), x);
        }),
        ("drop provenance.confidence", |v| {
            v["provenance"]
                .as_object_mut()
                .unwrap()
                .remove("confidence");
        }),
        ("drop provenance.cache", |v| {
            v["provenance"].as_object_mut().unwrap().remove("cache");
        }),
        ("drop provenance.source", |v| {
            v["provenance"].as_object_mut().unwrap().remove("source");
        }),
        ("rename value -> report", |v| {
            let o = v.as_object_mut().unwrap();
            let x = o.remove("value").unwrap();
            o.insert("report".into(), x);
        }),
        ("rename value.element_count -> elementCount", |v| {
            let o = v["value"].as_object_mut().unwrap();
            let x = o.remove("element_count").unwrap();
            o.insert("elementCount".into(), x);
        }),
        ("lowercase a severity", |v| {
            v["value"]["summary"] =
                Value::from(v["value"]["summary"].as_str().unwrap().to_ascii_lowercase());
        }),
        ("status measured -> ok", |v| {
            v["status"] = Value::from("ok");
        }),
    ];

    let base = as_json(&fixture(POPULATED));
    parse(base.clone()).expect("the unmutated fixture parses (precondition)");
    for (what, mutate) in mutations {
        let mut copy = base.clone();
        mutate(&mut copy);
        assert!(
            parse(copy).is_err(),
            "mutation `{what}` still parsed: the Rust canon no longer pins that part of the SDK wire"
        );
    }

    let empty = as_json(&fixture(EMPTY_REGISTRY));
    let mut copy = empty.clone();
    copy["unknown"]["code"] = Value::from("producerNotRun");
    assert!(
        parse(copy).is_err(),
        "a respelled UnknownCode must not parse"
    );
    let mut copy = empty;
    let p = copy["provenance"].as_object_mut().unwrap();
    let x = p.remove("observedAt").unwrap();
    p.insert("observed_at".into(), x);
    assert!(
        parse(copy).is_err(),
        "renaming observedAt on the unknown fixture must not parse"
    );
}
