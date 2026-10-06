//! A ticket's memory reservation from its own key's measured history (D2).

use super::types::{Estimate, EstimateKey, EstimateSource, Policy};

/// One completed lease's measurement, in completion order (oldest first).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Measurement {
    pub key: EstimateKey,
    /// Peak anonymous (+ unevictable) bytes, by the key's measure method.
    pub peak_bytes: u64,
    /// The job count it ran with.
    pub jobs: u32,
    /// Run time, lease start to end, minus paused seconds.
    pub run_s: u64,
}

/// A labelled starting value for a key with too little history.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Seed {
    pub key: EstimateKey,
    pub bytes: u64,
}

/// Nearest-rank percentile of `vals` (`q` in `0..=1`); `None` when empty.
pub fn nearest_rank(vals: &[u64], q: f64) -> Option<u64> {
    if vals.is_empty() {
        return None;
    }
    let mut s = vals.to_vec();
    s.sort_unstable();
    let idx = ((q * s.len() as f64).ceil() as usize).clamp(1, s.len()) - 1;
    Some(s[idx])
}

/// The estimate for `key`.
///
/// - At least `policy.min_measurements` measurements of the same key: the p90
///   of the peaks over the last `policy.history_window`, source `measured(n)`.
///   When those measurements span more than one job count and `at_jobs` is
///   given, the p90 is shifted by the least-squares slope (bytes per job,
///   floored at 0 — more jobs is never assumed to use less memory) times
///   `at_jobs` minus the mean job count, and never below the smallest observed
///   peak.
/// - Otherwise the seed for the key, source `seed`.
/// - Otherwise `unknown`.
///
/// `duration_s` is the p50 run time of the same window when measured, and is
/// `None` for a seed or unknown estimate (backfill then never relies on it).
pub fn estimate(
    key: &EstimateKey,
    history: &[Measurement],
    seeds: &[Seed],
    at_jobs: Option<u32>,
    policy: &Policy,
) -> Estimate {
    let same: Vec<&Measurement> = history.iter().filter(|m| &m.key == key).collect();
    let window = policy.history_window as usize;
    let recent = &same[same.len().saturating_sub(window)..];
    if recent.len() >= policy.min_measurements as usize {
        let peaks: Vec<u64> = recent.iter().map(|m| m.peak_bytes).collect();
        let runs: Vec<u64> = recent.iter().map(|m| m.run_s).collect();
        let p90 = nearest_rank(&peaks, 0.9).unwrap_or(0);
        let bytes = match at_jobs {
            Some(j) => scale_to_jobs(recent, p90, j),
            None => p90,
        };
        return Estimate {
            bytes: Some(bytes),
            duration_s: nearest_rank(&runs, 0.5),
            source: EstimateSource::Measured {
                n: recent.len() as u32,
            },
        };
    }
    match seeds.iter().find(|s| &s.key == key) {
        Some(s) => Estimate {
            bytes: Some(s.bytes),
            duration_s: None,
            source: EstimateSource::Seed,
        },
        None => Estimate::UNKNOWN,
    }
}

