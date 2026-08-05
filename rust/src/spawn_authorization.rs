//! Agent-spawn authorization — the ONE declaration of the configuration
//! surface.
//!
//! Plan `2026-08-03-agent-spawn-authorization-ground-truth`, phase P0.
//!
//! ## The problem this exists to end
//!
//! > "There are too many places configuring these variables. Establish a
//! > ground truth." — the operator
//!
//! One decision — *may this spawn happen, and how wide* — was expressed in at
//! least eight places, in three key spaces, with two fold algorithms, and
//! nothing checked that any of them agreed. It had already caused an outage:
//! on 2026-08-03 coord seeded the nine spawn KINDS but not the per-PATH class
//! row the runner's unnamed funnel sites resolve, so **every coord-initiated
//! standing spawn was denied fleet-wide** while the registry cheerfully served
//! `gate_continuation, enabled: true`.
//!
//! `fanout_bound` was the still-live second instance: six declaration sites,
//! two disagreeing values (the DB default and the runner said 15; coord's seed
//! wrote 1), and two different ceilings for one field (coord 100, runner 1024).
//!
//! ## Why a DATA FILE and not Rust constants
//!
//! The values must reach Rust (coord + the runner), Python (the web seeder)
//! and TypeScript (the settings UI). The existing schemars → `export_schemas`
//! pipeline carries **type shapes only** — JSON Schema has no way to express
//! constant data — so the tables below could never have ridden it.
//!
//! [`CONTRACT_JSON`] is therefore the canonical artifact and this module is a
//! thin compile-time loader over it: `include_str!` + `serde`, so consumers
//! get a `const`-equivalent with no runtime fetch and no I/O.
//!
//! ## Editing rules
//!
//! - **Edit the JSON, never a consumer's literal.** The whole point is that no
//!   consumer re-declares these values. A copy that drifts is the bug.
//! - A class row that is deliberately NOT seeded records `seeded: false` **and
//!   a `reason`** ([`ClassSeed`]). An omission expressed as an absent literal
//!   is invisible; expressed as data, a test can assert on it.
//! - **Every key is declared.** All three types are `deny_unknown_fields`, so
//!   a key this module does not know fails the PARSE rather than being
//!   silently dropped. A hopeful `"disposition"` on a row type that has none
//!   would otherwise read as set and behave as absent — the 2026-08-03 shape,
//!   one level down. Adding a field to the JSON therefore means adding it here
//!   in the same change; that coupling is deliberate.
//! - Adding a spawn path or disposition is a vocabulary change: the DB columns
//!   are deliberately `CHECK`-free (`agent_registry.rs`'s
//!   "Vocabulary lives in Rust, not the DB", citing coord's
//!   `sessions.rs:356-377`), so no migration is required — but coord's
//!   authoring validators and this file must agree, which coord's tests pin.
//!
//! ## Scope — Rust only, deliberately
//!
//! Only the Rust loader ships here. One physical file cannot serve all three
//! languages: `ts/tsconfig.json` sets `rootDir: "./src"` so a JSON outside
//! `ts/src/` cannot enter the TS build at all, poetry pins the Python copy
//! under `src/qontinui_schemas/`, and a publishable cargo crate pins its copy
//! under `rust/`. The TS and Python arms are therefore GENERATED copies riding
//! `scripts/check-generated-drift.sh`, and they land with their first consumer
//! (the web seeder / settings UI) rather than speculatively — a generator with
//! no consumer is exactly the kind of artifact that goes vacuously green.

use serde::{Deserialize, Serialize};

/// The canonical contract, embedded at compile time.
///
/// Exposed so a consumer in another language (or a test asserting the file is
/// the one that was loaded) can read the exact bytes.
pub const CONTRACT_JSON: &str = include_str!("spawn_authorization.json");

/// One coord-initiated spawn KIND: a concrete reason coord starts an agent.
///
/// `deny_unknown_fields`, here and on the two types below, because serde's
/// default is to DROP a key it does not recognize. In an artifact whose whole
/// premise is that nothing silently disagrees, a mistyped or hopeful key
/// (`"dispostion"`, or a `"disposition"` on a row type that has none) would
/// read to a reviewer as set and behave as absent — the 2026-08-03 shape
/// exactly, one level down.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct KindSeed {
    pub agent_name: String,
    pub spawn_path: String,
    pub purpose: String,
}

