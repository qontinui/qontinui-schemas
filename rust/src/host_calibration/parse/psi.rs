//! Pressure-stall information: `/proc/pressure/{cpu,memory,io}` and the
//! cgroup v2 `{cpu,memory,io}.pressure` files, which share one format:
//!
//! ```text
//! some avg10=2.12 avg60=7.81 avg300=2.78 total=28433288250
//! full avg10=1.95 avg60=7.18 avg300=2.56 total=25269186748
//! ```
//!
//! This is the ONE PSI parser (plan vet correction 7). It accepts every input
//! the computers plan's host reader (runner#1982 `fleet/host_axes.rs`
//! `parse_psi`) accepts and yields the same `(avg10, avg60)` pairs through
//! [`PsiLine::pair`], plus `avg300` and `total`, so that reader can switch to
//! it without a behaviour change.

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use super::parse_nonneg_f64;

/// One PSI line. `avg10`/`avg60` are required for the line to count (the
/// host reader's rule); `avg300` and `total` are carried when present.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct PsiLine {
    /// Percent of wall time stalled, 10 s average.
    pub avg10: f64,
    /// 60 s average.
    pub avg60: f64,
    /// 300 s average.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub avg300: Option<f64>,
    /// Cumulative stall time in microseconds.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub total_usec: Option<u64>,
}

impl PsiLine {
    /// `(avg10, avg60)` — the host reader's shape.
    pub fn pair(&self) -> (f64, f64) {
        (self.avg10, self.avg60)
    }
}

/// One PSI file. `full` is `None` when its line is absent:
/// `/proc/pressure/cpu` has no `full` line before kernel 5.13, which is
/// UNKNOWN, not "zero stall".
#[derive(Debug, Clone, Copy, PartialEq, Default, Serialize, Deserialize, JsonSchema)]
pub struct Psi {
    /// Some task stalled.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub some: Option<PsiLine>,
    /// All non-idle tasks stalled.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub full: Option<PsiLine>,
}

/// Parse one PSI file. `None` when neither line parses (not a PSI file).
///
/// A repeated `some`/`full` line overwrites the earlier one, and a line
/// missing `avg10` or `avg60` clears its slot — both exactly as the host
/// reader does. Negative or non-finite averages are rejected.
pub fn parse_psi(text: &str) -> Option<Psi> {
    let mut psi = Psi::default();
    for line in text.lines() {
        let mut it = line.split_whitespace();
        let Some(kind) = it.next() else { continue };
        let (mut avg10, mut avg60, mut avg300, mut total) = (None, None, None, None);
        for kv in it {
            match kv.split_once('=') {
                Some(("avg10", v)) => avg10 = parse_nonneg_f64(v),
                Some(("avg60", v)) => avg60 = parse_nonneg_f64(v),
                Some(("avg300", v)) => avg300 = parse_nonneg_f64(v),
                Some(("total", v)) => total = v.parse::<u64>().ok(),
                _ => {}
            }
        }
        let parsed = match (avg10, avg60) {
            (Some(avg10), Some(avg60)) => Some(PsiLine {
                avg10,
                avg60,
                avg300,
                total_usec: total,
            }),
            _ => None,
        };
        match kind {
            "some" => psi.some = parsed,
            "full" => psi.full = parsed,
            _ => {}
        }
    }
    (psi.some.is_some() || psi.full.is_some()).then_some(psi)
}

#[cfg(test)]
mod tests {
    use super::*;

    const MEM: &str = "some avg10=2.12 avg60=7.81 avg300=2.78 total=28433288250\n\
                       full avg10=1.95 avg60=7.18 avg300=2.56 total=25269186748\n";

    #[test]
    fn parses_all_four_fields_of_both_lines() {
        let p = parse_psi(MEM).unwrap();
        let s = p.some.unwrap();
        assert_eq!(s.pair(), (2.12, 7.81));
        assert_eq!(s.avg300, Some(2.78));
        assert_eq!(s.total_usec, Some(28_433_288_250));
        assert_eq!(p.full.unwrap().pair(), (1.95, 7.18));
    }

    #[test]
    fn old_kernel_cpu_without_full_is_none_not_zero() {
        let p = parse_psi("some avg10=12.50 avg60=9.25 avg300=4.00 total=123\n").unwrap();
        assert_eq!(p.some.unwrap().pair(), (12.5, 9.25));
        assert_eq!(p.full, None);
        let z = parse_psi("some avg10=0 avg60=0 avg300=0 total=0\nfull avg10=0.00 avg60=0.00 avg300=0.00 total=0\n")
            .unwrap();
        assert_eq!(z.full.unwrap().pair(), (0.0, 0.0));
    }

    #[test]
    fn host_reader_superset_cases() {
        // avg10 + avg60 alone is still a line (the host reader requires only those).
        let p = parse_psi("some avg10=1.00 avg60=2.00\n").unwrap();
        assert_eq!(p.some.unwrap().avg300, None);
        // Missing avg60 → the line does not count.
        assert_eq!(parse_psi("some avg10=1.00 avg300=2.00\n"), None);
        // Negative / non-finite rejected.
        assert_eq!(parse_psi("some avg10=-1 avg60=2\n"), None);
        assert_eq!(parse_psi("some avg10=NaN avg60=2\n"), None);
        // A later bad line clears the slot, as the host reader does.
        let q = parse_psi("some avg10=1 avg60=1\nsome avg10=x avg60=1\nfull avg10=1 avg60=1\n")
            .unwrap();
        assert_eq!(q.some, None);
        assert!(q.full.is_some());
    }

    #[test]
    fn non_psi_text_is_none() {
        assert_eq!(parse_psi(""), None);
        assert_eq!(parse_psi("garbage here\n"), None);
        assert_eq!(parse_psi("100\n"), None);
    }
}
