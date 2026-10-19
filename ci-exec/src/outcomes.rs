//! Capturing a run's RUN OUTCOMES before cleanup removes them.
//!
//! Plan `2026-10-13-ci-node-forwards-run-outcomes-into-the-dispatch-result`
//! (F1, F2). coord's `run_outcome` anchor resolves a claim such as "the
//! harness executed N cases" or "the engine beats baseline B2" against a
//! `coord.run_outcomes` row, and the ONLY writer of that table is the CI
//! result door's optional `run_outcomes` field. This module is how a job's
//! numbers get there:
//!
//! 1. [`prepare`] creates `<dispatch root>/.ci-outputs/run_outcomes.jsonl`
//!    empty, before the first step. The executor exports its path to every
//!    step as [`RUN_OUTCOMES_ENV`] (an executor-owned key a manifest cannot
//!    set). The file is OUTSIDE the worktree, so a committed file can never be
//!    mistaken for this run's output, and INSIDE the dispatch root, so it goes
//!    with the dispatch's own cleanup unit.
//! 2. Steps append one JSON object per line — coord's entry shape, nothing
//!    else. Each evaluation harness maps its own rows to entries; the executor
//!    stays harness-agnostic, as it is for JUnit.
//! 3. [`capture`] reads the file after the last step and BEFORE the report,
//!    validating it with coord's own validator
//!    ([`qontinui_types::run_outcomes::validate_run_outcomes`]), so the node
//!    sends exactly what coord would accept or sends nothing and says why.
//!
//! Never fails the dispatch: the outcome channel is evidence ABOUT the run,
//! not part of its verdict. coord's validation is all-or-nothing, so an
//! invalid file is reported on the dispatch log and nothing is sent — never
//! trimmed to the entries that happened to pass.

use std::path::{Path, PathBuf};

use qontinui_types::run_outcomes::{
    validate_run_outcomes, RunOutcomeEntry, RunOutcomeRejection, RUN_OUTCOMES_MAX_BYTES,
};
use serde_json::Value;

/// The env var every step of a dispatch receives, naming the run-outcomes
/// output file. Executor-owned: [`crate::manifest`] refuses it in any `env`.
pub const RUN_OUTCOMES_ENV: &str = "QONTINUI_CI_RUN_OUTCOMES";

/// The directory, under the dispatch root, the executor's output files live in.
const OUTPUTS_DIR: &str = ".ci-outputs";

/// The run-outcomes file's name inside [`OUTPUTS_DIR`].
const OUTCOMES_FILE: &str = "run_outcomes.jsonl";

/// Where a dispatch's run-outcomes file lives. PURE.
pub fn outcomes_path(dispatch_root: &Path) -> PathBuf {
    dispatch_root.join(OUTPUTS_DIR).join(OUTCOMES_FILE)
}

/// Create the run-outcomes file EMPTY (truncating any leftover), returning its
/// path. An error means the channel is unavailable for this dispatch: the
/// executor logs it and exports nothing, and the verdict does not change.
pub fn prepare(dispatch_root: &Path) -> Result<PathBuf, String> {
    let path = outcomes_path(dispatch_root);
    let dir = path
        .parent()
        .ok_or_else(|| format!("{} has no parent directory", path.display()))?;
    std::fs::create_dir_all(dir).map_err(|e| format!("create {}: {e}", dir.display()))?;
    std::fs::write(&path, b"").map_err(|e| format!("create {}: {e}", path.display()))?;
    Ok(path)
}

