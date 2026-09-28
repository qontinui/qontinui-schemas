//! Validates the repo-root `fleet-nouns.toml` — the single fleet-noun
//! vocabulary every fleet-noun lint reads (plan
//! `2026-09-20-the-published-product-works-without-knowing-a-development-environment-exists`,
//! Phase A, "One vocabulary, one file").
//!
//! The load-bearing property is the last one: **no product constant and no
//! look-alike matches any class**. A class that flags `127.0.0.1:9876` would
//! make every consumer red on correct product text, and a consumer would then
//! be tempted to carve a local exception — a second vocabulary, which is what
//! this file exists to prevent.
//!
//! The file is read at run time (not `include_str!`) because it lives outside
//! this crate's package directory; a missing file is a failure with the path,
//! never a skipped test.

use std::collections::BTreeSet;
use std::path::PathBuf;

use regex::Regex;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct Vocabulary {
    version: u32,
    lookalikes: Vec<String>,
    portability_probes: Vec<String>,
    class: Vec<Class>,
    product_constant: Vec<ProductConstant>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct Class {
    id: String,
    plan_class_names: Vec<String>,
    pattern: String,
    #[serde(default)]
    exclude: Option<String>,
    #[serde(default)]
    literals: Vec<String>,
    examples: Vec<String>,
    why: String,
    next_action_for_author: String,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct ProductConstant {
    value: String,
    why: String,
}

/// Ids the vocabulary must keep: the five it shares with the runner roster
/// and the two it adds. A floor, so deleting a class cannot make the other
/// assertions vacuously pass.
const REQUIRED_IDS: &[&str] = &[
    "repo_layout",
    "dev_ports",
    "supervisor_dependency",
    "tenant_literal",
    "machine_path",
    "fleet_host_name",
    "fleet_device_uuid",
];

/// The plan's working class names; each must be absorbed by exactly one id.
const PLAN_CLASS_NAMES: &[&str] = &[
    "sibling_repo_name",
    "fleet_host_name",
    "fleet_device_uuid",
    "tenant_literal",
    "supervisor_port",
    "dev_stack_port",
    "plans_repo_name",
    "operator_drive_root",
];

/// The product constants the plan and check #71 name. A floor, like REQUIRED_IDS.
const REQUIRED_PRODUCT_CONSTANTS: &[&str] = &[
    "127.0.0.1:9876",
    "localhost:9876",
    "~/.qontinui/",
    "coord.qontinui.io",
    "api.qontinui.io",
    "$QONTINUI_PLANS_DIR",
];

/// Constructs outside the subset Rust `regex`, Python `re` and JS `RegExp`
/// share with one meaning. Lexical, because this test can run only one of the
/// three engines.
const NON_PORTABLE: &[(&str, &str)] = &[
    ("(?=", "lookahead (Rust regex has none)"),
    ("(?!", "negative lookahead (Rust regex has none)"),
    ("(?<=", "lookbehind (Rust regex has none)"),
    ("(?<!", "negative lookbehind (Rust regex has none)"),
    ("(?P<", "named group (Python/Rust spelling; JS differs)"),
    ("(?<", "named group (JS spelling; Python differs)"),
    ("(?i", "inline flag (JS has no leading inline flag)"),
    ("(?m", "inline flag (JS has no leading inline flag)"),
    ("(?s", "inline flag (JS has no leading inline flag)"),
    ("(?x", "inline flag (JS has no leading inline flag)"),
    ("(?u", "inline flag (JS has no leading inline flag)"),
    (
        "\\d",
        "\\d is Unicode in Rust/Python, ASCII in JS — write [0-9]",
    ),
    (
        "\\w",
        "\\w is Unicode in Rust/Python, ASCII in JS — write a class",
    ),
    ("\\s", "\\s differs per engine — write a class"),
    (
        "\\b",
        "\\b is a Unicode boundary in Rust/Python, ASCII in JS — write (?:^|[^A-Za-z0-9_])",
    ),
    (
        "\\B",
        "\\B is a Unicode non-boundary in Rust/Python, ASCII in JS",
    ),
    (
        "\\D",
        "\\D is Unicode in Rust/Python, ASCII in JS — write [^0-9]",
    ),
    (
        "\\W",
        "\\W is Unicode in Rust/Python, ASCII in JS — write a class",
    ),
    ("\\S", "\\S differs per engine — write a class"),
    ("\\A", "\\A is not JS"),
    ("\\z", "\\z is not JS/Python"),
    ("\\Z", "\\Z differs between Python and Rust"),
    ("\\p{", "Unicode classes need the JS u flag"),
];

/// Constructs that are non-portable only INSIDE a character class `[...]`:
/// Rust `regex` reads them as class syntax, Python `re` and JS `RegExp` (no
/// `v` flag) read them as literal characters.
const NON_PORTABLE_IN_CLASS: &[(&str, &str)] = &[
    (
        "[:",
        "POSIX class `[[:alpha:]]` (Rust syntax; literal in Python/JS)",
    ),
    (
        "[",
        "nested class (Rust syntax; literal in JS, FutureWarning in Python)",
    ),
    (
        "&&",
        "class intersection (Rust syntax; literal in Python/JS)",
    ),
    ("--", "class difference (Rust syntax; literal in Python/JS)"),
    (
        "~~",
        "class symmetric difference (Rust syntax; literal in Python/JS)",
    ),
];

fn vocabulary_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("fleet-nouns.toml")
}

fn load() -> Vocabulary {
    let path = vocabulary_path();
    let text = std::fs::read_to_string(&path).unwrap_or_else(|e| {
        panic!(
            "fleet-nouns.toml is unreadable at {} ({e}) — an absent vocabulary is a failure, not an empty one",
            path.display()
        )
    });
    toml::from_str(&text).unwrap_or_else(|e| panic!("{} does not parse: {e}", path.display()))
}

/// Backslash-escape aware scan: `\\d` (an escaped backslash then `d`) is not
/// `\d`. Byte-wise, so a non-ASCII pattern is reported rather than panicking on
/// a char boundary.
fn non_portable_constructs(pattern: &str) -> Vec<String> {
    let mut found = Vec::new();
    let bytes = pattern.as_bytes();
    let mut i = 0;
    let mut in_class = false;
    while i < bytes.len() {
        if in_class && bytes[i] != b'\\' {
            if bytes[i] == b']' {
                in_class = false;
                i += 1;
                continue;
            }
            // Longest token first: `[:` before `[`.
            if let Some((tok, why)) = NON_PORTABLE_IN_CLASS
                .iter()
                .find(|(t, _)| bytes[i..].starts_with(t.as_bytes()))
            {
                found.push(format!("`{tok}` inside a class at byte {i}: {why}"));
                i += tok.len();
                continue;
            }
            i += 1;
            continue;
        }
        if !in_class && bytes[i] == b'[' {
            in_class = true;
            i += 1;
            if bytes.get(i) == Some(&b'^') {
                i += 1;
            }
            // `[]` / `[^]`: a literal `]` in Python/Rust, an empty class in JS.
            if bytes.get(i) == Some(&b']') {
                found.push(format!(
                    "`]` first in a class at byte {i}: literal in Python/Rust, empty class in JS — escape it"
                ));
                i += 1;
            }
            continue;
        }
        if bytes[i] == b'\\' {
            let esc = &bytes[i..];
            for (tok, why) in NON_PORTABLE.iter().filter(|(t, _)| t.starts_with('\\')) {
                if esc.starts_with(tok.as_bytes()) {
                    found.push(format!("`{tok}` at byte {i}: {why}"));
                }
            }
            i += 2; // skip the escaped character, whatever it is
            continue;
        }
        if !in_class && bytes[i] == b'(' {
            let rest = &bytes[i..];
            for (tok, why) in NON_PORTABLE.iter().filter(|(t, _)| t.starts_with('(')) {
                // `(?<` is a prefix of the lookbehinds; report only the most specific.
                if rest.starts_with(tok.as_bytes())
                    && !(*tok == "(?<" && (rest.starts_with(b"(?<=") || rest.starts_with(b"(?<!")))
                {
                    found.push(format!("`{tok}` at byte {i}: {why}"));
                }
            }
        }
        i += 1;
    }
    found
}

struct Compiled<'a> {
    class: &'a Class,
    pattern: Regex,
    exclude: Option<Regex>,
}