/// One per-PATH CLASS row — the lever a user actually toggles for a whole
/// spawn class.
///
/// `seeded: false` carries a mandatory `reason`. That asymmetry is the point:
/// the 2026-08-03 outage was an omission nobody could see, because "we do not
/// seed this class" was expressed by a literal simply not being in a list.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ClassSeed {
    pub spawn_path: String,
    pub seeded: bool,
    /// Required when `seeded` is false; `None` when it is true.
    pub reason: Option<String>,
    pub purpose: String,
}

/// The whole contract.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SpawnAuthorizationContract {
    /// The file's own header — the line that routes whoever opens the JSON to
    /// the editing rules in this module.
    ///
    /// Declared rather than tolerated: `deny_unknown_fields` admits no key it
    /// does not know, and waving `$comment` through with a catch-all would
    /// re-open the hole for every OTHER stray top-level key. [`validate`]
    /// requires it non-empty, so it stays a routing device rather than
    /// decoration.
    ///
    /// [`validate`]: SpawnAuthorizationContract::validate
    #[serde(rename = "$comment")]
    pub comment: String,
    /// Every legal `spawn_path` wire value.
    pub spawn_paths: Vec<String>,
    /// Every legal `disposition` wire value.
    pub dispositions: Vec<String>,
    /// The fan-out bound a row gets when it declares none. Must equal the
    /// alembic column default and the runner's `DEFAULT_FANOUT_BOUND`.
    pub default_fanout_bound: i32,
    /// The ceiling coord enforces on the authoring surface.
    ///
    /// NOT yet fleet-single-valued: `qontinui-runner`'s own
    /// `MAX_FANOUT_BOUND` is still 1024 on `origin/main`, so coord will
    /// reject a bound of 500 the runner would clamp and honour. Coord's side
    /// is pinned to this value; the runner arm closes when it adopts the
    /// contract. The divergence is asserted explicitly in coord's
    /// `one_fanout_default_and_one_ceiling` so that closing it is a
    /// deliberate act rather than a silent one.
    pub max_fanout_bound: i32,
    /// What a freshly seeded row's `default_enabled` is.
    pub default_enabled: bool,
    pub kind_seeds: Vec<KindSeed>,
    pub class_seeds: Vec<ClassSeed>,
}

impl SpawnAuthorizationContract {
    /// Parse [`CONTRACT_JSON`].
    ///
    /// Prefer [`contract`], which parses once per process.
    pub fn load() -> Result<Self, serde_json::Error> {
        serde_json::from_str(CONTRACT_JSON)
    }

    /// The class rows that ARE seeded, in declaration order.
    pub fn seeded_classes(&self) -> impl Iterator<Item = &ClassSeed> {
        self.class_seeds.iter().filter(|c| c.seeded)
    }

    /// Is `value` a declared `spawn_path` wire value?
    ///
    /// The membership test [`validate`] itself uses, exposed because the
    /// consumers that must agree with this file — coord's authoring
    /// validators, per this module's editing rules — need exactly it, and a
    /// consumer that re-derives membership from its own copy of the list is
    /// the drift the contract exists to prevent.
    ///
    /// [`validate`]: SpawnAuthorizationContract::validate
    pub fn is_spawn_path(&self, value: &str) -> bool {
        self.spawn_paths.iter().any(|p| p == value)
    }

    /// Is `value` a declared `disposition` wire value?
    ///
    /// `dispositions` has no other reader in this crate — it is a vocabulary
    /// the DB deliberately does not `CHECK` (see this module's editing rules),
    /// so this accessor is where that vocabulary is enforceable at all.
    pub fn is_disposition(&self, value: &str) -> bool {
        self.dispositions.iter().any(|d| d == value)
    }

