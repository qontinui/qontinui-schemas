//! The glossary source (`glossary/terms.toml`) and its generated Rust
//! (`rust/src/glossary/generated.rs`) — plan
//! `2026-09-20-the-published-product-works-without-knowing-a-development-environment-exists`,
//! Phase C1.
//!
//! This test IS the generator: it renders the Rust table from the TOML and
//! fails when the checked-in file differs. To regenerate after editing the
//! TOML (and raising its `version`):
//!
//! ```text
//! QONTINUI_GLOSSARY_REGENERATE=1 cargo test -p qontinui-types --test glossary_source
//! ```
//!
//! It also enforces the source's invariants — including that no text names a
//! fleet noun from `fleet-nouns.toml`, because these definitions ship to every
//! user — and the version guard: a content change whose `version` is not above
//! the generated file's `GLOSSARY_VERSION` fails, in the regenerate mode too.
//!
//! Both files live outside this crate's package, so this is a repo test
//! (excluded from the published crate in `rust/Cargo.toml`).

mod common;

use std::collections::{BTreeMap, BTreeSet};
use std::path::PathBuf;

use serde::Deserialize;
use sha2::{Digest, Sha256};

use common::fleet_nouns::FleetNouns;

const SHORT_MAX: usize = 160;
const LONG_MAX: usize = 1200;
const REGENERATE_ENV: &str = "QONTINUI_GLOSSARY_REGENERATE";

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct Source {
    version: u32,
    term: Vec<Term>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct Term {
    id: String,
    term: String,
    short: String,
    long: String,
    see_also: Vec<String>,
    since: u32,
}

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("..")
}

fn source_path() -> PathBuf {
    repo_root().join("glossary").join("terms.toml")
}

fn generated_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("src")
        .join("glossary")
        .join("generated.rs")
}

/// The source text with CRLF normalised to LF, so the digest is the same on
/// every checkout.
fn source_text() -> String {
    let path = source_path();
    std::fs::read_to_string(&path)
        .unwrap_or_else(|e| panic!("glossary source unreadable at {} ({e})", path.display()))
        .replace("\r\n", "\n")
}

fn parse(text: &str) -> Source {
    toml::from_str(text)
        .unwrap_or_else(|e| panic!("{} does not parse: {e}", source_path().display()))
}

fn sha256_hex(text: &str) -> String {
    hex::encode(Sha256::digest(text.as_bytes()))
}

fn is_snake_case(id: &str) -> bool {
    !id.is_empty()
        && id.starts_with(|c: char| c.is_ascii_lowercase())
        && id
            .chars()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_')
        && !id.contains("__")
        && !id.ends_with('_')
}

fn pascal_case(id: &str) -> String {
    id.split('_')
        .map(|w| {
            let mut c = w.chars();
            c.next()
                .map(|f| f.to_ascii_uppercase().to_string() + c.as_str())
                .unwrap_or_default()
        })
        .collect()
}

/// Every invariant of the source, as a list of violations (empty = valid).
fn violations(src: &Source) -> Vec<String> {
    let mut out = Vec::new();
    if src.version < 1 {
        out.push("version must be >= 1".to_string());
    }
    if src.term.len() < 20 {
        out.push(format!(
            "only {} terms; the roster floor is 20",
            src.term.len()
        ));
    }
    let ids: BTreeSet<&str> = src.term.iter().map(|t| t.id.as_str()).collect();
    let mut seen = BTreeSet::new();
    let mut variants = BTreeMap::new();
    for t in &src.term {
        let id = t.id.as_str();
        if !seen.insert(id) {
            out.push(format!("duplicate id `{id}`"));
        }
        if !is_snake_case(id) {
            out.push(format!("id `{id}` is not snake_case"));
        }
        if let Some(prev) = variants.insert(pascal_case(id), id) {
            out.push(format!(
                "ids `{prev}` and `{id}` map to the same Rust variant"
            ));
        }
        if t.term.trim().is_empty() {
            out.push(format!("`{id}`: empty term"));
        }
        let short = t.short.trim().chars().count();
        if short == 0 || short > SHORT_MAX {
            out.push(format!("`{id}`: short is {short} chars (1..={SHORT_MAX})"));
        }
        if t.short.contains('\n') {
            out.push(format!("`{id}`: short must be one line"));
        }
        let long = t.long.trim().chars().count();
        if long == 0 || long > LONG_MAX {
            out.push(format!("`{id}`: long is {long} chars (1..={LONG_MAX})"));
        }
        let mut cited = BTreeSet::new();
        for s in &t.see_also {
            if !ids.contains(s.as_str()) {
                out.push(format!("`{id}`: see_also `{s}` is dangling (no such term)"));
            }
            if s == id {
                out.push(format!("`{id}`: see_also cites itself"));
            }
            if !cited.insert(s.as_str()) {
                out.push(format!("`{id}`: see_also lists `{s}` twice"));
            }
        }
        if t.since < 1 || t.since > src.version {
            out.push(format!(
                "`{id}`: since {} is outside 1..={}",
                t.since, src.version
            ));
        }
    }
    out
}

