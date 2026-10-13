//! Image-slot spec fixtures, authored as YAML the way a consumer repo writes
//! its specs, and the golden JSON Schema the bindings are generated from. Plan
//! `2026-10-13-image-generation-and-visual-judgment`, Phase 14.
//!
//! - `fixtures/image_slot/valid/<slot>.yaml` must parse, validate and survive
//!   YAML and JSON round trips unchanged. Each pins its own content hash on a
//!   `# content-sha256: <hex>` header line, so every valid fixture is a golden
//!   vector for another language's implementation of
//!   `ImageSlotSpec::content_sha256`. A bound fixture's
//!   `provenance.spec_sha256` is that same hash (a current binding), unless
//!   its header says `# binding: stale`: then the binding was made for an
//!   earlier version of the spec, the spec is still valid, and the same spec
//!   with its binding made current is refused. That pins the stale exemption a
//!   mirror of the binding obligations must honour.
//! - `fixtures/image_slot/invalid/*.yaml` each break exactly ONE rule, named on
//!   the first line:
//!   - `# refused-by: serde <text>`: deserialization fails, with `<text>` in
//!     the error;
//!   - `# refused-by: schema <path>`: it parses, and `validate` reports one
//!     rule the JSON Schema carries, at `<path>`;
//!   - `# refused-by: rule <path>`: it parses, and `validate` reports one
//!     cross-field rule, at `<path>`.
//!
//!   A validator generated from the schema (the pydantic model), validating
//!   strictly, refuses every `serde` and `schema` fixture; a `rule` fixture
//!   only one that also mirrors the cross-field rules by hand. In lax mode
//!   pydantic accepts `wrong-value-type.yaml` (it coerces `"yes"` to `true`);
//!   the `image_slot` module docs say how to validate strictly.
//! - `fixtures/image_slot/schema.json` is the whole `schema_for!(ImageSlotSpec)`
//!   output with its object keys sorted, compared byte for byte, so loosening
//!   any schema rule (a dropped `minItems`, `pattern` or range) fails here even
//!   where no fixture exercises it. To accept a deliberate schema change,
//!   regenerate it over the whole suite and review the diff:
//!
//!   ```text
//!   IMAGE_SLOT_SCHEMA_UPDATE=1 bash <qontinui-claude-config>/scripts/cargo-verify.sh \
//!     test -p qontinui-types
//!   ```
//!
//!   No test-name filter: a filtered run falls below cargo-verify's coverage
//!   floor and exits 3 after the snapshot is already rewritten. Only the value
//!   `1` regenerates; any other value, `0` included, compares as usual.

use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

use qontinui_types::image_slot::ImageSlotSpec;
use serde_json::Value;

const SCHEMA_UPDATE_VAR: &str = "IMAGE_SLOT_SCHEMA_UPDATE";

fn fixture_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/image_slot")
}

fn fixtures(kind: &str) -> Vec<PathBuf> {
    let dir = fixture_dir().join(kind);
    let mut paths: Vec<PathBuf> = fs::read_dir(&dir)
        .unwrap_or_else(|e| panic!("{}: {e}", dir.display()))
        .map(|entry| entry.unwrap().path())
        .filter(|path| path.extension().is_some_and(|ext| ext == "yaml"))
        .collect();
    paths.sort();
    paths
}

fn parse(path: &Path) -> Result<ImageSlotSpec, serde_yaml_ng::Error> {
    serde_yaml_ng::from_str(&fs::read_to_string(path).unwrap())
}

/// The hash a valid fixture pins on its `# content-sha256:` header line.
fn pinned_hash(path: &Path) -> String {
    let text = fs::read_to_string(path).unwrap();
    let pins: Vec<&str> = text
        .lines()
        .take_while(|line| line.starts_with('#'))
        .filter_map(|line| line.strip_prefix("# content-sha256: "))
        .collect();
    assert_eq!(
        pins.len(),
        1,
        "{}: the header must pin exactly one `# content-sha256: <hex>`",
        path.display()
    );
    pins[0].trim().to_owned()
}

#[test]
fn every_valid_fixture_parses_validates_and_round_trips() {
    let paths = fixtures("valid");
    assert!(paths.len() >= 6, "only {} valid fixtures", paths.len());
    for path in &paths {
        let name = path.display();
        let spec = parse(path).unwrap_or_else(|e| panic!("{name}: {e}"));
        spec.validate().unwrap_or_else(|e| panic!("{name}: {e}"));
        assert_eq!(
            path.file_stem().unwrap().to_str().unwrap(),
            spec.slot,
            "{name}: a fixture is named after its slot"
        );

        let yaml = serde_yaml_ng::to_string(&spec).unwrap();
        let again: ImageSlotSpec = serde_yaml_ng::from_str(&yaml).unwrap();
        assert_eq!(again, spec, "{name}: YAML round trip");
        assert_eq!(
            serde_yaml_ng::to_string(&again).unwrap(),
            yaml,
            "{name}: re-serialized YAML is not stable"
        );

        let json = serde_json::to_value(&spec).unwrap();
        let from_json: ImageSlotSpec = serde_json::from_value(json).unwrap();
        assert_eq!(from_json, spec, "{name}: JSON round trip");
    }
}