impl Compiled<'_> {
    /// The consumer contract in the file's header: `exclude` applies to the
    /// MATCHED SPAN — a pattern match is discarded when it overlaps any exclude
    /// match on the same text; the text is a hit when one pattern match survives.
    fn hits(&self, text: &str) -> bool {
        let excluded: Vec<(usize, usize)> = self
            .exclude
            .as_ref()
            .map(|x| x.find_iter(text).map(|m| (m.start(), m.end())).collect())
            .unwrap_or_default();
        self.pattern
            .find_iter(text)
            .any(|m| !excluded.iter().any(|&(s, e)| m.start() < e && s < m.end()))
    }
}

fn compile(v: &Vocabulary) -> Vec<Compiled<'_>> {
    v.class
        .iter()
        .map(|c| Compiled {
            class: c,
            pattern: Regex::new(&c.pattern)
                .unwrap_or_else(|e| panic!("class `{}` pattern does not compile: {e}", c.id)),
            exclude: c.exclude.as_ref().map(|x| {
                Regex::new(x)
                    .unwrap_or_else(|e| panic!("class `{}` exclude does not compile: {e}", c.id))
            }),
        })
        .collect()
}

#[test]
fn version_is_one() {
    assert_eq!(load().version, 1, "a consumer keys its parser on `version`");
}

