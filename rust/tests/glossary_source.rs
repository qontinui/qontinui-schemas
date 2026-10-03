//! The glossary source (`glossary/terms.toml`) and its two generated tables
//! — the Rust `rust/src/glossary/generated.rs` and the TypeScript
//! `ts/src/glossary/generated.ts` — plan
//! `2026-09-20-the-published-product-works-without-knowing-a-development-environment-exists`,
//! Phases C1 and C3.
//!
//! This test IS the generator: it renders both tables from the TOML and fails
//! when either checked-in file differs. The TypeScript table is what a web
//! surface renders a definition from without a request (C3), so it is
//! generated from the same source by the same code rather than copied. To
//! regenerate after editing the TOML (and raising its `version`):
//!
//! ```text
//! QONTINUI_GLOSSARY_REGENERATE=1 cargo test -p qontinui-types --test glossary_source
//! ```
//!
//! It also enforces the source's invariants — including that no text names a
//! fleet noun from `fleet-nouns.toml`, because these definitions ship to every
//! user — and the version guard.
//!
//! ## The version guard: `glossary/versions.lock`
//!
//! An append-only ledger of `<version> <sha256>` lines, where the digest is of
//! a CANONICAL rendering of the terms (their parsed fields as JSON — so
//! comments, whitespace and key order do not count, and `version` itself is
//! not hashed). Only the NEWEST row decides; older rows are history. It
//! fails when:
//!
//! - the ledger's versions are not strictly increasing;
//! - the current `version` is below the newest recorded version;
//! - the current `version` equals the newest but its digest differs (content
//!   changed without raising `version`);
//! - the current `version` is above the newest but not exactly newest + 1, or
//!   its digest equals the newest's (`version` raised with no content change);
//! - the current `(version, digest)` is a legitimate newest + 1 but not yet in
//!   the ledger — the regenerate mode appends it.
//!
//! Returning to OLDER content under a new version is allowed (a revert is a
//! new version whose digest may repeat an earlier row's).
//!
//! The ledger lives in git, so two branches that both claim the next version
//! conflict on the same appended line instead of both passing their own CI.
//!
//! Both files live outside this crate's package, so this is a repo test
//! (excluded from the published crate in `rust/Cargo.toml`).

mod common;

use std::collections::{BTreeMap, BTreeSet};
use std::path::PathBuf;

use serde::{Deserialize, Serialize};
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

#[derive(Debug, Deserialize, Serialize)]
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

/// The generated TypeScript table, published as `@qontinui/shared-types/glossary`.
/// Kept out of `ts/src/generated/`, which belongs to the JSON-Schema binding
/// generator and its drift gate.
fn generated_ts_path() -> PathBuf {
    repo_root()
        .join("ts")
        .join("src")
        .join("glossary")
        .join("generated.ts")
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

fn lock_path() -> PathBuf {
    repo_root().join("glossary").join("versions.lock")
}

/// The digest of the glossary's CONTENT: the parsed terms as JSON (struct
/// field order, trimmed text). Comments, formatting and `version` do not count.
fn content_sha(src: &Source) -> String {
    let canon: Vec<Term> = src
        .term
        .iter()
        .map(|t| Term {
            id: t.id.clone(),
            term: t.term.trim().to_string(),
            short: t.short.trim().to_string(),
            long: t.long.trim().to_string(),
            see_also: t.see_also.clone(),
            since: t.since,
        })
        .collect();
    sha256_hex(&serde_json::to_string(&canon).expect("terms serialise"))
}

/// `(version, sha)` rows of versions.lock; `#` comments and blank lines skipped.
fn parse_lock(text: &str) -> Result<Vec<(u32, String)>, String> {
    let mut rows = Vec::new();
    for (i, line) in text.lines().enumerate() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let mut parts = line.split_whitespace();
        let (Some(v), Some(sha), None) = (parts.next(), parts.next(), parts.next()) else {
            return Err(format!(
                "versions.lock line {}: expected `<version> <sha256>`",
                i + 1
            ));
        };
        let v: u32 = v
            .parse()
            .map_err(|_| format!("versions.lock line {}: `{v}` is not a version", i + 1))?;
        if sha.len() != 64 || !sha.chars().all(|c| c.is_ascii_hexdigit()) {
            return Err(format!(
                "versions.lock line {}: `{sha}` is not a sha256",
                i + 1
            ));
        }
        rows.push((v, sha.to_string()));
    }
    Ok(rows)
}