/// Whether a valid fixture's header declares its binding stale.
fn declares_stale_binding(path: &Path) -> bool {
    fs::read_to_string(path)
        .unwrap()
        .lines()
        .take_while(|line| line.starts_with('#'))
        .any(|line| line == "# binding: stale")
}

#[test]
fn every_valid_fixture_pins_its_hash_and_declares_its_binding() {
    let (mut current, mut stale) = (0, 0);
    let mut mismatches = Vec::new();
    for path in fixtures("valid") {
        let name = path.display();
        let spec = parse(&path).unwrap();
        let hash = spec.content_sha256().unwrap();
        let pinned = pinned_hash(&path);
        if pinned != hash {
            mismatches.push(format!("{name}: pins {pinned} but hashes to {hash}"));
        }
        match (&spec.provenance, declares_stale_binding(&path)) {
            (None, false) => {}
            (None, true) => {
                mismatches.push(format!("{name}: declares a stale binding but has none"))
            }
            (Some(binding), false) => {
                current += 1;
                if binding.spec_sha256 != hash {
                    mismatches.push(format!(
                        "{name}: binding records {} but the spec hashes to {hash}",
                        binding.spec_sha256
                    ));
                }
            }
            (Some(binding), true) => {
                stale += 1;
                if binding.spec_sha256 == hash {
                    mismatches.push(format!(
                        "{name}: declared stale, but the binding is current"
                    ));
                }
                // Made current, the same binding must break an obligation, or
                // the fixture would not exercise the stale exemption at all.
                let mut made_current = spec.clone();
                if let Some(binding) = &mut made_current.provenance {
                    binding.spec_sha256 = hash.clone();
                }
                if made_current.validate().is_ok() {
                    mismatches.push(format!(
                        "{name}: its binding breaks no obligation even when current"
                    ));
                }
            }
        }
    }
    assert!(
        current >= 4,
        "only {current} fixtures with a current binding"
    );
    assert!(stale >= 1, "no fixture with a stale binding");
    assert!(mismatches.is_empty(), "{}", mismatches.join("\n"));
}

#[test]
fn every_invalid_fixture_is_refused_for_its_declared_reason_alone() {
    let paths = fixtures("invalid");
    assert!(paths.len() >= 32, "only {} invalid fixtures", paths.len());
    let mut per_stage: BTreeMap<String, usize> = BTreeMap::new();
    for path in &paths {
        let name = path.display();
        let text = fs::read_to_string(path).unwrap();
        let header = text
            .lines()
            .next()
            .and_then(|line| line.strip_prefix("# refused-by: "))
            .unwrap_or_else(|| {
                panic!("{name}: the first line must be `# refused-by: <stage> <detail>`")
            });
        let (stage, detail) = header
            .split_once(' ')
            .unwrap_or_else(|| panic!("{name}: no detail in {header:?}"));
        *per_stage.entry(stage.to_owned()).or_default() += 1;

        let parsed = serde_yaml_ng::from_str::<ImageSlotSpec>(&text);
        match stage {
            "serde" => {
                let err = match parsed {
                    Ok(_) => panic!("{name}: parsed, but must be refused ({detail})"),
                    Err(err) => err.to_string(),
                };
                assert!(
                    err.contains(detail),
                    "{name}: refused for another reason: {err}"
                );
            }
            "schema" | "rule" => {
                let spec = parsed.unwrap_or_else(|e| panic!("{name}: must parse: {e}"));
                let refusal = spec
                    .validate()
                    .expect_err(&format!("{name}: validated, but must be refused"));
                assert_eq!(
                    refusal.issues.len(),
                    1,
                    "{name}: the fixture breaks more than one rule: {refusal}"
                );
                let issue = &refusal.issues[0];
                assert_eq!(issue.path, detail, "{name}: {refusal}");
                assert_eq!(
                    issue.problem.is_schema_rule(),
                    stage == "schema",
                    "{name}: {issue} is not a {stage} rule"
                );
            }
            other => panic!("{name}: unknown stage {other:?}"),
        }
    }
    for stage in ["serde", "schema", "rule"] {
        assert!(
            per_stage.get(stage).copied().unwrap_or(0) >= 10,
            "fewer than 10 `{stage}` fixtures: {per_stage:?}"
        );
    }
}

