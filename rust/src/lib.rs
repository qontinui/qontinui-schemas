//! # qontinui-types
//!
//! Canonical DTO types shared across the Qontinui stack.
//!
//! Rust is the **single source of truth**. TypeScript and Python bindings are
//! generated from the JSON Schemas emitted by `schemars` on these types; see
//! `qontinui-schemas/scripts/` and the `just generate-types` task.
//!
//! ## Conventions
//!
//! - Every field that is optional on the wire uses
//!   `#[serde(default, skip_serializing_if = "Option::is_none")]` so absence and
//!   `null` are distinguishable and round-trip faithfully.
//! - Dates and times are ISO 8601 `String`s. Do **not** introduce
//!   `chrono::DateTime<Utc>` here — it complicates JSON Schema output and
//!   couples the DTO layer to a particular chrono version. The types crate is a
//!   wire-format layer, not a domain model.
//! - UUIDs are `String`s (wire-format), not `uuid::Uuid`. Same reason.
//! - Polymorphic step arrays use `Vec<serde_json::Value>`. JSON Schema emits
//!   `{ "type": "array", "items": {} }` → `unknown[]` in TS, `list[Any]` in
//!   Python. Typed step discriminated unions are a future migration (Wave 4).
//!
//! Drift between these Rust sources and the checked-in TS/Python bindings is
//! caught in CI by `.github/workflows/schema-drift.yml`, which regenerates
//! the bindings on every PR touching `rust/src/**` and fails if `git diff`
//! reports any non-timestamp change. The codegen toolchain (Tauri Linux deps,
//! datamodel-code-generator version) is pinned in that workflow — bump the
//! pins deliberately when upstream output changes.

pub mod accessibility;
pub mod ai_workflows;
pub mod app_events;
pub mod apps;
pub mod canonical_hash;
pub mod config;
pub mod constraints;
/// Canonical `DEV-*` dev-event outcome signatures (effect-side classification).
/// Companion to [`dev_states`]; same `UB-*`-style plain-enum template.
pub mod dev_signatures;
/// Canonical dev-environment state vocabulary (cause-side context) for the
/// dev-event cause-effect ledger. Plain Rust enum mirroring the
/// [`ui_bridge_diagnostics`] `UB-*` pattern — intentionally no DB, not codegen'd
/// to TS/Python. See `plans/2026-06-07-twin-dev-event-cause-effect-ledger.md`.
pub mod dev_states;
pub mod discovery;
pub mod execution;
pub mod federation;
pub mod findings;
pub mod geometry;
pub mod git_ops;
pub mod ir;
pub mod mcp_config;
pub mod memory;
pub mod orchestration_config;
pub mod process_management;
pub mod rag;
pub mod runner;
pub mod scheduler;
pub mod sdk_features;
pub mod spec_api_events;
pub mod spec_check;
pub mod state_machine;
pub mod targets;
pub mod task_run;
pub mod terminal;
pub mod text_norm;
pub mod ticket_system;
pub mod tree_events;
pub mod ui_bridge;
/// Canonical UI Bridge diagnostic code enum. **Generated** from
/// `ui-bridge/diagnostics/codes.json` by `ui-bridge/scripts/gen-diagnostics.ts`
/// — not from a `schemars` Rust source like the rest of this crate. Hand-edits
/// are overwritten; drift is gated by ui-bridge's `diagnostics:check`.
pub mod ui_bridge_diagnostics;
pub mod verification;
pub mod worker_output;
pub mod workflow;
pub mod workflow_step;

/// Rust-only cross-crate runner↔supervisor types.
///
/// See `wire/mod.rs` for the strictness convention. NOT generated to TS or
/// Python — these types only ever flow between Rust crates inside this
/// workspace.
pub mod wire;

// =========================================================================
// Forward-compat device-namespace aliases — unified-devices-registry rollout
// =========================================================================
//
// The unified-devices plan renames `Runner` → `Device` across the qontinui
// ecosystem. This 0.2.0 release ships **only the forward-compat aliases**:
// the underlying Rust types remain named `Runner*`, with `Device*` aliases
// added so new consumer code can write `qontinui_types::device::*` against
// this release.
//
// The actual rename direction-flip (move struct defs from `runner` to
// `device`, reverse the alias direction) is deferred to a coordinated
// future PR after every consumer has migrated to `device::*` imports and
// the runner-side `schema_export.rs` registers Device-named schemars
// titles. Doing the flip prematurely would drift the codegen output
// (schemars emits the Rust struct's actual name as the schema title)
// against the runner's hardcoded `add!("Runner", qrn::Runner)` calls.
//
// Removed (along with the reverse `runner::*` aliases) once the flip
// lands. The aliases are additive public API; semver-wise this is a
// minor bump (0.1.x → 0.2.0).
pub mod device {
    pub use crate::runner::*;
    pub type Device = crate::runner::Runner;
    pub type DeviceStatus = crate::runner::RunnerStatus;
    pub type DeviceCrash = crate::runner::RunnerCrash;
    pub type DeviceUiError = crate::runner::RunnerUiError;
}
