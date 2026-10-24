//! [`UnknownCode`] and the ui-bridge diagnostic registry name the same closed
//! set.
//!
//! Every `UnknownCode` has a registry name, `UB-OBS-<CODE>`
//! ([`UnknownCode::diagnostic_code`]). The registry's source of truth is
//! `ui-bridge/diagnostics/codes.json`; its Rust mirror,
//! `rust/src/ui_bridge_diagnostics.rs` in this repo, is generated from it and
//! checked by ui-bridge's `diagnostics:check`. This test pins the two sides
//! against each other IN BOTH DIRECTIONS, so adding a member on one side only
//! — an `UnknownCode` with no registry entry, or a `UB-OBS-*` entry no
//! envelope can carry — is a red test rather than a code an agent cannot
//! look up.
//!
//! It reads the mirror as text (this crate does not depend on
//! `qontinui-types`), from the same repo checkout.

use std::collections::BTreeSet;
use std::path::PathBuf;

use qontinui_vision_core::UnknownCode;

fn registry_obs_codes() -> BTreeSet<String> {
    let path =
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../rust/src/ui_bridge_diagnostics.rs");
    let text =
        std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("read {}: {e}", path.display()));
    // The as_str arm of each variant: `... => "UB-OBS-...",`.
    let codes: BTreeSet<String> = text
        .lines()
        .filter_map(|line| {
            let (_, rhs) = line.split_once("=> \"UB-OBS-")?;
            let code = rhs.split('"').next()?;
            Some(format!("UB-OBS-{code}"))
        })
        .collect();
    assert!(
        codes.len() >= 9,
        "parsed only {} UB-OBS codes from {} — the probe is broken",
        codes.len(),
        path.display()
    );
    codes
}

#[test]
fn every_unknown_code_is_a_registry_code_and_vice_versa() {
    let registry = registry_obs_codes();
    let envelope: BTreeSet<String> = UnknownCode::ALL
        .iter()
        .map(|c| c.diagnostic_code())
        .collect();
    let only_envelope: Vec<_> = envelope.difference(&registry).collect();
    let only_registry: Vec<_> = registry.difference(&envelope).collect();
    assert!(
        only_envelope.is_empty() && only_registry.is_empty(),
        "UnknownCode without a registry entry: {only_envelope:?}; \
         UB-OBS registry entry without an UnknownCode: {only_registry:?}"
    );
}