/// Render `generated.rs`. Deterministic: same source, same bytes.
fn render(src: &Source, sha: &str) -> String {
    use std::fmt::Write;
    let lit = |s: &str| format!("{s:?}");
    let mut o = String::new();
    o.push_str(
        "// @generated from glossary/terms.toml by rust/tests/glossary_source.rs — do not edit.\n",
    );
    o.push_str("// Regenerate: QONTINUI_GLOSSARY_REGENERATE=1 cargo test -p qontinui-types --test glossary_source\n\n");
    o.push_str("use schemars::JsonSchema;\nuse serde::{Deserialize, Serialize};\n\nuse super::GlossaryEntry;\n\n");
    o.push_str("/// The glossary's `version` (glossary/terms.toml).\n");
    writeln!(o, "pub const GLOSSARY_VERSION: u32 = {};", src.version).unwrap();
    o.push_str(
        "/// SHA-256 of the LF-normalised glossary/terms.toml this table was generated from.\n",
    );
    writeln!(
        o,
        "pub const GLOSSARY_CONTENT_SHA256: &str = {};\n",
        lit(sha)
    )
    .unwrap();
    o.push_str("/// A term the product glossary defines, by its stable snake_case id.\n");
    o.push_str("#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize, JsonSchema)]\n");
    o.push_str("pub enum GlossaryTerm {\n");
    for t in &src.term {
        writeln!(o, "    #[doc = {}]", lit(t.short.trim())).unwrap();
        writeln!(o, "    #[serde(rename = {})]", lit(&t.id)).unwrap();
        writeln!(o, "    {},", pascal_case(&t.id)).unwrap();
    }
    o.push_str("}\n\nimpl GlossaryTerm {\n");
    o.push_str("    /// Every term, in glossary order.\n");
    o.push_str("    pub const ALL: &'static [GlossaryTerm] = &[\n");
    for t in &src.term {
        writeln!(o, "        GlossaryTerm::{},", pascal_case(&t.id)).unwrap();
    }
    o.push_str("    ];\n\n");
    o.push_str("    /// The stable id (the wire value).\n");
    o.push_str("    pub const fn as_str(self) -> &'static str {\n        match self {\n");
    for t in &src.term {
        writeln!(
            o,
            "            GlossaryTerm::{} => {},",
            pascal_case(&t.id),
            lit(&t.id)
        )
        .unwrap();
    }
    o.push_str("        }\n    }\n}\n\n");
    o.push_str("/// Every definition, in [`GlossaryTerm::ALL`] order.\n");
    o.push_str("pub static GLOSSARY: &[GlossaryEntry] = &[\n");
    for t in &src.term {
        o.push_str("    GlossaryEntry {\n");
        writeln!(o, "        id: GlossaryTerm::{},", pascal_case(&t.id)).unwrap();
        writeln!(o, "        term: {},", lit(t.term.trim())).unwrap();
        writeln!(o, "        short: {},", lit(t.short.trim())).unwrap();
        writeln!(o, "        long: {},", lit(t.long.trim())).unwrap();
        let see: Vec<String> = t
            .see_also
            .iter()
            .map(|s| format!("GlossaryTerm::{}", pascal_case(s)))
            .collect();
        writeln!(o, "        see_also: &[{}],", see.join(", ")).unwrap();
        writeln!(o, "        since: {},", t.since).unwrap();
        o.push_str("    },\n");
    }
    o.push_str("];\n");
    o
}

