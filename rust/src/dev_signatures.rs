//! Canonical `DEV-*` dev-event outcome signatures (the effect-side
//! classification).
//!
//! Where [`crate::dev_states`] names the *cause-side* context (the active
//! dev-state set at action time), this module names the *effect-side*
//! classification: the post-action signatures an attribution watcher folds into
//! an Action Snapshot's outcome
//! (`plans/2026-06-07-twin-dev-event-cause-effect-ledger.md` §5.1 / §5.2).
//!
//! It mirrors the shipped [`crate::ui_bridge_diagnostics`] `UB-*` registry
//! exactly: canonical string codes, `as_str` / `FromStr` round-trip, serde, and
//! — like that registry — **intentionally no DB** (resolved Q5). The five
//! signatures shipped on supervisor `main` (PR #79, `src/dev_action/attribution.rs`)
//! are pinned byte-for-byte by [`tests::matches_supervisor_phase1_literals`];
//! `DEV-COMPILE-PROC-MACRO-FLAKE` is introduced here for the shared vocabulary.
//!
//! Each signature carries its **default D3 outcome category** — the verdict
//! class it implies absent any contrary signal — via [`DevSignature::default_category`],
//! and an optional [`DevSignature::remediation_ref`] pointing at the memory /
//! feedback doc that fixes it.

use std::fmt;
use std::str::FromStr;

use serde::{Deserialize, Serialize};

/// The D3 outcome category for an action — the five `outcome_category` values
/// the coord calibration flywheel already keys on
/// (`OutcomeCounts{confirmed,surprise,failure,contradiction,partial}`). The
/// `snake_case` serde rename guarantees this serializes to exactly those five
/// strings, which is what keeps the Phase-4 calibration key compatible without
/// a translation layer (guarded by a serde round-trip test below).
///
/// This is the shared mirror of the supervisor's Phase-1 `D3Category`
/// (`src/dev_action/record.rs`); the two MUST serialize identically (the
/// supervisor's own test pins the same five strings).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum D3Category {
    /// The action did exactly what the ACK claimed (clean window).
    Confirmed,
    /// An unexpected-but-not-failing signal appeared.
    Surprise,
    /// The action's own machinery failed (panic on startup, port bind fail,
    /// compile flake).
    Failure,
    /// The ACK claimed success but observation refutes it — the motivating
    /// 2026-06-07 case (a "restarted successfully" ACK over an asset-missing
    /// white screen).
    Contradiction,
    /// A partial / mixed outcome.
    Partial,
}

/// A canonical `DEV-*` dev-event outcome signature.
///
/// Each variant's doc comment states its **matcher semantics** (what observable
/// pattern fires it) and its **default D3 category** (the verdict class it
/// implies). The asset-missing / webview-refused / ui-error-boundary signatures
/// default to [`D3Category::Contradiction`] (the ACK claimed success but the
/// surface refutes it); panic / port-bind / compile-flake default to
/// [`D3Category::Failure`] (the action's own machinery failed).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum DevSignature {
    /// **Matcher:** the runner's early log contains `tauri::manager` +
    /// `asset not found` (the exact 2026-06-07 line:
    /// `ERROR tauri::manager: asset not found: index.html`) — Tauri loaded a
    /// binary with no embedded `dist/`. **Default category:** Contradiction.
    DevTauriAssetMissing,
    /// **Matcher:** the early log contains `ERR_CONNECTION_REFUSED` — the
    /// webview could not reach its frontend (a dev-mode binary with no Vite
    /// server). **Default category:** Contradiction.
    DevWebviewConnRefused,
    /// **Matcher:** the early log shows a bind failure — `AddrInUse`,
    /// `address already in use`, or `failed to bind`. **Default category:**
    /// Failure (the action's own startup machinery failed).
    DevPortBindFail,
    /// **Matcher:** a fresh startup panic was recorded within the attribution
    /// window (parsed from the runner-written `runner-panic.log`). **Default
    /// category:** Failure.
    DevPanicStartup,
    /// **Matcher:** the runner's cached health reports a `ui_error` or its
    /// `derived_status` is `Errored` — the React error boundary tripped.
    /// **Default category:** Contradiction.
    DevUiErrorBoundary,
    /// **Matcher:** a proc-macro compile step failed flakily (an sccache /
    /// proc-macro miscompile that disappears on retry). **Default category:**
    /// Failure. (Introduced by this shared registry; not a Phase-1 supervisor
    /// const.)
    DevCompileProcMacroFlake,
}

