//! The `run_outcomes` wire contract of coord's CI result door, and its ONE
//! validator.
//!
//! Plan `2026-10-13-ci-node-forwards-run-outcomes-into-the-dispatch-result`
//! (F2). `POST /coord/ci/dispatches/:dispatch_id/result` carries an optional
//! `run_outcomes` array; coord validates it all-or-nothing and writes one
//! `coord.run_outcomes` row per entry, which the `run_outcome` anchor resolver
//! reads. The CI node validates the SAME payload before it posts it, so a file
//! coord would refuse is reported on the dispatch log instead of being sent
//! and silently rejected.
//!
//! Both sides call [`validate_run_outcomes`] from here rather than keeping a
//! copy each: a mirror drifts the moment one side's rules change, and the
//! node would then either trim what coord accepts or send what coord refuses.
//! This module was moved verbatim from qontinui-coord's
//! `crates/coord/src/run_outcomes.rs` (the pure half) and
//! `crates/coord/src/anchor_observer.rs` (the label and metric-name rules) at
//! coord `origin/main` `ee3fd66e5`; coord's write path, attribution and SQL
//! stay in coord.
//!
//! PURE: no I/O, no clock, no coord state.

use std::collections::BTreeSet;

use serde::Serialize;
use serde_json::{json, Map, Value};

/// Most outcomes one result POST may carry.
pub const RUN_OUTCOMES_MAX_ENTRIES: usize = 32;

/// Most metrics one outcome may carry.
pub const RUN_OUTCOME_MAX_METRICS: usize = 64;

/// Byte bound on the serialized `run_outcomes` value, checked before any walk
/// of its entries.
pub const RUN_OUTCOMES_MAX_BYTES: usize = 64 * 1024;

/// The reserved metric name that reads `coord.run_outcomes.executed_case_count`
/// rather than a key of `metrics`. One source for the count: the validator
/// refuses a `metrics` map carrying this key, so the two can never disagree.
pub const EXECUTED_CASES_METRIC: &str = "executed_cases";

/// Longest `source` / `subject` / `task` label, in bytes. Shared by coord's
/// anchor parser and the writer so a label one accepts the other can match.
pub const RUN_OUTCOME_LABEL_MAX_BYTES: usize = 200;

/// Longest metric name, in bytes.
pub const RUN_OUTCOME_METRIC_NAME_MAX_BYTES: usize = 128;

/// Keys a body may NOT carry on an outcome entry, because coord takes them
/// from the dispatch row. Refused by name
/// ([`RunOutcomeRejection::AttributionInBody`]).
pub const ATTRIBUTION_KEYS: &[&str] = &[
    "tenant_id",
    "tenant",
    "repo",
    "ref",
    "head_sha",
    "dispatch_id",
    "recorded_at",
];

/// The keys an outcome entry carries.
pub const ENTRY_KEYS: &[&str] = &[
    "source",
    "subject",
    "task",
    "metrics",
    "executed_case_count",
];

/// A `run_outcome` source / subject / task label: non-blank, at most
/// [`RUN_OUTCOME_LABEL_MAX_BYTES`], no control characters, and no leading or
/// trailing whitespace (two spellings of one label would be two lookup keys).
pub fn is_valid_run_outcome_label(s: &str) -> bool {
    !s.is_empty()
        && s.len() <= RUN_OUTCOME_LABEL_MAX_BYTES
        && s.trim() == s
        && !s.chars().any(char::is_control)
}

/// A metric name: `[A-Za-z0-9_.:-]`, 1..=[`RUN_OUTCOME_METRIC_NAME_MAX_BYTES`].
pub fn is_valid_metric_name(s: &str) -> bool {
    !s.is_empty()
        && s.len() <= RUN_OUTCOME_METRIC_NAME_MAX_BYTES
        && s.bytes()
            .all(|b| b.is_ascii_alphanumeric() || matches!(b, b'_' | b'.' | b':' | b'-'))
}

/// One validated outcome. Serializes to exactly the entry shape the result
/// door accepts (the five [`ENTRY_KEYS`]), so a validated entry can be sent as
/// it is.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct RunOutcomeEntry {
    pub source: String,
    pub subject: String,
    pub task: String,
    /// A JSON object of metric name to finite number.
    pub metrics: Value,
    pub executed_case_count: i64,
}

