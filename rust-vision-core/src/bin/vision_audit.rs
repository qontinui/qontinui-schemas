//! `vision-audit` — standalone, runner-less analyzer CLI for CI style gating.
//!
//! A hermetic, version-pinned wrapper over `qontinui-vision-core`'s analyzer +
//! assertion API. It runs the SAME library code the runner's `/vision/*` HTTP
//! endpoints call, but as a plain binary with no qontinui-runner process — so a
//! CI job can gate a build on layout/typography/color/element findings or on a
//! declarative assertion suite without standing up the full stack.
//!
//! Machine-readable JSON goes to **stdout**; a human-readable summary goes to
//! **stderr**. Exit codes are the contract CI keys off:
//!
//! | exit | meaning                                                       |
//! |------|--------------------------------------------------------------|
//! | 0    | OK (analyze: ran; or no finding at/above `--fail-on`. assert: allPassed) |
//! | 2    | GATE FAILED (a finding at/above `--fail-on`, or an assertion failed) |
//! | 1    | usage / IO / parse error                                      |
//!
//! Modes:
//!   * `vision-audit analyze  --snapshot S --frame F [--analyzer A] [--fail-on L]`
//!   * `vision-audit assert   --snapshot S --frame F --assertions A [--baseline-dir D]`
//!   * `vision-audit baseline --snapshot S --name N --baseline-dir D`
//!
//! SCOPE: OCR / VLM clients live runner-side; this bin runs the snapshot-text
//! path only and emits an `ocr_unavailable` note rather than pretending to OCR.

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

use qontinui_vision_core::analyzers::{self, AnalyzeInput, Analyzer, Finding, Severity};
use qontinui_vision_core::assertions::{
    evaluate as evaluate_assertion, Assertion, AssertionResult, BaselineEntry, EvalContext,
};
use qontinui_vision_core::element_snapshot::ElementSnapshot;
use qontinui_vision_core::frame::{Frame, FrameSource};

// ===========================================================================
// Exit-code contract (centralized so tests can reference the same constants).
// ===========================================================================

const EXIT_OK: u8 = 0;
const EXIT_USAGE: u8 = 1;
const EXIT_GATE_FAILED: u8 = 2;

// ===========================================================================
// main — thin. All real logic lives in testable functions below.
// ===========================================================================

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    match run(&args) {
        Ok(code) => ExitCode::from(code),
        Err(CliError::Usage(msg)) => {
            eprintln!("error: {msg}\n");
            eprintln!("{USAGE}");
            ExitCode::from(EXIT_USAGE)
        }
        Err(CliError::Io(msg)) => {
            eprintln!("error: {msg}");
            ExitCode::from(EXIT_USAGE)
        }
    }
}

const USAGE: &str = "\
vision-audit — runner-less vision analyzer / assertion gate for CI

USAGE:
    vision-audit analyze  --snapshot <file> --frame <file> [--analyzer <a>] [--fail-on <level>]
    vision-audit assert   --snapshot <file> --frame <file> --assertions <file> [--baseline-dir <dir>]
    vision-audit baseline --snapshot <file> --name <name> --baseline-dir <dir>

ANALYZE:
    --analyzer   layout | typography | color | elements | all   (default: all)
    --fail-on    warning | critical                             (default: never fail on findings)
    Exit 2 if any finding is at/above --fail-on; otherwise 0.

