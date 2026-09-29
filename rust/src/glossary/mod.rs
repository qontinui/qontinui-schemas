//! The product glossary, compiled in.
//!
//! Plan `2026-09-20-the-published-product-works-without-knowing-a-development-environment-exists`,
//! Phase C1 (fork 1 resolved: compiled in from this crate, not a
//! tenant-editable document).
//!
//! ## One source
//!
//! The source is the repo-root `glossary/terms.toml`. [`GlossaryTerm`] and
//! [`GLOSSARY`] live in `generated.rs`, which is **generated** from that file
//! and checked against it by the repo test `tests/glossary_source.rs` — edit
//! the TOML, never the generated Rust. That test also enforces the file's
//! invariants (stable snake_case ids, `short` ≤ 160 chars, `long` ≤ 1200,
//! no dangling `see_also`, no fleet noun in any text) and the version guard:
//! a content change without a raised `version` fails.
//!
//! ## Why compiled in
//!
//! - **Typed ids.** A [`crate::refusal::Refusal`] cites terms as
//!   [`GlossaryTerm`], so citing a term that does not exist is a compile error
//!   (or a parse error on the wire), not a dangling link.
//! - **Available offline.** Every surface renders the glossary from its own
//!   build, with no request, so a definition is shown even when the
//!   coordination service is unreachable.
//! - **Attributable drift.** [`GLOSSARY_VERSION`] and
//!   [`GLOSSARY_CONTENT_SHA256`] travel with the build, so two surfaces showing
//!   different text can be told apart by version.

use std::fmt;
use std::str::FromStr;

use serde::Serialize;

#[rustfmt::skip]
mod generated;

pub use generated::{GlossaryTerm, GLOSSARY, GLOSSARY_CONTENT_SHA256, GLOSSARY_VERSION};

// A glossary always has a version; checked at compile time.
const _: () = assert!(GLOSSARY_VERSION >= 1);

/// One glossary definition. Rows of [`GLOSSARY`], in [`GlossaryTerm::ALL`]
/// order.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct GlossaryEntry {
    pub id: GlossaryTerm,
    /// Display name.
    pub term: &'static str,
    /// Plain text, at most 160 characters (tooltip).
    pub short: &'static str,
    /// Markdown, at most 1200 characters.
    pub long: &'static str,
    /// Related terms.
    pub see_also: &'static [GlossaryTerm],
    /// The glossary version that introduced this term.
    pub since: u32,
}

impl GlossaryTerm {
    /// This term's definition.
    pub fn entry(self) -> &'static GlossaryEntry {
        // GLOSSARY is generated in declaration order; `glossary_is_in_enum_order`
        // pins that, so the discriminant is the row index.
        &GLOSSARY[self as usize]
    }

    /// The term with this stable id, or `None` when this version's glossary
    /// does not define it.
    pub fn from_id(id: &str) -> Option<Self> {
        Self::ALL.iter().copied().find(|t| t.as_str() == id)
    }
}

impl fmt::Display for GlossaryTerm {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A glossary id this version does not define.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[error("`{id}` is not a term in glossary version {known_version}")]
pub struct UnknownGlossaryTerm {
    pub id: String,
    pub known_version: u32,
}

impl FromStr for GlossaryTerm {
    type Err = UnknownGlossaryTerm;

    fn from_str(id: &str) -> Result<Self, Self::Err> {
        Self::from_id(id).ok_or_else(|| UnknownGlossaryTerm {
            id: id.to_string(),
            known_version: GLOSSARY_VERSION,
        })
    }
}

/// The definition for `id`, or `None` when this version does not define it.
pub fn lookup(id: &str) -> Option<&'static GlossaryEntry> {
    GlossaryTerm::from_id(id).map(GlossaryTerm::entry)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeSet;

    #[test]
    fn glossary_is_in_enum_order() {
        assert_eq!(GLOSSARY.len(), GlossaryTerm::ALL.len());
        for (i, (row, id)) in GLOSSARY.iter().zip(GlossaryTerm::ALL).enumerate() {
            assert_eq!(row.id, *id, "row {i}");
            assert_eq!(*id as usize, i, "discriminant of {id}");
            assert_eq!(id.entry(), row);
        }
    }

    /// The same invariants the source test enforces on the TOML, re-asserted
    /// on the compiled table so the published crate carries them too.
    #[test]
    fn compiled_table_invariants() {
        assert_eq!(GLOSSARY_CONTENT_SHA256.len(), 64);
        assert!(GLOSSARY.len() >= 20, "roster floor");
        let mut seen = BTreeSet::new();
        for e in GLOSSARY {
            let id = e.id.as_str();
            assert!(seen.insert(id), "duplicate id {id}");
            assert!(
                !id.is_empty()
                    && id.starts_with(|c: char| c.is_ascii_lowercase())
                    && id
                        .chars()
                        .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_')
                    && !id.contains("__")
                    && !id.ends_with('_'),
                "`{id}` is not snake_case"
            );
            assert!(!e.term.trim().is_empty(), "{id}: empty term");
            let short = e.short.chars().count();
            assert!((1..=160).contains(&short), "{id}: short is {short} chars");
            let long = e.long.chars().count();
            assert!((1..=1200).contains(&long), "{id}: long is {long} chars");
            assert!(!e.see_also.contains(&e.id), "{id}: cites itself");
            let uniq: BTreeSet<_> = e.see_also.iter().collect();
            assert_eq!(uniq.len(), e.see_also.len(), "{id}: duplicate see_also");
            assert!((1..=GLOSSARY_VERSION).contains(&e.since), "{id}: since");
        }
    }

    #[test]
    fn lookup_and_parse() {
        assert_eq!(lookup("gate").map(|e| e.id), Some(GlossaryTerm::Gate));
        assert_eq!(lookup("no_such_term"), None);
        assert_eq!(
            "work_unit".parse::<GlossaryTerm>(),
            Ok(GlossaryTerm::WorkUnit)
        );
        let err = "no_such_term".parse::<GlossaryTerm>().unwrap_err();
        assert_eq!(err.known_version, GLOSSARY_VERSION);
        assert_eq!(GlossaryTerm::Unknown.to_string(), "unknown");
    }

    #[test]
    fn wire_value_is_the_stable_id() {
        for t in GlossaryTerm::ALL {
            let v = serde_json::to_value(t).unwrap();
            assert_eq!(v, t.as_str());
            assert_eq!(serde_json::from_value::<GlossaryTerm>(v).unwrap(), *t);
        }
        assert!(serde_json::from_value::<GlossaryTerm>("no_such_term".into()).is_err());
    }
}