/// What the ledger says about the current `(version, sha)`.
#[derive(Debug, PartialEq, Eq)]
enum LockVerdict {
    /// Already recorded: nothing to do.
    Recorded,
    /// A legitimate new version: append `(version, sha)`.
    Append,
}

fn check_lock(rows: &[(u32, String)], version: u32, sha: &str) -> Result<LockVerdict, String> {
    for w in rows.windows(2) {
        if w[1].0 <= w[0].0 {
            return Err(format!(
                "versions.lock is not strictly increasing ({} after {})",
                w[1].0, w[0].0
            ));
        }
    }
    let Some((newest, newest_sha)) = rows.last() else {
        return if version == 1 {
            Ok(LockVerdict::Append)
        } else {
            Err(format!(
                "versions.lock is empty, so the first version must be 1, not {version}"
            ))
        };
    };
    let newest = *newest;
    if version < newest {
        return Err(format!(
            "version {version} is below the newest recorded version {newest}"
        ));
    }
    if version == newest {
        if sha == newest_sha {
            return Ok(LockVerdict::Recorded);
        }
        return Err(format!(
            "glossary content changed (digest {sha}) but version {version} is already recorded \
             in versions.lock with digest {newest_sha}. Raise `version` in glossary/terms.toml to \
             {} — every content change is a new glossary version.",
            newest + 1
        ));
    }
    if version != newest + 1 {
        return Err(format!(
            "version {version} skips ahead; the next version is {}",
            newest + 1
        ));
    }
    if sha == newest_sha {
        return Err(format!(
            "version raised to {version} but the content is unchanged since version {newest}; \
             put `version` back to {newest}"
        ));
    }
    Ok(LockVerdict::Append)
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
        "/// SHA-256 of the canonical glossary content (the parsed terms as JSON; see glossary/versions.lock).\n",
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

/// A TypeScript string literal. JSON's string grammar is a subset of
/// JavaScript's, so `serde_json` is the escaper.
fn ts_lit(s: &str) -> String {
    serde_json::to_string(s).expect("a string serialises")
}

/// Render `ts/src/glossary/generated.ts`. Deterministic: same source, same
/// bytes.
///
/// The table's type is a mapped type over the `GlossaryTerm` union that the
/// JSON-Schema binding generator emits (`ts/src/generated/GlossaryTerm.d.ts`),
/// so the two generators are checked against each other by `tsc`: a term the
/// union lacks is an excess property, a term the table lacks is a missing one.
/// Field names are the Rust `GlossaryEntry`'s serialised names (`see_also`),
/// so the same type describes a row read from a served glossary door.
fn render_ts(src: &Source, sha: &str) -> String {
    use std::fmt::Write;
    let mut o = String::new();
    o.push_str("/* eslint-disable */\n");
    o.push_str(
        "// @generated from glossary/terms.toml by rust/tests/glossary_source.rs — do not edit.\n",
    );
    o.push_str("// Regenerate: QONTINUI_GLOSSARY_REGENERATE=1 cargo test -p qontinui-types --test glossary_source\n\n");
    o.push_str("import type { GlossaryTerm } from \"../generated/GlossaryTerm\";\n\n");
    o.push_str("/** One glossary definition (the Rust `GlossaryEntry`, as it serialises). */\n");
    o.push_str("export interface GlossaryEntry<Id extends GlossaryTerm = GlossaryTerm> {\n");
    o.push_str("  readonly id: Id;\n");
    o.push_str("  /** Display name. */\n  readonly term: string;\n");
    o.push_str(
        "  /** Plain text, at most 160 characters (tooltip). */\n  readonly short: string;\n",
    );
    o.push_str("  /** Markdown, at most 1200 characters. */\n  readonly long: string;\n");
    o.push_str("  /** Related terms. */\n  readonly see_also: readonly GlossaryTerm[];\n");
    o.push_str(
        "  /** The glossary version that introduced this term. */\n  readonly since: number;\n",
    );
    o.push_str("}\n\n");
    o.push_str("/** The glossary's `version` (glossary/terms.toml). */\n");
    writeln!(
        o,
        "export const GLOSSARY_VERSION: number = {};",
        src.version
    )
    .unwrap();
    o.push_str(
        "/** SHA-256 of the canonical glossary content (the parsed terms as JSON; see glossary/versions.lock). */\n",
    );
    writeln!(
        o,
        "export const GLOSSARY_CONTENT_SHA256 = {};\n",
        ts_lit(sha)
    )
    .unwrap();
    o.push_str("/** Every term id, in glossary order. */\n");
    o.push_str("export const GLOSSARY_TERMS: readonly GlossaryTerm[] = [\n");
    for t in &src.term {
        writeln!(o, "  {},", ts_lit(&t.id)).unwrap();
    }
    o.push_str("];\n\n");
    o.push_str("/** Every definition, keyed by id. */\n");
    o.push_str("export const GLOSSARY: { readonly [Id in GlossaryTerm]: GlossaryEntry<Id> } = {\n");
    for t in &src.term {
        writeln!(o, "  {}: {{", ts_lit(&t.id)).unwrap();
        writeln!(o, "    id: {},", ts_lit(&t.id)).unwrap();
        writeln!(o, "    term: {},", ts_lit(t.term.trim())).unwrap();
        writeln!(o, "    short: {},", ts_lit(t.short.trim())).unwrap();
        writeln!(o, "    long: {},", ts_lit(t.long.trim())).unwrap();
        let see: Vec<String> = t.see_also.iter().map(|s| ts_lit(s)).collect();
        writeln!(o, "    see_also: [{}],", see.join(", ")).unwrap();
        writeln!(o, "    since: {},", t.since).unwrap();
        o.push_str("  },\n");
    }
    o.push_str("};\n");
    o
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
fn generated_tables_match_the_source_and_the_version_was_raised() {
    let src = parse(&source_text());
    let sha = content_sha(&src);
    let regenerate = std::env::var_os(REGENERATE_ENV).is_some();

    let lock = lock_path();
    let lock_text = std::fs::read_to_string(&lock)
        .unwrap_or_else(|e| panic!("versions.lock unreadable at {} ({e})", lock.display()))
        .replace("\r\n", "\n");
    let rows = parse_lock(&lock_text).unwrap_or_else(|e| panic!("{e}"));
    match check_lock(&rows, src.version, &sha) {
        Err(e) => panic!("{e}"),
        Ok(LockVerdict::Recorded) => {}
        Ok(LockVerdict::Append) if regenerate => {
            let mut text = lock_text.clone();
            if !text.is_empty() && !text.ends_with('\n') {
                text.push('\n');
            }
            text.push_str(&format!("{} {sha}\n", src.version));
            std::fs::write(&lock, text)
                .unwrap_or_else(|e| panic!("cannot write {}: {e}", lock.display()));
            eprintln!("appended version {} to {}", src.version, lock.display());
        }
        Ok(LockVerdict::Append) => panic!(
            "glossary version {} (digest {sha}) is not in versions.lock. Record it:\n  \
             {REGENERATE_ENV}=1 cargo test -p qontinui-types --test glossary_source",
            src.version
        ),
    }

    let mut stale = Vec::new();
    for (path, want) in [
        (generated_path(), render(&src, &sha)),
        (generated_ts_path(), render_ts(&src, &sha)),
    ] {
        let have = std::fs::read_to_string(&path)
            .unwrap_or_default()
            .replace("\r\n", "\n");
        if have == want {
            continue;
        }
        if regenerate {
            std::fs::write(&path, &want)
                .unwrap_or_else(|e| panic!("cannot write {}: {e}", path.display()));
            eprintln!("regenerated {}", path.display());
            continue;
        }
        stale.push(path.display().to_string());
    }
    assert!(
        stale.is_empty(),
        "stale against glossary/terms.toml: {}. Regenerate:\n  \
         {REGENERATE_ENV}=1 cargo test -p qontinui-types --test glossary_source",
        stale.join(", ")
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
fn the_version_lock_refuses_every_bad_transition() {
    let a = "a".repeat(64);
    let b = "b".repeat(64);
    let c = "c".repeat(64);
    let rows = vec![(1, a.clone()), (2, b.clone())];
    // The newest row, as-is.
    assert_eq!(check_lock(&rows, 2, &b), Ok(LockVerdict::Recorded));
    // Going back to an older version is refused, even with its old content.
    let e = check_lock(&rows, 1, &a).unwrap_err();
    assert!(e.contains("below the newest"), "{e}");
    // Content changed, version not raised.
    let e = check_lock(&rows, 2, &c).unwrap_err();
    assert!(e.contains("Raise `version`") && e.contains(" 3 "), "{e}");
    // Version raised, content unchanged.
    let e = check_lock(&rows, 3, &b).unwrap_err();
    assert!(e.contains("unchanged"), "{e}");
    // Version skipping ahead.
    let e = check_lock(&rows, 4, &c).unwrap_err();
    assert!(e.contains("skips ahead"), "{e}");
    // A new version with new content: append.
    assert_eq!(check_lock(&rows, 3, &c), Ok(LockVerdict::Append));
    // Reverting to OLDER content under a new version is allowed.
    assert_eq!(check_lock(&rows, 3, &a), Ok(LockVerdict::Append));
    let reverted = vec![(1, a.clone()), (2, b.clone()), (3, a.clone())];
    assert_eq!(check_lock(&reverted, 3, &a), Ok(LockVerdict::Recorded));
    // A malformed ledger.
    assert!(check_lock(&[(2, a.clone()), (1, b.clone())], 3, &c).is_err());
    assert!(parse_lock("1 nothex").is_err());
    assert!(parse_lock(&format!("x {a}")).is_err());
    assert_eq!(
        parse_lock(&format!("# c\n\n1 {a}\n")).unwrap(),
        vec![(1, a.clone())]
    );
    // An empty ledger accepts only version 1.
    assert_eq!(check_lock(&[], 1, &a), Ok(LockVerdict::Append));
    assert!(check_lock(&[], 2, &a).is_err());
}

#[test]
fn the_digest_is_of_content_not_formatting() {
    let text = source_text();
    let src = parse(&text);
    let sha = content_sha(&src);
    let a = render(&src, &sha);
    assert_eq!(a, render(&src, &content_sha(&parse(&text))));
    // A comment-only edit, or a version change, is NOT a content change.
    let commented = format!("{text}\n# edited\n");
    assert_eq!(content_sha(&parse(&commented)), sha);
    let bumped = text.replacen(
        &format!("\nversion = {}\n", src.version),
        &format!("\nversion = {}\n", src.version + 1),
        1,
    );
    assert_ne!(bumped, text, "the version line was not found");
    assert_eq!(content_sha(&parse(&bumped)), sha);
    // An edit to a definition is.
    let mut edited = parse(&text);
    edited.term[0].short.push_str(" More.");
    assert_ne!(content_sha(&edited), sha);
}
