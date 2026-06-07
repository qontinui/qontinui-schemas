//! Canonical dev-environment state vocabulary (the cause-side context).
//!
//! Dev-states are *named predicates over live observations* — evaluated, never
//! inferred (the paper's `s ∩ E_Ξ ≠ ∅`; Spinak 2025 §5.3). Multiple are active
//! simultaneously: the active set `S_Ξ` is the "You Are Here" context recorded
//! into every Action Snapshot
//! (`plans/2026-06-07-twin-dev-event-cause-effect-ledger.md` §5.1).
//!
//! This module is the shared (Phase-2) registry that supersedes the supervisor's
//! Phase-1 hardcoded seed consts in `src/dev_action/states.rs` — the canonical
//! strings here MUST match those byte-for-byte (a compile-guard test in this
//! module pins the overlap so Phase-2b's migration is a re-import, not a
//! rename). It mirrors the shipped [`crate::ui_bridge_diagnostics`] `UB-*`
//! registry exactly: canonical string ids, `as_str` / `FromStr` round-trip,
//! serde, and — like that registry — there is **intentionally no DB**. A
//! release train to add a state is acceptable because every consumer (the
//! supervisor evaluator, coord ingest, the mock harness) must change in
//! lockstep anyway (resolved Q5). DB-loadable predicate rows are a real future
//! need only once non-engineers author states; not v1.
//!
//! States = cause-side context; [`crate::dev_signatures`] `DEV-*` codes =
//! effect-side classification. The two registries sit side by side, the same
//! way `UB-*` codes do.

use std::fmt;
use std::str::FromStr;

use serde::{Deserialize, Serialize};

/// Which component can evaluate a given dev-state's predicate. The supervisor
/// holds the build/binary/slot/health facts in memory; coord holds the
/// git/CI/deploy facts. A state is evaluated only by its owning scope; the
/// other scope records it as [`Eval::Unknown`] (a blind spot, never a silent
/// absence — D4 honesty).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DevStateScope {
    /// Evaluated by `qontinui-supervisor` (build-pool, binary, dist, health).
    Supervisor,
    /// Evaluated by `qontinui-coord` (origin/main CI, sibling drift, deploy).
    Coord,
}

/// The truth value of a single dev-state predicate evaluation.
///
/// [`Eval::Unknown`] is a **D4 known-unknown**: the predicate's input was
/// unavailable (a probe that did not run, a path that could not be stat'd, a
/// state queried outside its evaluating scope). It is *blind*, not *absent* —
/// it MUST never be silently dropped or collapsed into `False`, because a state
/// that could not be evaluated is weak evidence, not negative evidence (the
/// matching layer applies an explicit coverage discount, resolved Q4). A
/// `False` means "the predicate ran and was not satisfied"; `Unknown` means
/// "the predicate could not run."
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Eval {
    /// The predicate ran and was satisfied.
    True,
    /// The predicate ran and was not satisfied.
    False,
    /// The predicate could not run — a known-unknown (D4). Never silently
    /// dropped; carries a coverage discount into matching.
    Unknown,
}

