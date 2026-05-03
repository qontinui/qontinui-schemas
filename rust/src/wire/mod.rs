//! Rust-only cross-crate runner↔supervisor types.
//!
//! Strictness convention: `#[non_exhaustive]` + `#[serde(deny_unknown_fields)]`.
//! NOT generated to TS/Python — no consumers outside this Rust workspace.
//!
//! This namespace exists separately from the top-level DTO modules because
//! the cross-crate strictness convention (`deny_unknown_fields`,
//! `non_exhaustive`) is a deliberate departure from the forward-compatible
//! DTO convention documented at the top of `lib.rs`. Keeping them split lets
//! the JSON-Schema-generated TS/Python bindings stay forward-compatible while
//! the Rust runner↔supervisor surface fails fast on shape drift.

pub mod placement;
pub mod runner_kind;

pub use runner_kind::RunnerKind;