    /// Every `(agent_name, spawn_path)` key a seeder writes — the KIND rows
    /// plus the seeded CLASS rows, whose `agent_name` IS the class.
    ///
    /// This is the set a divergence test compares against the keys the
    /// runner's lookup can resolve. A key the runner can resolve that is
    /// absent here (without a `seeded: false` + reason) is the 2026-08-03
    /// outage, exactly.
    pub fn seeded_keys(&self) -> Vec<(String, String)> {
        let mut keys: Vec<(String, String)> = self
            .kind_seeds
            .iter()
            .map(|k| (k.agent_name.clone(), k.spawn_path.clone()))
            .collect();
        keys.extend(
            self.seeded_classes()
                .map(|c| (c.spawn_path.clone(), c.spawn_path.clone())),
        );
        keys
    }

    /// Structural invariants the file must satisfy. Called by [`contract`], so
    /// a malformed contract is a panic at first use rather than a subtly wrong
    /// authorization decision later.
    pub fn validate(&self) -> Result<(), String> {
        if self.comment.trim().is_empty() {
            return Err("`$comment` must carry the file's header — it is what \
                        routes an editor to the rules in spawn_authorization.rs"
                .into());
        }
        // Both vocabularies, checked identically. A blank entry is not a wire
        // value (it would key a registry row nothing can look up), and a
        // REPEATED entry makes the contract disagree with itself one level
        // above the duplicate-class-row check below: the per-path loop would
        // resolve the same path twice while `seen_classes` still saw one row.
        for (label, values) in [
            ("spawn_paths", &self.spawn_paths),
            ("dispositions", &self.dispositions),
        ] {
            if values.is_empty() {
                return Err(format!("{label} must not be empty"));
            }
            let mut seen = std::collections::HashSet::new();
            for v in values {
                if v.trim().is_empty() {
                    return Err(format!("{label} contains a blank entry"));
                }
                if !seen.insert(v.as_str()) {
                    return Err(format!("{label} declares `{v}` twice"));
                }
            }
        }
        if self.default_fanout_bound < 1 || self.default_fanout_bound > self.max_fanout_bound {
            return Err(format!(
                "default_fanout_bound {} must be within 1..={}",
                self.default_fanout_bound, self.max_fanout_bound
            ));
        }
        // One kind is one row, so `agent_name` must resolve to exactly one
        // path. `seeded_keys` dedupes on the (name, path) PAIR and so cannot
        // see this: two rows for one kind on different paths both survive it,
        // leaving two answers to "which class governs this kind?".
        let mut seen_kinds = std::collections::HashSet::new();
        for k in &self.kind_seeds {
            if !self.is_spawn_path(&k.spawn_path) {
                return Err(format!(
                    "kind seed `{}` declares unknown spawn_path `{}`",
                    k.agent_name, k.spawn_path
                ));
            }
            if k.agent_name.trim().is_empty() {
                return Err("a kind seed has a blank agent_name".into());
            }
            // Class rows key as (spawn_path, spawn_path) in `seeded_keys`, so
            // a kind NAMED after a path shares that key space and is
            // indistinguishable from that path's class row.
            if self.is_spawn_path(&k.agent_name) {
                return Err(format!(
                    "kind seed `{}` is named after a declared spawn_path; class \
                     rows key as (spawn_path, spawn_path), so the two would be \
                     indistinguishable",
                    k.agent_name
                ));
            }
            if !seen_kinds.insert(k.agent_name.as_str()) {
                return Err(format!(
                    "kind `{}` is declared twice — one kind resolves to exactly \
                     one spawn_path, and `seeded_keys` dedupes on the (name, \
                     path) pair so it cannot catch a second path for one kind",
                    k.agent_name
                ));
            }
            if k.purpose.trim().is_empty() {
                return Err(format!("kind seed `{}` has an empty purpose", k.agent_name));
            }
        }
        // A class row for a path that does not exist, or TWO rows for one
        // path, would let `validate` (which uses `.find`, first wins) and
        // `seeded_classes` (which yields both) disagree about the same path —
        // inside the artifact whose entire purpose is that nothing disagrees.
        let mut seen_classes = std::collections::HashSet::new();
        for c in &self.class_seeds {
            if !self.is_spawn_path(&c.spawn_path) {
                return Err(format!(
                    "class_seeds entry `{}` names no declared spawn_path",
                    c.spawn_path
                ));
            }
            if !seen_classes.insert(c.spawn_path.as_str()) {
                return Err(format!(
                    "duplicate class_seeds entry for `{}` — `validate` takes the \
                     first and `seeded_classes` yields both, so the contract \
                     would disagree with itself",
                    c.spawn_path
                ));
            }
        }
        // EVERY path needs a class row — present-and-seeded, or
        // present-and-explicitly-not. Silence is what caused the outage.
        for path in &self.spawn_paths {
            let Some(c) = self.class_seeds.iter().find(|c| &c.spawn_path == path) else {
                return Err(format!(
                    "spawn_path `{path}` has no class_seeds entry — an omission \
                     must be RECORDED (`seeded: false` with a reason), never absent"
                ));
            };
            match (c.seeded, c.reason.as_deref().map(str::trim)) {
                (false, None) | (false, Some("")) => {
                    return Err(format!(
                        "class `{path}` is `seeded: false` with no reason — an \
                         unexplained omission is indistinguishable from an oversight"
                    ))
                }
                (true, Some(r)) if !r.is_empty() => {
                    return Err(format!(
                        "class `{path}` is seeded but carries a `reason` ({r}); \
                         reason documents an OMISSION only"
                    ))
                }
                _ => {}
            }
            if c.seeded && c.purpose.trim().is_empty() {
                return Err(format!("seeded class `{path}` has an empty purpose"));
            }
        }
        let mut seen = std::collections::HashSet::new();
        for (name, path) in self.seeded_keys() {
            if !seen.insert((name.clone(), path.clone())) {
                return Err(format!("duplicate seeded key ({name}, {path})"));
            }
        }
        Ok(())
    }
}