ASSERT:
    --assertions JSON array of assertion objects ({\"type\":\"no_overlap\", ...}).
    --baseline-dir  dir of <name>.json baselines for no_layout_shift_since.
    Exit 0 iff every assertion passed; else 2.

BASELINE:
    Serialize the snapshot's element bboxes to <baseline-dir>/<name>.json so a
    later `assert` run can evaluate no_layout_shift_since against it.

OUTPUT: machine JSON -> stdout, human summary -> stderr.
EXIT:   0 ok / gate passed, 2 gate failed, 1 usage|io|parse error.

NOTE: OCR/VLM run runner-side only; this bin uses the snapshot-text path and
      reports \"ocr_unavailable\" rather than erroring on OCR-dependent checks.";

// ===========================================================================
// Errors
// ===========================================================================

#[derive(Debug)]
enum CliError {
    /// Bad/missing flags, bad mode — exit 1 + print usage.
    Usage(String),
    /// IO or parse failure — exit 1, no usage dump.
    Io(String),
}

impl CliError {
    fn usage(m: impl Into<String>) -> Self {
        CliError::Usage(m.into())
    }
    fn io(m: impl Into<String>) -> Self {
        CliError::Io(m.into())
    }
}

// ===========================================================================
// Top-level dispatch
// ===========================================================================

fn run(args: &[String]) -> Result<u8, CliError> {
    let mode = args
        .first()
        .ok_or_else(|| CliError::usage("missing subcommand"))?
        .as_str();
    let rest = &args[1..];
    match mode {
        "analyze" => run_analyze(rest),
        "assert" => run_assert(rest),
        "baseline" => run_baseline(rest),
        "-h" | "--help" | "help" => {
            eprintln!("{USAGE}");
            Ok(EXIT_OK)
        }
        other => Err(CliError::usage(format!("unknown subcommand: {other:?}"))),
    }
}

// ===========================================================================
// Minimal argument parser (no external dep).
//
// Supports `--flag value` and `--flag=value`. Returns a map of flag->value.
// Unknown flags are rejected so a typo can't silently disable a gate.
// ===========================================================================

fn parse_flags(args: &[String], allowed: &[&str]) -> Result<HashMap<String, String>, CliError> {
    let mut out = HashMap::new();
    let mut i = 0;
    while i < args.len() {
        let arg = &args[i];
        if let Some(stripped) = arg.strip_prefix("--") {
            let (key, inline_val) = match stripped.split_once('=') {
                Some((k, v)) => (k.to_string(), Some(v.to_string())),
                None => (stripped.to_string(), None),
            };
            if !allowed.contains(&key.as_str()) {
                return Err(CliError::usage(format!("unknown flag: --{key}")));
            }
            let value = match inline_val {
                Some(v) => v,
                None => {
                    i += 1;
                    args.get(i)
                        .cloned()
                        .ok_or_else(|| CliError::usage(format!("flag --{key} needs a value")))?
                }
            };
            out.insert(key, value);
        } else {
            return Err(CliError::usage(format!(
                "unexpected positional arg: {arg:?}"
            )));
        }
        i += 1;
    }
    Ok(out)
}

fn require<'a>(flags: &'a HashMap<String, String>, key: &str) -> Result<&'a String, CliError> {
    flags
        .get(key)
        .ok_or_else(|| CliError::usage(format!("missing required flag --{key}")))
}

// ===========================================================================
// Snapshot loading + envelope unwrap
// ===========================================================================

/// Parse an `ElementSnapshot` from raw JSON bytes, transparently unwrapping the
/// common bridge `discover` envelopes so a CI script can pipe
/// `curl .../discover | jq .data` (or even the raw response) straight in:
///
///   * `{ "elements": [...] }`         -> the snapshot itself
///   * `{ "data": { "elements": [...] } }` -> unwrap `.data`
///   * `{ "data": [...] }`             -> `.data` is the element array
///   * `[ ... ]`                       -> a bare element array
pub fn parse_snapshot(bytes: &[u8]) -> Result<ElementSnapshot, String> {
    let value: serde_json::Value =
        serde_json::from_slice(bytes).map_err(|e| format!("snapshot is not valid JSON: {e}"))?;
    let inner = unwrap_snapshot_envelope(value);
    serde_json::from_value(inner)
        .map_err(|e| format!("snapshot does not match ElementSnapshot: {e}"))
}

/// Peel a single `{data: ...}` wrapper and normalize a bare element array into
/// `{elements: [...]}`. Conservative: only unwraps shapes we recognize.
fn unwrap_snapshot_envelope(value: serde_json::Value) -> serde_json::Value {
    use serde_json::Value;
    // Bare array -> {elements: [...]}.
    if value.is_array() {
        return serde_json::json!({ "elements": value });
    }
    if let Value::Object(map) = &value {
        // Already an ElementSnapshot.
        if map.contains_key("elements") {
            return value;
        }
        // {data: <snapshot|array>} bridge envelope — unwrap one level, then
        // re-normalize (data may itself be a bare array).
        if let Some(data) = map.get("data") {
            return unwrap_snapshot_envelope(data.clone());
        }
    }
    value
}

