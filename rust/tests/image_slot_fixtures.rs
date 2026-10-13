//! Image-slot spec fixtures, authored as YAML the way a consumer repo writes
//! its specs. Plan `2026-10-13-image-generation-and-visual-judgment`, Phase 14.
//!
//! - `fixtures/image_slot/valid/<slot>.yaml` must parse, validate and survive
//!   YAML and JSON round trips unchanged. A bound fixture's
//!   `provenance.spec_sha256` is the spec's own content hash, so each one is
//!   also a golden vector for another language's implementation of that hash.
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

use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

use qontinui_types::image_slot::ImageSlotSpec;

fn fixtures(kind: &str) -> Vec<PathBuf> {
    let dir = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/image_slot")
        .join(kind);
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

#[test]
fn every_valid_fixture_parses_validates_and_round_trips() {
    let paths = fixtures("valid");
    assert!(paths.len() >= 4, "only {} valid fixtures", paths.len());
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

#[test]
fn every_bound_fixture_is_bound_to_its_own_content_hash() {
    let mut bound = 0;
    let mut mismatches = Vec::new();
    for path in fixtures("valid") {
        let spec = parse(&path).unwrap();
        if let Some(provenance) = &spec.provenance {
            bound += 1;
            let hash = spec.content_sha256().unwrap();
            if provenance.spec_sha256 != hash {
                mismatches.push(format!(
                    "{}: records {} but hashes to {hash}",
                    path.display(),
                    provenance.spec_sha256
                ));
            }
        }
    }
    assert!(bound >= 2, "only {bound} bound fixtures");
    assert!(mismatches.is_empty(), "{}", mismatches.join("\n"));
}

#[test]
fn every_invalid_fixture_is_refused_for_its_declared_reason_alone() {
    let paths = fixtures("invalid");
    assert!(paths.len() >= 24, "only {} invalid fixtures", paths.len());
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
            per_stage.get(stage).copied().unwrap_or(0) >= 5,
            "fewer than 5 `{stage}` fixtures: {per_stage:?}"
        );
    }
}
