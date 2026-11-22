//! The ledger envelope's golden JSON Schema: a frozen version is never
//! edited.
//!
//! `tests/fixtures/ledger_envelope.v<N>.schema.json` is the schema
//! `schemars` emits for [`LedgerEnvelope`] at `ENVELOPE_SCHEMA_VERSION = N`.
//! This test regenerates it and fails on any STRUCTURAL difference — the
//! comparison strips the `description` and `title` schema KEYWORDS (never a
//! property or definition of that name) and sorts `required`, so a
//! doc-comment edit or a `schemars` upgrade that only rewords or reorders
//! cannot force a meaningless version bump, while a changed field, type,
//! pattern, enum token or `required` entry cannot merge without one. Each
//! frozen file's structural digest is pinned in [`FROZEN_DIGESTS`], so a
//! frozen file edited in place fails too. Plan
//! `2026-11-22-phase1-envelope-frozen-and-measurement-dispatch-proven`
//! Phase 1.
//!
//! The schema is also the validator non-Rust writers use, so
//! [`the_schema_and_validate_agree_on_the_corpus`] runs one corpus through a
//! JSON Schema validator (formats asserted) and through
//! `qontinui_types::ledger::validate`, and fails if they disagree.
//!
//! To change the envelope: bump `ENVELOPE_SCHEMA_VERSION`, run this test with
//! `LEDGER_ENVELOPE_BLESS=1` (it writes the NEW version's file only when it
//! does not exist yet, never overwriting one), then add the new file's digest
//! — printed by the failing pin test — to [`FROZEN_DIGESTS`]. Every earlier
//! version's file and digest stay.

use std::path::PathBuf;

use qontinui_types::canonical_hash::canonical_hash;
use qontinui_types::ledger::{validate, LedgerEnvelope, ENVELOPE_SCHEMA_VERSION};
use serde_json::{json, Map, Value};

/// `canonical_hash(structural(file))` of every frozen version, oldest first.
const FROZEN_DIGESTS: &[(u32, &str)] = &[(
    1,
    "sha256-8b3f21f91d093678509ab7fc4ce1753197d6f74cf3be525950e065ce72660cdf",
)];

fn fixture(version: u32) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures")
        .join(format!("ledger_envelope.v{version}.schema.json"))
}

fn emitted() -> Value {
    serde_json::to_value(schemars::schema_for!(LedgerEnvelope)).unwrap()
}

/// Keywords whose value is a map from NAMES to subschemas.
const NAMED_SUBSCHEMAS: &[&str] = &[
    "properties",
    "patternProperties",
    "$defs",
    "definitions",
    "dependentSchemas",
];
/// Keywords whose value is one subschema.
const ONE_SUBSCHEMA: &[&str] = &[
    "items",
    "additionalItems",
    "additionalProperties",
    "unevaluatedItems",
    "unevaluatedProperties",
    "contains",
    "propertyNames",
    "not",
    "if",
    "then",
    "else",
];
/// Keywords whose value is an array of subschemas.
const SUBSCHEMA_ARRAYS: &[&str] = &["allOf", "anyOf", "oneOf", "prefixItems"];

/// The canonical structural form of a SCHEMA: `description` / `title`
/// removed only where they are keywords of a schema object, `required`
/// sorted, and every other value (`const`, `enum`, `default`, property and
/// definition names) kept verbatim. `serde_json::Value` equality is
/// key-order independent.
fn structural(schema: &Value) -> Value {
    let Value::Object(map) = schema else {
        return schema.clone(); // `true` / `false` schemas
    };
    let mut out = Map::new();
    for (key, value) in map {
        let k = key.as_str();
        let kept = if k == "description" || k == "title" {
            continue;
        } else if NAMED_SUBSCHEMAS.contains(&k) {
            match value {
                Value::Object(named) => Value::Object(
                    named
                        .iter()
                        .map(|(name, sub)| (name.clone(), structural(sub)))
                        .collect(),
                ),
                other => other.clone(),
            }
        } else if ONE_SUBSCHEMA.contains(&k) {
            structural(value)
        } else if SUBSCHEMA_ARRAYS.contains(&k) {
            match value {
                Value::Array(items) => Value::Array(items.iter().map(structural).collect()),
                other => other.clone(),
            }
        } else if k == "required" {
            match value {
                Value::Array(names) => {
                    let mut names = names.clone();
                    names.sort_by_key(|n| n.to_string());
                    Value::Array(names)
                }
                other => other.clone(),
            }
        } else {
            value.clone()
        };
        out.insert(key.clone(), kept);
    }
    Value::Object(out)
}

