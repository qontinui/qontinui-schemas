//! Golden table for `host_sizing::build_jobs`.
//!
//! `tests/fixtures/build-jobs-golden.tsv` is the ONE table the Rust formula and
//! the bash engine (`qontinui-claude-config/scripts/lib/build-jobs.sh`, which
//! vendors a copy of this file and pins its sha256) are both asserted against.
//! Editing a row here changes the contract for both engines: re-vendor the file
//! and restamp the pinned digest there. Plan
//! `2026-10-02-cargo-jobs-supervisor-build-pool-and-physical-disk-floors-follow-the-machine`, D2.
//!
//! Columns (tab-separated): `mem_bytes  cpus  others  jobs  note`.
//! `mem_bytes` `unknown` is an unreadable memory probe (`None`); `#` lines are
//! comments.

use qontinui_types::host_sizing::{build_jobs, HostCapacity};

/// Minimum number of data rows; guards against a truncated or emptied table
/// passing vacuously.
const MIN_ROWS: usize = 10;

#[test]
fn build_jobs_matches_every_golden_row() {
    let path = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/fixtures/build-jobs-golden.tsv"
    );
    let table = std::fs::read_to_string(path).expect("read build-jobs-golden.tsv");

    let mut rows = 0usize;
    let mut failures = Vec::new();
    for (idx, line) in table.lines().enumerate() {
        let lineno = idx + 1;
        if line.trim().is_empty() || line.starts_with('#') {
            continue;
        }
        let cols: Vec<&str> = line.split('\t').collect();
        assert_eq!(
            cols.len(),
            5,
            "line {lineno}: expected 5 tab-separated columns, got {}: {line:?}",
            cols.len()
        );
        let mem_bytes = match cols[0] {
            "unknown" => None,
            v => Some(
                v.parse::<u64>()
                    .unwrap_or_else(|e| panic!("line {lineno}: bad mem_bytes {v:?}: {e}")),
            ),
        };
        let parse_u32 = |name: &str, v: &str| -> u32 {
            v.parse::<u32>()
                .unwrap_or_else(|e| panic!("line {lineno}: bad {name} {v:?}: {e}"))
        };
        let cpus = parse_u32("cpus", cols[1]);
        let others = parse_u32("others", cols[2]);
        let want = parse_u32("jobs", cols[3]);
        let note = cols[4];

        let got = build_jobs(HostCapacity { mem_bytes, cpus }, others);
        if got != want {
            failures.push(format!(
                "line {lineno} ({note}): build_jobs(mem={}, cpus={cpus}, others={others}) = {got}, golden says {want}",
                cols[0]
            ));
        }
        rows += 1;
    }

    assert!(
        rows >= MIN_ROWS,
        "golden table read only {rows} data rows (< {MIN_ROWS}); refusing a vacuous pass"
    );
    assert!(
        failures.is_empty(),
        "{} of {rows} golden rows disagree:\n{}",
        failures.len(),
        failures.join("\n")
    );
}