fn load_snapshot(path: &Path) -> Result<ElementSnapshot, CliError> {
    let bytes = std::fs::read(path)
        .map_err(|e| CliError::io(format!("cannot read snapshot {}: {e}", path.display())))?;
    parse_snapshot(&bytes).map_err(CliError::io)
}

// ===========================================================================
// Frame loading (PNG/JPEG/WebP via the `image` dep the analyzers already use)
// ===========================================================================

fn load_frame(path: &Path) -> Result<Frame, CliError> {
    let bytes = std::fs::read(path)
        .map_err(|e| CliError::io(format!("cannot read frame {}: {e}", path.display())))?;
    let decoded = image::load_from_memory(&bytes)
        .map_err(|e| CliError::io(format!("cannot decode frame {}: {e}", path.display())))?;
    Ok(Frame::from_rgba(
        decoded.to_rgba8(),
        FrameSource::synthetic_now(),
    ))
}

// ===========================================================================
// Severity / fail-on helpers
// ===========================================================================

/// Numeric rank so we can compare a finding's severity to the `--fail-on`
/// threshold. Higher = worse.
fn severity_rank(s: Severity) -> u8 {
    match s {
        Severity::Info => 0,
        Severity::Warning => 1,
        Severity::Critical => 2,
    }
}

/// Parse a `--fail-on` value into a threshold rank. `None` means "never fail on
/// findings" (the analyze default).
pub fn parse_fail_on(value: Option<&str>) -> Result<Option<u8>, String> {
    match value {
        None => Ok(None),
        Some("warning") => Ok(Some(severity_rank(Severity::Warning))),
        Some("critical") => Ok(Some(severity_rank(Severity::Critical))),
        Some(other) => Err(format!(
            "--fail-on must be 'warning' or 'critical', got {other:?}"
        )),
    }
}

fn parse_analyzer(value: Option<&str>) -> Result<Vec<Analyzer>, String> {
    match value.unwrap_or("all") {
        "all" => Ok(vec![
            Analyzer::Layout,
            Analyzer::Typography,
            Analyzer::Color,
            Analyzer::Elements,
        ]),
        "layout" => Ok(vec![Analyzer::Layout]),
        "typography" => Ok(vec![Analyzer::Typography]),
        "color" => Ok(vec![Analyzer::Color]),
        "elements" => Ok(vec![Analyzer::Elements]),
        // `dynamic` needs a prior frame the bin doesn't take; surface that.
        "dynamic" => Err(
            "the 'dynamic' analyzer needs two frames and is not supported by vision-audit"
                .to_string(),
        ),
        other => Err(format!(
            "--analyzer must be one of layout|typography|color|elements|all, got {other:?}"
        )),
    }
}

// ===========================================================================
// analyze
// ===========================================================================

fn run_analyze(args: &[String]) -> Result<u8, CliError> {
    let flags = parse_flags(args, &["snapshot", "frame", "analyzer", "fail-on"])?;
    let snapshot = load_snapshot(Path::new(require(&flags, "snapshot")?))?;
    let frame = load_frame(Path::new(require(&flags, "frame")?))?;
    let analyzers_to_run =
        parse_analyzer(flags.get("analyzer").map(String::as_str)).map_err(CliError::usage)?;
    let fail_on =
        parse_fail_on(flags.get("fail-on").map(String::as_str)).map_err(CliError::usage)?;

    let report = analyze(&snapshot, &frame, &analyzers_to_run);
    let exit = analyze_exit_code(&report, fail_on);

    println!(
        "{}",
        serde_json::to_string_pretty(&report).map_err(|e| CliError::io(e.to_string()))?
    );
    eprintln!("{}", analyze_summary(&report, fail_on, exit));
    Ok(exit)
}

