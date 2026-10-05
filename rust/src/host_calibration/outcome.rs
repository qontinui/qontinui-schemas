//! The build-outcome ledger line: one JSON object per guarded build, which
//! cargo-guard and cargo-verify append to the calibration `builds.jsonl`
//! (plan Phase 2b). The runner ships aggregates of these, never raw lines.
//!
//! Times are Unix epoch MILLISECONDS rather than the crate's usual ISO
//! strings: the writer is a bash wrapper (`date +%s%3N`), and wall time is
//! `ended_at_ms − started_at_ms` with no date parsing on either side.

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// The ledger line shape's version. Bumped on any non-additive change.
pub const BUILD_OUTCOME_SCHEMA_VERSION: u32 = 1;

/// Which wrapper ran the build.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum BuildWrapper {
    /// `cargo-guard.sh` (qontinui-runner).
    CargoGuard,
    /// `cargo-verify.sh` (coord, schemas, supervisor, inspect).
    CargoVerify,
}

/// One guarded build.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct BuildOutcome {
    /// [`BUILD_OUTCOME_SCHEMA_VERSION`] at write time.
    #[serde(default = "default_schema_version")]
    pub schema_version: u32,
    /// Repo name (`qontinui-runner`), never a path.
    pub repo: String,
    /// Build start, epoch ms.
    pub started_at_ms: u64,
    /// Build end, epoch ms.
    pub ended_at_ms: u64,
    /// Hash of the crate set built (workspace members + features + profile),
    /// so builds of the same set compare.
    pub crate_set_hash: String,
    /// Count of `Compiling` lines; `None` when the wrapper could not count.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub crates_recompiled: Option<u32>,
    /// Filesystem type of the target dir (`ext4`, `tmpfs`, `ReFS`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub target_fs_type: Option<String>,
    /// Which wrapper.
    pub wrapper: BuildWrapper,
    /// The CPU-weight slice the build ran in, when it was placed in one.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cpu_weight_slice: Option<String>,
    /// Whether the compile cache wrapper was active for this build.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub compile_cache_active: Option<bool>,
    /// Active calibration trial ids from the profile at build start.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub trial_ids: Vec<String>,
    /// cargo's exit status.
    pub exit: i32,
}

fn default_schema_version() -> u32 {
    BUILD_OUTCOME_SCHEMA_VERSION
}

impl BuildOutcome {
    /// Wall time in ms; `None` when the clock went backwards.
    pub fn wall_ms(&self) -> Option<u64> {
        self.ended_at_ms.checked_sub(self.started_at_ms)
    }

    /// True for a zero exit.
    pub fn succeeded(&self) -> bool {
        self.exit == 0
    }
}

/// A parsed ledger: the good lines, and how many lines did not parse.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct BuildLedger {
    /// Lines that parsed, in file order.
    pub outcomes: Vec<BuildOutcome>,
    /// Non-blank lines that did not (a torn last line after a crash, …).
    pub malformed_lines: usize,
}

/// Parse one ledger line.
pub fn parse_build_outcome_line(line: &str) -> Result<BuildOutcome, serde_json::Error> {
    serde_json::from_str(line.trim())
}

/// Parse a whole `builds.jsonl`. Malformed lines are counted, not fatal: a
/// torn write must not hide every other build.
pub fn parse_build_ledger(text: &str) -> BuildLedger {
    let mut ledger = BuildLedger::default();
    for line in text.lines().filter(|l| !l.trim().is_empty()) {
        match parse_build_outcome_line(line) {
            Ok(o) => ledger.outcomes.push(o),
            Err(_) => ledger.malformed_lines += 1,
        }
    }
    ledger
}

#[cfg(test)]
mod tests {
    use super::*;

    const LINE: &str = r#"{"schema_version":1,"repo":"qontinui-runner","started_at_ms":1759659600000,"ended_at_ms":1759659720500,"crate_set_hash":"a1b2","crates_recompiled":14,"target_fs_type":"ext4","wrapper":"cargo_guard","cpu_weight_slice":"qontinui-builds.slice","trial_ids":["t-1"],"exit":0}"#;

    #[test]
    fn a_line_round_trips() {
        let o = parse_build_outcome_line(LINE).unwrap();
        assert_eq!(o.wall_ms(), Some(120_500));
        assert!(o.succeeded());
        assert_eq!(o.wrapper, BuildWrapper::CargoGuard);
        assert_eq!(o.compile_cache_active, None);
        let back = serde_json::to_string(&o).unwrap();
        assert_eq!(parse_build_outcome_line(&back).unwrap(), o);
    }

    #[test]
    fn minimal_line_defaults_and_backwards_clock() {
        let o = parse_build_outcome_line(
            r#"{"repo":"r","started_at_ms":10,"ended_at_ms":5,"crate_set_hash":"h","wrapper":"cargo_verify","exit":101}"#,
        )
        .unwrap();
        assert_eq!(o.schema_version, BUILD_OUTCOME_SCHEMA_VERSION);
        assert_eq!(o.wall_ms(), None);
        assert!(!o.succeeded());
        assert_eq!(o.crates_recompiled, None);
    }

    #[test]
    fn ledger_counts_torn_lines() {
        let text = format!("{LINE}\n\n{{\"repo\":\"r\",\"start\n{LINE}\n");
        let l = parse_build_ledger(&text);
        assert_eq!(l.outcomes.len(), 2);
        assert_eq!(l.malformed_lines, 1);
    }
}