impl DevSignature {
    /// The canonical `DEV-` wire string for this signature.
    pub fn as_str(&self) -> &'static str {
        match self {
            DevSignature::DevTauriAssetMissing => "DEV-TAURI-ASSET-MISSING",
            DevSignature::DevWebviewConnRefused => "DEV-WEBVIEW-CONN-REFUSED",
            DevSignature::DevPortBindFail => "DEV-PORT-BIND-FAIL",
            DevSignature::DevPanicStartup => "DEV-PANIC-STARTUP",
            DevSignature::DevUiErrorBoundary => "DEV-UI-ERROR-BOUNDARY",
            DevSignature::DevCompileProcMacroFlake => "DEV-COMPILE-PROC-MACRO-FLAKE",
        }
    }

    /// The D3 outcome category this signature defaults to, absent any contrary
    /// signal. Contradiction for the surface-refutes-success signatures
    /// (asset-missing, webview-refused, ui-error-boundary); Failure for the
    /// own-machinery-failed signatures (panic, port-bind, compile-flake).
    pub fn default_category(&self) -> D3Category {
        match self {
            DevSignature::DevTauriAssetMissing
            | DevSignature::DevWebviewConnRefused
            | DevSignature::DevUiErrorBoundary => D3Category::Contradiction,
            DevSignature::DevPortBindFail
            | DevSignature::DevPanicStartup
            | DevSignature::DevCompileProcMacroFlake => D3Category::Failure,
        }
    }

    /// The memory / feedback doc that documents how to fix this signature, if
    /// one exists. `None` for signatures without a recorded remediation yet.
    pub fn remediation_ref(&self) -> Option<&'static str> {
        match self {
            DevSignature::DevTauriAssetMissing => {
                Some("reference_runner_asset_not_found_legacy_exe_fallback")
            }
            DevSignature::DevWebviewConnRefused => Some("feedback_runner_manual_build"),
            DevSignature::DevPortBindFail
            | DevSignature::DevPanicStartup
            | DevSignature::DevUiErrorBoundary
            | DevSignature::DevCompileProcMacroFlake => None,
        }
    }

    /// Every signature, in declaration (stable) order.
    pub fn all() -> &'static [DevSignature] {
        &[
            DevSignature::DevTauriAssetMissing,
            DevSignature::DevWebviewConnRefused,
            DevSignature::DevPortBindFail,
            DevSignature::DevPanicStartup,
            DevSignature::DevUiErrorBoundary,
            DevSignature::DevCompileProcMacroFlake,
        ]
    }
}

impl fmt::Display for DevSignature {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// Returned when a string is not a recognized canonical `DEV-` signature.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParseDevSignatureError(pub String);

impl fmt::Display for ParseDevSignatureError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "unrecognized dev-event signature: {}", self.0)
    }
}

impl std::error::Error for ParseDevSignatureError {}

impl FromStr for DevSignature {
    type Err = ParseDevSignatureError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "DEV-TAURI-ASSET-MISSING" => Ok(DevSignature::DevTauriAssetMissing),
            "DEV-WEBVIEW-CONN-REFUSED" => Ok(DevSignature::DevWebviewConnRefused),
            "DEV-PORT-BIND-FAIL" => Ok(DevSignature::DevPortBindFail),
            "DEV-PANIC-STARTUP" => Ok(DevSignature::DevPanicStartup),
            "DEV-UI-ERROR-BOUNDARY" => Ok(DevSignature::DevUiErrorBoundary),
            "DEV-COMPILE-PROC-MACRO-FLAKE" => Ok(DevSignature::DevCompileProcMacroFlake),
            other => Err(ParseDevSignatureError(other.to_string())),
        }
    }
}

