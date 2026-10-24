//! Edge timing — the latency of one journey EDGE as a distribution over its
//! observations, never a single "the latency" (plan
//! `2026-09-20-ui-bridge-represents-the-users-path-and-the-passage-of-time`,
//! Phase 4: `GET /apps/{app_id}/journey/edge-timing?from=&to=`).
//!
//! The runner selects the ledger rows of one edge (its `from`/`to` node keys,
//! under whatever filter it applies), parses each row's `timeline` column as
//! a [`ChangeTimeline`] (the [`ChangeTimeline::for_ledger`] projection it
//! stored), and hands them to [`edge_timing`] as [`EdgeTimingSample`]s. The
//! answer carries every per-observation value (as the [`Observation`] the SDK
//! produced, so an `unknown` stays an `unknown`), the sample size, and per
//! metric `min` / `median` / `p90` / `max` over the MEASURED values only, with
//! the absent and unknown counts beside them — so `n = 12, measured = 3` can
//! never read as a 12-sample median.
//!
//! Pure, I/O-free, deterministic: rows are ordered by `(observedAt,
//! observationId)`; percentiles are nearest-rank (`rank = ceil(p * n)`, so the
//! median of an even count is the LOWER middle value) over values sorted with
//! `f64::total_cmp`; no clock is read; no threshold is applied — whether a
//! p90 is "slow" is the caller's call.

use std::collections::{BTreeMap, BTreeSet};

use chrono::{DateTime, SecondsFormat, Utc};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::observation::{Observation, ObservationState};
use crate::timeline::ChangeTimeline;

/// One stored observation of an edge: a ledger row's identity and its parsed
/// `timeline` column. A row whose `timeline` is NULL (recorded with
/// `timeline: false`, or by a choke point that records none) is passed with
/// `timeline: None` — it counts toward `n` and toward `withoutTimeline`, and
/// contributes to no metric.
#[derive(Debug, Clone, PartialEq)]
pub struct EdgeTimingSample {
    /// `journey_edge_observations.id`.
    pub observation_id: String,
    /// `journey_edge_observations.observed_at`.
    pub observed_at: DateTime<Utc>,
    /// `journey_edge_observations.app_version`; `None` = not reported.
    pub app_version: Option<String>,
    /// The parsed `timeline` column; `None` = the row has none.
    pub timeline: Option<ChangeTimeline>,
}

/// One observation's values, exactly as its timeline summary stated them.
/// `null` (for every metric) = the row has no timeline.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct EdgeTimingRow {
    /// The ledger row id — a `/vision/assert` `timeline_ref`.
    pub observation_id: String,
    /// RFC 3339 UTC.
    pub observed_at: String,
    /// `null` = not reported.
    pub app_version: Option<String>,
    /// Whether the row carried a timeline.
    pub has_timeline: bool,
    /// `summary.firstResponseMs`.
    pub first_response_ms: Option<Observation<f64>>,
    /// `summary.settleMs`.
    pub settle_ms: Option<Observation<f64>>,
    /// `summary.layoutShiftTotal`.
    pub layout_shift_total: Option<Observation<f64>>,
    /// `summary.layoutShiftCount`.
    pub layout_shift_count: Option<Observation<u64>>,
    /// `summary.layoutShiftAfterFirstResponseMs`.
    pub layout_shift_after_first_response_ms: Option<Observation<f64>>,
    /// `summary.longTaskTotalMs`.
    pub long_task_total_ms: Option<Observation<f64>>,
}

/// One metric over every sample. The order statistics are over the MEASURED
/// values only (degraded ones included — their value stands) and are `null`
/// when none was measured; `absent` and `unknownByCode` say what the rest
/// were. `measured + absent + Σ unknownByCode + withoutTimeline = n`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct MetricDistribution {
    /// Samples with a measured value.
    pub measured: u64,
    /// Of those, how many were degraded (`coverage.unmeasured` non-empty).
    pub degraded: u64,
    /// Samples whose producer looked and found the thing did not occur.
    pub absent: u64,
    /// Samples the producer could not answer, per `UnknownCode` wire value.
    pub unknown_by_code: BTreeMap<String, u64>,
    /// Samples with no timeline at all.
    pub without_timeline: u64,
    /// Smallest measured value.
    pub min: Option<f64>,
    /// Nearest-rank 50th percentile (the lower middle value of an even count).
    pub median: Option<f64>,
    /// Nearest-rank 90th percentile.
    pub p90: Option<f64>,
    /// Largest measured value.
    pub max: Option<f64>,
}

