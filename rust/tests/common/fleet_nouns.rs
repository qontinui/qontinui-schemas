//! A consumer of the repo-root `fleet-nouns.toml`, implementing the file's
//! "HOW A CONSUMER MATCHES" contract: one line at a time, terminator
//! stripped, `exclude` applied to the matched SPAN.
//!
//! Shared by the repo tests that must prove a product string carries no fleet
//! noun (`glossary_source.rs`, `refusal_render.rs`). The vocabulary's own
//! validation stays in `fleet_nouns_vocabulary.rs`.
//!
//! The file lives outside this crate's package, so it is read at run time and
//! a missing file is a failure with the path — an absent vocabulary is
//! UNKNOWN, never "no fleet nouns".

use std::path::PathBuf;

use regex::Regex;
use serde::Deserialize;

#[derive(Deserialize)]
struct Vocabulary {
    class: Vec<ClassRow>,
}

#[derive(Deserialize)]
struct ClassRow {
    id: String,
    pattern: String,
    #[serde(default)]
    exclude: Option<String>,
    examples: Vec<String>,
}

pub struct Class {
    pub id: String,
    pattern: Regex,
    exclude: Option<Regex>,
}

pub struct FleetNouns {
    pub classes: Vec<Class>,
}

impl Class {
    fn hits_line(&self, line: &str) -> bool {
        let excluded: Vec<(usize, usize)> = self
            .exclude
            .as_ref()
            .map(|x| x.find_iter(line).map(|m| (m.start(), m.end())).collect())
            .unwrap_or_default();
        self.pattern
            .find_iter(line)
            .any(|m| !excluded.iter().any(|&(s, e)| m.start() < e && s < m.end()))
    }
}

impl FleetNouns {
    /// Load and compile `../fleet-nouns.toml`, and prove the matcher is live:
    /// at least seven classes, and every class hits its own first example.
    pub fn load() -> Self {
        let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("..")
            .join("fleet-nouns.toml");
        let text = std::fs::read_to_string(&path).unwrap_or_else(|e| {
            panic!(
                "fleet-nouns.toml is unreadable at {} ({e}) — an absent vocabulary is a failure, not an empty one",
                path.display()
            )
        });
        let v: Vocabulary = toml::from_str(&text)
            .unwrap_or_else(|e| panic!("{} does not parse: {e}", path.display()));
        let mut classes = Vec::new();
        let mut firsts = Vec::new();
        for c in v.class {
            let compiled = Class {
                pattern: Regex::new(&c.pattern)
                    .unwrap_or_else(|e| panic!("class `{}` pattern: {e}", c.id)),
                exclude: c.exclude.as_deref().map(|x| {
                    Regex::new(x).unwrap_or_else(|e| panic!("class `{}` exclude: {e}", c.id))
                }),
                id: c.id,
            };
            firsts.push(c.examples.first().cloned().unwrap_or_default());
            classes.push(compiled);
        }
        assert!(
            classes.len() >= 7,
            "only {} fleet-noun classes loaded from {}",
            classes.len(),
            path.display()
        );
        let me = Self { classes };
        for (class, example) in me.classes.iter().zip(&firsts) {
            assert!(
                class.hits_line(example),
                "vacuity guard: class `{}` does not hit its own first example {example:?}",
                class.id
            );
        }
        me
    }

    /// The ids of every class that hits any line of `text`.
    pub fn hits(&self, text: &str) -> Vec<&str> {
        let mut out = Vec::new();
        for class in &self.classes {
            let hit = text
                .split('\n')
                .map(|l| l.strip_suffix('\r').unwrap_or(l))
                .any(|l| class.hits_line(l));
            if hit {
                out.push(class.id.as_str());
            }
        }
        out
    }
}
