//! The ledger envelope's golden JSON Schema: a frozen version is never
//! edited.
//!
//! `tests/fixtures/ledger_envelope.v<N>.schema.json` is the schema
//! `schemars` emits for [`LedgerEnvelope`] at `ENVELOPE_SCHEMA_VERSION = N`.
//! This test regenerates it and fails on any STRUCTURAL difference — the
//! comparison strips every `description` and `title` key, so a doc-comment
//! edit or a `schemars` upgrade that only rewords cannot force a meaningless
//! version bump, while a changed field, type, enum token or `required` entry
//! cannot merge without one. Plan
//! `2026-11-22-phase1-envelope-frozen-and-measurement-dispatch-proven`
//! Phase 1.
//!
//! To change the envelope: bump `ENVELOPE_SCHEMA_VERSION`, then run this test
//! with `LEDGER_ENVELOPE_BLESS=1`, which writes the NEW version's file only
//! when it does not exist yet. It never overwrites a file, and every earlier
//! version's file must stay.

use std::path::PathBuf;

use qontinui_types::ledger::{LedgerEnvelope, ENVELOPE_SCHEMA_VERSION};
use serde_json::{json, Value};

fn fixture(version: u32) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures")
        .join(format!("ledger_envelope.v{version}.schema.json"))
}

fn emitted() -> Value {
    serde_json::to_value(schemars::schema_for!(LedgerEnvelope)).unwrap()
}

/// The schema with every `description` / `title` key removed, recursively.
/// `serde_json::Value` equality is key-order independent, so this is the
/// canonical structural form.
fn structural(value: &Value) -> Value {
    match value {
        Value::Object(map) => Value::Object(
            map.iter()
                .filter(|(k, _)| k.as_str() != "description" && k.as_str() != "title")
                .map(|(k, v)| (k.clone(), structural(v)))
                .collect(),
        ),
        Value::Array(items) => Value::Array(items.iter().map(structural).collect()),
        other => other.clone(),
    }
}

/// `Ok` when `emitted` is structurally the frozen `golden`; otherwise the
/// reason, naming the remedy.
fn golden_verdict(emitted: &Value, golden: &Value, version: u32) -> Result<(), String> {
    if structural(emitted) == structural(golden) {
        return Ok(());
    }
    Err(format!(
        "the emitted LedgerEnvelope schema differs STRUCTURALLY from the frozen \
         ledger_envelope.v{version}.schema.json. A frozen version is never edited: bump \
         ENVELOPE_SCHEMA_VERSION to {next} and run this test with LEDGER_ENVELOPE_BLESS=1 to \
         add ledger_envelope.v{next}.schema.json (keep v{version}).\nemitted:\n{}",
        serde_json::to_string_pretty(emitted).unwrap(),
        next = version + 1,
    ))
}

/// The frozen schema of the current version. With `LEDGER_ENVELOPE_BLESS`
/// set it first writes the file — only when it does not exist yet.
fn golden() -> Value {
    let path = fixture(ENVELOPE_SCHEMA_VERSION);
    if !path.exists() && std::env::var_os("LEDGER_ENVELOPE_BLESS").is_some() {
        let mut text = serde_json::to_string_pretty(&emitted()).unwrap();
        text.push('\n');
        // Write-then-rename so a parallel test never reads a half-written file.
        let tmp = path.with_extension(format!("tmp.{}", std::process::id()));
        std::fs::write(&tmp, text).unwrap();
        std::fs::rename(&tmp, &path).unwrap();
    }
    let bytes = std::fs::read(&path).unwrap_or_else(|e| {
        panic!(
            "{} is missing ({e}). ENVELOPE_SCHEMA_VERSION = {ENVELOPE_SCHEMA_VERSION} needs its \
             golden file: run this test once with LEDGER_ENVELOPE_BLESS=1 and commit the file",
            path.display()
        )
    });
    serde_json::from_slice(&bytes).expect("golden schema is JSON")
}

#[test]
fn emitted_schema_matches_the_frozen_golden_file() {
    if let Err(reason) = golden_verdict(&emitted(), &golden(), ENVELOPE_SCHEMA_VERSION) {
        panic!("{reason}");
    }
}

