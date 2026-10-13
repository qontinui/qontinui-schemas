//! Pure parsers over already-read file contents. No IO here: the runner's
//! collector and the CI host agent read the files, tests pass fixtures.
//!
//! Every parser returns `None` / `Err` for text that is not the file it
//! parses — never a zeroed value — so a caller maps a failed parse to
//! [`super::measured::Measured::Unavailable`].

pub mod cgroup;
pub mod proc;
pub mod psi;
pub mod sccache;

/// A finite, non-negative float, or `None`.
pub(crate) fn parse_nonneg_f64(tok: &str) -> Option<f64> {
    tok.parse::<f64>()
        .ok()
        .filter(|v| v.is_finite() && *v >= 0.0)
}