/// A canonical dev-environment state.
///
/// Each variant is a named predicate over live observations. The doc comment on
/// each states its **predicate** (observable source + condition) and its
/// **evaluator scope** (which component can evaluate it). The supervisor-scoped
/// six are shipped on supervisor `main` as hardcoded consts in
/// `src/dev_action/states.rs` (PR #79) — their canonical strings are pinned
/// byte-for-byte by [`tests::matches_supervisor_phase1_literals`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum DevState {
    /// **Predicate:** no slot exe is present in the build pool
    /// (`target-pool/slot-*/debug/`). **Source:** the supervisor's
    /// `build_pool.slots` exe paths — none exist on disk. A fresh checkout or a
    /// wiped pool where resolution must fall back to LKG or the legacy exe.
    /// **Scope:** supervisor.
    SlotsEmpty,
    /// **Predicate:** resolution fell back to the legacy `target/debug` exe (no
    /// build-pool slot). **Source:** `resolve_source_exe_with_slot` returned a
    /// `None` slot id (preference 3) — the genuinely-new signal §2 of the plan
    /// calls out, distinct from `compute_stale_binary` (slot-drift), which is
    /// structurally incapable of catching the no-slot case. This is the state
    /// that white-screened the primary on 2026-06-07 (a months-old binary with
    /// no embedded assets). **Scope:** supervisor.
    LegacyExeFallback,
    /// **Predicate:** the LKG exe was built before the newest source mtime —
    /// pinning to it would run stale code. **Source:** `lkg.json` `built_at`
    /// `<` newest src mtime (the existing `lkg_stale_warning` logic).
    /// **Scope:** supervisor.
    LkgStale,
    /// **Predicate:** `dist/index.html` exists but is OLDER than the newest
    /// frontend source — the embedded UI does not reflect current source.
    /// **Source:** dist mtime vs src mtimes. **Scope:** supervisor.
    DistStale,
    /// **Predicate:** `dist/index.html` is absent or empty. **Source:**
    /// `std::fs::metadata` on the dist index — missing, zero-length, or not a
    /// file. **Scope:** supervisor.
    DistMissing,
    /// **Predicate:** recent proc-macro compile flakes are above threshold —
    /// sccache is serving degraded (local-disk fallback / cache misses).
    /// **Source:** the supervisor's recent build-event flake counts.
    /// **Scope:** supervisor. (Not yet a Phase-1 supervisor const; introduced
    /// by this registry for the coord/supervisor-shared vocabulary.)
    SccacheDegraded,
    /// **Predicate:** the primary runner is not responding to its `/health`
    /// probe. **Source:** the supervisor's cached primary health
    /// (`runner_responding == false`; the existing `derived_status`).
    /// **Scope:** supervisor.
    PrimaryDown,
    /// **Predicate:** the latest CI run on `origin/main` is a failure.
    /// **Source:** coord's view of the GitHub Actions status for the repo's
    /// default branch. **Scope:** coord.
    MainRed,
    /// **Predicate:** a path-dep sibling's `main` has moved past the locally
    /// pinned stamp — a cross-crate build off this tree may inherit a break.
    /// **Source:** coord compares the sibling `origin/main` SHA against the
    /// recorded stamp. **Scope:** coord.
    SiblingDrifted,
    /// **Predicate:** a change is merged but not yet rolled to the live service
    /// (merged-not-yet-deployed). **Source:** coord's existing deploy-freshness
    /// watchers (expected revision vs live revision). **Scope:** coord.
    DeployPending,
}

impl DevState {
    /// The canonical wire string for this state. These are the ids recorded
    /// (by id only) into Action Snapshots' active state set.
    pub fn as_str(&self) -> &'static str {
        match self {
            DevState::SlotsEmpty => "SLOTS_EMPTY",
            DevState::LegacyExeFallback => "LEGACY_EXE_FALLBACK",
            DevState::LkgStale => "LKG_STALE",
            DevState::DistStale => "DIST_STALE",
            DevState::DistMissing => "DIST_MISSING",
            DevState::SccacheDegraded => "SCCACHE_DEGRADED",
            DevState::PrimaryDown => "PRIMARY_DOWN",
            DevState::MainRed => "MAIN_RED",
            DevState::SiblingDrifted => "SIBLING_DRIFTED",
            DevState::DeployPending => "DEPLOY_PENDING",
        }
    }

    /// Which component can evaluate this state's predicate.
    pub fn scope(&self) -> DevStateScope {
        match self {
            DevState::SlotsEmpty
            | DevState::LegacyExeFallback
            | DevState::LkgStale
            | DevState::DistStale
            | DevState::DistMissing
            | DevState::SccacheDegraded
            | DevState::PrimaryDown => DevStateScope::Supervisor,
            DevState::MainRed | DevState::SiblingDrifted | DevState::DeployPending => {
                DevStateScope::Coord
            }
        }
    }

    /// Every state, in declaration (stable) order.
    pub fn all() -> &'static [DevState] {
        &[
            DevState::SlotsEmpty,
            DevState::LegacyExeFallback,
            DevState::LkgStale,
            DevState::DistStale,
            DevState::DistMissing,
            DevState::SccacheDegraded,
            DevState::PrimaryDown,
            DevState::MainRed,
            DevState::SiblingDrifted,
            DevState::DeployPending,
        ]
    }
}

impl fmt::Display for DevState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// Returned when a string is not a recognized canonical dev-state id.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParseDevStateError(pub String);

impl fmt::Display for ParseDevStateError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "unrecognized dev-state id: {}", self.0)
    }
}

impl std::error::Error for ParseDevStateError {}

impl FromStr for DevState {
    type Err = ParseDevStateError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "SLOTS_EMPTY" => Ok(DevState::SlotsEmpty),
            "LEGACY_EXE_FALLBACK" => Ok(DevState::LegacyExeFallback),
            "LKG_STALE" => Ok(DevState::LkgStale),
            "DIST_STALE" => Ok(DevState::DistStale),
            "DIST_MISSING" => Ok(DevState::DistMissing),
            "SCCACHE_DEGRADED" => Ok(DevState::SccacheDegraded),
            "PRIMARY_DOWN" => Ok(DevState::PrimaryDown),
            "MAIN_RED" => Ok(DevState::MainRed),
            "SIBLING_DRIFTED" => Ok(DevState::SiblingDrifted),
            "DEPLOY_PENDING" => Ok(DevState::DeployPending),
            other => Err(ParseDevStateError(other.to_string())),
        }
    }
}

