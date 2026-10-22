// SPDX-License-Identifier: MIT
//! `vision-audit` — standalone, runner-less analyzer CLI for CI style gating.
//!
//! A hermetic, version-pinned wrapper over `qontinui-vision-core`'s analyzer +
//! assertion API. It runs the SAME library code the runner's `/vision/*` HTTP
//! endpoints call, but as a plain binary with no qontinui-runner process — so a
//! CI job can gate a build on layout/typography/color/element findings or on a
//! declarative assertion suite without standing up the full stack. The `image`
//! analyzer rides along under `all` and reports VALUES (`imageValues`); it
//! emits Info findings only and blocks only without a frame, so it never
//! trips `--fail-on` on its own findings.
//!
//! Machine-readable JSON goes to **stdout**; a human-readable summary goes to
//! **stderr**. Exit codes are the contract CI keys off:
//!
//! | exit | meaning                                                       |
//! |------|--------------------------------------------------------------|
//! | 0    | OK (analyze: ran; or no finding at/above `--fail-on`. assert: allPassed) |
//! | 2    | GATE FAILED (a finding at/above `--fail-on`, an analyzer that could not run, or an assertion failed) |
//! | 1    | usage / IO / parse error                                      |
//!
//! Modes:
//!   * `vision-audit analyze  --snapshot S [--frame F --scale-factor N [--frame-text-hidden T]] [--analyzer A] [--fail-on L]`
//!   * `vision-audit assert   --snapshot S [--frame F --scale-factor N [--frame-text-hidden T]] --assertions A [--baseline-dir D]`
//!   * `vision-audit baseline --snapshot S --name N --baseline-dir D`
//!
//! A frame always travels with its scale factor: snapshot bboxes are CSS
//! pixels, the frame is device pixels, and the analyzers map one onto the
//! other through `FrameSource::scale_factor`. The report echoes the frame it
//! analyzed (`frame: {width, height, scaleFactor}`, or `null`).
//!
//! `--frame-text-hidden` is the same page captured with its text hidden
//! (plan 2026-10-13-image-generation-and-visual-judgment, Phase 16), read at
//! the same `--scale-factor`. It feeds the image analyzer's per-image
//! `textContrast` value and the `text_over_image_contrast` assertion, and is
//! echoed as `frameTextHidden` (or `null`).
//!
//! SCOPE: OCR / VLM clients live runner-side; this bin runs the snapshot-text
//! path only and emits an `ocr_unavailable` note rather than pretending to OCR.

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

use qontinui_vision_core::analyzers::image::ImageValues;
use qontinui_vision_core::analyzers::{
    self, AnalyzeInput, Analyzer, AnalyzerVerdict, Finding, Severity,
};
use qontinui_vision_core::assertions::{
    evaluate as evaluate_assertion, Assertion, AssertionOutcome, AssertionResult, BaselineEntry,
    EvalContext,
};
use qontinui_vision_core::coverage::SnapshotCoverage;
use qontinui_vision_core::element_snapshot::{display_snapshot_id, ElementSnapshot};
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
    vision-audit analyze  --snapshot <file> [--frame <file> --scale-factor <n> [--frame-text-hidden <file>]] [--analyzer <a>] [--fail-on <level>]
    vision-audit assert   --snapshot <file> [--frame <file> --scale-factor <n> [--frame-text-hidden <file>]] --assertions <file> [--baseline-dir <dir>]
    vision-audit baseline --snapshot <file> --name <name> --baseline-dir <dir>

FRAME:
    --frame         PNG/JPEG/WebP screenshot captured in lockstep with the snapshot.
                    Optional: without it the pixel analyzers (color, image) report
                    themselves BLOCKED and the geometric ones run unchanged.
    --scale-factor  The frame's device pixel ratio: frame pixels per CSS pixel
                    (1 for a DPR-1 capture, 2 for Retina, 2.625 for many phones).
                    REQUIRED with --frame, refused without one. Snapshot bboxes are
                    CSS pixels and every pixel read maps them onto the frame
                    through this value; the report echoes it as frame.scaleFactor.
    --frame-text-hidden  The same page captured with its text hidden (UI Bridge
                    control/text-visibility), read at the same --scale-factor.
                    Optional, and only with --frame. The glyphs of text over an
                    image are where the two frames differ; text_over_image_contrast
                    answers unknown (needs_multi_frame_input) without it, and the
                    image analyzer's textContrast value too. Echoed as frameTextHidden.

ANALYZE:
    --analyzer   layout | typography | color | elements | image | all   (default: all)
    --fail-on    warning | critical                             (default: never fail on findings)
    Exit 2 if any finding is at/above --fail-on; otherwise 0.
    `image` reports per-image values under `imageValues` and Info findings only;
    with a frame it never blocks (an image-free page is DEGRADED, which is green).

ASSERT:
    --assertions JSON array of assertion objects ({\"type\":\"no_overlap\", ...}).
    --baseline-dir  dir of <name>.json baselines for no_layout_shift_since.
    Exit 0 iff every assertion PASSED; else 2. An assertion that could not be
    evaluated (missing element, missing measurement, unregistered baseline)
    reports `outcome: \"unknown\"` and is NOT a pass -- it counts toward exit 2,
    because an unchecked assertion must never read as a green one.

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

/// Parse an `ElementSnapshot` from raw JSON bytes, peeling a single `{data:
/// ...}` transport wrapper and normalizing a bare element array:
///
///   * `{ "elements": [...] }`         -> the snapshot itself
///   * `{ "data": { "elements": [...] } }` -> unwrap `.data`
///   * `{ "data": [...] }`             -> `.data` is the element array
///   * `[ ... ]`                       -> a bare element array
///
/// **This is not a `curl .../discover | jq .data` pipe, and must not be
/// advertised as one.** The bridge's `DiscoveredElement` shares exactly ONE
/// field with [`ElementSnapshot`]'s `Element`: `id`. It carries no `bbox`, no
/// `text` and no `interactable`, so a discover payload does parse — into a
/// snapshot where every element is `interactable: false, text: None`, on a
/// page with forty working buttons. `analyzers::elements` then emits
/// `no_interactive` + `no_text`, `analyze --fail-on warning` exits 2, and the
/// summary stamps the capture's own `snapshotId` on that verdict: a
/// confidently wrong answer wearing authoritative attribution. The
/// `a_raw_discover_payload_is_not_a_supported_input` test pins that, so the
/// claim cannot quietly come back.
///
/// The unwrap exists for a caller that has ALREADY projected into this shape
/// and merely left a transport envelope around it. Projecting a discover
/// payload — mapping geometry, text and interactivity onto `Element` — is the
/// caller's job; there is no shortcut for it here.
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

/// Load a frame and stamp the caller's scale factor on it — the value the
/// analyzers map every CSS-pixel bbox onto its pixels through.
fn load_frame(path: &Path, scale_factor: f64) -> Result<Frame, CliError> {
    let bytes = std::fs::read(path)
        .map_err(|e| CliError::io(format!("cannot read frame {}: {e}", path.display())))?;
    let decoded = image::load_from_memory(&bytes)
        .map_err(|e| CliError::io(format!("cannot decode frame {}: {e}", path.display())))?;
    let mut source = FrameSource::synthetic_now();
    source.scale_factor = scale_factor;
    Ok(Frame::from_rgba(decoded.to_rgba8(), source))
}

/// Parse `--scale-factor`: a positive, finite device pixel ratio.
pub fn parse_scale_factor(value: &str) -> Result<f64, String> {
    match value.trim().parse::<f64>() {
        Ok(s) if s.is_finite() && s > 0.0 => Ok(s),
        _ => Err(format!(
            "--scale-factor must be a positive finite number (the frame's device pixel \
             ratio, e.g. 1 or 2), got {value:?}"
        )),
    }
}

/// The `--frame` / `--scale-factor` pair, both or neither.
///
/// A frame without a scale factor is refused rather than read at 1.0: the
/// CLI cannot know the ratio a screenshot was taken at, and assuming 1.0 on a
/// HiDPI frame samples the wrong pixels for every element with nothing to say
/// so. A scale factor without a frame describes nothing and is refused as the
/// typo it almost certainly is.
fn load_frame_flags(flags: &HashMap<String, String>) -> Result<Option<Frame>, CliError> {
    match (flags.get("frame"), flags.get("scale-factor")) {
        (None, None) => Ok(None),
        (Some(path), Some(scale)) => {
            let scale = parse_scale_factor(scale).map_err(CliError::usage)?;
            Ok(Some(load_frame(Path::new(path), scale)?))
        }
        (Some(_), None) => Err(CliError::usage(
            "--frame needs --scale-factor: the frame's device pixel ratio (1 for a DPR-1 \
             capture, 2 for Retina). Snapshot bboxes are CSS pixels and are mapped onto the \
             frame through it.",
        )),
        (None, Some(_)) => Err(CliError::usage(
            "--scale-factor describes a frame, and no --frame was given",
        )),
    }
}

/// `--frame-text-hidden`: the frame's text-hidden twin, read at the frame's
/// scale factor. Refused without `--frame` — a twin of nothing. Whether it
/// PAIRS with the frame (one size) is the library's question, not a usage
/// error: the image value and the assertion answer unknown with the reason.
fn load_text_hidden_flag(
    flags: &HashMap<String, String>,
    frame: Option<&Frame>,
) -> Result<Option<Frame>, CliError> {
    match (flags.get("frame-text-hidden"), frame) {
        (None, _) => Ok(None),
        (Some(path), Some(f)) => Ok(Some(load_frame(Path::new(path), f.source.scale_factor)?)),
        (Some(_), None) => Err(CliError::usage(
            "--frame-text-hidden is the text-hidden twin of --frame, and no --frame was given",
        )),
    }
}

/// What a report says about the frame it analyzed: `null` on the wire when
/// there was none, so a reader can tell "no pixels were read" from silence.
#[derive(Debug, Clone, Copy, PartialEq, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FrameEcho {
    width: u32,
    height: u32,
    scale_factor: f64,
}

impl FrameEcho {
    fn of(frame: Option<&Frame>) -> Option<Self> {
        frame.map(|f| FrameEcho {
            width: f.width,
            height: f.height,
            scale_factor: f.source.scale_factor,
        })
    }

    /// The summary line, so a CI log names the DPR the pixels were read at.
    fn summary(echo: Option<&Self>) -> String {
        match echo {
            Some(f) => format!(
                "  frame: {}x{} px at scale factor {} (snapshot bboxes mapped through it)",
                f.width, f.height, f.scale_factor
            ),
            None => "  frame: none (no pixels read; pixel analyzers report BLOCKED)".to_string(),
        }
    }
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
            Analyzer::Image,
        ]),
        "layout" => Ok(vec![Analyzer::Layout]),
        "typography" => Ok(vec![Analyzer::Typography]),
        "color" => Ok(vec![Analyzer::Color]),
        "elements" => Ok(vec![Analyzer::Elements]),
        "image" => Ok(vec![Analyzer::Image]),
        // `dynamic` needs a prior frame the bin doesn't take; surface that.
        "dynamic" => Err(
            "the 'dynamic' analyzer needs two frames and is not supported by vision-audit"
                .to_string(),
        ),
        other => Err(format!(
            "--analyzer must be one of layout|typography|color|elements|image|all, got {other:?}"
        )),
    }
}