impl<'a> TryFrom<&'a str> for DevSignature {
    type Error = ParseDevSignatureError;
    fn try_from(s: &'a str) -> Result<Self, Self::Error> {
        s.parse()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn as_str_from_str_round_trip_for_every_variant() {
        for &sig in DevSignature::all() {
            let s = sig.as_str();
            let parsed = DevSignature::from_str(s).expect("round-trip");
            assert_eq!(parsed, sig, "round-trip mismatch for {s}");
            assert_eq!(DevSignature::try_from(s).unwrap(), sig);
            assert_eq!(format!("{sig}"), s);
        }
    }

    #[test]
    fn from_str_rejects_unknown() {
        let err = DevSignature::from_str("DEV-NOPE").unwrap_err();
        assert_eq!(err, ParseDevSignatureError("DEV-NOPE".to_string()));
    }

    #[test]
    fn serde_round_trip_for_every_variant() {
        for &sig in DevSignature::all() {
            let json = serde_json::to_string(&sig).expect("serialize");
            let back: DevSignature = serde_json::from_str(&json).expect("deserialize");
            assert_eq!(back, sig);
        }
    }

    #[test]
    fn default_category_mapping() {
        // Contradiction: surface refutes the success ACK.
        assert_eq!(
            DevSignature::DevTauriAssetMissing.default_category(),
            D3Category::Contradiction
        );
        assert_eq!(
            DevSignature::DevWebviewConnRefused.default_category(),
            D3Category::Contradiction
        );
        assert_eq!(
            DevSignature::DevUiErrorBoundary.default_category(),
            D3Category::Contradiction
        );
        // Failure: the action's own machinery failed.
        assert_eq!(
            DevSignature::DevPortBindFail.default_category(),
            D3Category::Failure
        );
        assert_eq!(
            DevSignature::DevPanicStartup.default_category(),
            D3Category::Failure
        );
        assert_eq!(
            DevSignature::DevCompileProcMacroFlake.default_category(),
            D3Category::Failure
        );
    }

    #[test]
    fn remediation_ref_mapping() {
        assert_eq!(
            DevSignature::DevTauriAssetMissing.remediation_ref(),
            Some("reference_runner_asset_not_found_legacy_exe_fallback")
        );
        assert_eq!(
            DevSignature::DevWebviewConnRefused.remediation_ref(),
            Some("feedback_runner_manual_build")
        );
        assert_eq!(DevSignature::DevPortBindFail.remediation_ref(), None);
        assert_eq!(DevSignature::DevPanicStartup.remediation_ref(), None);
        assert_eq!(DevSignature::DevUiErrorBoundary.remediation_ref(), None);
        assert_eq!(
            DevSignature::DevCompileProcMacroFlake.remediation_ref(),
            None
        );
    }

    #[test]
    fn d3_category_serializes_to_five_outcome_category_strings() {
        let cases = [
            (D3Category::Confirmed, "\"confirmed\""),
            (D3Category::Surprise, "\"surprise\""),
            (D3Category::Failure, "\"failure\""),
            (D3Category::Contradiction, "\"contradiction\""),
            (D3Category::Partial, "\"partial\""),
        ];
        for (cat, wire) in cases {
            assert_eq!(serde_json::to_string(&cat).unwrap(), wire, "{cat:?}");
            let back: D3Category = serde_json::from_str(wire).unwrap();
            assert_eq!(back, cat);
        }
    }

    /// Compile-guard: the five `DEV-*` signature strings hardcoded on
    /// supervisor main (`src/dev_action/attribution.rs`, PR #79) MUST match
    /// this shared registry byte-for-byte. The sixth
    /// (`DEV-COMPILE-PROC-MACRO-FLAKE`) is introduced here and is asserted
    /// present so Phase 2b's migration is a re-import, not a rename.
    #[test]
    fn matches_supervisor_phase1_literals() {
        // The exact five `pub const` strings shipped on supervisor main.
        const SUPERVISOR_PHASE1: &[&str] = &[
            "DEV-TAURI-ASSET-MISSING",
            "DEV-WEBVIEW-CONN-REFUSED",
            "DEV-PORT-BIND-FAIL",
            "DEV-PANIC-STARTUP",
            "DEV-UI-ERROR-BOUNDARY",
        ];
        for &lit in SUPERVISOR_PHASE1 {
            let sig = DevSignature::from_str(lit)
                .unwrap_or_else(|_| panic!("shared registry missing supervisor literal {lit}"));
            assert_eq!(sig.as_str(), lit, "byte-for-byte mismatch for {lit}");
        }
        // The new shared-only signature is present.
        assert!(DevSignature::from_str("DEV-COMPILE-PROC-MACRO-FLAKE").is_ok());
    }
}