/// Machine-readable analyze output.
#[derive(Debug, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AnalyzeReport {
    /// Per-analyzer findings, keyed by wire-name (`"layout"`, ...).
    findings: HashMap<String, Vec<Finding>>,
    /// Total finding count across all analyzers.
    total: usize,
    /// Counts by severity (`"info"`/`"warning"`/`"critical"`).
    counts: HashMap<String, usize>,
    /// OCR/VLM are runner-side only; flagged so CI knows OCR checks did not run.
    notes: Vec<String>,
}

/// Pure: run the requested analyzers over the snapshot+frame and tally.
pub fn analyze(snapshot: &ElementSnapshot, frame: &Frame, which: &[Analyzer]) -> AnalyzeReport {
    let input = AnalyzeInput {
        frame: Some(frame),
        snapshot: Some(snapshot),
        prior_frame: None,
    };
    let mut findings = HashMap::new();
    let mut counts: HashMap<String, usize> = HashMap::new();
    let mut total = 0usize;
    for &a in which {
        let fs = analyzers::run(a, &input);
        for f in &fs {
            total += 1;
            *counts
                .entry(severity_name(f.severity).to_string())
                .or_insert(0) += 1;
        }
        findings.insert(a.name().to_string(), fs);
    }
    AnalyzeReport {
        findings,
        total,
        counts,
        notes: vec!["ocr_unavailable: OCR/VLM run runner-side only; \
                     contains_text falls back to snapshot text"
            .to_string()],
    }
}

fn severity_name(s: Severity) -> &'static str {
    match s {
        Severity::Info => "info",
        Severity::Warning => "warning",
        Severity::Critical => "critical",
    }
}

/// Pure: map an analyze report + threshold to an exit code.
pub fn analyze_exit_code(report: &AnalyzeReport, fail_on: Option<u8>) -> u8 {
    let Some(threshold) = fail_on else {
        return EXIT_OK;
    };
    let tripped = report
        .findings
        .values()
        .flatten()
        .any(|f| severity_rank(f.severity) >= threshold);
    if tripped {
        EXIT_GATE_FAILED
    } else {
        EXIT_OK
    }
}

fn analyze_summary(report: &AnalyzeReport, fail_on: Option<u8>, exit: u8) -> String {
    let mut lines = Vec::new();
    lines.push(format!(
        "vision-audit analyze: {} finding(s) [info={} warning={} critical={}]",
        report.total,
        report.counts.get("info").copied().unwrap_or(0),
        report.counts.get("warning").copied().unwrap_or(0),
        report.counts.get("critical").copied().unwrap_or(0),
    ));
    match fail_on {
        None => lines.push("gate: --fail-on not set; exit 0 regardless of findings".to_string()),
        Some(t) => {
            let level = if t >= severity_rank(Severity::Critical) {
                "critical"
            } else {
                "warning"
            };
            lines.push(format!(
                "gate: --fail-on {level} -> {}",
                if exit == EXIT_GATE_FAILED {
                    "FAILED (exit 2)"
                } else {
                    "passed (exit 0)"
                }
            ));
        }
    }
    lines.join("\n")
}

// ===========================================================================
// assert
// ===========================================================================

fn run_assert(args: &[String]) -> Result<u8, CliError> {
    let flags = parse_flags(args, &["snapshot", "frame", "assertions", "baseline-dir"])?;
    let snapshot = load_snapshot(Path::new(require(&flags, "snapshot")?))?;
    let frame = load_frame(Path::new(require(&flags, "frame")?))?;
    let assertions = load_assertions(Path::new(require(&flags, "assertions")?))?;
    let baselines = match flags.get("baseline-dir") {
        Some(dir) => load_baselines(Path::new(dir))?,
        None => HashMap::new(),
    };

    let report = assert_all(&snapshot, &frame, &assertions, &baselines);
    let exit = if report.all_passed {
        EXIT_OK
    } else {
        EXIT_GATE_FAILED
    };

    println!(
        "{}",
        serde_json::to_string_pretty(&report).map_err(|e| CliError::io(e.to_string()))?
    );
    eprintln!("{}", assert_summary(&report));
    Ok(exit)
}