/// What [`capture`] found. Every arm carries a [`CaptureOutcome::log_line`].
#[derive(Debug, Clone, PartialEq)]
pub enum CaptureOutcome {
    /// The file held at least one entry and coord's validator accepted all of
    /// them.
    Captured(Vec<RunOutcomeEntry>),
    /// No entries: the file is missing, empty, or blank lines only. The
    /// common case — most jobs are not evaluation runs — and it sends no
    /// `run_outcomes` key at all, because coord refuses `[]`.
    Absent,
    /// The output file could not be created, so no step was given the path.
    NotExported { detail: String },
    /// A line is not JSON, or coord's validator refused the payload. `line`
    /// is the 1-based file line of the offending entry, `None` when the
    /// refusal is of the payload as a whole.
    Invalid { line: Option<usize>, reason: String },
    /// Too large to send. `of` says what was measured: the file itself (over
    /// the pre-read memory guard, [`READ_GUARD_BYTES`]) or the payload the
    /// node would send (over coord's [`RUN_OUTCOMES_MAX_BYTES`]).
    TooLarge { bytes: u64, of: Measured },
    /// The file exists and could not be read (permissions, not UTF-8, …).
    Unreadable { detail: String },
}

/// What a [`CaptureOutcome::TooLarge`] size was measured on.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Measured {
    /// The file on disk, before it was read.
    File,
    /// The compact serialized `run_outcomes` array the node would POST — the
    /// value coord's own byte cap is applied to.
    Payload,
}

/// The most the node will read from the outcomes file. A generous multiple of
/// coord's cap: coord measures the COMPACT re-serialized array, so a file with
/// whitespace or blank lines can be larger on disk than the payload it yields.
/// This is a memory guard only; the exact cap is enforced on the payload.
pub const READ_GUARD_BYTES: u64 = 4 * RUN_OUTCOMES_MAX_BYTES as u64;

impl CaptureOutcome {
    /// The entries to send, or `None` for every arm that sends nothing.
    pub fn entries(&self) -> Option<&[RunOutcomeEntry]> {
        match self {
            CaptureOutcome::Captured(e) => Some(e),
            _ => None,
        }
    }

    /// The dispatch-log line naming this outcome. Every arm that sends nothing
    /// says so, because the log is the operator's only clue to a missing row.
    pub fn log_line(&self) -> String {
        match self {
            CaptureOutcome::Captured(e) => format!(
                "[ci-node] run outcomes: captured {} entr{} from ${RUN_OUTCOMES_ENV}; forwarding to coord",
                e.len(),
                if e.len() == 1 { "y" } else { "ies" }
            ),
            CaptureOutcome::Absent => {
                format!("[ci-node] run outcomes: none (no entries in ${RUN_OUTCOMES_ENV})")
            }
            CaptureOutcome::NotExported { detail } => format!(
                "[ci-node] run outcomes: NOT exported — the output file could not be created ({detail}); \
                 no step received ${RUN_OUTCOMES_ENV}, so none are sent"
            ),
            CaptureOutcome::Invalid { line, reason } => match line {
                Some(n) => format!(
                    "[ci-node] run outcomes: INVALID at line {n} ({reason}); none are sent — coord \
                     validates all-or-nothing, so a partial send would be refused anyway"
                ),
                None => format!(
                    "[ci-node] run outcomes: INVALID ({reason}); none are sent — coord validates \
                     all-or-nothing, so a partial send would be refused anyway"
                ),
            },
            CaptureOutcome::TooLarge {
                bytes,
                of: Measured::File,
            } => format!(
                "[ci-node] run outcomes: file is {bytes} bytes, over the node's {READ_GUARD_BYTES}-byte \
                 read guard; none are sent"
            ),
            CaptureOutcome::TooLarge {
                bytes,
                of: Measured::Payload,
            } => format!(
                "[ci-node] run outcomes: the payload serializes to {bytes} bytes, over coord's \
                 {RUN_OUTCOMES_MAX_BYTES}-byte cap; none are sent"
            ),
            CaptureOutcome::Unreadable { detail } => {
                format!("[ci-node] run outcomes: file unreadable ({detail}); none are sent")
            }
        }
    }
}