fn scale_to_jobs(recent: &[&Measurement], p90: u64, at_jobs: u32) -> u64 {
    let n = recent.len() as f64;
    let mean_j = recent.iter().map(|m| m.jobs as f64).sum::<f64>() / n;
    let mean_b = recent.iter().map(|m| m.peak_bytes as f64).sum::<f64>() / n;
    let var: f64 = recent
        .iter()
        .map(|m| (m.jobs as f64 - mean_j).powi(2))
        .sum();
    if var == 0.0 {
        return p90;
    }
    let cov: f64 = recent
        .iter()
        .map(|m| (m.jobs as f64 - mean_j) * (m.peak_bytes as f64 - mean_b))
        .sum();
    let slope = (cov / var).max(0.0);
    let floor = recent.iter().map(|m| m.peak_bytes).min().unwrap_or(0) as f64;
    let shifted = p90 as f64 + slope * (at_jobs as f64 - mean_j);
    shifted.max(floor).round() as u64
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::build_admission::types::{MeasureMethod, Subcommand, TargetDirKind, GIB};

    fn key(repo: &str) -> EstimateKey {
        EstimateKey {
            repo: repo.to_owned(),
            subcommand: Subcommand::Test,
            profile: "dev".to_owned(),
            target_dir_kind: TargetDirKind::SharedWarm,
            measure_method: MeasureMethod::CgroupAnon,
        }
    }

    fn m(repo: &str, gib: u64, jobs: u32) -> Measurement {
        Measurement {
            key: key(repo),
            peak_bytes: gib * GIB,
            jobs,
            run_s: gib * 60,
        }
    }

    #[test]
    fn unknown_without_history_or_seed() {
        let e = estimate(&key("a"), &[], &[], None, &Policy::default());
        assert_eq!(e, Estimate::UNKNOWN);
    }

    #[test]
    fn seed_until_three_measurements() {
        let seeds = [Seed {
            key: key("a"),
            bytes: 15 * GIB,
        }];
        let hist = [m("a", 10, 8), m("a", 11, 8), m("b", 99, 8)];
        let e = estimate(&key("a"), &hist, &seeds, None, &Policy::default());
        assert_eq!(e.source, EstimateSource::Seed);
        assert_eq!(e.bytes, Some(15 * GIB));
        assert_eq!(e.duration_s, None);
    }

    #[test]
    fn p90_of_the_last_twenty_of_the_same_key() {
        // 25 measurements 1..=25 GiB; the window keeps 6..=25, p90 = rank 18 = 23.
        let hist: Vec<Measurement> = (1..=25).map(|g| m("a", g, 8)).collect();
        let e = estimate(&key("a"), &hist, &[], None, &Policy::default());
        assert_eq!(e.source, EstimateSource::Measured { n: 20 });
        assert_eq!(e.bytes, Some(23 * GIB));
        assert_eq!(e.duration_s, Some(15 * 60));
    }

    #[test]
    fn scales_by_the_measured_slope_only_when_jobs_vary() {
        let flat = [m("a", 10, 8), m("a", 10, 8), m("a", 10, 8)];
        let e = estimate(&key("a"), &flat, &[], Some(32), &Policy::default());
        assert_eq!(e.bytes, Some(10 * GIB));
        // 1 GiB per job: 8 jobs -> 8 GiB, 16 -> 16, 24 -> 24; mean 16; p90 24.
        let sloped = [m("a", 8, 8), m("a", 16, 16), m("a", 24, 24)];
        let e = estimate(&key("a"), &sloped, &[], Some(32), &Policy::default());
        assert_eq!(e.bytes, Some(40 * GIB));
        // Fewer jobs shrinks it, but never below the smallest observed peak.
        let e = estimate(&key("a"), &sloped, &[], Some(1), &Policy::default());
        assert_eq!(e.bytes, Some(9 * GIB));
        let e = estimate(&key("a"), &sloped, &[], Some(0), &Policy::default());
        assert_eq!(e.bytes, Some(8 * GIB));
    }

    #[test]
    fn a_negative_slope_is_not_extrapolated() {
        let inverse = [m("a", 24, 8), m("a", 16, 16), m("a", 8, 24)];
        let e = estimate(&key("a"), &inverse, &[], Some(48), &Policy::default());
        assert_eq!(e.bytes, Some(24 * GIB));
    }

    #[test]
    fn nearest_rank_matches_the_metrics_script() {
        assert_eq!(nearest_rank(&[], 0.9), None);
        assert_eq!(nearest_rank(&(1..=10).collect::<Vec<_>>(), 0.9), Some(9));
        assert_eq!(nearest_rank(&[5, 1, 3], 0.5), Some(3));
    }
}
