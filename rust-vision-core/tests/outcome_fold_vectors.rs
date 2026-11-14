// SPDX-License-Identifier: MIT
//! Runs every golden vector in `tests/vectors/outcome-fold/` through this
//! crate's [`fold_outcomes`]. `qontinui-types` runs the same files through its
//! twin (`rust/tests/outcome_fold_vectors.rs`), so the two Rust copies of the
//! fold cannot drift apart without one of them going red.

use std::collections::BTreeSet;
use std::path::PathBuf;

use qontinui_vision_core::{fold_outcomes, Verdict};
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
    let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/vectors/outcome-fold");
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