fn load_assertions(path: &Path) -> Result<Vec<Assertion>, CliError> {
    let bytes = std::fs::read(path)
        .map_err(|e| CliError::io(format!("cannot read assertions {}: {e}", path.display())))?;
    parse_assertions(&bytes).map_err(CliError::io)
}

/// Parse an assertions file: a JSON array of assertion objects, or a
/// `{ "assertions": [...] }` wrapper.
pub fn parse_assertions(bytes: &[u8]) -> Result<Vec<Assertion>, String> {
    let value: serde_json::Value =
        serde_json::from_slice(bytes).map_err(|e| format!("assertions is not valid JSON: {e}"))?;
    let arr = match value {
        serde_json::Value::Array(_) => value,
        serde_json::Value::Object(ref map) if map.contains_key("assertions") => {
            map.get("assertions").cloned().unwrap()
        }
        _ => {
            return Err("assertions must be a JSON array (or {\"assertions\": [...]})".to_string())
        }
    };
    serde_json::from_value(arr).map_err(|e| format!("invalid assertion in list: {e}"))
}

#[derive(Debug, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AssertReport {
    all_passed: bool,
    passed: usize,
    failed: usize,
    results: Vec<AssertionResult>,
    notes: Vec<String>,
}

/// Pure: evaluate every assertion against the snapshot/frame/baselines.
pub fn assert_all(
    snapshot: &ElementSnapshot,
    frame: &Frame,
    assertions: &[Assertion],
    baselines: &HashMap<String, BaselineEntry>,
) -> AssertReport {
    let ctx = EvalContext {
        snapshot: Some(snapshot),
        frame: Some(frame),
        ocr_blocks: None, // runner-side only
        baselines: Some(baselines),
    };
    let results: Vec<AssertionResult> = assertions
        .iter()
        .map(|a| evaluate_assertion(a, &ctx))
        .collect();
    let passed = results.iter().filter(|r| r.passed).count();
    let failed = results.len() - passed;
    AssertReport {
        all_passed: failed == 0,
        passed,
        failed,
        results,
        notes: vec!["ocr_unavailable: OCR/VLM run runner-side only; \
                     contains_text uses snapshot text"
            .to_string()],
    }
}

fn assert_summary(report: &AssertReport) -> String {
    let mut lines = vec![format!(
        "vision-audit assert: {} passed, {} failed of {} -> {}",
        report.passed,
        report.failed,
        report.passed + report.failed,
        if report.all_passed {
            "PASS (exit 0)"
        } else {
            "FAIL (exit 2)"
        }
    )];
    for r in &report.results {
        if !r.passed {
            lines.push(format!(
                "  FAIL {}: {}",
                assertion_type_name(&r.assertion),
                r.detail.as_deref().unwrap_or("(no detail)")
            ));
        }
    }
    lines.join("\n")
}

/// The wire `type` discriminant of an assertion, for human summaries.
fn assertion_type_name(a: &Assertion) -> String {
    serde_json::to_value(a)
        .ok()
        .and_then(|v| v.get("type").and_then(|t| t.as_str()).map(str::to_string))
        .unwrap_or_else(|| "assertion".to_string())
}

// ===========================================================================
// baseline
// ===========================================================================

fn run_baseline(args: &[String]) -> Result<u8, CliError> {
    let flags = parse_flags(args, &["snapshot", "name", "baseline-dir"])?;
    let snapshot = load_snapshot(Path::new(require(&flags, "snapshot")?))?;
    let name = require(&flags, "name")?;
    let dir = PathBuf::from(require(&flags, "baseline-dir")?);

    let entry = BaselineEntry::from_snapshot(&snapshot);
    let path = baseline_path(&dir, name)?;
    std::fs::create_dir_all(&dir)
        .map_err(|e| CliError::io(format!("cannot create {}: {e}", dir.display())))?;
    let json = serde_json::to_string_pretty(&entry).map_err(|e| CliError::io(e.to_string()))?;
    std::fs::write(&path, &json)
        .map_err(|e| CliError::io(format!("cannot write {}: {e}", path.display())))?;

    println!("{json}");
    eprintln!(
        "vision-audit baseline: wrote {} element bbox(es) to {}",
        entry.element_bboxes.len(),
        path.display()
    );
    Ok(EXIT_OK)
}

