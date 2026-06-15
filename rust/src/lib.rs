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
//!
//! ## The functional-spec-contract v0 artifact set (frozen 2026-06-13)
//!
//! Three modules form the keystone contract for the website → mobile
//! regeneration program (`2026-06-13-functional-spec-contract.md`):
//!
//! - [`functional_spec`] — **Artifact 1**, the backend-agnostic [`functional_spec::FunctionalSpec`]
//!   that comprehension writes and generation reads. Its `ui_states` / `navigation`
//!   reuse [`ir`]`::{IrState, IrTransition}` (a literal type-level superset of the IR).
//! - [`completeness_verdict`] — **Artifact 2**, the Completeness Rubric +
//!   [`completeness_verdict::CompletenessVerdict`]. Borrows the `DriftVerdict` field
//!   vocabulary and embeds [`spec_check::SpecCheckResult`] for the UI dimension.
//! - [`priorities_profile`] — **Artifact 3**, the [`priorities_profile::Profile`] that
//!   parameterizes the generators.
//!
//! **v0 stability contract:** the `specVersion` / `profileVersion` fields are `"0"`.
//! Until they bump to `"1"`, changes are **additive-only** — new optional fields and
//! new enum variants only, never a rename, removal, or type change of an existing
//! field. (`deny_unknown_fields` is deliberately omitted on these three so a reader
//! tolerates a forward document.) Downstream plans #1 (app-gen), #2 (backend-gen),
//! and #3 (comprehension) build against this frozen interface in parallel.

pub mod accessibility;
pub mod ai_workflows;
pub mod app_events;
pub mod apps;
pub mod canonical_hash;
/// Completeness **evaluation** — the deterministic rubric walk that turns a
/// [`functional_spec::FunctionalSpec`] + coverage evidence into a
/// [`completeness_verdict::CompletenessVerdict`] (node enumeration + gap diff +
/// per-section breakdown). The verify-phase rubric the website→mobile slice proves.
pub mod completeness_eval;
/// Completeness Rubric + Verdict — Artifact 2 of the functional-spec-contract
/// keystone. Scores a generated app against the [`functional_spec`] in spec
/// units; embeds [`spec_check`] for the UI dimension and borrows the
/// `DriftVerdict` field vocabulary. See
/// `2026-06-13-functional-spec-contract.md`.
pub mod completeness_verdict;
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
/// Functional Spec — Artifact 1 of the functional-spec-contract keystone: the
/// backend-agnostic contract between comprehension and generation. Reuses
/// [`ir`]`::{IrState, IrTransition}` for the `ui_states`/`navigation` section.
/// See `2026-06-13-functional-spec-contract.md`.
pub mod functional_spec;
pub mod geometry;
pub mod git_ops;
pub mod ir;
pub mod mcp_config;
pub mod memory;
pub mod orchestration_config;
/// Priorities Profile — Artifact 3 of the functional-spec-contract keystone:
/// the declarative, gate-checkable generator parameterization. See
/// `2026-06-13-functional-spec-contract.md`.
pub mod priorities_profile;
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