#[test]
fn every_class_is_complete_and_ids_are_unique() {
    let v = load();
    assert!(
        !v.class.is_empty(),
        "no [[class]] entries — a vacuous vocabulary"
    );
    let id_shape = Regex::new(r"^[a-z][a-z0-9_]*$").unwrap();
    let mut ids = BTreeSet::new();
    let mut problems = Vec::new();
    for c in &v.class {
        for (field, value) in [
            ("id", &c.id),
            ("pattern", &c.pattern),
            ("why", &c.why),
            ("next_action_for_author", &c.next_action_for_author),
        ] {
            if value.trim().is_empty() {
                problems.push(format!("class `{}` has an empty `{field}`", c.id));
            }
        }
        if !id_shape.is_match(&c.id) {
            problems.push(format!("class id `{}` is not snake_case", c.id));
        }
        if !ids.insert(c.id.as_str()) {
            problems.push(format!("class id `{}` is declared twice", c.id));
        }
        if c.examples.is_empty() {
            problems.push(format!(
                "class `{}` has no examples — nothing proves its pattern can match",
                c.id
            ));
        }
    }
    for req in REQUIRED_IDS {
        if !ids.contains(req) {
            problems.push(format!("required class id `{req}` is missing"));
        }
    }
    assert!(problems.is_empty(), "{}", problems.join("\n"));
}

#[test]
fn each_plan_class_name_maps_to_exactly_one_id() {
    let v = load();
    let mut problems = Vec::new();
    for name in PLAN_CLASS_NAMES {
        let owners: Vec<&str> = v
            .class
            .iter()
            .filter(|c| c.plan_class_names.iter().any(|n| n == name))
            .map(|c| c.id.as_str())
            .collect();
        if owners.len() != 1 {
            problems.push(format!(
                "plan class `{name}` maps to {owners:?}, want exactly one id"
            ));
        }
    }
    assert!(problems.is_empty(), "{}", problems.join("\n"));
}

#[test]
fn every_pattern_compiles_and_stays_portable() {
    let v = load();
    let _ = compile(&v); // panics naming the class on a compile error
    let mut problems = Vec::new();
    for c in &v.class {
        for (field, p) in std::iter::once(("pattern", &c.pattern))
            .chain(c.exclude.as_ref().map(|x| ("exclude", x)))
        {
            if !p.is_ascii() {
                problems.push(format!("class `{}` {field} is not ASCII", c.id));
            }
            for f in non_portable_constructs(p) {
                problems.push(format!("class `{}` {field}: {f}", c.id));
            }
        }
    }
    assert!(problems.is_empty(), "{}", problems.join("\n"));
}

#[test]
fn every_example_and_literal_matches_its_own_class() {
    let v = load();
    let compiled = compile(&v);
    let mut problems = Vec::new();
    for c in &compiled {
        for text in c.class.examples.iter().chain(&c.class.literals) {
            if !c.hits(text) {
                problems.push(format!(
                    "class `{}` does not match its own entry {text:?}",
                    c.class.id
                ));
            }
        }
    }
    assert!(problems.is_empty(), "{}", problems.join("\n"));
}