/// Resolve `<dir>/<name>.json`, rejecting a `name` that would escape the dir.
fn baseline_path(dir: &Path, name: &str) -> Result<PathBuf, CliError> {
    if name.is_empty() || name.contains(['/', '\\', '.']) {
        return Err(CliError::usage(format!(
            "--name must be a simple file stem (no path separators or dots), got {name:?}"
        )));
    }
    Ok(dir.join(format!("{name}.json")))
}

/// Load every `<name>.json` baseline in a dir into a name->entry map.
fn load_baselines(dir: &Path) -> Result<HashMap<String, BaselineEntry>, CliError> {
    let mut out = HashMap::new();
    let entries = std::fs::read_dir(dir)
        .map_err(|e| CliError::io(format!("cannot read baseline dir {}: {e}", dir.display())))?;
    for entry in entries {
        let entry = entry.map_err(|e| CliError::io(e.to_string()))?;
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) != Some("json") {
            continue;
        }
        let Some(stem) = path.file_stem().and_then(|s| s.to_str()) else {
            continue;
        };
        let bytes = std::fs::read(&path)
            .map_err(|e| CliError::io(format!("cannot read {}: {e}", path.display())))?;
        let parsed: BaselineEntry = serde_json::from_slice(&bytes)
            .map_err(|e| CliError::io(format!("invalid baseline {}: {e}", path.display())))?;
        out.insert(stem.to_string(), parsed);
    }
    Ok(out)
}