/// The version guard: content that changed must carry a raised version.
/// `committed` is what the compiled crate carries.
fn version_guard(src_version: u32, sha: &str, committed: (u32, &str)) -> Result<(), String> {
    let (committed_version, committed_sha) = committed;
    if sha == committed_sha {
        return Ok(());
    }
    if src_version <= committed_version {
        return Err(format!(
            "glossary/terms.toml changed (sha256 {sha}, generated from {committed_sha}) but its \
             version is {src_version}, not above the generated GLOSSARY_VERSION {committed_version}. \
             Raise `version` to {} — every content change is a new glossary version.",
            committed_version + 1
        ));
    }
    Ok(())
}

#[test]
fn source_is_valid() {
    let src = parse(&source_text());
    let v = violations(&src);
    assert!(v.is_empty(), "glossary/terms.toml:\n  {}", v.join("\n  "));
}

#[test]
fn no_text_names_a_fleet_noun() {
    let src = parse(&source_text());
    let nouns = FleetNouns::load();
    let mut bad = Vec::new();
    for t in &src.term {
        let fields = [
            ("id", t.id.as_str()),
            ("term", t.term.as_str()),
            ("short", t.short.as_str()),
            ("long", t.long.as_str()),
        ];
        for (field, text) in fields {
            let hits = nouns.hits(text);
            if !hits.is_empty() {
                bad.push(format!("`{}`.{field}: {hits:?}", t.id));
            }
        }
    }
    assert!(
        bad.is_empty(),
        "glossary text names fleet nouns (these definitions ship to every user):\n  {}",
        bad.join("\n  ")
    );
}

#[test]
fn generated_rust_matches_the_source_and_the_version_was_raised() {
    let text = source_text();
    let src = parse(&text);
    let sha = sha256_hex(&text);
    let committed = (
        qontinui_types::glossary::GLOSSARY_VERSION,
        qontinui_types::glossary::GLOSSARY_CONTENT_SHA256,
    );
    if let Err(e) = version_guard(src.version, &sha, committed) {
        panic!("{e}");
    }
    let want = render(&src, &sha);
    let path = generated_path();
    let have = std::fs::read_to_string(&path)
        .unwrap_or_default()
        .replace("\r\n", "\n");
    if have == want {
        return;
    }
    if std::env::var_os(REGENERATE_ENV).is_some() {
        std::fs::write(&path, &want)
            .unwrap_or_else(|e| panic!("cannot write {}: {e}", path.display()));
        eprintln!("regenerated {}", path.display());
        return;
    }
    panic!(
        "{} is stale against glossary/terms.toml. Regenerate:\n  \
         {REGENERATE_ENV}=1 cargo test -p qontinui-types --test glossary_source",
        path.display()
    );
}

/// The compiled table is the source, field for field — so the checked-in file
/// being byte-equal is not the only thing standing between the two.
#[test]
fn compiled_table_equals_the_source() {
    let src = parse(&source_text());
    let table = qontinui_types::glossary::GLOSSARY;
    assert_eq!(table.len(), src.term.len());
    for (row, t) in table.iter().zip(&src.term) {
        assert_eq!(row.id.as_str(), t.id);
        assert_eq!(row.term, t.term.trim());
        assert_eq!(row.short, t.short.trim());
        assert_eq!(row.long, t.long.trim());
        let see: Vec<&str> = row.see_also.iter().map(|s| s.as_str()).collect();
        assert_eq!(see, t.see_also);
        assert_eq!(row.since, t.since);
    }
    assert_eq!(qontinui_types::glossary::GLOSSARY_VERSION, src.version);
}