fn structural_digest(schema: &Value) -> String {
    canonical_hash(&structural(schema)).unwrap()
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

fn read_schema(version: u32) -> Value {
    let path = fixture(version);
    let bytes = std::fs::read(&path).unwrap_or_else(|e| {
        panic!(
            "{} is missing ({e}). Every version up to ENVELOPE_SCHEMA_VERSION = \
             {ENVELOPE_SCHEMA_VERSION} needs its golden file: run this test once with \
             LEDGER_ENVELOPE_BLESS=1 and commit the file",
            path.display()
        )
    });
    serde_json::from_slice(&bytes).unwrap_or_else(|e| panic!("{} is not JSON: {e}", path.display()))
}

/// The frozen schema of the current version. With `LEDGER_ENVELOPE_BLESS`
/// set it first writes the file — only when it does not exist yet.
fn golden() -> Value {
    let path = fixture(ENVELOPE_SCHEMA_VERSION);
    if !path.exists() && std::env::var_os("LEDGER_ENVELOPE_BLESS").is_some() {
        let mut text = serde_json::to_string_pretty(&emitted()).unwrap();
        text.push('\n');
        // Write-then-rename so a parallel test never reads a half-written file.
        // One temp file per thread: tests bless concurrently.
        let thread = format!("{:?}", std::thread::current().id());
        let thread: String = thread.chars().filter(char::is_ascii_digit).collect();
        let tmp = path.with_extension(format!("tmp.{}.{thread}", std::process::id()));
        std::fs::write(&tmp, text).unwrap();
        std::fs::rename(&tmp, &path).unwrap();
    }
    read_schema(ENVELOPE_SCHEMA_VERSION)
}

#[test]
fn emitted_schema_matches_the_frozen_golden_file() {
    if let Err(reason) = golden_verdict(&emitted(), &golden(), ENVELOPE_SCHEMA_VERSION) {
        panic!("{reason}");
    }
}

#[test]
fn every_frozen_file_is_kept_and_matches_its_pinned_digest() {
    golden(); // bless first, if asked, so the current file exists
    let pinned: Vec<u32> = FROZEN_DIGESTS.iter().map(|(v, _)| *v).collect();
    let expected: Vec<u32> = (1..=ENVELOPE_SCHEMA_VERSION).collect();
    let mut problems = Vec::new();
    if pinned != expected {
        problems.push(format!(
            "FROZEN_DIGESTS pins versions {pinned:?}; it must pin exactly {expected:?}"
        ));
    }
    for version in 1..=ENVELOPE_SCHEMA_VERSION {
        let actual = structural_digest(&read_schema(version));
        match FROZEN_DIGESTS.iter().find(|(v, _)| *v == version) {
            Some((_, pin)) if *pin == actual => {}
            Some((_, pin)) => problems.push(format!(
                "ledger_envelope.v{version}.schema.json was edited in place: structural digest \
                 {actual}, pinned {pin}. A frozen version is never edited"
            )),
            None => problems.push(format!(
                "v{version} has no pinned digest; add ({version}, \"{actual}\") to FROZEN_DIGESTS"
            )),
        }
    }
    assert!(problems.is_empty(), "{}", problems.join("\n"));
}

#[test]
fn a_structural_mutation_is_caught_and_a_reworded_description_is_not() {
    let golden = golden();
    golden_verdict(&golden, &golden, ENVELOPE_SCHEMA_VERSION).expect("identical");

    // Reworded docs and titles, and a reordered `required`: not a change.
    let mut reworded = golden.clone();
    reworded["description"] = json!("reworded");
    reworded["title"] = json!("Renamed");
    reworded["properties"]["id"]["description"] = json!("reworded too");
    reworded["required"].as_array_mut().unwrap().reverse();
    golden_verdict(&reworded, &golden, ENVELOPE_SCHEMA_VERSION)
        .expect("description/title edits and required order are not structural");
    assert_eq!(structural_digest(&reworded), structural_digest(&golden));

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
            "a relaxed pattern",
            Box::new(|s| {
                s["properties"]["id"]["pattern"] = json!("^.*$");
            }),
        ),
        (
            "a plain-enum token appended (SourceClass)",
            Box::new(|s| {
                let tokens = s["$defs"]["SourceClass"]["enum"]
                    .as_array_mut()
                    .expect("SourceClass is a plain `enum` array");
                tokens.push(json!("vendor"));
            }),
        ),
        (
            "a documented-enum variant appended (EvidenceGrade)",
            Box::new(|s| {
                let variants = s["$defs"]["EvidenceGrade"]["oneOf"]
                    .as_array_mut()
                    .expect("EvidenceGrade is a `oneOf` of documented consts");
                variants.push(json!({"const": "audited", "type": "string"}));
            }),
        ),
        (
            "an enum token renamed",
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
        assert_ne!(
            structural_digest(&mutated),
            structural_digest(&golden),
            "mutation `{name}` kept the digest"
        );
    }
}