#[test]
fn every_earlier_version_file_is_kept() {
    for version in 1..=ENVELOPE_SCHEMA_VERSION {
        let path = fixture(version);
        let parsed: Value = serde_json::from_slice(
            &std::fs::read(&path)
                .unwrap_or_else(|e| panic!("frozen {} must be kept: {e}", path.display())),
        )
        .unwrap_or_else(|e| panic!("{} is not JSON: {e}", path.display()));
        assert!(parsed.is_object(), "{} is not a schema", path.display());
    }
}

#[test]
fn a_structural_mutation_is_caught_and_a_reworded_description_is_not() {
    let golden = golden();
    golden_verdict(&golden, &golden, ENVELOPE_SCHEMA_VERSION).expect("identical");

    // Reworded docs and titles only: not a version change.
    let mut reworded = golden.clone();
    reworded["description"] = json!("reworded");
    reworded["title"] = json!("Renamed");
    reworded["properties"]["id"]["description"] = json!("reworded too");
    golden_verdict(&reworded, &golden, ENVELOPE_SCHEMA_VERSION)
        .expect("description/title edits are not structural");

    // Each of these is a change in meaning and must fail.
    type Mutation = Box<dyn Fn(&mut Value)>;
    let mutations: Vec<(&str, Mutation)> = vec![
        (
            "assignment dropped from required",
            Box::new(|s| {
                let req = s["required"].as_array_mut().unwrap();
                req.retain(|k| k != "assignment");
            }),
        ),
        (
            "a new property",
            Box::new(|s| {
                s["properties"]["extra"] = json!({"type": "string"});
            }),
        ),
        (
            "a changed type",
            Box::new(|s| {
                s["properties"]["source_seq"]["type"] = json!("string");
            }),
        ),
        (
            "an enum token added",
            Box::new(|s| {
                let defs = s["$defs"].as_object_mut().unwrap();
                let grade = defs.get_mut("EvidenceGrade").unwrap();
                let text = serde_json::to_string(grade).unwrap();
                assert!(text.contains("gate_grade"), "EvidenceGrade def: {text}");
                *grade = serde_json::from_str(&text.replace("\"gate_grade\"", "\"gate_grade2\""))
                    .unwrap();
            }),
        ),
        (
            "additionalProperties relaxed",
            Box::new(|s| {
                s["additionalProperties"] = json!(true);
            }),
        ),
    ];
    for (name, mutate) in mutations {
        let mut mutated = golden.clone();
        mutate(&mut mutated);
        assert!(
            golden_verdict(&mutated, &golden, ENVELOPE_SCHEMA_VERSION).is_err(),
            "mutation `{name}` was not caught"
        );
    }
}

#[test]
fn the_frozen_schema_requires_every_envelope_field_and_refuses_unknown_ones() {
    let golden = golden();
    let mut properties: Vec<&str> = golden["properties"]
        .as_object()
        .unwrap()
        .keys()
        .map(String::as_str)
        .collect();
    let mut required: Vec<&str> = golden["required"]
        .as_array()
        .unwrap()
        .iter()
        .map(|v| v.as_str().unwrap())
        .collect();
    properties.sort_unstable();
    required.sort_unstable();
    assert_eq!(
        required, properties,
        "every envelope field is required in v1"
    );
    assert!(required.contains(&"assignment"));
    assert_eq!(golden["additionalProperties"], json!(false));

    // Required is not non-null: these fields are spelled `null` when absent.
    for field in [
        "assignment",
        "causation_id",
        "correlation_id",
        "policy_version",
        "prev_hash",
    ] {
        let text = serde_json::to_string(&golden["properties"][field]).unwrap();
        assert!(
            text.contains("\"null\""),
            "`{field}` must admit null: {text}"
        );
    }
    let part = serde_json::to_string(&golden["$defs"]["PrincipalPart"]).unwrap();
    assert!(
        part.contains("\"null\""),
        "PrincipalPart.value must admit null: {part}"
    );
}