// ===========================================================================
// analyze
// ===========================================================================

fn run_analyze(args: &[String]) -> Result<u8, CliError> {
    let flags = parse_flags(
        args,
        &[
            "snapshot",
            "frame",
            "scale-factor",
            "frame-text-hidden",
            "analyzer",
            "fail-on",
        ],
    )?;
    let snapshot = load_snapshot(Path::new(require(&flags, "snapshot")?))?;
    // `--frame` is OPTIONAL. Three of the six analyzers — layout,
    // typography, elements — are pure geometry over the snapshot and never
    // read a pixel, and `analyzers::run` already degrades color/dynamic/image
    // to an explicit "skipped" finding without one. Requiring it made the geometric
    // checks unavailable to exactly the callers that need them most: a CI job
    // or a headless probe holding a snapshot and no screenshot. When it IS
    // given, `--scale-factor` must come with it — see `load_frame_flags`.
    let frame = load_frame_flags(&flags)?;
    let text_hidden = load_text_hidden_flag(&flags, frame.as_ref())?;
    let analyzers_to_run =
        parse_analyzer(flags.get("analyzer").map(String::as_str)).map_err(CliError::usage)?;
    let fail_on =
        parse_fail_on(flags.get("fail-on").map(String::as_str)).map_err(CliError::usage)?;

    let report = analyze_frames(
        &snapshot,
        frame.as_ref(),
        text_hidden.as_ref(),
        &analyzers_to_run,
    );
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
    ///
    /// **Never read this without `verdicts`.** An empty list under a
    /// `blocked` verdict is not a clean page — it is an analyzer that could
    /// not examine anything, and before `verdicts` existed the two were
    /// byte-identical on this surface.
    findings: HashMap<String, Vec<Finding>>,
    /// Per-analyzer verdict, keyed by the same wire-name as `findings`.
    /// Every key in `findings` appears here and vice versa.
    verdicts: HashMap<String, AnalyzerVerdict>,
    /// The gate bit for the run: false when ANY analyzer was blocked.
    /// Derived, never set independently — see [`analyze_exit_code`].
    conclusive: bool,
    /// What the analyzed snapshot actually carried. One measurement for the
    /// whole run, because it is a property of the SNAPSHOT rather than of
    /// any analyzer, and every analyzer here was handed the same one.
    ///
    /// This is the evidence behind the verdicts: a reader who disagrees with
    /// a `blocked` can check the counts that produced it without re-running
    /// anything.
    coverage: SnapshotCoverage,
    /// Total finding count across all analyzers.
    total: usize,
    /// Counts by severity (`"info"`/`"warning"`/`"critical"`).
    counts: HashMap<String, usize>,
    /// The frame the pixel analyzers read, or `null` when none was given —
    /// its size and the scale factor every CSS-pixel bbox was mapped through.
    frame: Option<FrameEcho>,
    /// The text-hidden twin of `frame` (`--frame-text-hidden`), or `null`.
    frame_text_hidden: Option<FrameEcho>,
    /// Which snapshot these findings describe — echoed verbatim from
    /// [`ElementSnapshot::snapshot_id`], exactly as [`AssertReport`] does.
    /// The two report shapes are the same surface to a caller, so attribution
    /// that worked on only one of them would be a half-surface. `None` when
    /// the snapshot carried no id; omitted from the JSON in that case so a
    /// consumer cannot read `null` as an id.
    #[serde(skip_serializing_if = "Option::is_none")]
    snapshot_id: Option<String>,
    /// The `image` analyzer's per-image and page values, forwarded verbatim
    /// from [`analyzers::AnalyzerResult::image_values`]. Omitted when the
    /// image analyzer did not run or measured nothing (blocked: no frame).
    /// Forwarding it here is load-bearing: a report that projected the
    /// result field by field without it would compute the values and drop
    /// them.
    #[serde(skip_serializing_if = "Option::is_none")]
    image_values: Option<ImageValues>,
    /// OCR/VLM are runner-side only; flagged so CI knows OCR checks did not run.
    notes: Vec<String>,
}

/// Pure: run the requested analyzers over the snapshot (+ optional frame)
/// and tally. `frame: None` is a supported mode, not a degraded one — the
/// snapshot-only analyzers produce identical output either way, and the
/// pixel-dependent ones report themselves skipped.
pub fn analyze(
    snapshot: &ElementSnapshot,
    frame: Option<&Frame>,
    which: &[Analyzer],
) -> AnalyzeReport {
    analyze_frames(snapshot, frame, None, which)
}