/// The parsed, validated contract — parsed once per process.
///
/// Panics if the embedded JSON is malformed or violates [`validate`]. That is
/// deliberate: both are compile-time-constant faults that a test in this crate
/// catches, so a panic here can only mean the crate shipped broken.
///
/// [`validate`]: SpawnAuthorizationContract::validate
pub fn contract() -> &'static SpawnAuthorizationContract {
    static CONTRACT: std::sync::OnceLock<SpawnAuthorizationContract> = std::sync::OnceLock::new();
    CONTRACT.get_or_init(|| {
        let c = SpawnAuthorizationContract::load()
            .expect("embedded spawn_authorization.json must parse");
        c.validate()
            .expect("embedded spawn_authorization.json must satisfy its invariants");
        c
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_embedded_contract_parses_and_validates() {
        let c = contract();
        assert_eq!(c.kind_seeds.len(), 9, "the nine coord spawn kinds");
        assert_eq!(c.spawn_paths.len(), 3);
        assert_eq!(c.dispositions, vec!["block", "degrade", "warn_proceed"]);
    }

    /// The values the whole plan exists to unify.
    ///
    /// `default_fanout_bound` is 15 — matching the alembic column default and
    /// the runner's `DEFAULT_FANOUT_BOUND`. coord's seed used to hardcode
    /// **1**, which is why `parallel_fanout` could not be seeded at all: a
    /// class row carrying 1 would have replaced the runner's 15 and serialized
    /// every fan-out on the fleet.
    #[test]
    fn the_unified_fanout_values_are_the_agreed_ones() {
        let c = contract();
        assert_eq!(c.default_fanout_bound, 15);
        assert_eq!(c.max_fanout_bound, 100);
        // Pinned for the same reason as the two above: a freshly seeded row's
        // enabled state is a value the seeder reads, and it is the one field
        // of the contract that nothing else in this crate touches — so without
        // this assertion it could be flipped with no test going red.
        assert!(c.default_enabled);
    }

    /// The vocabularies are queryable, and the accessors actually discriminate.
    ///
    /// The negative half is the load-bearing one: an `is_*` that returned
    /// `true` unconditionally would satisfy every positive assertion.
    #[test]
    fn the_vocabularies_are_queryable() {
        let c = contract();
        for p in &c.spawn_paths {
            assert!(c.is_spawn_path(p));
        }
        for d in &c.dispositions {
            assert!(c.is_disposition(d));
        }
        assert!(!c.is_spawn_path("not_a_path"));
        assert!(!c.is_disposition("not_a_disposition"));
        // The two vocabularies are separate namespaces, not one pool.
        assert!(!c.is_spawn_path("block"));
        assert!(!c.is_disposition("parallel_fanout"));
    }

    /// `parallel_fanout` IS seeded now, and the reason it could not be is gone.
    #[test]
    fn parallel_fanout_is_seeded_and_in_session_is_explicitly_not() {
        let c = contract();
        let by = |p: &str| {
            c.class_seeds
                .iter()
                .find(|x| x.spawn_path == p)
                .unwrap_or_else(|| panic!("no class row for {p}"))
        };
        assert!(by("parallel_fanout").seeded);
        assert!(by("standing_continuation").seeded);

        let in_session = by("in_session_subagent");
        assert!(!in_session.seeded);
        assert!(
            in_session.reason.as_deref().is_some_and(|r| !r.is_empty()),
            "a deliberate omission must carry its reason as DATA — an absent \
             literal is what made the 2026-08-03 outage invisible"
        );
    }

    /// Every seeded key is unique and every kind's path is a declared path.
    #[test]
    fn seeded_keys_are_well_formed() {
        let c = contract();
        let keys = c.seeded_keys();
        assert_eq!(keys.len(), 9 + 2, "9 kinds + 2 seeded classes");
        for (_, path) in &keys {
            assert!(c.spawn_paths.contains(path), "unknown path {path}");
        }
    }

    // ── Assert the assertions: each invariant must be able to FAIL ──────────
    //
    // `check-generated-drift.sh`'s own header records that both TypeScript
    // drift arms were vacuously green for their entire life because they
    // diffed against an untracked directory: "a gate that cannot go red is
    // worse than no gate: it manufactures confidence." `validate` is this
    // crate's gate, so every arm of it is fed a deliberately broken contract
    // and REQUIRED to reject it.

    fn valid() -> SpawnAuthorizationContract {
        SpawnAuthorizationContract::load().expect("fixture base must parse")
    }

    #[test]
    fn validate_rejects_a_class_omission_with_no_reason() {
        let mut c = valid();
        let row = c
            .class_seeds
            .iter_mut()
            .find(|x| !x.seeded)
            .expect("fixture has an unseeded class");
        row.reason = None;
        assert!(
            c.validate().is_err(),
            "an unexplained omission must be refused — that is the whole \
             mechanism protecting against the 2026-08-03 failure"
        );
    }

    #[test]
    fn validate_rejects_a_path_with_no_class_row_at_all() {
        let mut c = valid();
        c.spawn_paths.push("brand_new_class".to_string());
        assert!(
            c.validate().is_err(),
            "a path with no class row is the outage shape: silence, not a decision"
        );
    }

    #[test]
    fn validate_rejects_a_kind_pointing_at_an_unknown_path() {
        let mut c = valid();
        c.kind_seeds[0].spawn_path = "not_a_real_path".to_string();
        assert!(c.validate().is_err());
    }

    #[test]
    fn validate_rejects_a_default_bound_above_the_ceiling() {
        let mut c = valid();
        c.default_fanout_bound = c.max_fanout_bound + 1;
        assert!(c.validate().is_err());
    }

    #[test]
    fn validate_rejects_a_duplicate_seeded_key() {
        let mut c = valid();
        let dup = c.kind_seeds[0].clone();
        c.kind_seeds.push(dup);
        assert!(c.validate().is_err());
    }

    #[test]
    fn validate_rejects_a_whitespace_only_reason() {
        let mut c = valid();
        c.class_seeds
            .iter_mut()
            .find(|x| !x.seeded)
            .expect("fixture has an unseeded class")
            .reason = Some("   ".to_string());
        assert!(
            c.validate().is_err(),
            "`   ` is not a reason; it would satisfy every `omission is \
             recorded` assertion in both repos while explaining nothing"
        );
    }

    #[test]
    fn validate_rejects_two_class_rows_for_one_path() {
        let mut c = valid();
        let mut dup = c.class_seeds[0].clone();
        dup.seeded = !dup.seeded;
        dup.reason = if dup.seeded {
            None
        } else {
            Some("injected".into())
        };
        c.class_seeds.push(dup);
        assert!(
            c.validate().is_err(),
            "`validate` takes the FIRST row and `seeded_classes` yields BOTH, \
             so a duplicate makes the contract disagree with itself"
        );
    }

    #[test]
    fn validate_rejects_a_class_row_for_an_undeclared_path() {
        let mut c = valid();
        let mut orphan = c.class_seeds[0].clone();
        orphan.spawn_path = "not_in_spawn_paths".to_string();
        c.class_seeds.push(orphan);
        assert!(c.validate().is_err());
    }

    #[test]
    fn validate_rejects_a_duplicate_spawn_path() {
        let mut c = valid();
        c.spawn_paths.push(c.spawn_paths[0].clone());
        assert!(
            c.validate().is_err(),
            "a repeated vocabulary entry makes the contract disagree with itself \
             one level ABOVE the duplicate-class-row check: the class loop resolves \
             the same path twice while `seen_classes` still sees one row"
        );
    }

    #[test]
    fn validate_rejects_a_duplicate_disposition() {
        let mut c = valid();
        c.dispositions.push(c.dispositions[0].clone());
        assert!(c.validate().is_err());
    }

    #[test]
    fn validate_rejects_a_blank_vocabulary_entry() {
        let mut c = valid();
        c.spawn_paths.push("   ".to_string());
        assert!(
            c.validate().is_err(),
            "a blank wire value is not a path; it would key a registry row \
             nothing can ever look up"
        );

        let mut c = valid();
        c.dispositions.push(String::new());
        assert!(c.validate().is_err());
    }

    #[test]
    fn validate_rejects_one_kind_declared_on_two_paths() {
        let mut c = valid();
        let mut twin = c.kind_seeds[0].clone();
        twin.spawn_path = c
            .spawn_paths
            .iter()
            .find(|p| *p != &c.kind_seeds[0].spawn_path)
            .expect("fixture declares more than one path")
            .clone();
        c.kind_seeds.push(twin);
        assert!(
            c.validate().is_err(),
            "`seeded_keys` dedupes on the (name, path) PAIR, so one kind on two \
             paths survives it — and the contract then gives two answers to \
             `which class governs this kind?`"
        );
    }

    #[test]
    fn validate_rejects_a_kind_named_after_a_spawn_path() {
        let mut c = valid();
        let path = c.spawn_paths[0].clone();
        c.kind_seeds.push(KindSeed {
            agent_name: path,
            // A path DIFFERENT from its name, so the (name, path) pair is
            // unique and the existing duplicate-key check waves it through.
            spawn_path: c.spawn_paths[1].clone(),
            purpose: "injected".into(),
        });
        assert!(
            c.validate().is_err(),
            "class rows key as (spawn_path, spawn_path), so a kind named after a \
             path is indistinguishable from that path's class row"
        );
    }

    #[test]
    fn an_unknown_field_in_the_contract_is_refused() {
        // Silently dropping a key is the failure mode this whole artifact
        // exists to end: a `disposition` a reviewer believes they set, on a row
        // whose type has no such field, reads as set and behaves as absent.
        //
        // The injected key must be a PURE ADDITION. Renaming an existing one
        // (`"seeded"` -> `"seeded_typo"`) also fails the parse today, but on
        // the REQUIRED FIELD MISSING arm — a green that says nothing about
        // whether unknown keys are refused. That is the vacuity this repo's
        // own `check-generated-drift.sh` header was written about.
        let anchor = "\"spawn_path\": \"parallel_fanout\"";
        assert!(
            CONTRACT_JSON.contains(anchor),
            "anchor must exist to inject at"
        );
        let broken = CONTRACT_JSON.replacen(
            anchor,
            "\"disposition\": \"block\", \"spawn_path\": \"parallel_fanout\"",
            1,
        );
        assert_ne!(
            broken, CONTRACT_JSON,
            "the fixture must actually be altered"
        );
        assert!(
            serde_json::from_str::<SpawnAuthorizationContract>(&broken).is_err(),
            "an unrecognized key must fail the parse, not vanish — every \
             required field is still present here, so this can only go green \
             on `deny_unknown_fields`"
        );
    }

    #[test]
    fn validate_accepts_the_shipped_contract() {
        // The companion to EVERY rejection above: proves they are refusing the
        // INJECTED fault, not failing for some unrelated reason that would make
        // every fixture red regardless. (Deliberately not a count — the "five"
        // this said originally was already stale by the time it landed, and a
        // number here rots on every added arm.)
        assert!(valid().validate().is_ok());
    }
}
