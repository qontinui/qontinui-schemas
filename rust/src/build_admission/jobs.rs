//! `CARGO_BUILD_JOBS` for one lease (D2).
//!
//! The arithmetic is qontinui-runner's CI sizing (`src-tauri/src/ci_node/host_sizing.rs`
//! `derive`), applied to a PER-LEASE memory share rather than to the whole
//! host. The identity, pinned by `matches_runner_derive_of_share` below, is
//!
//! ```text
//! jobs(m, c, n) == derive(share({mem: m·n, cpus: c}, n)).cargo_build_jobs
//! ```
//!
//! i.e. the runner's `share` divides HOST memory by `n`, while this function is
//! handed a memory share that is already per lease (D2's `mem_share` = this
//! lease's reservation plus its share of unreserved budget), so only the cpus
//! are divided here. The golden table below is therefore keyed by the
//! per-lease share, NOT by host memory: it is not the host-memory table plan
//! `2026-10-02-cargo-jobs-supervisor-build-pool-and-physical-disk-floors-follow-the-machine`
//! D2 describes, and must not be byte-compared against one.

/// Bytes of headroom assumed per concurrent cargo job (3 GiB).
pub const BYTES_PER_BUILD_TOKEN: u64 = 3 * 1024 * 1024 * 1024;
/// At or below this memory share the job count is pinned to 1 (measured on the
/// runner's CI lane: `JOBS=2` still OOM-killed a 7 GB host).
pub const SMALL_SHARE_BYTES: u64 = 8 * 1024 * 1024 * 1024;
/// Job count when the memory share is unknown: an unknown host is a small one.
pub const UNKNOWN_SHARE_JOBS: u32 = 1;

/// Jobs for a lease whose PER-LEASE memory share is `lease_mem_share_bytes`, on
/// a host with `cpus` usable cpus shared by `concurrent` running builds (this
/// one included).
///
/// The cpu share is `max(1, cpus / concurrent)`; the job count is
/// `share / 3 GiB` clamped to `[1, cpu share]`, then pinned to 1 when the
/// share is at most 8 GiB — the pin applied LAST so it overrides the cpu cap,
/// as in `host_sizing::derive`.
pub fn jobs(lease_mem_share_bytes: Option<u64>, cpus: u32, concurrent: u32) -> u32 {
    let cpu_share = (cpus.max(1) / concurrent.max(1)).max(1);
    let Some(mem) = lease_mem_share_bytes.filter(|m| *m > 0) else {
        return UNKNOWN_SHARE_JOBS;
    };
    let mut j = (mem / BYTES_PER_BUILD_TOKEN).clamp(1, u32::MAX as u64) as u32;
    j = j.min(cpu_share);
    if mem <= SMALL_SHARE_BYTES {
        j = 1;
    }
    j
}

#[cfg(test)]
mod tests {
    use super::*;

    const G: u64 = 1 << 30;

    /// The PER-LEASE golden table: `(lease share GiB, cpus, concurrent) -> jobs`.
    /// Rows cover the unknown share, the <= 8 GiB pin, a memory-bound share, a
    /// cpu-bound share and a 48-cpu box alone and shared.
    #[test]
    fn golden_table() {
        let rows: &[(Option<f64>, u32, u32, u32)] = &[
            (None, 48, 1, 1),
            (Some(0.0), 48, 1, 1),
            (Some(7.0), 16, 1, 1),
            (Some(8.0), 16, 1, 1),
            (Some(9.0), 16, 1, 3),
            (Some(31.7), 16, 1, 10),
            (Some(64.0), 8, 1, 8),
            (Some(368.0), 48, 1, 48),
            (Some(148.0), 48, 1, 48),
            (Some(74.0), 48, 2, 24),
            (Some(30.0), 48, 4, 10),
            (Some(15.0), 48, 12, 4),
            (Some(15.0), 48, 100, 1),
            (Some(368.0), 1, 1, 1),
        ];
        for &(gib, cpus, concurrent, want) in rows {
            let mem = gib.map(|g| (g * G as f64) as u64);
            assert_eq!(
                jobs(mem, cpus, concurrent),
                want,
                "jobs({gib:?} GiB, {cpus} cpus, {concurrent} concurrent)"
            );
        }
    }

    /// A line-for-line port of the runner's `derive` + `share`
    /// (`host_sizing.rs` on qontinui-runner main) for the build-jobs field.
    fn runner_derive_of_share(host_mem: Option<u64>, cpus: u32, n: u32) -> u32 {
        let n = n.max(1);
        let (mem, cpus) = (host_mem.map(|m| m / n as u64), (cpus / n).max(1));
        let cpus = cpus.max(1);
        let Some(mem) = mem.filter(|m| *m > 0) else {
            return 1;
        };
        let mut j = (mem / BYTES_PER_BUILD_TOKEN).clamp(1, u32::MAX as u64) as u32;
        j = j.max(1).min(cpus);
        if mem <= SMALL_SHARE_BYTES {
            j = 1;
        }
        j
    }

    #[test]
    fn matches_runner_derive_of_share() {
        for share_gib in [0u64, 1, 3, 7, 8, 9, 12, 15, 31, 64, 148, 368] {
            for cpus in [0u32, 1, 2, 7, 16, 48, 64] {
                for n in [0u32, 1, 2, 3, 4, 12, 100] {
                    let share = share_gib * G;
                    let host = share * n.max(1) as u64;
                    assert_eq!(
                        jobs(Some(share), cpus, n),
                        runner_derive_of_share(Some(host), cpus, n),
                        "share {share_gib} GiB, {cpus} cpus, {n} concurrent"
                    );
                }
            }
            assert_eq!(jobs(None, 48, 1), runner_derive_of_share(None, 48, 1));
        }
    }

    #[test]
    fn zero_cpus_and_zero_concurrent_are_clamped() {
        assert_eq!(jobs(Some(64 * G), 0, 0), 1);
    }
}