/// The plan's roster (C1) — the terms an operator meets in the product's own
/// surfaces. Removing one is a deliberate act that edits this list.
#[test]
fn plan_roster_is_present() {
    let src = parse(&source_text());
    let ids: BTreeSet<&str> = src.term.iter().map(|t| t.id.as_str()).collect();
    for id in [
        "gate",
        "work_unit",
        "merge_train",
        "attestation",
        "tier",
        "claim",
        "continuation",
        "finding",
        "dossier",
        "policy",
        "clause",
        "intent_document",
        "drain",
        "restart_readiness",
        "session_status",
        "rung",
        "tenant",
        "device",
        "worktree_allocation",
        "landed",
        "unknown",
    ] {
        assert!(ids.contains(id), "roster term `{id}` is missing");
    }
}

// ── The checks catch what they name (mutation proofs, in-process) ──────────

fn mutated(f: impl FnOnce(&mut Source)) -> Vec<String> {
    let mut src = parse(&source_text());
    f(&mut src);
    violations(&src)
}

#[test]
fn a_dangling_see_also_is_caught() {
    let v = mutated(|s| s.term[0].see_also.push("no_such_term".into()));
    assert!(v.iter().any(|m| m.contains("dangling")), "{v:?}");
}

#[test]
fn length_id_and_since_violations_are_caught() {
    let v = mutated(|s| s.term[0].short = "x".repeat(SHORT_MAX + 1));
    assert!(v.iter().any(|m| m.contains("short is")), "{v:?}");
    let v = mutated(|s| s.term[0].long = "x".repeat(LONG_MAX + 1));
    assert!(v.iter().any(|m| m.contains("long is")), "{v:?}");
    let v = mutated(|s| s.term[0].id = "Work-Unit".into());
    assert!(v.iter().any(|m| m.contains("not snake_case")), "{v:?}");
    let v = mutated(|s| {
        let dup = s.term[1].id.clone();
        s.term[0].id = dup;
    });
    assert!(v.iter().any(|m| m.contains("duplicate id")), "{v:?}");
    let v = mutated(|s| s.term[0].since = s.version + 1);
    assert!(v.iter().any(|m| m.contains("since")), "{v:?}");
    let v = mutated(|s| {
        let me = s.term[0].id.clone();
        s.term[0].see_also.push(me);
    });
    assert!(v.iter().any(|m| m.contains("cites itself")), "{v:?}");
}

#[test]
fn the_fleet_noun_check_is_not_vacuous() {
    let nouns = FleetNouns::load();
    assert!(!nouns
        .hits("write the plan to qontinui-dev-notes/plans/")
        .is_empty());
    assert!(!nouns.hits("curl http://localhost:8000/api").is_empty());
    assert!(nouns.hits("A gate resumes work when it clears.").is_empty());
}

#[test]
fn the_version_guard_refuses_an_unbumped_change() {
    let committed = (3, "aaaa");
    // Unchanged content: fine at the same version.
    assert!(version_guard(3, "aaaa", committed).is_ok());
    // Changed content at the same (or a lower) version: refused.
    let e = version_guard(3, "bbbb", committed).unwrap_err();
    assert!(e.contains("Raise `version` to 4"), "{e}");
    assert!(version_guard(2, "bbbb", committed).is_err());
    // Changed content with a raised version: accepted (then regenerated).
    assert!(version_guard(4, "bbbb", committed).is_ok());
}

#[test]
fn render_is_deterministic_and_changes_with_content() {
    let text = source_text();
    let src = parse(&text);
    let a = render(&src, &sha256_hex(&text));
    assert_eq!(a, render(&src, &sha256_hex(&text)));
    // Even a comment-only edit is a content change: new digest, new output.
    let edited = format!("{text}\n# edited\n");
    assert_ne!(sha256_hex(&edited), sha256_hex(&text));
    assert_ne!(render(&parse(&edited), &sha256_hex(&edited)), a);
}