/// `value` with every object's keys in sorted order, whatever map type
/// serde_json was built with (feature unification can switch `preserve_order`
/// on for one invocation and off for another).
fn sorted_keys(value: &Value) -> Value {
    match value {
        Value::Object(map) => {
            let sorted: BTreeMap<&String, Value> =
                map.iter().map(|(k, v)| (k, sorted_keys(v))).collect();
            Value::Object(sorted.into_iter().map(|(k, v)| (k.clone(), v)).collect())
        }
        Value::Array(items) => Value::Array(items.iter().map(sorted_keys).collect()),
        other => other.clone(),
    }
}

/// A line diff of `expected` against `actual` (longest common subsequence),
/// `-` for a line only in `expected`, `+` for a line only in `actual`, with two
/// lines of context around each change.
fn line_diff(expected: &str, actual: &str) -> String {
    let a: Vec<&str> = expected.lines().collect();
    let b: Vec<&str> = actual.lines().collect();
    let mut lcs = vec![vec![0usize; b.len() + 1]; a.len() + 1];
    for i in (0..a.len()).rev() {
        for j in (0..b.len()).rev() {
            lcs[i][j] = if a[i] == b[j] {
                lcs[i + 1][j + 1] + 1
            } else {
                lcs[i + 1][j].max(lcs[i][j + 1])
            };
        }
    }
    let mut ops: Vec<(char, &str)> = Vec::new();
    let (mut i, mut j) = (0, 0);
    while i < a.len() || j < b.len() {
        if i < a.len() && j < b.len() && a[i] == b[j] {
            ops.push((' ', a[i]));
            i += 1;
            j += 1;
        } else if j < b.len() && (i == a.len() || lcs[i][j + 1] >= lcs[i + 1][j]) {
            ops.push(('+', b[j]));
            j += 1;
        } else {
            ops.push(('-', a[i]));
            i += 1;
        }
    }
    let near_change = |k: usize| {
        ops[k.saturating_sub(2)..(k + 3).min(ops.len())]
            .iter()
            .any(|(op, _)| *op != ' ')
    };
    let mut out = String::new();
    let mut gap = false;
    for (k, (op, line)) in ops.iter().enumerate() {
        if near_change(k) {
            if gap {
                out.push_str("  ...\n");
            }
            gap = false;
            out.push_str(&format!("{op} {line}\n"));
        } else {
            gap = true;
        }
    }
    out
}

#[test]
fn the_schema_matches_its_golden_snapshot() {
    let schema = serde_json::to_value(schemars::schema_for!(ImageSlotSpec)).unwrap();
    let actual = serde_json::to_string_pretty(&sorted_keys(&schema)).unwrap() + "\n";
    let path = fixture_dir().join("schema.json");
    if regenerate_requested(std::env::var(SCHEMA_UPDATE_VAR).ok().as_deref()) {
        fs::write(&path, &actual).unwrap();
        eprintln!(
            "{SCHEMA_UPDATE_VAR} is set: rewrote {}; review its diff",
            path.display()
        );
    }
    let expected = fs::read_to_string(&path).unwrap_or_else(|e| {
        panic!(
            "{}: {e}; generate it with {SCHEMA_UPDATE_VAR}=1 over the whole suite",
            path.display()
        )
    });
    assert!(
        actual == expected,
        "the ImageSlotSpec schema no longer matches {} (- snapshot, + now):\n{}\n\
         If the change is deliberate, regenerate with {SCHEMA_UPDATE_VAR}=1 over the whole suite \
         (this file's header has the command) and review the diff.",
        path.display(),
        line_diff(&expected, &actual)
    );
}

/// Regenerate the snapshot only when the variable is exactly `1`.
fn regenerate_requested(value: Option<&str>) -> bool {
    value == Some("1")
}

#[test]
fn regeneration_runs_only_on_exactly_one() {
    assert!(regenerate_requested(Some("1")));
    for value in [
        None,
        Some("0"),
        Some(""),
        Some("true"),
        Some(" 1"),
        Some("10"),
    ] {
        assert!(!regenerate_requested(value), "{value:?}");
    }
}

#[test]
fn the_snapshot_diff_names_the_changed_line() {
    let diff = line_diff("a\nb\nc\nd\n", "a\nb\nX\nd\n");
    assert!(diff.contains("- c\n") && diff.contains("+ X\n"), "{diff}");
    assert!(line_diff("same\n", "same\n").is_empty());
}
