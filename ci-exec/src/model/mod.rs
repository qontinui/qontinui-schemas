//! Model access for CI steps (plan
//! `2026-10-13-ci-node-runs-model-dependent-eval-steps`).
//!
//! A step never holds a model credential. A repo declares what its steps may
//! spend in `[model]` ([`declaration`]); the steps that need a model set
//! `model = true`; and the executor hands only those steps a dispatch-bound
//! capability for a per-dispatch loopback broker ([`broker`]), which calls the
//! model through the HOST's [`crate::host::ModelInvoker`]. The step talks to the
//! broker through the `qontinui-ci-model` shim ([`shim`]), which mirrors
//! `claude -p`.
//!
//! | Module | What it owns |
//! |---|---|
//! | [`declaration`] | the `[model]` table, the closed [`declaration::MODEL_REGISTRY`] and the budget ceilings |
//! | [`protocol`] | the broker's request, its `claude -p`-compatible envelope and the error subtypes |
//! | [`broker`] | the per-dispatch listener: capability token, request checks, budgets, the exhaustion hold |
//! | [`ledger`] | the per-call ledger and the result's `summary.model` section |
//! | [`shim`] | the `qontinui-ci-model` client a step runs |
//! | [`scrub`] | the env every CI step loses, and the empty `CLAUDE_CONFIG_DIR` it gets instead |
//! | [`access`] | the executor's ordered refusal checks and the per-step exports |
//!
//! # What credential separation covers (D11)
//!
//! Everything the executor and broker hand a step: the step's environment has
//! the model-credential class, every inherited `*_MODEL_CMD` and the host's own
//! credential values removed, and `CLAUDE_CONFIG_DIR` pointed at an empty
//! directory; its argv and protocol carry only the shim's path and a
//! dispatch-bound capability. It does NOT cover a step that deliberately
//! reaches for what the node owner's OS user can read (the account login
//! files, the runner's own process environment): every Local-lane step runs as
//! that user. That residual is recorded as `isolation: "env"` in every ledger.

pub(crate) mod access;
pub mod broker;
pub mod declaration;
pub mod ledger;
pub mod protocol;
pub mod scrub;
pub mod shim;

pub use scrub::MODEL_CREDENTIAL_ENV;

/// The shim's command line, exported to `model = true` steps on a grant.
pub const MODEL_CMD_ENV: &str = "QONTINUI_CI_MODEL_CMD";
/// The broker's base URL (`http://127.0.0.1:<port>`), exported on a grant.
pub const MODEL_URL_ENV: &str = "QONTINUI_CI_MODEL_URL";
/// The per-dispatch capability, exported on a grant. Sent only as
/// `Authorization: Bearer`.
pub const MODEL_TOKEN_ENV: &str = "QONTINUI_CI_MODEL_TOKEN";
/// The closed refusal reason, exported to `model = true` steps when model
/// access was withheld, so a harness can write UNKNOWN rows that say why.
pub const MODEL_REFUSED_ENV: &str = "QONTINUI_CI_MODEL_REFUSED";
/// The dispatch id, exported to every step.
pub const DISPATCH_ID_ENV: &str = "QONTINUI_CI_DISPATCH_ID";
/// Set for every step to an empty per-dispatch directory (D11).
pub const CLAUDE_CONFIG_DIR_ENV: &str = "CLAUDE_CONFIG_DIR";

/// Refusal reasons coord decides (D12). Passed through from the dispatch
/// payload's `model_access.reason`.
pub const COORD_REFUSALS: &[&str] = &[
    "fork_pr",
    "fork_status_unknown",
    "repo_not_tenant_owned",
    "trigger_not_trusted",
    "preference_off_tenant",
    "preference_off_repo",
    "preference_unknown",
    "rate_capped",
    "already_evaluated",
    "schema_pending",
];

/// Refusal reasons the node decides (D12).
pub const RUNNER_REFUSALS: &[&str] = &[
    "not_offered_by_coord",
    "device_consent_off",
    "host_has_no_broker",
    "shim_missing",
    "cli_missing",
    "cli_version_unsupported",
    "no_model_account",
    "gateway_unresolved",
    "broker_start_failed",
];

/// Whether `reason` is one of the closed refusal reasons.
pub fn is_known_refusal(reason: &str) -> bool {
    COORD_REFUSALS.contains(&reason) || RUNNER_REFUSALS.contains(&reason)
}

/// Constant-time equality for secrets (the broker's capability token).
///
/// `ct_eq` requires equal lengths; the length is not secret (the token is a
/// fixed-width hex string), so comparing it first is safe and is what every
/// constant-time string compare does. Mirrors the runner's private
/// `coord_mcp::secret_eq`.
pub fn secret_eq(a: &[u8], b: &[u8]) -> bool {
    use subtle::ConstantTimeEq;
    a.len() == b.len() && bool::from(a.ct_eq(b))
}

#[cfg(test)]
pub(crate) mod test_support;

#[cfg(test)]
mod tests {
    use super::*;

    /// Correct verdicts are what a unit test can assert; the timing property
    /// is `subtle`'s.
    #[test]
    fn secret_eq_compares_length_then_content() {
        assert!(secret_eq(b"abcd", b"abcd"));
        assert!(!secret_eq(b"abcd", b"abce"));
        assert!(!secret_eq(b"abcd", b"abc"));
        assert!(!secret_eq(b"", b"a"));
        assert!(secret_eq(b"", b""));
    }

    /// The reason vocabulary is closed and the two halves do not overlap.
    #[test]
    fn refusal_reasons_are_closed_and_disjoint() {
        for r in COORD_REFUSALS {
            assert!(!RUNNER_REFUSALS.contains(r), "{r}");
            assert!(is_known_refusal(r));
        }
        assert!(is_known_refusal("host_has_no_broker"));
        assert!(!is_known_refusal("all_accounts_exhausted"));
    }
}
