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

pub mod accessibility;
pub mod config;
pub mod constraints;
pub mod execution;
pub mod findings;
pub mod geometry;
pub mod process_management;
pub mod rag;
pub mod scheduler;
pub mod state_machine;
pub mod targets;
pub mod task_run;
pub mod ticket_system;
pub mod tree_events;
pub mod workflow;
pub mod workflow_step;