/// The answer of [`edge_timing`].
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct EdgeTiming {
    /// Samples (ledger rows) the answer is over.
    pub n: u64,
    /// Of those, rows with no timeline.
    pub without_timeline: u64,
    /// Earliest `observedAt`; `null` when `n` is 0.
    pub oldest_observed_at: Option<String>,
    /// Latest `observedAt`; `null` when `n` is 0.
    pub newest_observed_at: Option<String>,
    /// Distinct reported app versions, sorted, `null` (= not reported) first.
    pub app_versions: Vec<Option<String>>,
    /// `summary.firstResponseMs`.
    pub first_response_ms: MetricDistribution,
    /// `summary.settleMs`.
    pub settle_ms: MetricDistribution,
    /// `summary.layoutShiftTotal`.
    pub layout_shift_total: MetricDistribution,
    /// `summary.layoutShiftCount`.
    pub layout_shift_count: MetricDistribution,
    /// `summary.layoutShiftAfterFirstResponseMs`.
    pub layout_shift_after_first_response_ms: MetricDistribution,
    /// `summary.longTaskTotalMs`.
    pub long_task_total_ms: MetricDistribution,
    /// Every observation's raw values, ordered by `(observedAt,
    /// observationId)`.
    pub observations: Vec<EdgeTimingRow>,
}

/// Nearest-rank percentile of an ascending, non-empty slice.
fn nearest_rank(sorted: &[f64], p: f64) -> f64 {
    let n = sorted.len();
    let rank = ((p * n as f64).ceil() as usize).clamp(1, n);
    sorted[rank - 1]
}

fn distribution<'a, T: 'a + Copy + Into<f64>>(
    values: impl Iterator<Item = Option<&'a Observation<T>>>,
) -> MetricDistribution {
    let mut d = MetricDistribution {
        measured: 0,
        degraded: 0,
        absent: 0,
        unknown_by_code: BTreeMap::new(),
        without_timeline: 0,
        min: None,
        median: None,
        p90: None,
        max: None,
    };
    let mut measured: Vec<f64> = Vec::new();
    for v in values {
        let Some(o) = v else {
            d.without_timeline += 1;
            continue;
        };
        match o.state() {
            ObservationState::Measured(x) => {
                d.measured += 1;
                if !o.provenance().coverage.is_full() {
                    d.degraded += 1;
                }
                measured.push((*x).into());
            }
            ObservationState::Absent => d.absent += 1,
            ObservationState::Unknown(u) => {
                *d.unknown_by_code
                    .entry(u.code.as_str().to_string())
                    .or_default() += 1;
            }
        }
    }
    if !measured.is_empty() {
        measured.sort_by(f64::total_cmp);
        d.min = measured.first().copied();
        d.max = measured.last().copied();
        d.median = Some(nearest_rank(&measured, 0.5));
        d.p90 = Some(nearest_rank(&measured, 0.9));
    }
    d
}

/// `u64` has no lossless `Into<f64>`; counts on one edge are far below 2^53.
#[derive(Clone, Copy)]
struct Count(u64);

impl From<Count> for f64 {
    fn from(c: Count) -> f64 {
        c.0 as f64
    }
}

fn rfc3339(t: DateTime<Utc>) -> String {
    t.to_rfc3339_opts(SecondsFormat::AutoSi, true)
}

