//! Runs every golden vector in `rust-vision-core/tests/vectors/outcome-fold/`
//! through this crate's [`fold_outcomes`]. `qontinui-vision-core` runs the same
//! files through its twin, so the two Rust copies of the fold cannot drift
//! apart without one of them going red. Excluded from the published crate (it
//! reads outside the package).

use std::collections::BTreeSet;
use std::path::PathBuf;

use qontinui_types::outcome::{fold_outcomes, Verdict};
use serde_json::Value;

/// The cases the plan names. The test refuses a directory missing any of
/// them, so a deleted vector cannot read as a passing run.
const REQUIRED: [&str; 8] = [
    "empty",
    "passed",
    "failed",
    "unknown",
    "failed-unknown",
    "unknown-passed",
    "passed-x3",
    "failed-passed",
];

#[test]
fn every_outcome_fold_vector_holds() {
    let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../rust-vision-core/tests/vectors/outcome-fold");
    let mut seen = BTreeSet::new();
    let mut entries: Vec<_> = std::fs::read_dir(&dir)
        .unwrap_or_else(|e| panic!("read {}: {e}", dir.display()))
        .map(|e| e.unwrap().path())
        .filter(|p| p.extension().is_some_and(|x| x == "json"))
        .collect();
    entries.sort();
    for path in entries {
        let name = path.file_stem().unwrap().to_string_lossy().into_owned();
        let doc: Value = serde_json::from_slice(&std::fs::read(&path).unwrap())
            .unwrap_or_else(|e| panic!("{name}: not JSON: {e}"));
        let input: Vec<Verdict> = serde_json::from_value(doc["input"].clone())
            .unwrap_or_else(|e| panic!("{name}: input: {e}"));
        let got = serde_json::to_value(fold_outcomes(&input)).unwrap();
        assert_eq!(got, doc["expected"], "vector {name}");
        seen.insert(name);
    }
    for r in REQUIRED {
        assert!(
            seen.contains(r),
            "vector {r}.json is missing from {}",
            dir.display()
        );
    }
}

/// The vectors pin behaviour; this pins the rest — serde attributes, the
/// hand-written JSON Schema, the wire checks — by requiring the marked region
/// of the two Rust copies to be byte-identical. An edit to one copy alone
/// (say, `deny_unknown_fields` on one wire struct) goes red here.
#[test]
fn the_two_rust_copies_of_the_fold_are_byte_identical() {
    fn region(rel: &str) -> String {
        let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(rel);
        let text = std::fs::read_to_string(&path)
            .unwrap_or_else(|e| panic!("read {}: {e}", path.display()));
        let begin = text
            .find("// --- twin: begin")
            .unwrap_or_else(|| panic!("{rel}: no twin-begin marker"));
        let end = text
            .find("// --- twin: end ---")
            .unwrap_or_else(|| panic!("{rel}: no twin-end marker"));
        assert!(begin < end, "{rel}: twin markers out of order");
        text[begin..end].to_string()
    }
    let types = region("src/outcome.rs");
    let vision = region("../rust-vision-core/src/assertions/fold.rs");
    assert!(
        types.contains("pub fn fold_outcomes("),
        "twin region lost the fold"
    );
    assert!(
        types == vision,
        "the twinned region of rust/src/outcome.rs and \
         rust-vision-core/src/assertions/fold.rs has diverged; make the same edit in both"
    );
}
