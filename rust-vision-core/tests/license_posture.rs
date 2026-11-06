// SPDX-License-Identifier: MIT
//! This crate is MIT inside an AGPL-3.0-or-later workspace. That exception
//! holds only while the facts `CONTRIBUTING.md` states about it stay true, so
//! these tests check them.
//!
//! The header check exists because it already drifted once: the relicense commit
//! missed five source files, and six more landed without the header within a week
//! of it.

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

const SPDX_HEADER: &str = "// SPDX-License-Identifier: MIT";
const CRATES_IO: &str = "registry+https://github.com/rust-lang/crates.io-index";

fn crate_root() -> &'static Path {
    Path::new(env!("CARGO_MANIFEST_DIR"))
}

fn collect_rust_sources(dir: &Path, sources: &mut Vec<PathBuf>) {
    for entry in fs::read_dir(dir).unwrap_or_else(|e| panic!("read {}: {e}", dir.display())) {
        let path = entry.expect("read directory entry").path();
        if path == crate_root().join("target") {
            continue;
        }
        if path.is_dir() {
            collect_rust_sources(&path, sources);
        } else if path.extension().is_some_and(|ext| ext == "rs") {
            sources.push(path);
        }
    }
}

#[test]
fn every_rust_source_starts_with_the_mit_spdx_header() {
    let mut sources = Vec::new();
    collect_rust_sources(crate_root(), &mut sources);

    // A walk that found nothing would pass the check below, so name files that
    // must have been seen.
    for expected in [
        "src/lib.rs",
        "src/bin/vision_audit.rs",
        "tests/license_posture.rs",
    ] {
        assert!(
            sources.contains(&crate_root().join(expected)),
            "the source walk did not reach {expected}"
        );
    }

    let missing: Vec<PathBuf> = sources
        .into_iter()
        .filter(|path| {
            let text =
                fs::read_to_string(path).unwrap_or_else(|e| panic!("read {}: {e}", path.display()));
            text.lines().next() != Some(SPDX_HEADER)
        })
        .collect();
    assert!(
        missing.is_empty(),
        "these files must have `{SPDX_HEADER}` as their first line: {missing:#?}"
    );
}

#[test]
fn the_manifest_says_mit_and_the_license_file_is_mit() {
    assert_eq!(env!("CARGO_PKG_LICENSE"), "MIT");

    let license = fs::read_to_string(crate_root().join("LICENSE")).expect("read LICENSE");
    assert!(
        license.starts_with("MIT License")
            && license.contains("Permission is hereby granted, free of charge"),
        "rust-vision-core/LICENSE must hold the MIT text"
    );
}

/// Asks cargo rather than parsing `Cargo.toml`, so that a dependency inherited
/// from `[workspace.dependencies]`, or declared under a `[target.*]` table, is
/// seen with its real source.
///
/// The name check catches a crate of this workspace that declares crates.io as
/// its source: a published version of it, or one `[patch]`ed onto its local path.
/// Neither shows in `source`. This test cannot see a published AGPL crate under
/// any other name, a `[patch]` on any other crate, or transitive dependencies.
#[test]
fn dependencies_come_from_crates_io_and_from_outside_this_workspace() {
    let output = Command::new(env!("CARGO"))
        .args(["metadata", "--format-version", "1", "--no-deps"])
        .arg("--manifest-path")
        .arg(crate_root().join("Cargo.toml"))
        .output()
        .expect("run cargo metadata");
    assert!(
        output.status.success(),
        "cargo metadata failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let metadata: serde_json::Value =
        serde_json::from_slice(&output.stdout).expect("parse cargo metadata output");
    let packages = metadata["packages"]
        .as_array()
        .expect("metadata lists packages");
    assert!(
        packages.len() > 1,
        "metadata did not list the other workspace members, so the name check would pass without looking"
    );
    let workspace_crates: Vec<&serde_json::Value> =
        packages.iter().map(|package| &package["name"]).collect();

    let package = packages
        .iter()
        .find(|package| package["name"] == env!("CARGO_PKG_NAME"))
        .expect("metadata lists this crate");
    let dependencies = package["dependencies"]
        .as_array()
        .expect("package lists dependencies");
    assert!(
        !dependencies.is_empty(),
        "metadata reported no dependencies, so this check would pass without looking"
    );

    let forbidden: Vec<String> = dependencies
        .iter()
        .filter(|dep| dep["source"] != CRATES_IO || workspace_crates.contains(&&dep["name"]))
        .map(|dep| format!("{} (source: {})", dep["name"], dep["source"]))
        .collect();
    assert!(
        forbidden.is_empty(),
        "qontinui-vision-core may only depend on crates.io crates from outside this \
         workspace. Anything else can pull AGPL code into an MIT crate, and the crate \
         could then no longer be distributed under MIT. Forbidden: {forbidden:#?}"
    );
}
