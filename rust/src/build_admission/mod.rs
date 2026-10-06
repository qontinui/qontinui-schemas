//! Build admission — the PURE decision core.
//!
//! Plan `2026-10-06-builds-are-admitted-per-invocation-against-measured-host-memory`,
//! Phase 1. Every function here is a pure function of its arguments: no I/O, no
//! clock, no environment. The runner's host broker (Phase 2) and governor
//! (Phase 4) call it to MAKE decisions; coord (Phase 6) calls the same
//! functions only to EXPLAIN a host's published verdicts.
//!
//! ## What is decided where
//!
//! | Function | Decides |
//! |---|---|
//! | [`estimate::estimate`] | a ticket's memory reservation from its own key's measured history (p90 of the last 20), else a labelled seed, else UNKNOWN |
//! | [`budget::budget`] | the bytes builds may reserve on this host (D3) |
//! | [`admit::admit`] | whether ONE ticket may start now, and with how many jobs |
//! | [`order::schedule`] | which queued ticket starts next: class, age, promotion, EASY backfill, the paused-holder skip |
//! | [`overload::overload_step`] | the governor ladder as a state machine: pause / resume / hold |
//! | [`degraded::degraded_slots`] | the slot count of the wrapper's broker-less arm (D4) |
//! | [`jobs::jobs`] | `CARGO_BUILD_JOBS` for a memory share, the runner's CI sizing arithmetic |
//!
//! ## UNKNOWN is never free capacity
//!
//! Every host input is a [`Fact`]: `measured`, `live_only`, `seed`,
//! `not_supported` or `unknown`. `not_supported` (the platform cannot provide
//! it — memory PSI on Windows) DROPS its conjunct; `unknown` (it should be
//! readable and is not) WITHHOLDS every admission except the progress floor —
//! the head of the queue on an otherwise idle host — so an unknown bounds the
//! box at one build rather than stalling it or admitting the herd.
//!
//! ## No host literals
//!
//! No hostname, uid or path appears in a rule or a default; the
//! `no_host_literals_in_rules` test reads this module's own sources and fails
//! on one. Every number is a measurement handed in by the caller or a named
//! [`Policy`] parameter.

pub mod admit;
pub mod budget;
pub mod degraded;
pub mod estimate;
pub mod jobs;
pub mod order;
pub mod overload;
pub mod types;

#[cfg(test)]
mod golden_tests;

pub use types::*;