#[test]
fn no_product_constant_or_lookalike_matches_any_class() {
    let v = load();
    let compiled = compile(&v);
    let mut problems = Vec::new();

    let values: BTreeSet<&str> = v
        .product_constant
        .iter()
        .map(|k| k.value.as_str())
        .collect();
    for req in REQUIRED_PRODUCT_CONSTANTS {
        if !values.contains(req) {
            problems.push(format!("required product constant `{req}` is missing"));
        }
    }
    for k in &v.product_constant {
        if k.value.trim().is_empty() || k.why.trim().is_empty() {
            problems.push(format!(
                "product constant {:?} has an empty value or why",
                k.value
            ));
        }
    }
    assert!(
        !v.lookalikes.is_empty(),
        "no lookalikes — the negative fixture set is vacuous"
    );

    // Product constants: against the PATTERN alone — an exclude must never be
    // what keeps a product constant clean.
    for k in &v.product_constant {
        for c in compiled.iter().filter(|c| c.pattern.is_match(&k.value)) {
            problems.push(format!(
                "product constant {:?} matches class `{}`",
                k.value, c.class.id
            ));
        }
    }
    // Look-alikes: under the consumer contract (exclude applied to the span),
    // since some exist precisely to prove an exclude works.
    for text in &v.lookalikes {
        for c in compiled.iter().filter(|c| c.hits(text)) {
            problems.push(format!("lookalike {text:?} matches class `{}`", c.class.id));
        }
    }
    assert!(problems.is_empty(), "{}", problems.join("\n"));
}

#[test]
fn portability_scan_catches_what_it_names() {
    // The lexical guard is itself load-bearing; prove it is not vacuous.
    assert!(!non_portable_constructs(r"(?i)abc").is_empty());
    assert!(!non_portable_constructs(r"(?<![a-z])x").is_empty());
    assert!(!non_portable_constructs(r"(?<name>x)").is_empty());
    assert!(!non_portable_constructs(r"\d+").is_empty());
    assert!(!non_portable_constructs(r"x\by").is_empty());
    // Non-ASCII around an escape or a group must be scanned, not panic.
    assert!(!non_portable_constructs("é\\bé(?iü").is_empty());
    assert!(non_portable_constructs(r"\\d \\b [0-9] (?:a|b)").is_empty());
    // Rust class syntax that Python/JS read literally.
    assert!(!non_portable_constructs(r"[[:alpha:]]").is_empty());
    assert!(!non_portable_constructs(r"[a-z&&[^aeiou]]").is_empty());
    assert!(!non_portable_constructs(r"[a-z--b]").is_empty());
    assert!(!non_portable_constructs(r"[a~~b]").is_empty());
    assert!(!non_portable_constructs(r"[]a]").is_empty());
    // ...but the same characters outside a class, escaped, or a trailing `-`
    // inside one are fine.
    assert!(non_portable_constructs(r"a&&b--c~~d [a\-\-b] [^A-Za-z0-9_.~$}-] \[:x\]").is_empty());
}

/// Every text the vocabulary carries: examples, literals, look-alikes, product
/// constants and the portability probes, in file order, deduplicated.
fn all_texts(v: &Vocabulary) -> Vec<&str> {
    let mut seen = BTreeSet::new();
    v.class
        .iter()
        .flat_map(|c| c.examples.iter().chain(&c.literals))
        .chain(&v.lookalikes)
        .chain(v.product_constant.iter().map(|k| &k.value))
        .chain(&v.portability_probes)
        .map(String::as_str)
        .filter(|t| seen.insert(*t))
        .collect()
}

/// Rust's side of the cross-engine check. Always computed (so a panic here is a
/// red on every run); written as JSON — `[[text, [class ids hit]], ...]` — only
/// when `FLEET_NOUNS_MATRIX_OUT` names a path. rust-ci's
/// `.github/scripts/fleet-nouns-portability.py` compares it with Python `re`
/// and Node `RegExp` and fails on any disagreement.
#[test]
fn hit_matrix_for_the_cross_engine_check() {
    let v = load();
    let compiled = compile(&v);
    assert!(
        !v.portability_probes.is_empty(),
        "no portability_probes — the non-ASCII cross-engine fixtures are gone"
    );
    let rows: Vec<serde_json::Value> = all_texts(&v)
        .into_iter()
        .map(|t| {
            let ids: Vec<&str> = compiled
                .iter()
                .filter(|c| c.hits(t))
                .map(|c| c.class.id.as_str())
                .collect();
            serde_json::json!([t, ids])
        })
        .collect();
    if let Ok(out) = std::env::var("FLEET_NOUNS_MATRIX_OUT") {
        std::fs::write(&out, serde_json::to_string_pretty(&rows).unwrap())
            .unwrap_or_else(|e| panic!("cannot write the hit matrix to {out}: {e}"));
    }
}