// ===========================================================================
// Tests — core paths: arg parsing, envelope unwrap, exit-code mapping,
// fail-on parsing, baseline round-trip, and a golden analyze/assert run.
// ===========================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use qontinui_vision_core::element_snapshot::{Element, Rgb};
    use qontinui_vision_core::frame::Region;

    fn s(v: &str) -> String {
        v.to_string()
    }

    // ---- arg parser ----

    #[test]
    fn parse_flags_space_and_equals() {
        let args = vec![s("--snapshot"), s("a.json"), s("--frame=b.png")];
        let f = parse_flags(&args, &["snapshot", "frame"]).unwrap();
        assert_eq!(f.get("snapshot").unwrap(), "a.json");
        assert_eq!(f.get("frame").unwrap(), "b.png");
    }

    #[test]
    fn parse_flags_rejects_unknown() {
        let args = vec![s("--bogus"), s("x")];
        assert!(matches!(
            parse_flags(&args, &["snapshot"]),
            Err(CliError::Usage(_))
        ));
    }

    #[test]
    fn parse_flags_rejects_missing_value() {
        let args = vec![s("--snapshot")];
        assert!(matches!(
            parse_flags(&args, &["snapshot"]),
            Err(CliError::Usage(_))
        ));
    }

    #[test]
    fn parse_flags_rejects_positional() {
        let args = vec![s("stray")];
        assert!(matches!(
            parse_flags(&args, &["snapshot"]),
            Err(CliError::Usage(_))
        ));
    }

    // ---- envelope unwrap ----

    #[test]
    fn snapshot_plain_shape() {
        let json = br#"{"elements":[{"id":"a","bbox":{"x":0,"y":0,"w":10,"h":10}}]}"#;
        let snap = parse_snapshot(json).unwrap();
        assert_eq!(snap.elements.len(), 1);
        assert_eq!(snap.elements[0].id, "a");
    }

    #[test]
    fn snapshot_data_envelope_unwrapped() {
        let json = br#"{"data":{"elements":[{"id":"a"}]}}"#;
        let snap = parse_snapshot(json).unwrap();
        assert_eq!(snap.elements.len(), 1);
    }

    #[test]
    fn snapshot_data_bare_array_unwrapped() {
        let json = br#"{"data":[{"id":"a"},{"id":"b"}]}"#;
        let snap = parse_snapshot(json).unwrap();
        assert_eq!(snap.elements.len(), 2);
    }

    #[test]
    fn snapshot_bare_array_normalized() {
        let json = br#"[{"id":"a"}]"#;
        let snap = parse_snapshot(json).unwrap();
        assert_eq!(snap.elements.len(), 1);
    }

    #[test]
    fn snapshot_bad_json_errors() {
        assert!(parse_snapshot(b"not json").is_err());
    }

    // ---- fail-on parsing ----

    #[test]
    fn fail_on_parse_table() {
        assert_eq!(parse_fail_on(None).unwrap(), None);
        assert_eq!(parse_fail_on(Some("warning")).unwrap(), Some(1));
        assert_eq!(parse_fail_on(Some("critical")).unwrap(), Some(2));
        assert!(parse_fail_on(Some("bogus")).is_err());
    }

    // ---- assertions parsing ----

    #[test]
    fn assertions_array_and_wrapper() {
        let arr = br#"[{"type":"no_clipping"}]"#;
        assert_eq!(parse_assertions(arr).unwrap().len(), 1);
        let wrapped = br#"{"assertions":[{"type":"no_clipping"}]}"#;
        assert_eq!(parse_assertions(wrapped).unwrap().len(), 1);
    }

    #[test]
    fn assertions_bad_type_errors() {
        assert!(parse_assertions(br#"{"type":"no_clipping"}"#).is_err());
    }

    // ---- exit-code mapping ----

    fn frame_1x1() -> Frame {
        Frame::from_rgba(
            image::RgbaImage::from_pixel(1, 1, image::Rgba([255, 255, 255, 255])),
            FrameSource::synthetic_now(),
        )
    }

    fn snap_with_overlap() -> ElementSnapshot {
        // interactable: true — the layout analyzer only flags overlaps among
        // interactive elements.
        let mk = |id: &str, x: u32| Element {
            id: id.into(),
            bbox: Some(Region {
                x,
                y: 0,
                w: 100,
                h: 50,
            }),
            text: None,
            role: None,
            interactable: true,
            fg_color: None,
            bg_color: None,
            font_size_px: None,
            font_family: None,
            line_height_px: None,
            parent_id: None,
            children_ids: vec![],
        };
        ElementSnapshot {
            elements: vec![mk("a", 0), mk("b", 50)],
        }
    }

    #[test]
    fn analyze_exit_default_is_ok_even_with_findings() {
        let snap = snap_with_overlap();
        let report = analyze(&snap, &frame_1x1(), &[Analyzer::Layout]);
        // No --fail-on -> always 0.
        assert_eq!(analyze_exit_code(&report, None), EXIT_OK);
    }

    #[test]
    fn analyze_exit_fail_on_warning_trips_on_overlap() {
        let snap = snap_with_overlap();
        let report = analyze(&snap, &frame_1x1(), &[Analyzer::Layout]);
        assert!(report.total >= 1, "expected at least one finding");
        // The overlap finding is at warning-or-above, so fail-on warning trips.
        assert_eq!(
            analyze_exit_code(&report, Some(severity_rank(Severity::Warning))),
            EXIT_GATE_FAILED
        );
    }

    #[test]
    fn analyze_clean_snapshot_passes_fail_on() {
        // Two disjoint elements -> no overlap finding.
        let mk = |id: &str, x: u32| Element {
            id: id.into(),
            bbox: Some(Region {
                x,
                y: 0,
                w: 10,
                h: 10,
            }),
            text: None,
            role: None,
            interactable: false,
            fg_color: None,
            bg_color: None,
            font_size_px: None,
            font_family: None,
            line_height_px: None,
            parent_id: None,
            children_ids: vec![],
        };
        let snap = ElementSnapshot {
            elements: vec![mk("a", 0), mk("b", 500)],
        };
        let report = analyze(&snap, &frame_1x1(), &[Analyzer::Layout]);
        assert_eq!(
            analyze_exit_code(&report, Some(severity_rank(Severity::Critical))),
            EXIT_OK
        );
    }

    // ---- assert exit mapping ----

    #[test]
    fn assert_all_passed_vs_failed() {
        let snap = snap_with_overlap();
        let bl = HashMap::new();
        // Passing assertion: no_clipping (no parent relationships -> passes).
        let pass = assert_all(
            &snap,
            &frame_1x1(),
            &[Assertion::NoClipping { region: None }],
            &bl,
        );
        assert!(pass.all_passed);

        // Failing assertion: a and b overlap.
        let fail = assert_all(
            &snap,
            &frame_1x1(),
            &[Assertion::NoOverlap {
                elements: ["a".into(), "b".into()],
                tolerance_px: None,
            }],
            &bl,
        );
        assert!(!fail.all_passed);
        assert_eq!(fail.failed, 1);
    }

    // ---- baseline path safety + round-trip ----

    #[test]
    fn baseline_path_rejects_traversal() {
        let dir = Path::new("/tmp/bl");
        assert!(baseline_path(dir, "../evil").is_err());
        assert!(baseline_path(dir, "a/b").is_err());
        assert!(baseline_path(dir, "with.dot").is_err());
        assert!(baseline_path(dir, "good").is_ok());
    }

    #[test]
    fn baseline_entry_serializes_and_loads() {
        let dir = tempfile::tempdir().unwrap();
        let mut e = Element {
            id: "hdr".into(),
            bbox: Some(Region {
                x: 1,
                y: 2,
                w: 3,
                h: 4,
            }),
            text: None,
            role: None,
            interactable: false,
            fg_color: None,
            bg_color: None,
            font_size_px: None,
            font_family: None,
            line_height_px: None,
            parent_id: None,
            children_ids: vec![],
        };
        e.fg_color = Some(Rgb::new(0, 0, 0));
        let snap = ElementSnapshot { elements: vec![e] };
        let entry = BaselineEntry::from_snapshot(&snap);
        let path = baseline_path(dir.path(), "v1").unwrap();
        std::fs::write(&path, serde_json::to_string(&entry).unwrap()).unwrap();

        let loaded = load_baselines(dir.path()).unwrap();
        assert!(loaded.contains_key("v1"));
        assert_eq!(loaded["v1"].element_bboxes.len(), 1);
    }

    #[test]
    fn no_layout_shift_uses_loaded_baseline() {
        // Baseline puts 'a' at x=0; current snapshot shifts it to x=50 -> fail.
        let dir = tempfile::tempdir().unwrap();
        let base_el = Element {
            id: "a".into(),
            bbox: Some(Region {
                x: 0,
                y: 0,
                w: 100,
                h: 50,
            }),
            text: None,
            role: None,
            interactable: false,
            fg_color: None,
            bg_color: None,
            font_size_px: None,
            font_family: None,
            line_height_px: None,
            parent_id: None,
            children_ids: vec![],
        };
        let base_snap = ElementSnapshot {
            elements: vec![base_el],
        };
        let entry = BaselineEntry::from_snapshot(&base_snap);
        let path = baseline_path(dir.path(), "v1").unwrap();
        std::fs::write(&path, serde_json::to_string(&entry).unwrap()).unwrap();
        let baselines = load_baselines(dir.path()).unwrap();

        let report = assert_all(
            &snap_with_overlap(), // 'a' is at x=0 here, 'b' at x=50; a matches baseline
            &frame_1x1(),
            &[Assertion::NoLayoutShiftSince {
                baseline: "v1".into(),
                tolerance_px: Some(2),
            }],
            &baselines,
        );
        assert!(report.all_passed, "a did not move -> should pass");
    }

    // ---- top-level run() dispatch ----

    #[test]
    fn run_unknown_subcommand_is_usage_error() {
        assert!(matches!(run(&[s("frobnicate")]), Err(CliError::Usage(_))));
    }

    #[test]
    fn run_no_subcommand_is_usage_error() {
        assert!(matches!(run(&[]), Err(CliError::Usage(_))));
    }
}