/// Why a `run_outcomes` payload was not recorded. The wire token
/// ([`RunOutcomeRejection::as_str`]) is what coord reports back under
/// `run_outcomes.rejected`, so a missing row is always attributable to a named
/// cause. The variants after [`RunOutcomeRejection::DuplicateTask`] are
/// produced by coord's write path, never by [`validate_run_outcomes`]; they
/// live here so the vocabulary is one table both sides parse.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RunOutcomeRejection {
    /// Not a JSON array.
    NotAnArray,
    /// An empty array — nothing to record, and almost certainly a harness bug.
    Empty,
    /// More than [`RUN_OUTCOMES_MAX_ENTRIES`] entries.
    TooMany,
    /// Serialized larger than [`RUN_OUTCOMES_MAX_BYTES`].
    TooLarge,
    /// An entry is not an object, misses a required key, or carries an
    /// unknown one.
    Malformed,
    /// An entry names a key coord takes from the dispatch row.
    AttributionInBody,
    /// `source` / `subject` / `task` is blank, too long, padded, or carries a
    /// control character.
    InvalidLabel,
    /// `metrics` is not an object, carries too many keys, or names an invalid
    /// metric.
    InvalidMetrics,
    /// A metric value is not a finite number.
    InvalidMetricValue,
    /// `metrics` names [`EXECUTED_CASES_METRIC`], which is read from
    /// `executed_case_count` and must have exactly one source.
    ReservedMetric,
    /// `executed_case_count` is not a non-negative integer.
    InvalidCaseCount,
    /// Two entries share one `(source, subject, task)`.
    DuplicateTask,
    /// The dispatch concluded `cancelled`; its numbers describe an unfinished
    /// run.
    DispatchCancelled,
    /// The dispatch row carries no tenant to attribute the outcomes to.
    Unattributable,
    /// `coord.run_outcomes` does not exist yet.
    TableAbsent,
    /// The write itself failed.
    WriteFailed,
    /// A re-POST arrived after coord's retry window closed.
    RetryWindowClosed,
    /// The dispatch row carries no completion time.
    CompletionUnknown,
    /// The ledger refused the POST itself, so its outcomes were not considered.
    NotAccepting,
}

impl RunOutcomeRejection {
    /// Every variant, in declaration order.
    pub const ALL: &'static [RunOutcomeRejection] = &[
        RunOutcomeRejection::NotAnArray,
        RunOutcomeRejection::Empty,
        RunOutcomeRejection::TooMany,
        RunOutcomeRejection::TooLarge,
        RunOutcomeRejection::Malformed,
        RunOutcomeRejection::AttributionInBody,
        RunOutcomeRejection::InvalidLabel,
        RunOutcomeRejection::InvalidMetrics,
        RunOutcomeRejection::InvalidMetricValue,
        RunOutcomeRejection::ReservedMetric,
        RunOutcomeRejection::InvalidCaseCount,
        RunOutcomeRejection::DuplicateTask,
        RunOutcomeRejection::DispatchCancelled,
        RunOutcomeRejection::Unattributable,
        RunOutcomeRejection::TableAbsent,
        RunOutcomeRejection::WriteFailed,
        RunOutcomeRejection::RetryWindowClosed,
        RunOutcomeRejection::CompletionUnknown,
        RunOutcomeRejection::NotAccepting,
    ];

    /// The wire token.
    pub fn as_str(self) -> &'static str {
        match self {
            RunOutcomeRejection::NotAnArray => "run_outcomes_not_an_array",
            RunOutcomeRejection::Empty => "run_outcomes_empty",
            RunOutcomeRejection::TooMany => "run_outcomes_too_many",
            RunOutcomeRejection::TooLarge => "run_outcomes_too_large",
            RunOutcomeRejection::Malformed => "run_outcome_malformed",
            RunOutcomeRejection::AttributionInBody => "run_outcome_attribution_in_body",
            RunOutcomeRejection::InvalidLabel => "run_outcome_invalid_label",
            RunOutcomeRejection::InvalidMetrics => "run_outcome_invalid_metrics",
            RunOutcomeRejection::InvalidMetricValue => "run_outcome_invalid_metric_value",
            RunOutcomeRejection::ReservedMetric => "run_outcome_reserved_metric",
            RunOutcomeRejection::InvalidCaseCount => "run_outcome_invalid_case_count",
            RunOutcomeRejection::DuplicateTask => "run_outcome_duplicate_task",
            RunOutcomeRejection::DispatchCancelled => "dispatch_cancelled",
            RunOutcomeRejection::Unattributable => "dispatch_row_missing_tenant",
            RunOutcomeRejection::TableAbsent => "table_absent",
            RunOutcomeRejection::WriteFailed => "write_failed",
            RunOutcomeRejection::RetryWindowClosed => "retry_window_closed",
            RunOutcomeRejection::CompletionUnknown => "dispatch_completion_unknown",
            RunOutcomeRejection::NotAccepting => "dispatch_not_accepting",
        }
    }

    /// Parse a wire token back. `None` for a token this build does not know —
    /// a coord newer than this crate — which a reader must treat as UNKNOWN,
    /// never as one of the known causes.
    pub fn from_wire(token: &str) -> Option<Self> {
        Self::ALL.iter().copied().find(|r| r.as_str() == token)
    }
}

