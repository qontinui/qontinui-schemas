//! Build census — the PURE core: who owns a build, what a class of builds
//! costs, and which class (if any) a capacity problem is attributable to.
//!
//! Plan `2026-10-21-every-build-on-a-box-has-an-owner-a-cost-and-a-bound`,
//! Phase 1 (decisions D1, D2, D5, D6). Every function here is a pure function
//! of its arguments: no I/O, no clock, no environment. The runner's census
//! (Phase 4) reads the facts and calls these to classify; coord (Phase 3)
//! calls the same functions to serve the verdict.
//!
//! | Function | Decides |
//! |---|---|
//! | [`owner::classify_owner`] | the [`OwnerClass`] of one build from the facts a census can read |
//! | [`rollup::class_rollup`] | per owner class: count, anon RSS, placed / unplaced |
//! | [`cause::capacity_cause`] | [`Cause`]: which class a stalled host is attributable to, on positive evidence only |
//!
//! ## UNKNOWN is never a default
//!
//! An unreadable ancestry, an undated ancestor, or no positive owner evidence
//! is [`OwnerClass::Unknown`], never `InteractiveHuman`. An unattributable
//! build is counted at its measured RSS in every total; an RSS that could not
//! be read is carried as an explicit [`types::Reading::Unknown`] count, never 0.
//! A class whose slice pressure cannot be read makes the cause `Unknown`,
//! never `Other`.
//!
//! ## No host literals
//!
//! No hostname, uid or path appears in a rule: the
//! `no_host_literals_in_rules` test reads this module's own sources and fails
//! on a string literal that is not a serde attribute value.

pub mod cause;
pub mod owner;
pub mod rollup;
pub mod types;

#[cfg(test)]
mod golden_tests;

pub use cause::*;
pub use owner::*;
pub use rollup::*;
pub use types::*;
