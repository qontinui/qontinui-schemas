//! `CARGO_BUILD_JOBS` for one lease (D2).
//!
//! This is the sizing arithmetic of qontinui-runner's CI lane
//! (`src-tauri/src/ci_node/host_sizing.rs`: `derive(share(cap, n))`)
//! operation for operation, so a CI job and an admitted agent build of the same
//! share get the same job count. Plan
//! `2026-10-02-cargo-jobs-supervisor-build-pool-and-physical-disk-floors-follow-the-machine`
//! D2 names that formula and its golden table; the table lives in this
//! module's tests and is the one the other engines are to be checked against.

/// Bytes of headroom assumed per concurrent cargo job (3 GiB).
pub const BYTES_PER_BUILD_TOKEN: u64 = 3 * 1024 * 1024 * 1024;
/// At or below this memory share the job count is pinned to 1 (measured on the
/// runner's CI lane: `JOBS=2` still OOM-killed a 7 GB host).
pub const SMALL_SHARE_BYTES: u64 = 8 * 1024 * 1024 * 1024;
/// Job count when the memory share is unknown: an unknown host is a small one.
pub const UNKNOWN_SHARE_JOBS: u32 = 1;

/// Jobs for a lease whose memory share is `mem_share_bytes`, on a host with
/// `cpus` usable cpus shared by `concurrent` running builds (this one included).
///
/// The cpu share is `max(1, cpus / concurrent)`; the job count is
/// `mem_share / 3 GiB` clamped to `[1, cpu share]`, then pinned to 1 when the
/// share is at most 8 GiB — the pin applied LAST so it overrides the cpu cap,
/// as in `host_sizing::derive`.
pub fn jobs(mem_share_bytes: Option<u64>, cpus: u32, concurrent: u32) -> u32 {
    let cpu_share = (cpus.max(1) / concurrent.max(1)).max(1);
    let Some(mem) = mem_share_bytes.filter(|m| *m > 0) else {
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

    /// The golden table: `(mem GiB, cpus, concurrent) -> jobs`. Rows cover the
    /// unknown share, the <= 8 GiB pin, a memory-bound box, a cpu-bound box and
    /// a 48-cpu / 368 GiB box alone and shared.
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

    #[test]
    fn zero_cpus_and_zero_concurrent_are_clamped() {
        assert_eq!(jobs(Some(64 * G), 0, 0), 1);
    }
}
