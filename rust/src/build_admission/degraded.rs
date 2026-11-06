//! The slot count of the wrapper's broker-less degraded arm (D4).
//!
//! With no broker the wrapper evaluates D3's budget from what it can read on
//! its own — MemTotal, the CI slices' cgroup files, and the non-build p95 and
//! per-key estimates from the broker's last `estimates.json` — and allows
//! `N = max(1, floor(budget / max_est))` concurrent builds (each also holding a
//! per-output-directory slot). If ANY term is unreadable, `N = 1`: a dead
//! broker degrades admission to coarser, never to none.

use super::types::Policy;

/// What the wrapper could read. `None` = unreadable.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DegradedInputs {
    pub mem_total_bytes: Option<u64>,
    pub non_build_p95_bytes: Option<u64>,
    pub ci_reservation_bytes: Option<u64>,
    /// The largest per-key estimate in the last `estimates.json`.
    pub max_est_bytes: Option<u64>,
}

/// `N` for the degraded arm.
pub fn degraded_slots(inputs: &DegradedInputs, policy: &Policy) -> u32 {
    let (Some(total), Some(non_build), Some(ci), Some(max_est)) = (
        inputs.mem_total_bytes,
        inputs.non_build_p95_bytes,
        inputs.ci_reservation_bytes,
        inputs.max_est_bytes.filter(|b| *b > 0),
    ) else {
        return 1;
    };
    let budget = total
        .saturating_sub(policy.reserve_floor(total))
        .saturating_sub(non_build)
        .saturating_sub(ci);
    ((budget / max_est).min(u32::MAX as u64) as u32).max(1)
}