/// Reduce one edge's stored observations to its timing distribution.
///
/// Order-independent: the same samples in any order give an equal answer.
pub fn edge_timing(samples: &[EdgeTimingSample]) -> EdgeTiming {
    let mut sorted: Vec<&EdgeTimingSample> = samples.iter().collect();
    sorted.sort_by(|a, b| {
        a.observed_at
            .cmp(&b.observed_at)
            .then_with(|| a.observation_id.cmp(&b.observation_id))
    });

    let rows: Vec<EdgeTimingRow> = sorted
        .iter()
        .map(|s| {
            let sum = s.timeline.as_ref().map(|t| &t.summary);
            EdgeTimingRow {
                observation_id: s.observation_id.clone(),
                observed_at: rfc3339(s.observed_at),
                app_version: s.app_version.clone(),
                has_timeline: s.timeline.is_some(),
                first_response_ms: sum.map(|x| x.first_response_ms.clone()),
                settle_ms: sum.map(|x| x.settle_ms.clone()),
                layout_shift_total: sum.map(|x| x.layout_shift_total.clone()),
                layout_shift_count: sum.map(|x| x.layout_shift_count.clone()),
                layout_shift_after_first_response_ms: sum
                    .map(|x| x.layout_shift_after_first_response_ms.clone()),
                long_task_total_ms: sum.map(|x| x.long_task_total_ms.clone()),
            }
        })
        .collect();

    let counts: Vec<Option<Observation<Count>>> = rows
        .iter()
        .map(|r| r.layout_shift_count.clone().map(|o| o.map(Count)))
        .collect();

    let app_versions: BTreeSet<Option<&str>> =
        sorted.iter().map(|s| s.app_version.as_deref()).collect();

    EdgeTiming {
        n: rows.len() as u64,
        without_timeline: rows.iter().filter(|r| !r.has_timeline).count() as u64,
        oldest_observed_at: sorted.first().map(|s| rfc3339(s.observed_at)),
        newest_observed_at: sorted.last().map(|s| rfc3339(s.observed_at)),
        app_versions: app_versions
            .into_iter()
            .map(|v| v.map(str::to_string))
            .collect(),
        first_response_ms: distribution(rows.iter().map(|r| r.first_response_ms.as_ref())),
        settle_ms: distribution(rows.iter().map(|r| r.settle_ms.as_ref())),
        layout_shift_total: distribution(rows.iter().map(|r| r.layout_shift_total.as_ref())),
        layout_shift_count: distribution(counts.iter().map(Option::as_ref)),
        layout_shift_after_first_response_ms: distribution(
            rows.iter()
                .map(|r| r.layout_shift_after_first_response_ms.as_ref()),
        ),
        long_task_total_ms: distribution(rows.iter().map(|r| r.long_task_total_ms.as_ref())),
        observations: rows,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;
    use serde_json::{json, Value};

    fn prov(unmeasured: Value) -> Value {
        json!({
            "producer": { "id": "sdk/timeline/events", "version": "test" },
            "observedAt": "2026-10-20T00:00:00Z",
            "evaluatedAt": "2026-10-20T00:00:00Z",
            "coverage": { "considered": 1, "measured": 1, "unmeasured": unmeasured },
            "confidence": null, "cache": null, "source": null
        })
    }
    fn measured(v: Value) -> Value {
        json!({ "status": "measured", "value": v, "provenance": prov(json!([])) })
    }
    fn unknown(code: &str) -> Value {
        json!({ "status": "unknown", "unknown": { "code": code, "detail": "t" }, "provenance": prov(json!([])) })
    }

    fn timeline(first_response: Value, settle: Value) -> ChangeTimeline {
        serde_json::from_value(json!({
            "events": [{ "type": "action", "offsetMs": 0, "summary": "a" }],
            "recordedMs": 1000, "settled": true, "observedSources": ["snapshot-poll"],
            "summary": {
                "firstResponseMs": first_response,
                "settleMs": settle,
                "layoutShiftTotal": unknown("performance_observer_unsupported"),
                "layoutShiftCount": measured(json!(2)),
                "layoutShiftAfterFirstResponseMs": unknown("performance_observer_unsupported"),
                "animationsUnsettledAtEnd": measured(json!([])),
                "longTaskTotalMs": measured(json!(0)),
                "expectedStates": []
            }
        }))
        .unwrap()
    }

    fn sample(id: &str, secs: i64, t: Option<ChangeTimeline>) -> EdgeTimingSample {
        EdgeTimingSample {
            observation_id: id.into(),
            observed_at: Utc.timestamp_opt(1_790_000_000 + secs, 0).unwrap(),
            app_version: Some(format!("v{}", secs % 2)),
            timeline: t,
        }
    }

    fn ledger() -> Vec<EdgeTimingSample> {
        vec![
            sample(
                "e1",
                1,
                Some(timeline(measured(json!(40)), measured(json!(300)))),
            ),
            sample(
                "e2",
                2,
                Some(timeline(measured(json!(10)), measured(json!(100)))),
            ),
            sample(
                "e3",
                3,
                Some(timeline(
                    measured(json!(30)),
                    unknown("no_event_before_timeout"),
                )),
            ),
            sample(
                "e4",
                4,
                Some(timeline(measured(json!(20)), measured(json!(200)))),
            ),
            sample("e5", 5, None),
        ]
    }

    #[test]
    fn a_distribution_carries_its_sample_size_and_what_was_not_measured() {
        let t = edge_timing(&ledger());
        assert_eq!(t.n, 5);
        assert_eq!(t.without_timeline, 1);
        let f = &t.first_response_ms;
        assert_eq!((f.measured, f.absent, f.without_timeline), (4, 0, 1));
        assert_eq!(
            (f.min, f.median, f.p90, f.max),
            (Some(10.0), Some(20.0), Some(40.0), Some(40.0))
        );
        let s = &t.settle_ms;
        assert_eq!(s.measured, 3);
        assert_eq!(s.unknown_by_code["no_event_before_timeout"], 1);
        assert_eq!(
            (s.min, s.median, s.max),
            (Some(100.0), Some(200.0), Some(300.0))
        );
        // Nothing measured: no order statistic, never a 0.
        let l = &t.layout_shift_total;
        assert_eq!(l.measured, 0);
        assert_eq!(l.unknown_by_code["performance_observer_unsupported"], 4);
        assert_eq!((l.min, l.median, l.p90, l.max), (None, None, None, None));
        let v = serde_json::to_value(&t).unwrap();
        assert_eq!(v["layoutShiftTotal"]["median"], json!(null));
        // Counts are a distribution too.
        assert_eq!(t.layout_shift_count.median, Some(2.0));
        // Every sample is accounted for.
        for d in [&t.first_response_ms, &t.settle_ms, &t.layout_shift_total] {
            let unknown: u64 = d.unknown_by_code.values().sum();
            assert_eq!(d.measured + d.absent + unknown + d.without_timeline, t.n);
        }
    }

    #[test]
    fn raw_rows_keep_each_observation_and_order_is_irrelevant() {
        let mut shuffled = ledger();
        shuffled.reverse();
        shuffled.swap(0, 3);
        let a = edge_timing(&ledger());
        let b = edge_timing(&shuffled);
        assert_eq!(
            serde_json::to_string(&a).unwrap(),
            serde_json::to_string(&b).unwrap()
        );
        let ids: Vec<&str> = a
            .observations
            .iter()
            .map(|r| r.observation_id.as_str())
            .collect();
        assert_eq!(ids, ["e1", "e2", "e3", "e4", "e5"]);
        assert_eq!(
            a.observations[2].settle_ms.as_ref().unwrap().status(),
            crate::observation::ObservationStatus::Unknown
        );
        assert!(!a.observations[4].has_timeline);
        assert_eq!(
            a.app_versions,
            [Some("v0".to_string()), Some("v1".to_string())]
        );
        assert_eq!(
            a.oldest_observed_at,
            Some(rfc3339(Utc.timestamp_opt(1_790_000_001, 0).unwrap()))
        );
        assert_eq!(
            a.newest_observed_at,
            Some(rfc3339(Utc.timestamp_opt(1_790_000_005, 0).unwrap()))
        );
    }

    #[test]
    fn no_samples_is_an_empty_distribution_not_zeros() {
        let t = edge_timing(&[]);
        assert_eq!(t.n, 0);
        assert_eq!(t.oldest_observed_at, None);
        assert_eq!(t.first_response_ms.median, None);
        assert_eq!(t.first_response_ms.measured, 0);
    }

    #[test]
    fn nearest_rank_percentiles() {
        let v = [1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0, 10.0];
        assert_eq!(nearest_rank(&v, 0.5), 5.0);
        assert_eq!(nearest_rank(&v, 0.9), 9.0);
        assert_eq!(nearest_rank(&[7.0], 0.9), 7.0);
    }
}