/// Read and validate the run-outcomes file. `path` is what [`prepare`]
/// returned, or the reason it failed. **Must run before the dispatch's
/// cleanup** — the file is inside the tree cleanup deletes.
pub fn capture(path: Result<&Path, &str>) -> CaptureOutcome {
    let path = match path {
        Ok(p) => p,
        Err(detail) => {
            return CaptureOutcome::NotExported {
                detail: detail.to_string(),
            }
        }
    };
    // A step controls this path, so it is checked WITHOUT following a link:
    // a FIFO would block the read forever (after containment reaped every
    // writer) and a link to a device would read without bound, both before
    // the report and the cleanup. Only a regular file is read.
    let bytes = match std::fs::symlink_metadata(path) {
        Ok(m) if m.file_type().is_file() => m.len(),
        Ok(_) => {
            return CaptureOutcome::Unreadable {
                detail: "not a regular file".to_string(),
            }
        }
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return CaptureOutcome::Absent,
        Err(e) => {
            return CaptureOutcome::Unreadable {
                detail: e.to_string(),
            }
        }
    };
    if bytes > READ_GUARD_BYTES {
        return CaptureOutcome::TooLarge {
            bytes,
            of: Measured::File,
        };
    }
    // The read is bounded too, which closes the window between the size check
    // and the read (a step may still be appending).
    let mut buf = Vec::new();
    let read = std::fs::File::open(path).and_then(|f| {
        std::io::Read::read_to_end(&mut std::io::Read::take(f, READ_GUARD_BYTES + 1), &mut buf)
    });
    if let Err(e) = read {
        return CaptureOutcome::Unreadable {
            detail: e.to_string(),
        };
    }
    if buf.len() as u64 > READ_GUARD_BYTES {
        return CaptureOutcome::TooLarge {
            bytes: buf.len() as u64,
            of: Measured::File,
        };
    }
    match String::from_utf8(buf) {
        Ok(raw) => parse(&raw),
        Err(e) => CaptureOutcome::Unreadable {
            detail: format!("not UTF-8: {e}"),
        },
    }
}