/// A rejection plus the index of the entry that caused it, when one did.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Rejected {
    pub reason: RunOutcomeRejection,
    pub index: Option<usize>,
}

impl Rejected {
    /// A rejection of the payload as a whole.
    pub fn whole(reason: RunOutcomeRejection) -> Self {
        Rejected {
            reason,
            index: None,
        }
    }

    /// A rejection caused by the entry at `index`.
    pub fn at(index: usize, reason: RunOutcomeRejection) -> Self {
        Rejected {
            reason,
            index: Some(index),
        }
    }

    /// The `run_outcomes` value coord reports back to the caller.
    pub fn to_json(self) -> Value {
        match self.index {
            Some(i) => json!({"rejected": self.reason.as_str(), "index": i}),
            None => json!({"rejected": self.reason.as_str()}),
        }
    }
}

/// Validate a raw `run_outcomes` value. PURE.
///
/// `Ok(None)` when the key is absent or `null` — the common case: most
/// dispatches are not evaluation runs. Otherwise every entry is validated and
/// the first failure rejects the WHOLE payload.
pub fn validate_run_outcomes(
    value: Option<&Value>,
) -> Result<Option<Vec<RunOutcomeEntry>>, Rejected> {
    let Some(value) = value else {
        return Ok(None);
    };
    if value.is_null() {
        return Ok(None);
    }
    let Some(entries) = value.as_array() else {
        return Err(Rejected::whole(RunOutcomeRejection::NotAnArray));
    };
    if entries.is_empty() {
        return Err(Rejected::whole(RunOutcomeRejection::Empty));
    }
    if entries.len() > RUN_OUTCOMES_MAX_ENTRIES {
        return Err(Rejected::whole(RunOutcomeRejection::TooMany));
    }
    // Serializing a Value cannot fail; an error here would only mean a
    // non-string map key, which serde_json cannot produce.
    let size = serde_json::to_vec(value)
        .map(|v| v.len())
        .unwrap_or(usize::MAX);
    if size > RUN_OUTCOMES_MAX_BYTES {
        return Err(Rejected::whole(RunOutcomeRejection::TooLarge));
    }

    let mut out = Vec::with_capacity(entries.len());
    let mut seen: BTreeSet<(String, String, String)> = BTreeSet::new();
    for (i, entry) in entries.iter().enumerate() {
        let e = validate_entry(entry).map_err(|r| Rejected::at(i, r))?;
        if !seen.insert((e.source.clone(), e.subject.clone(), e.task.clone())) {
            return Err(Rejected::at(i, RunOutcomeRejection::DuplicateTask));
        }
        out.push(e);
    }
    Ok(Some(out))
}

fn validate_entry(entry: &Value) -> Result<RunOutcomeEntry, RunOutcomeRejection> {
    let obj = entry.as_object().ok_or(RunOutcomeRejection::Malformed)?;
    if obj.keys().any(|k| ATTRIBUTION_KEYS.contains(&k.as_str())) {
        return Err(RunOutcomeRejection::AttributionInBody);
    }
    if obj.keys().any(|k| !ENTRY_KEYS.contains(&k.as_str())) {
        return Err(RunOutcomeRejection::Malformed);
    }
    let label = |key: &str| -> Result<String, RunOutcomeRejection> {
        let s = obj
            .get(key)
            .ok_or(RunOutcomeRejection::Malformed)?
            .as_str()
            .ok_or(RunOutcomeRejection::InvalidLabel)?;
        if is_valid_run_outcome_label(s) {
            Ok(s.to_string())
        } else {
            Err(RunOutcomeRejection::InvalidLabel)
        }
    };
    let source = label("source")?;
    let subject = label("subject")?;
    let task = label("task")?;
    let metrics = validate_metrics(obj.get("metrics").ok_or(RunOutcomeRejection::Malformed)?)?;
    let executed_case_count = obj
        .get("executed_case_count")
        .ok_or(RunOutcomeRejection::Malformed)?
        .as_u64()
        .and_then(|n| i64::try_from(n).ok())
        .ok_or(RunOutcomeRejection::InvalidCaseCount)?;
    Ok(RunOutcomeEntry {
        source,
        subject,
        task,
        metrics,
        executed_case_count,
    })
}