/// [`analyze`] with the frame's text-hidden twin (`--frame-text-hidden`),
/// which only the image analyzer reads.
pub fn analyze_frames(
    snapshot: &ElementSnapshot,
    frame: Option<&Frame>,
    text_hidden: Option<&Frame>,
    which: &[Analyzer],
) -> AnalyzeReport {
    let input = AnalyzeInput {
        frame,
        snapshot: Some(snapshot),
        prior_frame: None,
        text_hidden_frame: text_hidden,
    };
    let mut findings = HashMap::new();
    let mut verdicts = HashMap::new();
    let mut counts: HashMap<String, usize> = HashMap::new();
    let mut total = 0usize;
    let mut conclusive = true;
    let mut image_values = None;
    for &a in which {
        let mut result = analyzers::run(a, &input);
        if let Some(v) = result.image_values.take() {
            image_values = Some(v);
        }
        for f in &result.findings {
            total += 1;
            *counts
                .entry(severity_name(f.severity).to_string())
                .or_insert(0) += 1;
        }
        // One blocked analyzer poisons the run's gate bit. It is not that
        // the others' findings are suspect — they stand — but that the run
        // as a whole did not check what it was asked to check, and a caller
        // folding this to a single boolean must not be told otherwise.
        conclusive &= result.conclusive;
        findings.insert(a.name().to_string(), result.findings);
        verdicts.insert(a.name().to_string(), result.verdict);
    }
    AnalyzeReport {
        findings,
        verdicts,
        conclusive,
        coverage: SnapshotCoverage::of(snapshot),
        total,
        counts,
        frame: FrameEcho::of(frame),
        frame_text_hidden: FrameEcho::of(text_hidden),
        snapshot_id: snapshot.snapshot_id.clone(),
        image_values,
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
///
/// Two independent ways to fail the gate, and the second is the point of
/// this whole surface:
///
/// 1. A finding at or above `--fail-on`. The original contract.
/// 2. **An analyzer that could not run at all.** A blocked analyzer produces
///    no findings, so under (1) alone `--fail-on critical` returned 0 for a
///    snapshot whose every element had lost its geometry — the same 0 a
///    healthy page returns. A gate that cannot be tripped by the absence of
///    evidence is not a gate.
///
/// `--fail-on` remains the opt-in for both: with no threshold set, the caller
/// has said it is not gating on this run, and (2) does not override that.
/// The verdicts and coverage are still on the report either way.
pub fn analyze_exit_code(report: &AnalyzeReport, fail_on: Option<u8>) -> u8 {
    let Some(threshold) = fail_on else {
        return EXIT_OK;
    };
    if !report.conclusive {
        return EXIT_GATE_FAILED;
    }
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
    // Name the snapshot analyzed, when the producer supplied one — same
    // reason as the assert summary: a CI reader looking at findings needs to
    // know WHICH capture produced them. Rendered through
    // `display_snapshot_id`: stdout JSON is serde-escaped, this line is not,
    // and the id is unvalidated by design.
    if let Some(id) = &report.snapshot_id {
        lines.push(format!("  snapshot: {}", display_snapshot_id(id)));
    }
    // The coverage the verdicts were derived from, in the same wording the
    // projector's `--stats` uses, so a reader comparing the two surfaces is
    // reading one sentence rather than two dialects of it.
    lines.push(format!("  coverage: {}", report.coverage.summary()));
    lines.push(FrameEcho::summary(report.frame.as_ref()));
    if let Some(t) = report.frame_text_hidden.as_ref() {
        lines.push(format!(
            "  frame-text-hidden: {}x{} px at scale factor {}",
            t.width, t.height, t.scale_factor
        ));
    }
    // Name every analyzer that did not reach a conclusion, and why. Ordered
    // so the same run prints the same lines — a HashMap iteration order in a
    // CI log is a diff nobody can read.
    let mut non_checked: Vec<(&String, &AnalyzerVerdict)> = report
        .verdicts
        .iter()
        .filter(|(_, v)| !matches!(v, AnalyzerVerdict::Checked))
        .collect();
    non_checked.sort_by(|a, b| a.0.cmp(b.0));
    for (name, verdict) in non_checked {
        let label = if verdict.is_blocked() {
            "BLOCKED"
        } else {
            "degraded"
        };
        lines.push(format!(
            "  {name}: {label} — {}",
            verdict.reason().unwrap_or_default()
        ));
    }
    match fail_on {
        None => lines.push("gate: --fail-on not set; exit 0 regardless of findings".to_string()),
        Some(t) => {
            let level = if t >= severity_rank(Severity::Critical) {
                "critical"
            } else {
                "warning"
            };
            lines.push(format!(
                "gate: --fail-on {level} -> {}{}",
                if exit == EXIT_GATE_FAILED {
                    "FAILED (exit 2)"
                } else {
                    "passed (exit 0)"
                },
                if report.conclusive {
                    ""
                } else {
                    " [an analyzer was BLOCKED: this run checked less than it was asked to]"
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
    let flags = parse_flags(
        args,
        &[
            "snapshot",
            "frame",
            "scale-factor",
            "frame-text-hidden",
            "assertions",
            "baseline-dir",
        ],
    )?;
    let snapshot = load_snapshot(Path::new(require(&flags, "snapshot")?))?;
    // Optional, as in `analyze`: only the L1 image assertions read
    // `ctx.frame` (image_palette_within, no_near_duplicate_images, …), and
    // without one they answer `unknown{input_missing}` rather than pass. Every
    // other assertion evaluates from the snapshot, the OCR blocks or the
    // baseline registry, so a required `--frame` would gate values they never
    // consult. Given, it is paired with `--scale-factor` exactly as in
    // `analyze`.
    let frame = load_frame_flags(&flags)?;
    let text_hidden = load_text_hidden_flag(&flags, frame.as_ref())?;
    let assertions = load_assertions(Path::new(require(&flags, "assertions")?))?;
    let baselines = match flags.get("baseline-dir") {
        Some(dir) => load_baselines(Path::new(dir))?,
        None => HashMap::new(),
    };

    let report = assert_all_frames(
        &snapshot,
        frame.as_ref(),
        text_hidden.as_ref(),
        &assertions,
        &baselines,
    );
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
    /// Assertions that were EVALUATED and VIOLATED. Before the tri-state
    /// landed this counted `unknown` too, which made a suite whose operands
    /// were simply absent from the snapshot read as a page full of layout
    /// bugs. The gate is unchanged — `all_passed` is still
    /// `failed + unknown == 0` — only the attribution is now honest.
    failed: usize,
    /// Assertions that could NOT be evaluated: a missing operand, a
    /// measurement field the producer does not populate, an unregistered
    /// baseline. Counted separately because "we did not check" and "we
    /// checked and it is broken" call for different work.
    unknown: usize,
    results: Vec<AssertionResult>,
    /// The frame handed to the evaluators, or `null` — see [`AnalyzeReport`].
    frame: Option<FrameEcho>,
    /// The text-hidden twin handed to the evaluators, or `null`.
    frame_text_hidden: Option<FrameEcho>,
    /// Which snapshot this verdict judged — echoed verbatim from
    /// [`ElementSnapshot::snapshot_id`]. `None` when the snapshot carried no
    /// id, which is the honest answer: the verdict is real but unattributable,
    /// not attributable to "some default snapshot". Omitted from the JSON in
    /// that case so a consumer cannot read `null` as an id.
    #[serde(skip_serializing_if = "Option::is_none")]
    snapshot_id: Option<String>,
    notes: Vec<String>,
}

/// Pure: evaluate every assertion against the snapshot/frame/baselines.
pub fn assert_all(
    snapshot: &ElementSnapshot,
    frame: Option<&Frame>,
    assertions: &[Assertion],
    baselines: &HashMap<String, BaselineEntry>,
) -> AssertReport {
    assert_all_frames(snapshot, frame, None, assertions, baselines)
}

/// [`assert_all`] with the frame's text-hidden twin (`--frame-text-hidden`),
/// which only `text_over_image_contrast` reads.
pub fn assert_all_frames(
    snapshot: &ElementSnapshot,
    frame: Option<&Frame>,
    text_hidden: Option<&Frame>,
    assertions: &[Assertion],
    baselines: &HashMap<String, BaselineEntry>,
) -> AssertReport {
    let ctx = EvalContext {
        snapshot: Some(snapshot),
        frame,
        text_hidden_frame: text_hidden,
        ocr_blocks: None, // runner-side only
        baselines: Some(baselines),
        timeline: None, // a timeline arrives through the runner's /vision/assert only
    };
    let results: Vec<AssertionResult> = assertions
        .iter()
        .map(|a| evaluate_assertion(a, &ctx))
        .collect();
    let passed = results
        .iter()
        .filter(|r| r.outcome == AssertionOutcome::Passed)
        .count();
    let unknown = results.iter().filter(|r| r.outcome.is_unknown()).count();
    let failed = results.len() - passed - unknown;
    AssertReport {
        // An un-evaluated assertion is not a pass. Same gate as before the
        // tri-state: only `Passed` is green.
        all_passed: failed + unknown == 0,
        passed,
        failed,
        unknown,
        results,
        frame: FrameEcho::of(frame),
        frame_text_hidden: FrameEcho::of(text_hidden),
        snapshot_id: snapshot.snapshot_id.clone(),
        notes: vec!["ocr_unavailable: OCR/VLM run runner-side only; \
                     contains_text uses snapshot text"
            .to_string()],
    }
}

fn assert_summary(report: &AssertReport) -> String {
    let mut lines = vec![format!(
        "vision-audit assert: {} passed, {} failed, {} not evaluated of {} -> {}",
        report.passed,
        report.failed,
        report.unknown,
        report.passed + report.failed + report.unknown,
        if report.all_passed {
            "PASS (exit 0)"
        } else {
            "FAIL (exit 2)"
        }
    )];
    // Name the snapshot judged, when the producer supplied one — a CI reader
    // looking at a red gate needs to know WHICH capture failed, not merely
    // that one did. Same format-time guard as the analyze summary.
    if let Some(id) = &report.snapshot_id {
        lines.push(format!("  snapshot: {}", display_snapshot_id(id)));
    }
    lines.push(FrameEcho::summary(report.frame.as_ref()));
    if let Some(t) = report.frame_text_hidden.as_ref() {
        lines.push(format!(
            "  frame-text-hidden: {}x{} px at scale factor {}",
            t.width, t.height, t.scale_factor
        ));
    }
    // Label the two red kinds apart. A reader triaging a red gate needs to
    // know which lines send them to the UI and which send them to the
    // producer; before the tri-state both said FAIL.
    for r in &report.results {
        let label = match r.outcome {
            AssertionOutcome::Passed => continue,
            AssertionOutcome::Failed => "FAIL",
            AssertionOutcome::Unknown => "UNKNOWN",
        };
        lines.push(format!(
            "  {label} {}: {}",
            assertion_type_name(&r.assertion),
            r.detail.as_deref().unwrap_or("(no detail)")
        ));
    }
    if report.unknown > 0 {
        lines.push(
            "  note: UNKNOWN lines were never evaluated — the snapshot (or the caller) did not \
             carry what they needed. They are red because an unchecked assertion must not read \
             as a passing one, NOT because the page is known to be wrong."
                .to_string(),
        );
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
    // Name the capture the baseline was taken FROM. A later
    // `no_layout_shift_since` failure names both ends of the delta, so the
    // operator recording the baseline should be able to see which end this
    // file becomes. Same format-time guard as the analyze/assert summaries.
    eprintln!(
        "vision-audit baseline: wrote {} element bbox(es) to {} (from snapshot {})",
        entry.element_bboxes.len(),
        path.display(),
        entry
            .snapshot_id
            .as_deref()
            .map(display_snapshot_id)
            .unwrap_or_else(|| "<unattributed>".to_string()),
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
    fn transport_envelope_peel_keeps_an_already_projected_id() {
        // A snapshot ALREADY in `ElementSnapshot` shape with one `{data: ...}`
        // transport wrapper left around it. Peeling the wrapper must not drop
        // the id with it.
        let json = br#"{"data":{"elements":[{"id":"a","bbox":{"x":0,"y":0,"w":10,"h":10},"interactable":true}],"snapshotId":"ubs2_1_1_aaaaaaaaaaaaaaaa_bbbbbbbbbbbbbbbb"}}"#;
        let snap = parse_snapshot(json).unwrap();
        assert_eq!(
            snap.snapshot_id.as_deref(),
            Some("ubs2_1_1_aaaaaaaaaaaaaaaa_bbbbbbbbbbbbbbbb")
        );

        // A bare element array carries no envelope and therefore no id — the
        // unattributed case stays unattributed rather than inventing one.
        assert!(parse_snapshot(br#"[{"id":"a"}]"#)
            .unwrap()
            .snapshot_id
            .is_none());
    }

    #[test]
    fn a_raw_discover_payload_is_not_a_supported_input() {
        // Withdrawn endorsement, pinned executable so it cannot quietly come
        // back. `DiscoveredElement` shares exactly ONE field with `Element`:
        // `id`. Everything the analyzers judge on — geometry, text,
        // interactivity — is spelled differently and is silently dropped.
        let discover = br#"{"data":{"elements":[
            {"id":"save","category":"button","state":{"textContent":"Save"},"rect":{"x":10,"y":20,"width":80,"height":32}},
            {"id":"cancel","category":"button","state":{"textContent":"Cancel"},"rect":{"x":100,"y":20,"width":80,"height":32}},
            {"id":"title","category":"text","state":{"textContent":"Settings"},"rect":{"x":10,"y":0,"width":200,"height":18}},
            {"id":"tab-a","category":"tab","state":{"textContent":"A"},"rect":{"x":10,"y":60,"width":40,"height":24}},
            {"id":"tab-b","category":"tab","state":{"textContent":"B"},"rect":{"x":54,"y":60,"width":40,"height":24}},
            {"id":"tab-c","category":"tab","state":{"textContent":"C"},"rect":{"x":98,"y":60,"width":40,"height":24}},
            {"id":"submit","category":"button","state":{"textContent":"Apply"},"rect":{"x":10,"y":100,"width":80,"height":32}}
        ]}}"#;
        let snap = parse_snapshot(discover).expect("it parses — that is the trap");

        assert_eq!(snap.elements.len(), 7);
        assert!(
            snap.elements
                .iter()
                .all(|e| e.bbox.is_none() && e.text.is_none() && !e.interactable),
            "every field the analyzers judge on is dropped: {:?}",
            snap.elements
        );

        // ...and that under-populated snapshot trips the elements analyzer,
        // so `--fail-on warning` would exit 2 on a page whose seven controls
        // all work. This is the attributed-garbage verdict the doc claim used
        // to advertise a pipe to.
        let report = analyze(&snap, Some(&frame_1x1()), &[Analyzer::Elements]);
        let kinds: Vec<&str> = report
            .findings
            .values()
            .flatten()
            .map(|f| f.kind.as_str())
            .collect();
        assert!(kinds.contains(&"no_interactive"), "{kinds:?}");
        assert!(kinds.contains(&"no_text"), "{kinds:?}");
        assert_eq!(
            analyze_exit_code(&report, Some(severity_rank(Severity::Warning))),
            EXIT_GATE_FAILED,
            "a healthy page would fail the gate"
        );
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
        let mk = |id: &str, x: i32| Element {
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
            ..Default::default()
        };
        ElementSnapshot {
            elements: vec![mk("a", 0), mk("b", 50)],
            ..Default::default()
        }
    }

    #[test]
    fn analyze_exit_default_is_ok_even_with_findings() {
        let snap = snap_with_overlap();
        let report = analyze(&snap, Some(&frame_1x1()), &[Analyzer::Layout]);
        // No --fail-on -> always 0.
        assert_eq!(analyze_exit_code(&report, None), EXIT_OK);
    }

    #[test]
    fn analyze_exit_fail_on_warning_trips_on_overlap() {
        let snap = snap_with_overlap();
        let report = analyze(&snap, Some(&frame_1x1()), &[Analyzer::Layout]);
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
        //
        // Nothing here is `interactable`, so layout reports `Degraded` (its
        // pairwise-overlap pass had no candidates) — and this test is
        // therefore also the gate-level pin that **a degraded verdict does
        // not fail the gate**. It is asserted below rather than left
        // implicit in the exit code.
        let mk = |id: &str, x: i32| Element {
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
            ..Default::default()
        };
        let snap = ElementSnapshot {
            elements: vec![mk("a", 0), mk("b", 500)],
            ..Default::default()
        };
        let report = analyze(&snap, Some(&frame_1x1()), &[Analyzer::Layout]);
        assert!(
            report.verdicts["layout"].is_degraded(),
            "expected the degraded arm: {:?}",
            report.verdicts
        );
        assert!(
            report.conclusive,
            "a Degraded verdict must NOT move the gate bit: {:?}",
            report.verdicts
        );
        assert_eq!(
            analyze_exit_code(&report, Some(severity_rank(Severity::Critical))),
            EXIT_OK
        );
    }

    #[test]
    fn a_blocked_analyzer_fails_the_gate_with_zero_findings() {
        // The whole point of wiring the verdict into the exit code. Three
        // elements, every one stripped of geometry: layout produces NO
        // findings, so the severity fold below cannot trip, and before the
        // verdict existed `--fail-on critical` returned 0 — the same 0 a
        // healthy page returns.
        let mk = |id: &str| Element {
            id: id.into(),
            bbox: None,
            text: Some("label".into()),
            interactable: true,
            ..Default::default()
        };
        let snap = ElementSnapshot {
            elements: vec![mk("a"), mk("b"), mk("c")],
            ..Default::default()
        };
        let report = analyze(&snap, Some(&frame_1x1()), &[Analyzer::Layout]);

        assert_eq!(report.total, 0, "the premise: no findings at all");
        assert!(!report.conclusive);
        assert!(report.verdicts["layout"].is_blocked());
        assert_eq!(
            analyze_exit_code(&report, Some(severity_rank(Severity::Critical))),
            EXIT_GATE_FAILED,
            "a gate that cannot be tripped by the absence of evidence is not a gate"
        );

        // `--fail-on` stays the opt-in for BOTH failure routes: a caller that
        // set no threshold has said it is not gating this run, and the block
        // does not override that. The verdict is still on the report.
        assert_eq!(analyze_exit_code(&report, None), EXIT_OK);
    }

    #[test]
    fn the_report_carries_the_coverage_its_verdicts_were_derived_from() {
        // Evidence, not just a verdict: a reader who disagrees with a block
        // can check the counts that produced it without re-running anything.
        let snap = snap_with_overlap();
        let report = analyze(&snap, Some(&frame_1x1()), &[Analyzer::Layout]);
        assert_eq!(report.coverage, SnapshotCoverage::of(&snap));

        let wire = serde_json::to_value(&report).unwrap();
        assert_eq!(wire["coverage"]["withGeometry"], serde_json::json!(2));
        assert_eq!(wire["coverage"]["interactable"], serde_json::json!(2));
        // The two elements intersect and carry no stacking order, so
        // occlusion is UNKNOWN: `degraded`, and still green.
        assert_eq!(
            wire["verdicts"]["layout"]["state"],
            serde_json::json!("degraded")
        );
        assert!(wire["verdicts"]["layout"]["reason"].is_string());
        assert_eq!(wire["conclusive"], serde_json::json!(true));

        // …and the human summary names it, in the projector's own wording.
        let summary = analyze_summary(&report, None, EXIT_OK);
        assert!(
            summary.contains("coverage: 2 elements: 2 with geometry"),
            "{summary}"
        );
    }

    #[test]
    fn the_summary_names_every_analyzer_that_did_not_reach_a_conclusion() {
        let snap = ElementSnapshot {
            elements: vec![Element {
                id: "a".into(),
                text: Some("label".into()),
                interactable: true,
                ..Default::default()
            }],
            ..Default::default()
        };
        let report = analyze(&snap, None, &[Analyzer::Layout, Analyzer::Typography]);
        let exit = analyze_exit_code(&report, Some(severity_rank(Severity::Critical)));
        let summary = analyze_summary(&report, Some(severity_rank(Severity::Critical)), exit);

        assert!(summary.contains("layout: BLOCKED"), "{summary}");
        assert!(
            summary.contains("an analyzer was BLOCKED"),
            "the gate line must say WHY it failed with no findings: {summary}"
        );
        // Typography had its whole input (text present) and must not be
        // listed at all.
        assert!(!summary.contains("typography:"), "{summary}");
    }

    // ---- analyze attribution ----

    #[test]
    fn analyze_report_names_the_snapshot_it_analyzed() {
        let mut snap = snap_with_overlap();
        let id = "ubs2_2_2_9f1c0a4b7e3d2610_00000191a4c3f2d8";
        snap.snapshot_id = Some(id.to_string());

        let report = analyze(&snap, Some(&frame_1x1()), &[Analyzer::Layout]);
        assert!(report.total >= 1, "expected at least one finding");
        assert_eq!(report.snapshot_id.as_deref(), Some(id));

        // On the machine-JSON stdout contract CI keys off...
        let wire = serde_json::to_value(&report).unwrap();
        assert_eq!(wire.get("snapshotId").and_then(|v| v.as_str()), Some(id));

        // ...and on the human summary, so findings name their capture.
        assert!(
            analyze_summary(&report, None, EXIT_OK).contains(id),
            "summary must name the snapshot"
        );
    }

    #[test]
    fn analyze_report_omits_attribution_for_an_unattributed_snapshot() {
        let report = analyze(
            &snap_with_overlap(),
            Some(&frame_1x1()),
            &[Analyzer::Layout],
        );
        assert!(report.snapshot_id.is_none());
        let wire = serde_json::to_value(&report).unwrap();
        assert!(
            wire.get("snapshotId").is_none(),
            "unattributed findings must omit the key, not emit null"
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
            Some(&frame_1x1()),
            &[Assertion::NoClipping { region: None }],
            &bl,
        );
        assert!(pass.all_passed);

        // Failing assertion: a and b overlap.
        let fail = assert_all(
            &snap,
            Some(&frame_1x1()),
            &[Assertion::NoOverlap {
                elements: ["a".into(), "b".into()],
                tolerance_px: None,
            }],
            &bl,
        );
        assert!(!fail.all_passed);
        assert_eq!(fail.failed, 1);
    }

    // ---- assert attribution ----

    #[test]
    fn assert_report_names_the_snapshot_it_judged() {
        let mut snap = snap_with_overlap();
        let id = "ubs2_2_2_9f1c0a4b7e3d2610_00000191a4c3f2d8";
        snap.snapshot_id = Some(id.to_string());
        let bl = HashMap::new();

        let report = assert_all(
            &snap,
            Some(&frame_1x1()),
            &[Assertion::NoOverlap {
                elements: ["a".into(), "b".into()],
                tolerance_px: None,
            }],
            &bl,
        );
        assert!(!report.all_passed, "a and b overlap");
        assert_eq!(report.snapshot_id.as_deref(), Some(id));

        // Reachable by a caller, not merely stored: it is on the machine-JSON
        // stdout contract CI keys off...
        let wire = serde_json::to_value(&report).unwrap();
        assert_eq!(wire.get("snapshotId").and_then(|v| v.as_str()), Some(id));

        // ...and on the human summary, so a red gate says WHICH capture failed.
        assert!(
            assert_summary(&report).contains(id),
            "summary must name the snapshot"
        );
    }

    #[test]
    fn assert_report_omits_attribution_for_an_unattributed_snapshot() {
        let report = assert_all(
            &snap_with_overlap(),
            Some(&frame_1x1()),
            &[Assertion::NoClipping { region: None }],
            &HashMap::new(),
        );
        assert!(report.all_passed);
        assert!(report.snapshot_id.is_none());
        let wire = serde_json::to_value(&report).unwrap();
        assert!(
            wire.get("snapshotId").is_none(),
            "an unattributed verdict must omit the key, not emit null"
        );
    }

    // ---- log-forgery containment ----

    /// A snapshot id is deliberately unvalidated (this crate is a consumer of
    /// the fold, never a producer), so a hostile snapshot file can carry a
    /// newline plus a convincing verdict. Stdout JSON is serde-escaped; the
    /// stderr summary is not. The guard sits at FORMAT time so the fail-open
    /// "opaque token" property survives — exit codes are untouched either way.
    const FORGED_ID: &str = "x\ngate: --fail-on critical -> passed (exit 0)";

    #[test]
    fn analyze_summary_cannot_be_forged_through_the_snapshot_id() {
        let mut snap = snap_with_overlap();
        snap.snapshot_id = Some(FORGED_ID.to_string());
        let report = analyze(&snap, Some(&frame_1x1()), &[Analyzer::Layout]);

        // The token still reaches the report and the JSON verbatim — nothing
        // is rejected or rewritten at parse time.
        assert_eq!(report.snapshot_id.as_deref(), Some(FORGED_ID));
        let wire = serde_json::to_value(&report).unwrap();
        assert_eq!(
            wire.get("snapshotId").and_then(|v| v.as_str()),
            Some(FORGED_ID)
        );

        // ...but the unescaped human summary gains no forged line.
        let summary = analyze_summary(&report, Some(severity_rank(Severity::Critical)), EXIT_OK);
        let gate_lines = summary.lines().filter(|l| l.starts_with("gate: ")).count();
        assert_eq!(gate_lines, 1, "exactly one gate verdict line: {summary:?}");
        assert!(summary.contains("\\u{000a}gate:"), "{summary:?}");
    }

    #[test]
    fn assert_summary_cannot_be_forged_through_the_snapshot_id() {
        let mut snap = snap_with_overlap();
        snap.snapshot_id = Some(FORGED_ID.to_string());
        let report = assert_all(
            &snap,
            Some(&frame_1x1()),
            &[Assertion::NoOverlap {
                elements: ["a".into(), "b".into()],
                tolerance_px: None,
            }],
            &HashMap::new(),
        );
        assert!(!report.all_passed);

        let summary = assert_summary(&report);
        assert!(
            !summary.lines().any(|l| l.starts_with("gate: ")),
            "assert summaries carry no bare gate line at all: {summary:?}"
        );
        assert!(summary.contains("\\u{000a}gate:"), "{summary:?}");
        // The verdict line the summary DOES own is still the real one.
        assert!(summary.starts_with("vision-audit assert: 0 passed, 1 failed"));
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
            ..Default::default()
        };
        e.fg_color = Some(Rgb::new(0, 0, 0));
        let snap = ElementSnapshot {
            elements: vec![e],
            ..Default::default()
        };
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
            ..Default::default()
        };
        let base_snap = ElementSnapshot {
            elements: vec![base_el],
            ..Default::default()
        };
        let entry = BaselineEntry::from_snapshot(&base_snap);
        let path = baseline_path(dir.path(), "v1").unwrap();
        std::fs::write(&path, serde_json::to_string(&entry).unwrap()).unwrap();
        let baselines = load_baselines(dir.path()).unwrap();

        let report = assert_all(
            &snap_with_overlap(), // 'a' is at x=0 here, 'b' at x=50; a matches baseline
            Some(&frame_1x1()),
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

    #[test]
    fn geometry_analyzers_run_with_no_frame_at_all() {
        // The headless case: a CI job or a probe against a runner whose
        // window is missing holds a snapshot and no screenshot. Requiring a
        // frame made exactly these callers unable to run the checks that
        // need no pixels.
        let report = analyze(&snap_with_overlap(), None, &[Analyzer::Layout]);
        assert!(
            report.total > 0,
            "layout is pure geometry and must produce findings without a frame"
        );
    }

    #[test]
    fn frameless_layout_matches_framed_layout_exactly() {
        // Not merely "it runs" — the frame contributes NOTHING to these
        // analyzers, so the two runs must agree finding-for-finding. If they
        // ever diverge, one of them is reading something it should not.
        let snap = snap_with_overlap();
        let with = analyze(&snap, Some(&frame_1x1()), &[Analyzer::Layout]);
        let without = analyze(&snap, None, &[Analyzer::Layout]);
        assert_eq!(with.total, without.total);
        assert_eq!(
            serde_json::to_string(&with.findings).unwrap(),
            serde_json::to_string(&without.findings).unwrap()
        );
    }

    // ---- --scale-factor: the frame's DPR ----

    #[test]
    fn scale_factor_parse_table() {
        assert_eq!(parse_scale_factor("1").unwrap(), 1.0);
        assert_eq!(parse_scale_factor("2.625").unwrap(), 2.625);
        for bad in ["0", "-1", "nan", "inf", "abc", ""] {
            assert!(parse_scale_factor(bad).is_err(), "{bad:?} must be refused");
        }
    }

    /// A DPR-2 screenshot of a page whose one text element has CSS box
    /// (40, 40, 20, 20) and no declared colours: a ~1.4:1 pair where the
    /// element rendered (buffer 80..120), black on white at the unscaled
    /// position (buffer 40..60). Returns the PNG path and the snapshot path.
    fn hidpi_inputs(dir: &Path) -> (PathBuf, PathBuf) {
        let mut img = image::RgbaImage::from_pixel(200, 200, image::Rgba([0xff, 0xff, 0xff, 0xff]));
        for y in 40..45u32 {
            for x in 40..60u32 {
                img.put_pixel(x, y, image::Rgba([0, 0, 0, 0xff]));
            }
        }
        for y in 80..120u32 {
            for x in 80..120u32 {
                let v = if y < 90 { 0x77 } else { 0x88 };
                img.put_pixel(x, y, image::Rgba([v, v, v, 0xff]));
            }
        }
        let png = dir.join("frame.png");
        img.save(&png).unwrap();
        let snap = dir.join("snapshot.json");
        std::fs::write(
            &snap,
            br#"{"elements":[{"id":"hero-caption","bbox":{"x":40,"y":40,"w":20,"h":20},"text":"caption"}]}"#,
        )
        .unwrap();
        (png, snap)
    }

    #[test]
    fn load_frame_stamps_the_scale_factor_it_is_given() {
        let dir = tempfile::tempdir().unwrap();
        let (png, _) = hidpi_inputs(dir.path());
        let frame = load_frame(&png, 2.0).unwrap();
        assert_eq!((frame.width, frame.height), (200, 200));
        assert_eq!(frame.source.scale_factor, 2.0);
    }

    #[test]
    fn a_hidpi_frame_is_analyzed_through_its_scale_factor() {
        let dir = tempfile::tempdir().unwrap();
        let (png, snap_path) = hidpi_inputs(dir.path());
        let snap = load_snapshot(&snap_path).unwrap();

        // At its real DPR the caption's low-contrast pair is found...
        let at_2 = analyze(
            &snap,
            Some(&load_frame(&png, 2.0).unwrap()),
            &[Analyzer::Color],
        );
        let low: Vec<_> = at_2.findings["color"]
            .iter()
            .filter(|f| f.kind == "low_contrast")
            .collect();
        assert_eq!(low.len(), 1, "{:?}", at_2.findings);
        assert_eq!(low[0].elements, vec!["hero-caption".to_string()]);

        // ...and read as a DPR-1 frame the same PNG yields the black-on-white
        // unscaled position instead: nothing to report.
        let at_1 = analyze(
            &snap,
            Some(&load_frame(&png, 1.0).unwrap()),
            &[Analyzer::Color],
        );
        assert!(
            !at_1.findings["color"]
                .iter()
                .any(|f| f.kind == "low_contrast"),
            "{:?}",
            at_1.findings
        );
    }

    #[test]
    fn the_report_echoes_the_frame_and_its_scale_factor_or_null() {
        let dir = tempfile::tempdir().unwrap();
        let (png, snap_path) = hidpi_inputs(dir.path());
        let snap = load_snapshot(&snap_path).unwrap();
        let frame = load_frame(&png, 2.0).unwrap();

        let wire = serde_json::to_value(analyze(&snap, Some(&frame), &[Analyzer::Layout])).unwrap();
        assert_eq!(
            wire["frame"],
            serde_json::json!({"width": 200, "height": 200, "scaleFactor": 2.0})
        );
        let none = serde_json::to_value(analyze(&snap, None, &[Analyzer::Layout])).unwrap();
        assert!(none["frame"].is_null(), "no frame is a stated null: {none}");

        let assert_wire = serde_json::to_value(assert_all(
            &snap,
            Some(&frame),
            &[Assertion::NoClipping { region: None }],
            &HashMap::new(),
        ))
        .unwrap();
        assert_eq!(assert_wire["frame"]["scaleFactor"], serde_json::json!(2.0));

        let summary = analyze_summary(
            &analyze(&snap, Some(&frame), &[Analyzer::Layout]),
            None,
            EXIT_OK,
        );
        assert!(
            summary.contains("frame: 200x200 px at scale factor 2"),
            "{summary}"
        );

        // ...and so does the assert summary, from the same echo.
        let assert_text = assert_summary(&assert_all(
            &snap,
            Some(&frame),
            &[Assertion::NoClipping { region: None }],
            &HashMap::new(),
        ));
        assert!(
            assert_text.contains("frame: 200x200 px at scale factor 2"),
            "{assert_text}"
        );
    }

    #[test]
    fn a_frame_without_a_scale_factor_is_refused_not_read_at_1() {
        let dir = tempfile::tempdir().unwrap();
        let (png, snap) = hidpi_inputs(dir.path());
        for mode in ["analyze", "assert"] {
            let mut args = vec![
                s(mode),
                s("--snapshot"),
                snap.display().to_string(),
                s("--frame"),
                png.display().to_string(),
            ];
            if mode == "assert" {
                let spec = dir.path().join("a.json");
                std::fs::write(&spec, br#"[{"type":"no_clipping"}]"#).unwrap();
                args.extend([s("--assertions"), spec.display().to_string()]);
            }
            match run(&args) {
                Err(CliError::Usage(msg)) => assert!(msg.contains("--scale-factor"), "{msg}"),
                other => panic!("{mode}: expected a usage error, got {other:?}"),
            }
        }
    }

    #[test]
    fn a_scale_factor_without_a_frame_is_refused() {
        let dir = tempfile::tempdir().unwrap();
        let (_, snap) = hidpi_inputs(dir.path());
        let args = vec![
            s("analyze"),
            s("--snapshot"),
            snap.display().to_string(),
            s("--scale-factor"),
            s("2"),
        ];
        assert!(matches!(run(&args), Err(CliError::Usage(m)) if m.contains("no --frame")));
    }

    #[test]
    fn a_frame_with_a_bad_scale_factor_is_refused() {
        let dir = tempfile::tempdir().unwrap();
        let (png, snap) = hidpi_inputs(dir.path());
        let args = vec![
            s("analyze"),
            s("--snapshot"),
            snap.display().to_string(),
            s("--frame"),
            png.display().to_string(),
            s("--scale-factor"),
            s("0"),
        ];
        assert!(matches!(run(&args), Err(CliError::Usage(m)) if m.contains("positive finite")));
    }

    #[test]
    fn the_usage_names_the_flag_and_its_pairing() {
        // verify-page-verdict.sh probes `--help` for exactly these tokens to
        // decide whether it may pass a scale factor and whether it may omit a
        // frame; renaming either silently downgrades that caller.
        assert!(USAGE.contains("--scale-factor <n>"));
        assert!(USAGE.contains("[--frame <file>"));
    }

    #[test]
    fn assertions_evaluate_with_no_frame() {
        // Only the L1 image assertions read `ctx.frame`; the rest evaluate
        // without one, so a required frame would gate values they never use.
        let bl = HashMap::new();
        let r = assert_all(
            &snap_with_overlap(),
            None,
            &[Assertion::NoClipping { region: None }],
            &bl,
        );
        assert!(r.all_passed);
    }

    // ---- the image analyzer ----

    #[test]
    fn all_includes_the_image_analyzer_and_image_parses_alone() {
        assert!(parse_analyzer(None).unwrap().contains(&Analyzer::Image));
        assert_eq!(
            parse_analyzer(Some("image")).unwrap(),
            vec![Analyzer::Image]
        );
        assert!(USAGE.contains("| image |"));
        assert!(parse_analyzer(Some("bogus"))
            .unwrap_err()
            .contains("|image|"));
    }

    /// The style gate runs `--analyzer all --fail-on critical`. Adding
    /// `image` to `all` must not make any route newly exit 2: on a page with
    /// no images, and on the frame-bearing run the gate does, the exit code is
    /// the one the run had without it.
    #[test]
    fn adding_image_to_all_newly_fails_no_style_gate_run() {
        let without: Vec<Analyzer> = parse_analyzer(Some("all"))
            .unwrap()
            .into_iter()
            .filter(|a| *a != Analyzer::Image)
            .collect();
        let all = parse_analyzer(Some("all")).unwrap();
        let critical = parse_fail_on(Some("critical")).unwrap();
        for snap in [snap_with_overlap(), image_page_snapshot()] {
            let frame = image_page_frame();
            let a = analyze(&snap, Some(&frame), &all);
            let b = analyze(&snap, Some(&frame), &without);
            assert_eq!(
                analyze_exit_code(&a, critical),
                analyze_exit_code(&b, critical)
            );
            assert!(
                a.verdicts["image"].conclusive(),
                "{:?}",
                a.verdicts["image"]
            );
            assert!(a.findings["image"]
                .iter()
                .all(|f| f.severity == Severity::Info));
        }
    }

    fn image_page_frame() -> Frame {
        let mut buf = image::RgbaImage::from_pixel(240, 120, image::Rgba([250, 250, 250, 255]));
        for y in 20..80 {
            for x in 120..200 {
                let v = ((x * 7 + y * 13) % 200) as u8;
                buf.put_pixel(x, y, image::Rgba([v, 255 - v, 90, 255]));
            }
        }
        Frame::from_rgba(buf, FrameSource::synthetic_now())
    }

    fn image_page_snapshot() -> ElementSnapshot {
        ElementSnapshot {
            elements: vec![Element {
                id: "hero".into(),
                bbox: Some(Region {
                    x: 120,
                    y: 20,
                    w: 80,
                    h: 60,
                }),
                // `fill` paints the whole box, so the box's pixels are the
                // image's (a fit that may letterbox, unplaced, is unreadable).
                media: Some(qontinui_vision_core::ElementMedia {
                    object_fit: Some(qontinui_vision_core::ObjectFit::Fill),
                    ..qontinui_vision_core::ElementMedia::new(qontinui_vision_core::MediaKind::Img)
                }),
                ..Default::default()
            }],
            ..Default::default()
        }
    }

    /// `AnalyzeReport` projects each `AnalyzerResult` field by field, so it
    /// must forward the image values or they are computed and dropped.
    #[test]
    fn the_analyze_report_forwards_image_values() {
        let report = analyze(
            &image_page_snapshot(),
            Some(&image_page_frame()),
            &[Analyzer::Image],
        );
        let wire = serde_json::to_value(&report).unwrap();
        let img = &wire["imageValues"]["images"][0];
        assert_eq!(img["element"], "hero");
        assert_eq!(img["sharpness"]["status"], "measured");
        assert!(wire["imageValues"]["page"]["colourfulness"]["value"].is_number());
        // No image run, no key.
        let layout_only = analyze(&image_page_snapshot(), None, &[Analyzer::Layout]);
        assert!(serde_json::to_value(&layout_only)
            .unwrap()
            .get("imageValues")
            .is_none());
    }

    /// Without a frame the image analyzer BLOCKS, exactly like `color`: no
    /// pixels were read, and a gate must hear that rather than a pass.
    #[test]
    fn without_a_frame_the_image_analyzer_blocks_the_gate_like_color() {
        let report = analyze(&image_page_snapshot(), None, &[Analyzer::Image]);
        assert!(report.verdicts["image"].is_blocked());
        assert_eq!(
            analyze_exit_code(&report, parse_fail_on(Some("critical")).unwrap()),
            EXIT_GATE_FAILED
        );
        assert!(serde_json::to_value(&report)
            .unwrap()
            .get("imageValues")
            .is_none());
    }

    #[test]
    fn an_image_assertion_reads_the_frame_through_assert_all() {
        let a = Assertion::NoPlaceholderImages { element: None };
        let snap = image_page_snapshot();
        let framed = assert_all(
            &snap,
            Some(&image_page_frame()),
            std::slice::from_ref(&a),
            &HashMap::new(),
        );
        assert!(framed.all_passed, "{:?}", framed.results[0].detail);
        assert!(framed.results[0].confidence.is_some());
        let frameless = assert_all(&snap, None, &[a], &HashMap::new());
        assert_eq!(frameless.unknown, 1);
    }

    // ---- arming tests (plan 2026-10-13 Phase 15) ----
    //
    // Written against the PRE-Phase-15 API on purpose: they name the image
    // analyzer and the image assertions only by their WIRE names, so the
    // arming record's revert run compiles them against the base and they go
    // RED there on an assertion, not on a build failure.

    /// A textured frame with one image row, in wire JSON.
    fn arming_page() -> (ElementSnapshot, Frame) {
        let snap: ElementSnapshot = serde_json::from_value(serde_json::json!({"elements": [
            {"id": "hero", "bbox": {"x": 120, "y": 20, "w": 80, "h": 60},
             "media": {"kind": "img", "object_fit": "fill"}}
        ]}))
        .unwrap();
        let buf = image::RgbaImage::from_fn(240, 120, |x, y| {
            if (120..200).contains(&x) && (20..80).contains(&y) {
                let v = ((x * 7 + y * 13) % 200) as u8;
                image::Rgba([v, 255 - v, 90, 255])
            } else {
                image::Rgba([250, 250, 250, 255])
            }
        });
        (snap, Frame::from_rgba(buf, FrameSource::synthetic_now()))
    }

    /// `vision-audit analyze --analyzer image` through the CLI's own
    /// dispatcher, from files on disk as CI runs it.
    #[test]
    fn the_cli_accepts_the_image_analyzer() {
        let dir = tempfile::tempdir().unwrap();
        let (snap, frame) = arming_page();
        let snap_path = dir.path().join("snapshot.json");
        let frame_path = dir.path().join("frame.png");
        std::fs::write(&snap_path, serde_json::to_vec(&snap).unwrap()).unwrap();
        frame.buffer.save(&frame_path).unwrap();
        let args: Vec<String> = [
            "analyze",
            "--snapshot",
            snap_path.to_str().unwrap(),
            "--frame",
            frame_path.to_str().unwrap(),
            "--scale-factor",
            "1",
            "--analyzer",
            "image",
        ]
        .iter()
        .map(|s| s.to_string())
        .collect();
        assert!(
            matches!(run(&args), Ok(code) if code == EXIT_OK),
            "`--analyzer image` must be accepted"
        );
    }

    #[test]
    fn analyze_delivers_image_values_on_its_report() {
        let image: Analyzer =
            serde_json::from_str("\"image\"").expect("vision-core has an `image` analyzer");
        let (snap, frame) = arming_page();
        let report = analyze(&snap, Some(&frame), &[image]);
        let wire = serde_json::to_value(&report).unwrap();
        assert_eq!(wire["imageValues"]["images"][0]["element"], "hero");
        assert_eq!(
            wire["imageValues"]["images"][0]["sharpness"]["status"],
            "measured"
        );
    }

    #[test]
    fn assert_all_fails_a_flat_slot_sized_raster_as_a_placeholder() {
        let a: Assertion =
            serde_json::from_value(serde_json::json!({"type": "no_placeholder_images"}))
                .expect("`no_placeholder_images` is an assertion");
        let snap: ElementSnapshot = serde_json::from_value(serde_json::json!({"elements": [
            {"id": "slot", "bbox": {"x": 10, "y": 10, "w": 400, "h": 300},
             "media": {"kind": "img", "object_fit": "fill",
                       "natural_px": {"w": 400.0, "h": 300.0},
                       "rendered_px": {"w": 400.0, "h": 300.0}}}
        ]}))
        .unwrap();
        let frame = Frame::from_rgba(
            image::RgbaImage::from_pixel(420, 320, image::Rgba([229, 231, 235, 255])),
            FrameSource::synthetic_now(),
        );
        let r = assert_all(&snap, Some(&frame), &[a], &HashMap::new());
        assert_eq!(r.failed, 1, "{:?}", r.results[0].detail);
    }

    // ---- --frame-text-hidden (plan Phase 16) ----

    /// A fixture pair from `tests/fixtures/text_over_image/`, written out as a
    /// snapshot the CLI reads: the hero image and the heading over it.
    fn text_over_image_inputs(dir: &Path, case: &str) -> (PathBuf, PathBuf, PathBuf) {
        let fixtures = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/text_over_image");
        let manifest: serde_json::Value =
            serde_json::from_slice(&std::fs::read(fixtures.join("manifest.json")).unwrap())
                .unwrap();
        let c = &manifest["cases"][case];
        let snap = serde_json::json!({"elements": [
            {"id": "hero", "bbox": c["image"],
             "media": {"kind": "img", "object_fit": "fill"}},
            {"id": "heading", "bbox": c["text"], "text": manifest["text"],
             "fg_color": {"r": c["fg"][0], "g": c["fg"][1], "b": c["fg"][2]}},
        ]});
        let snap_path = dir.join(format!("{case}.json"));
        std::fs::write(&snap_path, serde_json::to_vec(&snap).unwrap()).unwrap();
        (
            snap_path,
            fixtures.join(c["frame"].as_str().unwrap()),
            fixtures.join(c["text_hidden"].as_str().unwrap()),
        )
    }

    fn text_over_image_args(
        mode: &str,
        snap: &Path,
        frame: &Path,
        hidden: Option<&Path>,
        dir: &Path,
    ) -> Vec<String> {
        let mut args = vec![
            s(mode),
            s("--snapshot"),
            snap.display().to_string(),
            s("--frame"),
            frame.display().to_string(),
            s("--scale-factor"),
            s("1"),
        ];
        if let Some(h) = hidden {
            args.extend([s("--frame-text-hidden"), h.display().to_string()]);
        }
        if mode == "assert" {
            let spec = dir.join("toic.json");
            std::fs::write(
                &spec,
                br#"[{"type":"text_over_image_contrast","min_ratio":4.5}]"#,
            )
            .unwrap();
            args.extend([s("--assertions"), spec.display().to_string()]);
        } else {
            args.extend([s("--analyzer"), s("all"), s("--fail-on"), s("critical")]);
        }
        args
    }

    #[test]
    fn the_usage_names_the_text_hidden_flag() {
        assert!(USAGE.contains("[--frame-text-hidden <file>]"));
    }

    #[test]
    fn a_text_hidden_frame_without_a_frame_is_refused() {
        let dir = tempfile::tempdir().unwrap();
        let (snap, _, hidden) = text_over_image_inputs(dir.path(), "scrim");
        let args = vec![
            s("analyze"),
            s("--snapshot"),
            snap.display().to_string(),
            s("--frame-text-hidden"),
            hidden.display().to_string(),
        ];
        assert!(matches!(run(&args), Err(CliError::Usage(m)) if m.contains("no --frame")));
    }

    /// End to end through the CLI: the low-contrast overlay gates (exit 2),
    /// the scrim passes (exit 0), and without the twin the assertion is not a
    /// pass (exit 2, outcome unknown).
    #[test]
    fn assert_reads_the_text_hidden_frame() {
        let dir = tempfile::tempdir().unwrap();
        for (case, hidden, want) in [
            ("low_contrast", true, EXIT_GATE_FAILED),
            ("scrim", true, EXIT_OK),
            ("scrim", false, EXIT_GATE_FAILED),
        ] {
            let (snap, a, b) = text_over_image_inputs(dir.path(), case);
            let args = text_over_image_args(
                "assert",
                &snap,
                &a,
                hidden.then_some(b.as_path()),
                dir.path(),
            );
            assert_eq!(run(&args).unwrap(), want, "{case} hidden={hidden}");
        }
        let (snap, a, b) = text_over_image_inputs(dir.path(), "low_contrast");
        let report = assert_all_frames(
            &load_snapshot(&snap).unwrap(),
            Some(&load_frame(&a, 1.0).unwrap()),
            Some(&load_frame(&b, 1.0).unwrap()),
            &parse_assertions(br#"[{"type":"text_over_image_contrast","min_ratio":4.5}]"#).unwrap(),
            &HashMap::new(),
        );
        let wire = serde_json::to_value(&report).unwrap();
        assert_eq!(wire["results"][0]["outcome"], "failed");
        assert_eq!(
            wire["frameTextHidden"],
            serde_json::json!({"width": 320, "height": 180, "scaleFactor": 1.0})
        );
    }

    /// The no-gating rule: `analyze --analyzer all --fail-on critical` exits
    /// the same with and without `--frame-text-hidden` — on a failing overlay,
    /// a passing scrim, and a twin of the wrong size — and the image analyzer
    /// stays conclusive and Info-only.
    #[test]
    fn the_text_hidden_frame_newly_fails_no_style_gate_run() {
        let dir = tempfile::tempdir().unwrap();
        let critical = parse_fail_on(Some("critical")).unwrap();
        let all = parse_analyzer(Some("all")).unwrap();
        for case in ["low_contrast", "scrim"] {
            let (snap_path, a, b) = text_over_image_inputs(dir.path(), case);
            let snap = load_snapshot(&snap_path).unwrap();
            let frame = load_frame(&a, 1.0).unwrap();
            let twin = load_frame(&b, 1.0).unwrap();
            let wrong = Frame::from_rgba(
                image::imageops::crop_imm(&twin.buffer, 0, 0, 50, 50).to_image(),
                FrameSource::synthetic_now(),
            );
            let without = analyze_exit_code(&analyze(&snap, Some(&frame), &all), critical);
            for t in [&twin, &wrong] {
                let report = analyze_frames(&snap, Some(&frame), Some(t), &all);
                assert_eq!(analyze_exit_code(&report, critical), without, "{case}");
                assert!(report.verdicts["image"].conclusive());
                assert!(report.findings["image"]
                    .iter()
                    .all(|f| f.severity == Severity::Info));
            }
            // And through the CLI's own flag handling.
            let args = text_over_image_args("analyze", &snap_path, &a, Some(&b), dir.path());
            assert_eq!(run(&args).unwrap(), without, "{case} via the CLI");
        }
    }

    /// `AnalyzeReport` forwards the per-image `textContrast` value.
    #[test]
    fn the_analyze_report_forwards_text_contrast() {
        let dir = tempfile::tempdir().unwrap();
        let (snap, a, b) = text_over_image_inputs(dir.path(), "scrim");
        let report = analyze_frames(
            &load_snapshot(&snap).unwrap(),
            Some(&load_frame(&a, 1.0).unwrap()),
            Some(&load_frame(&b, 1.0).unwrap()),
            &[Analyzer::Image],
        );
        let wire = serde_json::to_value(&report).unwrap();
        let tc = &wire["imageValues"]["images"][0]["textContrast"];
        assert_eq!(tc["status"], "measured", "{tc}");
        assert_eq!(tc["value"][0]["element"], "heading");
        assert_eq!(tc["value"][0]["method"], "glyph_mask");
        assert!(tc["value"][0]["percentileRatio"].as_f64().unwrap() > 4.5);
    }
}