impl<'a> TryFrom<&'a str> for DevState {
    type Error = ParseDevStateError;
    fn try_from(s: &'a str) -> Result<Self, Self::Error> {
        s.parse()
    }
}

/// One dev-state predicate evaluation: the state + its truth value at action
/// time. Mirrors the supervisor's Phase-1 `DevStateEval` (which carries the id
/// as a `&'static str`); the shared form carries the typed [`DevState`] so the
/// id and scope are derivable, while still serializing the id as its canonical
/// string.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct DevStateEval {
    pub state: DevState,
    pub value: Eval,
}

impl DevStateEval {
    /// Construct an evaluation.
    pub fn new(state: DevState, value: Eval) -> Self {
        Self { state, value }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn as_str_from_str_round_trip_for_every_variant() {
        for &state in DevState::all() {
            let s = state.as_str();
            let parsed = DevState::from_str(s).expect("round-trip");
            assert_eq!(parsed, state, "round-trip mismatch for {s}");
            // TryFrom path too.
            assert_eq!(DevState::try_from(s).unwrap(), state);
            // Display agrees with as_str.
            assert_eq!(format!("{state}"), s);
        }
    }

    #[test]
    fn from_str_rejects_unknown() {
        let err = DevState::from_str("NOPE").unwrap_err();
        assert_eq!(err, ParseDevStateError("NOPE".to_string()));
    }

    #[test]
    fn serde_round_trip_for_every_variant() {
        for &state in DevState::all() {
            let json = serde_json::to_string(&state).expect("serialize");
            let back: DevState = serde_json::from_str(&json).expect("deserialize");
            assert_eq!(back, state);
        }
    }

    #[test]
    fn eval_serde_round_trip_and_wire_form() {
        let cases = [
            (Eval::True, "\"true\""),
            (Eval::False, "\"false\""),
            (Eval::Unknown, "\"unknown\""),
        ];
        for (eval, wire) in cases {
            assert_eq!(serde_json::to_string(&eval).unwrap(), wire);
            let back: Eval = serde_json::from_str(wire).unwrap();
            assert_eq!(back, eval);
        }
    }

    #[test]
    fn scope_assignment_matches_seed_table() {
        // Supervisor-scoped per §5.1.
        for s in [
            DevState::SlotsEmpty,
            DevState::LegacyExeFallback,
            DevState::LkgStale,
            DevState::DistStale,
            DevState::DistMissing,
            DevState::SccacheDegraded,
            DevState::PrimaryDown,
        ] {
            assert_eq!(s.scope(), DevStateScope::Supervisor, "{s}");
        }
        // Coord-scoped per §5.1.
        for s in [
            DevState::MainRed,
            DevState::SiblingDrifted,
            DevState::DeployPending,
        ] {
            assert_eq!(s.scope(), DevStateScope::Coord, "{s}");
        }
    }

    #[test]
    fn dev_state_eval_serde_round_trip() {
        let e = DevStateEval::new(DevState::LegacyExeFallback, Eval::True);
        let json = serde_json::to_string(&e).expect("serialize");
        let back: DevStateEval = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(back, e);
    }

    /// Compile-guard: the canonical strings of the states the supervisor
    /// hardcoded in Phase 1 (`src/dev_action/states.rs`, shipped on supervisor
    /// main as of PR #79) MUST match this shared registry byte-for-byte — else
    /// Phase 2b's migration is a rename, not a re-import. These literals are
    /// copied from the supervisor's shipped consts.
    #[test]
    fn matches_supervisor_phase1_literals() {
        // The exact six `pub const` strings shipped on supervisor main.
        const SUPERVISOR_PHASE1: &[&str] = &[
            "SLOTS_EMPTY",
            "LEGACY_EXE_FALLBACK",
            "LKG_STALE",
            "DIST_STALE",
            "DIST_MISSING",
            "PRIMARY_DOWN",
        ];
        for &lit in SUPERVISOR_PHASE1 {
            // Each must parse to a variant whose `as_str` is byte-identical.
            let state = DevState::from_str(lit)
                .unwrap_or_else(|_| panic!("shared registry missing supervisor literal {lit}"));
            assert_eq!(state.as_str(), lit, "byte-for-byte mismatch for {lit}");
        }
    }
}