fn validate_metrics(v: &Value) -> Result<Value, RunOutcomeRejection> {
    let m = v.as_object().ok_or(RunOutcomeRejection::InvalidMetrics)?;
    if m.len() > RUN_OUTCOME_MAX_METRICS {
        return Err(RunOutcomeRejection::InvalidMetrics);
    }
    let mut out = Map::with_capacity(m.len());
    for (name, value) in m {
        if name == EXECUTED_CASES_METRIC {
            return Err(RunOutcomeRejection::ReservedMetric);
        }
        if !is_valid_metric_name(name) {
            return Err(RunOutcomeRejection::InvalidMetrics);
        }
        let n = value
            .as_f64()
            .filter(|f| f.is_finite())
            .ok_or(RunOutcomeRejection::InvalidMetricValue)?;
        out.insert(name.clone(), json!(n));
    }
    Ok(Value::Object(out))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn entry() -> Value {
        json!({
            "source": "reference-eval",
            "subject": "oracle",
            "task": "synthetic/spec_defect",
            "metrics": {"recall": 98.5, "runs_total": 3},
            "executed_case_count": 40
        })
    }

    fn reject(v: Value) -> Rejected {
        validate_run_outcomes(Some(&v)).expect_err("must be rejected")
    }

    #[test]
    fn absent_and_null_are_nothing_to_record() {
        assert_eq!(validate_run_outcomes(None), Ok(None));
        assert_eq!(validate_run_outcomes(Some(&Value::Null)), Ok(None));
    }

    #[test]
    fn a_valid_entry_round_trips_to_the_wire_shape() {
        let got = validate_run_outcomes(Some(&json!([entry()])))
            .unwrap()
            .unwrap();
        assert_eq!(got.len(), 1);
        assert_eq!(got[0].executed_case_count, 40);
        // Serializes back to exactly the entry keys the door accepts.
        let wire = serde_json::to_value(&got[0]).unwrap();
        let keys: Vec<&str> = wire
            .as_object()
            .unwrap()
            .keys()
            .map(String::as_str)
            .collect();
        let mut want = ENTRY_KEYS.to_vec();
        want.sort_unstable();
        let mut keys_sorted = keys.clone();
        keys_sorted.sort_unstable();
        assert_eq!(keys_sorted, want);
        // Re-validating the serialized entry accepts it unchanged.
        let again = validate_run_outcomes(Some(&json!([wire])))
            .unwrap()
            .unwrap();
        assert_eq!(again, got);
    }

    #[test]
    fn whole_payload_rules() {
        assert_eq!(reject(json!({})).reason, RunOutcomeRejection::NotAnArray);
        assert_eq!(reject(json!([])).reason, RunOutcomeRejection::Empty);
        let many: Vec<Value> = (0..=RUN_OUTCOMES_MAX_ENTRIES)
            .map(|i| {
                let mut e = entry();
                e["task"] = json!(format!("t{i}"));
                e
            })
            .collect();
        assert_eq!(reject(json!(many)).reason, RunOutcomeRejection::TooMany);
        let mut big = entry();
        big["source"] = json!("x".repeat(RUN_OUTCOMES_MAX_BYTES));
        assert_eq!(reject(json!([big])).reason, RunOutcomeRejection::TooLarge);
    }

    #[test]
    fn exactly_the_cap_is_accepted() {
        let at_cap: Vec<Value> = (0..RUN_OUTCOMES_MAX_ENTRIES)
            .map(|i| {
                let mut e = entry();
                e["task"] = json!(format!("t{i}"));
                e
            })
            .collect();
        let got = validate_run_outcomes(Some(&json!(at_cap)))
            .unwrap()
            .unwrap();
        assert_eq!(got.len(), RUN_OUTCOMES_MAX_ENTRIES);
    }

    #[test]
    fn attribution_keys_are_refused_by_name() {
        for key in ATTRIBUTION_KEYS {
            let mut e = entry();
            e[*key] = json!("x");
            let r = reject(json!([e]));
            assert_eq!(r.reason, RunOutcomeRejection::AttributionInBody, "{key}");
            assert_eq!(r.index, Some(0));
        }
    }

    #[test]
    fn unknown_and_missing_keys_are_malformed() {
        let mut e = entry();
        e["extra"] = json!(1);
        assert_eq!(reject(json!([e])).reason, RunOutcomeRejection::Malformed);
        for key in ENTRY_KEYS {
            let mut e = entry();
            e.as_object_mut().unwrap().remove(*key);
            assert_eq!(
                reject(json!([e])).reason,
                RunOutcomeRejection::Malformed,
                "missing {key}"
            );
        }
        assert_eq!(reject(json!([1])).reason, RunOutcomeRejection::Malformed);
    }

    #[test]
    fn label_rules() {
        for bad in [json!(""), json!(" padded"), json!("tab\there"), json!(7)] {
            let mut e = entry();
            e["subject"] = bad.clone();
            assert_eq!(
                reject(json!([e])).reason,
                RunOutcomeRejection::InvalidLabel,
                "{bad}"
            );
        }
        let mut e = entry();
        e["task"] = json!("t".repeat(RUN_OUTCOME_LABEL_MAX_BYTES + 1));
        assert_eq!(reject(json!([e])).reason, RunOutcomeRejection::InvalidLabel);
        assert!(is_valid_run_outcome_label(
            &"t".repeat(RUN_OUTCOME_LABEL_MAX_BYTES)
        ));
    }

    #[test]
    fn metric_rules() {
        let mut e = entry();
        e["metrics"] = json!([1]);
        assert_eq!(
            reject(json!([e])).reason,
            RunOutcomeRejection::InvalidMetrics
        );

        let mut e = entry();
        e["metrics"] = json!({"bad name": 1});
        assert_eq!(
            reject(json!([e])).reason,
            RunOutcomeRejection::InvalidMetrics
        );

        let many: Map<String, Value> = (0..=RUN_OUTCOME_MAX_METRICS)
            .map(|i| (format!("m{i}"), json!(1)))
            .collect();
        let mut e = entry();
        e["metrics"] = Value::Object(many);
        assert_eq!(
            reject(json!([e])).reason,
            RunOutcomeRejection::InvalidMetrics
        );

        let mut e = entry();
        e["metrics"] = json!({"recall": "high"});
        assert_eq!(
            reject(json!([e])).reason,
            RunOutcomeRejection::InvalidMetricValue
        );

        let mut e = entry();
        e["metrics"] = json!({"recall": null});
        assert_eq!(
            reject(json!([e])).reason,
            RunOutcomeRejection::InvalidMetricValue
        );

        let mut e = entry();
        e["metrics"] = json!({ EXECUTED_CASES_METRIC: 3 });
        assert_eq!(
            reject(json!([e])).reason,
            RunOutcomeRejection::ReservedMetric
        );

        assert!(is_valid_metric_name("recall.min"));
        assert!(is_valid_metric_name("a:b-c_d"));
        assert!(!is_valid_metric_name(
            &"m".repeat(RUN_OUTCOME_METRIC_NAME_MAX_BYTES + 1)
        ));
    }

    #[test]
    fn case_count_must_be_a_non_negative_integer() {
        for bad in [json!(-1), json!(1.5), json!("3"), Value::Null] {
            let mut e = entry();
            e["executed_case_count"] = bad.clone();
            assert_eq!(
                reject(json!([e])).reason,
                RunOutcomeRejection::InvalidCaseCount,
                "{bad}"
            );
        }
        let mut e = entry();
        e["executed_case_count"] = json!(0);
        assert!(validate_run_outcomes(Some(&json!([e]))).is_ok());
    }

    #[test]
    fn duplicate_triples_are_refused_at_the_second_index() {
        let r = reject(json!([entry(), entry()]));
        assert_eq!(r.reason, RunOutcomeRejection::DuplicateTask);
        assert_eq!(r.index, Some(1));
        // A different task is a different triple.
        let mut other = entry();
        other["task"] = json!("other");
        assert!(validate_run_outcomes(Some(&json!([entry(), other]))).is_ok());
    }

    #[test]
    fn wire_tokens_are_unique_and_parse_back() {
        let mut seen = BTreeSet::new();
        for r in RunOutcomeRejection::ALL {
            assert!(seen.insert(r.as_str()), "duplicate token {}", r.as_str());
            assert_eq!(RunOutcomeRejection::from_wire(r.as_str()), Some(*r));
        }
        assert_eq!(RunOutcomeRejection::from_wire("from_a_newer_coord"), None);
        assert_eq!(
            Rejected::at(2, RunOutcomeRejection::Empty).to_json(),
            json!({"rejected": "run_outcomes_empty", "index": 2})
        );
        assert_eq!(
            Rejected::whole(RunOutcomeRejection::WriteFailed).to_json(),
            json!({"rejected": "write_failed"})
        );
    }
}