#[test]
fn structural_strips_doc_keywords_but_never_a_field_named_like_one() {
    let schema = json!({
        "title": "T",
        "description": "D",
        "type": "object",
        "properties": {
            "description": {"type": "string", "description": "the field's doc"},
            "title": {"type": "integer"}
        },
        "required": ["title", "description"],
        "$defs": {"title": {"const": {"description": "a value, not a keyword"}}}
    });
    let s = structural(&schema);
    assert!(s.get("title").is_none() && s.get("description").is_none());
    assert_eq!(s["properties"]["description"], json!({"type": "string"}));
    assert_eq!(s["properties"]["title"], json!({"type": "integer"}));
    assert_eq!(s["required"], json!(["description", "title"]));
    assert_eq!(
        s["$defs"]["title"]["const"],
        json!({"description": "a value, not a keyword"})
    );
    // Dropping the `description` FIELD is structural.
    let mut without = schema.clone();
    without["properties"]
        .as_object_mut()
        .unwrap()
        .remove("description");
    assert_ne!(structural(&without), s);
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

// ============================================================================
// Schema / validate() agreement
// ============================================================================

const V7: &str = "019a8f2e-7c3b-7d4e-8a1b-2c3d4e5f6a7b";

fn sample() -> Value {
    let part = |value: Value, grade: &str| json!({"value": value, "attribution_grade": grade});
    json!({
        "id": V7,
        "source": "coord.process_log",
        "source_seq": 42,
        "occurred_at": "2026-11-22T10:00:00Z",
        "recorded_at": "2026-11-22T10:00:01.250+00:00",
        "time_basis": "event",
        "activity": "worktree.allocate",
        "objects": [
            {"object_type": "worktree", "object_id": "wt-1", "qualifier": "allocated"}
        ],
        "principal": {
            "session": part(json!("s-1"), "observed"),
            "agent": part(Value::Null, "declared"),
            "device": part(json!("d-1"), "corroborated"),
            "tenant": part(json!("t-1"), "gate_grade"),
        },
        "evidence_grade": "observed",
        "source_class": "coord_observed",
        "world": {"id": "prod", "kind": "prod"},
        "correlation_id": "corr-1",
        "causation_id": null,
        "assignment": {
            "unit": "session:s-1",
            "arm": "treatment",
            "probability": 0.5,
            "covariates": {"device_class": "linux"},
            "policy_version": "premise-notice@1",
            "assignment_seed_id": "seed-2026-11-22"
        },
        "policy_version": null,
        "schema_version": 1,
        "prev_hash": null,
        "payload": {"kind": "inline", "value": {"note": "hi"}}
    })
}

/// `sample()` with one JSON pointer replaced (`None` removes the key).
fn with(pointer: &str, value: Option<Value>) -> Value {
    let mut v = sample();
    match value {
        Some(value) => match v.pointer_mut(pointer) {
            Some(slot) => *slot = value,
            None => {
                // A new key (an unknown field).
                let (parent, key) = pointer.rsplit_once('/').unwrap();
                v.pointer_mut(parent)
                    .and_then(Value::as_object_mut)
                    .unwrap_or_else(|| panic!("{pointer}"))
                    .insert(key.to_owned(), value);
            }
        },
        None => {
            let (parent, key) = pointer.rsplit_once('/').unwrap();
            v.pointer_mut(parent)
                .unwrap()
                .as_object_mut()
                .unwrap()
                .remove(key);
        }
    }
    v
}

fn corpus() -> Vec<(String, Value)> {
    let mut c: Vec<(String, Value)> = vec![("sample".into(), sample())];
    let mut set = |pointer: &str, value: Value| {
        c.push((format!("{pointer} = {value}"), with(pointer, Some(value))));
    };
    // ids
    for id in [
        V7,
        "019a8f2e-7c3b-4d4e-8a1b-2c3d4e5f6a7b",
        "019a8f2e-7c3b-1d4e-8a1b-2c3d4e5f6a7b",
        "019a8f2e7c3b7d4e8a1b2c3d4e5f6a7b",
        "019a8f2e-7c3b-7d4e-ca1b-2c3d4e5f6a7b",
        "019A8F2E-7C3B-7D4E-8A1B-2C3D4E5F6A7B",
        "019a8f2e-7c3b-7d4e-8a1b-2c3d4e5f6a7B",
        "not-a-uuid",
        "",
    ] {
        set("/id", json!(id));
    }
    // timestamps
    for t in [
        "2026-11-22T10:00:00Z",
        "2026-11-22T10:00:00.123456789Z",
        "2026-11-22T10:00:00+05:30",
        "2026-11-22T10:00:00-23:59",
        "2024-02-29T23:59:59Z",
        "2026-11-22 10:00:00",
        "2026-11-22T10:00:00",
        "yesterday",
        "2026-11-22 10:00:00Z",
        "2026-11-22t10:00:00z",
        "2026-11-22T10:00:00z",
        "2026-11-22T10:00:60Z",
        "2026-02-30T10:00:00Z",
        "2026-13-01T10:00:00Z",
        "2026-11-22T24:00:00Z",
        "2026-11-22T10:00:00.Z",
        "2026-11-22T10:00:00+24:00",
        "2026-11-22T10:00:00+0530",
    ] {
        set("/occurred_at", json!(t));
        set("/recorded_at", json!(t));
    }
    // activity
    for a in [
        "worktree.allocate",
        "train.occupancy_hourly",
        "ci.run2",
        "allocate",
        "worktree.",
        ".allocate",
        "Worktree.allocate",
        "worktree.allocate.extra",
        "work-tree.allocate",
        "1ci.run",
        "ci. run",
    ] {
        set("/activity", json!(a));
    }
    // A trailing newline, which Python's `$` would otherwise admit.
    set("/activity", json!("ci.run\n"));
    set("/id", json!(format!("{V7}\n")));
    set("/occurred_at", json!("2026-11-22T10:00:00Z\n"));
    set("/prev_hash", json!(format!("sha256-{}\n", "a".repeat(64))));
    set("/source", json!("x\n"));
    // Integers: negative, past u64::MAX, and the integer-valued float (a
    // DECLARED exception, see `DECLARED_EXCEPTIONS`).
    set("/source_seq", json!(-1));
    set("/source_seq", json!(u64::MAX));
    set("/source_seq", json!(18_446_744_073_709_551_616.0_f64));
    set("/source_seq", json!(1.0));
    set("/schema_version", json!(1.0));
    set("/schema_version", json!(-1));
    set("/schema_version", json!(4_294_967_297_u64));
    // probability
    for p in [0.0, -0.1, 1e-9, 0.5, 1.0, 1.000_001, 2.0] {
        set("/assignment/probability", json!(p));
    }
    set("/assignment", Value::Null);
    // schema version
    set("/schema_version", json!(2));
    set("/schema_version", json!(0));
    // hashes
    let good_hash = format!("sha256-{}", "a".repeat(64));
    for h in [
        good_hash.clone(),
        "abc".into(),
        format!("sha256-{}", "A".repeat(64)),
        format!("sha256-{}", "a".repeat(63)),
    ] {
        set("/prev_hash", json!(h));
        set(
            "/payload",
            json!({"kind": "erasable", "salted_hash": h, "store": "rack-pg.payloads"}),
        );
    }
    set(
        "/payload",
        json!({"kind": "erasable", "salted_hash": good_hash, "store": " s"}),
    );
    // labels, at every label site
    for site in [
        "/source",
        "/objects/0/object_type",
        "/objects/0/object_id",
        "/objects/0/qualifier",
        "/principal/session/value",
        "/world/id",
        "/correlation_id",
        "/causation_id",
        "/policy_version",
        "/assignment/unit",
        "/assignment/arm",
        "/assignment/policy_version",
        "/assignment/assignment_seed_id",
    ] {
        for label in [
            "ok",
            "a b",
            "a\u{FEFF}b",
            "",
            " x",
            "x ",
            "\u{FEFF}x",
            "x\u{3000}",
            "x\u{a0}",
            "a\nb",
            "a\u{85}b",
            "a\u{7f}b",
            "x\n",
        ] {
            set(site, json!(label));
        }
    }
    // closed enums
    set("/evidence_grade", json!("gate-grade"));
    set("/source_class", json!("coordObserved"));
    set("/time_basis", json!("wallclock"));
    set("/world/kind", json!("staging"));
    // unknown fields and a missing required key
    set("/extra", json!(1));
    set("/principal/agent/extra", json!(1));
    set("/objects/0/extra", json!(1));
    set("/payload/extra", json!(1));
    c.push(("assignment removed".into(), with("/assignment", None)));
    c.push(("prev_hash removed".into(), with("/prev_hash", None)));
    c.push((
        "principal.agent.value removed".into(),
        with("/principal/agent/value", None),
    ));
    c
}

/// The corpus cases where the schema and `validate()` are DOCUMENTED to
/// disagree (module docs of `qontinui_types::ledger`, "Integers are emitted
/// as JSON integers"): JSON Schema's `type: integer` admits an
/// integer-valued float and no keyword can refuse it, while Rust stays
/// strict. Each must disagree exactly that way (schema admits, Rust
/// refuses), and each must appear in the corpus.
const DECLARED_EXCEPTIONS: &[&str] = &["/source_seq = 1.0", "/schema_version = 1.0"];

fn rust_accepts(v: &Value) -> bool {
    serde_json::from_value::<LedgerEnvelope>(v.clone())
        .map(|e| validate(&e).is_ok())
        .unwrap_or(false)
}

#[test]
fn the_schema_and_validate_agree_on_the_corpus() {
    let schema = golden();
    let validator = jsonschema::options()
        .should_validate_formats(true)
        .build(&schema)
        .expect("the frozen schema compiles");
    let corpus = corpus();
    let mut accepted = 0;
    let mut disagreements = Vec::new();
    let mut exceptions_seen = Vec::new();
    for (name, instance) in &corpus {
        let by_schema = validator.is_valid(instance);
        let by_rust = rust_accepts(instance);
        if DECLARED_EXCEPTIONS.contains(&name.as_str()) {
            // Must disagree exactly as declared: schema admits, Rust refuses.
            assert!(
                by_schema && !by_rust,
                "declared exception `{name}` no longer behaves as documented: \
                 schema={by_schema} validate={by_rust}"
            );
            exceptions_seen.push(name.as_str());
        } else if by_schema != by_rust {
            disagreements.push(format!("{name}: schema={by_schema} validate={by_rust}"));
        }
        accepted += usize::from(by_rust);
    }
    assert!(
        disagreements.is_empty(),
        "the frozen schema and validate() disagree:\n{}",
        disagreements.join("\n")
    );
    exceptions_seen.sort_unstable();
    let mut declared = DECLARED_EXCEPTIONS.to_vec();
    declared.sort_unstable();
    assert_eq!(
        exceptions_seen, declared,
        "every declared exception is exercised"
    );
    // Guard against a vacuous corpus: both outcomes must be exercised.
    assert!(rust_accepts(&sample()));
    assert!(
        accepted > 20 && accepted < corpus.len() - 50,
        "accepted {accepted}/{}",
        corpus.len()
    );
}

// ============================================================================
// The committed patterns under Python's `re`
// ============================================================================

/// Python is a named non-Rust writer, and `re.search` differs from ECMA-262
/// in one way that matters here: `$` also matches before a trailing newline.
/// Run every committed pattern through `re.search` on cases including a
/// newline suffix. Needs `python3`; CI's rust-ci job has it (its
/// fleet-nouns step runs it), so under `CI` its absence fails rather than
/// skips.
#[test]
fn the_committed_patterns_behave_the_same_under_python_re() {
    use std::io::Write as _;
    use std::process::{Command, Stdio};

    let schema = golden();
    let pattern = |pointer: &str| {
        schema
            .pointer(pointer)
            .and_then(Value::as_str)
            .unwrap_or_else(|| panic!("no pattern at {pointer}"))
            .to_owned()
    };
    let id = pattern("/properties/id/pattern");
    let ts = pattern("/properties/occurred_at/pattern");
    let activity = pattern("/properties/activity/pattern");
    let hash = pattern("/properties/prev_hash/pattern");
    let label = pattern("/properties/source/pattern");
    let h = "a".repeat(64);
    let cases = json!([
        [id, V7, true],
        [id, format!("{V7}\n"), false],
        [id, V7.to_uppercase(), false],
        [ts, "2026-11-22T10:00:00Z", true],
        [ts, "2026-11-22T10:00:00.5+05:30", true],
        [ts, "2026-11-22T10:00:00Z\n", false],
        [ts, "2026-11-22t10:00:00z", false],
        [activity, "ci.run", true],
        [activity, "ci.run\n", false],
        [activity, "ci.run.x", false],
        [hash, format!("sha256-{h}"), true],
        [hash, format!("sha256-{h}\n"), false],
        [label, "x", true],
        [label, "a b", true],
        [label, "x\n", false],
        [label, " x", false],
        [label, "x\u{3000}", false],
        [label, "\u{FEFF}x", false],
        [label, "a\u{85}b", false],
    ]);
    let script = r#"
import json, re, sys
bad = []
for pattern, text, expected in json.load(sys.stdin):
    got = re.search(pattern, text) is not None
    if got != expected:
        bad.append([pattern, text, expected, got])
print(json.dumps(bad))
"#;
    let child = Command::new("python3")
        .args(["-I", "-c", script])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn();
    let mut child = match child {
        Ok(child) => child,
        Err(e) if std::env::var_os("CI").is_none() => {
            eprintln!("SKIPPED: python3 not runnable here ({e}); CI runs this test");
            return;
        }
        Err(e) => panic!("python3 is required under CI: {e}"),
    };
    child
        .stdin
        .take()
        .unwrap()
        .write_all(cases.to_string().as_bytes())
        .unwrap();
    let out = child.wait_with_output().unwrap();
    assert!(
        out.status.success(),
        "python3 failed: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    let bad: Value = serde_json::from_slice(&out.stdout).expect("python printed JSON");
    assert_eq!(bad, json!([]), "patterns disagree under Python re.search");
}