/// Parse and validate JSONL text. PURE. Blank lines are skipped, never
/// invalid; the file is created empty, so blank-only is the same as no output.
pub fn parse(raw: &str) -> CaptureOutcome {
    let mut values = Vec::new();
    let mut lines = Vec::new();
    for (i, text) in raw.lines().enumerate() {
        if text.trim().is_empty() {
            continue;
        }
        match serde_json::from_str::<Value>(text) {
            Ok(v) => {
                values.push(v);
                lines.push(i + 1);
            }
            Err(e) => {
                return CaptureOutcome::Invalid {
                    line: Some(i + 1),
                    reason: format!("not JSON: {e}"),
                }
            }
        }
    }
    if values.is_empty() {
        return CaptureOutcome::Absent;
    }
    let payload = Value::Array(values);
    match validate_run_outcomes(Some(&payload)) {
        Ok(Some(entries)) => {
            // What is SENT is the normalized entries, not the file's values:
            // the validator rewrites every metric as an f64 (`3` -> `3.0`), so
            // the sent payload can be larger than the one just checked. Coord
            // caps what it receives, so re-check what will go on the wire.
            let sent = serde_json::to_vec(&entries)
                .map(|v| v.len())
                .unwrap_or(usize::MAX);
            if sent > RUN_OUTCOMES_MAX_BYTES {
                return CaptureOutcome::TooLarge {
                    bytes: sent as u64,
                    of: Measured::Payload,
                };
            }
            CaptureOutcome::Captured(entries)
        }
        // Unreachable for a non-empty array, but never invent entries.
        Ok(None) => CaptureOutcome::Absent,
        Err(rejected) if rejected.reason == RunOutcomeRejection::TooLarge => {
            CaptureOutcome::TooLarge {
                bytes: serde_json::to_vec(&payload)
                    .map(|v| v.len() as u64)
                    .unwrap_or(u64::MAX),
                of: Measured::Payload,
            }
        }
        Err(rejected) => CaptureOutcome::Invalid {
            line: rejected.index.and_then(|i| lines.get(i).copied()),
            reason: rejected.reason.as_str().to_string(),
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use qontinui_types::run_outcomes::{RUN_OUTCOMES_MAX_ENTRIES, RUN_OUTCOME_MAX_METRICS};
    use serde_json::json;

    fn line(task: &str) -> String {
        json!({
            "source": "reference-eval",
            "subject": "oracle",
            "task": task,
            "metrics": {"recall": 98.0},
            "executed_case_count": 12
        })
        .to_string()
    }

    #[test]
    fn prepare_creates_an_empty_file_outside_the_worktree_inside_the_dispatch_root() {
        let tmp = tempfile::tempdir().unwrap();
        let dispatch_root = tmp.path().join(".ci-worktrees").join("d-1");
        std::fs::create_dir_all(dispatch_root.join("qontinui-x")).unwrap();
        let p = prepare(&dispatch_root).unwrap();
        assert!(p.starts_with(&dispatch_root));
        assert!(!p.starts_with(dispatch_root.join("qontinui-x")));
        assert_eq!(std::fs::read(&p).unwrap(), b"");
        // A leftover is truncated, never inherited by the next run.
        std::fs::write(&p, line("stale")).unwrap();
        prepare(&dispatch_root).unwrap();
        assert_eq!(std::fs::read(&p).unwrap(), b"");
        assert_eq!(capture(Ok(&p)), CaptureOutcome::Absent);
    }

    #[test]
    fn prepare_fails_cleanly_when_the_outputs_dir_cannot_be_created() {
        let tmp = tempfile::tempdir().unwrap();
        // A FILE where the dispatch root should be: nothing can be created
        // under it, so the channel is reported unavailable, never panics.
        let not_a_dir = tmp.path().join("dispatch-root");
        std::fs::write(&not_a_dir, b"x").unwrap();
        let err = prepare(&not_a_dir).unwrap_err();
        assert!(err.starts_with("create "), "{err}");
        let got = capture(Err(&err));
        assert!(got.entries().is_none());
        assert!(matches!(got, CaptureOutcome::NotExported { .. }));
    }

    #[test]
    fn missing_empty_and_blank_only_are_absent() {
        let tmp = tempfile::tempdir().unwrap();
        let p = tmp.path().join("run_outcomes.jsonl");
        assert_eq!(capture(Ok(&p)), CaptureOutcome::Absent);
        std::fs::write(&p, b"").unwrap();
        assert_eq!(capture(Ok(&p)), CaptureOutcome::Absent);
        std::fs::write(&p, b"\n  \n\t\n").unwrap();
        assert_eq!(capture(Ok(&p)), CaptureOutcome::Absent);
        assert!(CaptureOutcome::Absent.entries().is_none());
    }

    #[test]
    fn valid_lines_are_captured_and_blank_lines_skipped() {
        let tmp = tempfile::tempdir().unwrap();
        let p = tmp.path().join("run_outcomes.jsonl");
        std::fs::write(&p, format!("{}\n\n{}\n", line("a"), line("b"))).unwrap();
        let got = capture(Ok(&p));
        let entries = got.entries().expect("captured");
        assert_eq!(entries.len(), 2);
        assert_eq!(entries[1].task, "b");
        assert!(got.log_line().contains("captured 2 entries"));
    }

    #[test]
    fn a_creation_failure_is_reported_not_exported() {
        let got = capture(Err("permission denied"));
        assert!(got.entries().is_none());
        assert!(got.log_line().contains("NOT exported"));
        assert!(got.log_line().contains("permission denied"));
    }

    #[test]
    fn a_non_json_line_names_its_file_line() {
        let raw = format!("{}\n\nnot json\n", line("a"));
        match parse(&raw) {
            CaptureOutcome::Invalid { line, reason } => {
                assert_eq!(line, Some(3));
                assert!(reason.starts_with("not JSON"), "{reason}");
            }
            other => panic!("expected Invalid, got {other:?}"),
        }
    }

    /// Each class coord refuses reaches the node as `Invalid` naming coord's
    /// own wire token and the FILE line (blank lines counted), and nothing is
    /// sent. Drop a rule from the shared validator and its row here goes red.
    #[test]
    fn every_validator_refusal_is_invalid_with_the_file_line() {
        let ok = line("ok");
        let cases: Vec<(String, &str)> = vec![
            (
                json!({"source":"s","subject":"v","task":"t","metrics":{},"executed_case_count":1,"repo":"x"}).to_string(),
                "run_outcome_attribution_in_body",
            ),
            (
                json!({"source":"s","subject":"v","task":"t","metrics":{},"executed_case_count":1,"extra":1}).to_string(),
                "run_outcome_malformed",
            ),
            (
                json!({"source":"s","subject":"v","task":"t","metrics":{}}).to_string(),
                "run_outcome_malformed",
            ),
            (
                json!({"source":" s","subject":"v","task":"t","metrics":{},"executed_case_count":1}).to_string(),
                "run_outcome_invalid_label",
            ),
            (
                json!({"source":"s","subject":"v","task":"t","metrics":{"bad name":1},"executed_case_count":1}).to_string(),
                "run_outcome_invalid_metrics",
            ),
            (
                json!({"source":"s","subject":"v","task":"t","metrics":{"m":null},"executed_case_count":1}).to_string(),
                "run_outcome_invalid_metric_value",
            ),
            (
                json!({"source":"s","subject":"v","task":"t","metrics":{"executed_cases":1},"executed_case_count":1}).to_string(),
                "run_outcome_reserved_metric",
            ),
            (
                json!({"source":"s","subject":"v","task":"t","metrics":{},"executed_case_count":-1}).to_string(),
                "run_outcome_invalid_case_count",
            ),
            (ok.clone(), "run_outcome_duplicate_task"),
        ];
        for (bad, token) in cases {
            let raw = format!("{ok}\n\n{bad}\n");
            match parse(&raw) {
                CaptureOutcome::Invalid { line, reason } => {
                    assert_eq!(reason, token, "{bad}");
                    assert_eq!(line, Some(3), "{bad}");
                }
                other => panic!("{token}: expected Invalid, got {other:?}"),
            }
        }
    }

    #[test]
    fn the_entry_cap_is_file_wide_and_whole_payload() {
        let at_cap: Vec<String> = (0..RUN_OUTCOMES_MAX_ENTRIES)
            .map(|i| line(&format!("t{i}")))
            .collect();
        assert_eq!(
            parse(&at_cap.join("\n")).entries().map(<[_]>::len),
            Some(RUN_OUTCOMES_MAX_ENTRIES)
        );
        let mut over = at_cap;
        over.push(line("one-too-many"));
        assert_eq!(
            parse(&over.join("\n")),
            CaptureOutcome::Invalid {
                line: None,
                reason: "run_outcomes_too_many".to_string()
            }
        );
    }

    #[test]
    fn the_metric_cap_is_enforced() {
        let metrics: serde_json::Map<String, Value> = (0..=RUN_OUTCOME_MAX_METRICS)
            .map(|i| (format!("m{i}"), json!(1)))
            .collect();
        let bad = json!({"source":"s","subject":"v","task":"t","metrics":metrics,"executed_case_count":1});
        assert!(matches!(
            parse(&bad.to_string()),
            CaptureOutcome::Invalid { reason, .. } if reason == "run_outcome_invalid_metrics"
        ));
    }

    #[test]
    fn an_oversize_file_is_too_large_before_it_is_read() {
        let tmp = tempfile::tempdir().unwrap();
        let p = tmp.path().join("run_outcomes.jsonl");
        std::fs::write(&p, vec![b' '; READ_GUARD_BYTES as usize + 1]).unwrap();
        assert_eq!(
            capture(Ok(&p)),
            CaptureOutcome::TooLarge {
                bytes: READ_GUARD_BYTES + 1,
                of: Measured::File
            }
        );
    }

    /// Whitespace the file carries but the payload does not is not counted
    /// against coord's cap: coord measures the compact array.
    #[test]
    fn whitespace_on_disk_does_not_count_against_coords_cap() {
        let tmp = tempfile::tempdir().unwrap();
        let p = tmp.path().join("run_outcomes.jsonl");
        let mut raw = line("a");
        raw.push_str(&"\n".repeat(RUN_OUTCOMES_MAX_BYTES));
        std::fs::write(&p, raw).unwrap();
        assert_eq!(capture(Ok(&p)).entries().map(<[_]>::len), Some(1));
    }

    /// The payload the node SENDS is the normalized one (integers become
    /// floats), so a file whose own values fit coord's cap can still yield a
    /// payload over it — and that is refused here, never sent to be refused.
    #[test]
    fn a_payload_that_grows_past_the_cap_on_normalization_is_not_sent() {
        let metrics: serde_json::Map<String, Value> = (0..RUN_OUTCOME_MAX_METRICS)
            .map(|i| (format!("metric_{i:02}_{}", "x".repeat(20)), json!(1)))
            .collect();
        let entry = |i: usize| {
            json!({"source": "s", "subject": "v", "task": format!("t{i}"),
                   "metrics": metrics.clone(), "executed_case_count": 1})
        };
        // Add entries while the RAW payload stays within coord's cap.
        let mut entries: Vec<Value> = Vec::new();
        while entries.len() < RUN_OUTCOMES_MAX_ENTRIES {
            let mut next = entries.clone();
            next.push(entry(entries.len()));
            if serde_json::to_vec(&Value::Array(next.clone()))
                .unwrap()
                .len()
                > RUN_OUTCOMES_MAX_BYTES
            {
                break;
            }
            entries = next;
        }
        let raw_size = serde_json::to_vec(&Value::Array(entries.clone()))
            .unwrap()
            .len();
        assert!(raw_size <= RUN_OUTCOMES_MAX_BYTES);
        let text: Vec<String> = entries.iter().map(Value::to_string).collect();
        match parse(&text.join("\n")) {
            CaptureOutcome::TooLarge {
                bytes,
                of: Measured::Payload,
            } => assert!(bytes as usize > RUN_OUTCOMES_MAX_BYTES),
            other => panic!("expected a payload-size refusal, got {other:?} (raw {raw_size})"),
        }
    }

    #[cfg(unix)]
    #[test]
    fn a_symlink_or_fifo_is_never_read() {
        let tmp = tempfile::tempdir().unwrap();
        let link = tmp.path().join("link.jsonl");
        std::os::unix::fs::symlink("/dev/zero", &link).unwrap();
        assert_eq!(
            capture(Ok(&link)),
            CaptureOutcome::Unreadable {
                detail: "not a regular file".to_string()
            }
        );
        let fifo = tmp.path().join("fifo.jsonl");
        let made = std::process::Command::new("mkfifo").arg(&fifo).status();
        if made.map(|s| s.success()).unwrap_or(false) {
            assert_eq!(
                capture(Ok(&fifo)),
                CaptureOutcome::Unreadable {
                    detail: "not a regular file".to_string()
                }
            );
        }
    }

    #[test]
    fn a_non_utf8_file_is_unreadable() {
        let tmp = tempfile::tempdir().unwrap();
        let p = tmp.path().join("run_outcomes.jsonl");
        std::fs::write(&p, [0xff, 0xfe, 0x00]).unwrap();
        assert!(matches!(capture(Ok(&p)), CaptureOutcome::Unreadable { .. }));
    }
}
