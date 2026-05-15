//! Corpus integration test — Phase 6.0 scaffolding.
//!
//! Walks `tests/fixtures/analyzer_corpus/{frames,golden}/`, loads each
//! pair, and verifies the Rust analyzers + assertion DSL match the
//! expected output shape. See the directory's README.md for fixture
//! semantics + how to add cases.
//!
//! The gate is intentionally lenient — analyzers may surface extra
//! `info` findings without breaking the test, but they must emit at
//! least one of the named `kinds` for the layout/elements bucket the
//! fixture is checking. Assertions are exact (pass/fail must match).
//!
//! When the corpus grows past hand-crafted fixtures (see
//! `scripts/regenerate_vision_corpus.py` in qontinui/) and Python
//! ground truth is available, this same test reads them with no
//! changes — the goldens just become finer-grained.

use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

use qontinui_vision_core::{
    analyzers, AnalyzeInput, Analyzer, Assertion, ElementSnapshot, EvalContext,
};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct GoldenFile {
    #[serde(default)]
    analyzers: HashMap<String, AnalyzerExpectation>,
    #[serde(default)]
    assertions: Vec<AssertionExpectation>,
}

#[derive(Debug, Deserialize)]
struct AnalyzerExpectation {
    /// Required finding-kinds — the analyzer's output must include each
    /// of these (any severity).
    #[serde(default)]
    kinds: Vec<String>,
    /// Upper bound on total findings the analyzer is allowed to surface.
    /// Catches "the analyzer started over-firing" regressions.
    #[serde(default)]
    count_min: usize,
    #[serde(default = "default_count_max")]
    count_max: usize,
}

fn default_count_max() -> usize {
    100
}

#[derive(Debug, Deserialize)]
struct AssertionExpectation {
    assertion: Assertion,
    expect_passed: bool,
}

fn fixture_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
        .join("analyzer_corpus")
}

fn list_fixtures() -> Vec<(String, PathBuf, PathBuf)> {
    let frames = fixture_dir().join("frames");
    let golden = fixture_dir().join("golden");
    let mut out = Vec::new();
    let entries = match fs::read_dir(&frames) {
        Ok(e) => e,
        Err(_) => return out,
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.extension().map(|s| s != "json").unwrap_or(true) {
            continue;
        }
        let stem = path
            .file_stem()
            .and_then(|s| s.to_str())
            .map(String::from)
            .unwrap_or_default();
        let golden_path = golden.join(format!("{stem}.json"));
        if golden_path.exists() {
            out.push((stem, path, golden_path));
        }
    }
    out.sort_by(|a, b| a.0.cmp(&b.0));
    out
}

fn parse_analyzer(name: &str) -> Option<Analyzer> {
    match name {
        "layout" => Some(Analyzer::Layout),
        "typography" => Some(Analyzer::Typography),
        "color" => Some(Analyzer::Color),
        "dynamic" => Some(Analyzer::Dynamic),
        "elements" => Some(Analyzer::Elements),
        _ => None,
    }
}

fn check_one(stem: &str, frame_path: &Path, golden_path: &Path) -> Result<(), String> {
    let frame_text =
        fs::read_to_string(frame_path).map_err(|e| format!("[{stem}] read frame: {e}"))?;
    let snapshot: ElementSnapshot =
        serde_json::from_str(&frame_text).map_err(|e| format!("[{stem}] parse frame: {e}"))?;
    let golden_text =
        fs::read_to_string(golden_path).map_err(|e| format!("[{stem}] read golden: {e}"))?;
    let golden: GoldenFile =
        serde_json::from_str(&golden_text).map_err(|e| format!("[{stem}] parse golden: {e}"))?;

    let input = AnalyzeInput {
        frame: None,
        snapshot: Some(&snapshot),
        prior_frame: None,
    };

    for (analyzer_name, expectation) in &golden.analyzers {
        let analyzer = parse_analyzer(analyzer_name)
            .ok_or_else(|| format!("[{stem}] unknown analyzer '{analyzer_name}'"))?;
        let findings = analyzers::run(analyzer, &input);
        let count = findings.len();
        if count < expectation.count_min || count > expectation.count_max {
            return Err(format!(
                "[{stem}] {analyzer_name}: got {count} findings, expected [{}, {}]. Findings: {:?}",
                expectation.count_min,
                expectation.count_max,
                findings.iter().map(|f| &f.kind).collect::<Vec<_>>()
            ));
        }
        for required_kind in &expectation.kinds {
            if !findings.iter().any(|f| f.kind == *required_kind) {
                return Err(format!(
                    "[{stem}] {analyzer_name}: missing required kind '{required_kind}'. Got: {:?}",
                    findings.iter().map(|f| &f.kind).collect::<Vec<_>>()
                ));
            }
        }
    }

    let ctx = EvalContext {
        snapshot: Some(&snapshot),
        ..Default::default()
    };
    for (idx, exp) in golden.assertions.iter().enumerate() {
        let result = qontinui_vision_core::evaluate_assertion(&exp.assertion, &ctx);
        if result.passed != exp.expect_passed {
            return Err(format!(
                "[{stem}] assertion[{idx}] ({:?}): expected passed={}, got passed={} detail={:?}",
                exp.assertion, exp.expect_passed, result.passed, result.detail
            ));
        }
    }

    Ok(())
}

#[test]
fn analyzer_corpus_holds() {
    let fixtures = list_fixtures();
    assert!(
        !fixtures.is_empty(),
        "expected at least one fixture in tests/fixtures/analyzer_corpus/frames/"
    );
    let mut failures = Vec::new();
    for (stem, frame, golden) in &fixtures {
        if let Err(e) = check_one(stem, frame, golden) {
            failures.push(e);
        }
    }
    assert!(
        failures.is_empty(),
        "{} of {} fixtures failed:\n{}",
        failures.len(),
        fixtures.len(),
        failures.join("\n")
    );
    eprintln!(
        "corpus: {} fixtures pass against analyzers + assertion DSL",
        fixtures.len()
    );
}
