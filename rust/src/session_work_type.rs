//! Session work type — the ONE declaration of the session run-record
//! vocabulary.
//!
//! Plan `2026-10-23-session-duration-by-work-type-and-per-device-session-load`,
//! Phase 1 (design decisions D1, D2, D4).
//!
//! ## What it declares
//!
//! Every Claude session run coord records (`coord.session_runs`) carries two
//! axes that are never merged — who **initiated** it and what **work type** it
//! is — plus how it ended (`end_reason`, `end_source`) and how its session id
//! was bound (`bind_origin`). Coord derives `work_type` with one pure
//! classifier and stores the verdict beside [`classifier_version`], so a
//! vocabulary change is a re-classification of stored facts rather than a data
//! loss. This file is where that vocabulary and its version live.
//!
//! ## Why a DATA FILE and not a Rust enum
//!
//! Same reason as [`crate::spawn_authorization`]: the values are constant data
//! that consumers in several languages must agree on, which the schemars →
//! `export_schemas` pipeline (type shapes only) cannot carry. [`CONTRACT_JSON`]
//! is the canonical artifact; this module is a thin compile-time loader over
//! it (`include_str!` + `serde`), with no runtime fetch and no I/O.
//!
//! ## Editing rules
//!
//! - **Edit the JSON, never a consumer's literal.** A consumer that re-declares
//!   a work type is the drift this file exists to prevent.
//! - **Every key is declared.** All types are `deny_unknown_fields`, so a key
//!   this module does not know fails the PARSE rather than vanishing.
//! - **`unknown` is mandatory** in `work_types` and in `initiators`: an
//!   unclassifiable run is counted and shown as `unknown`, never folded into
//!   another type (served policy `verification-and-evidence`
//!   `unknown-must-not-render-as-a-default`). [`SessionWorkTypeContract::validate`]
//!   refuses a contract without it.
//! - **Changing what the classifier would decide for stored facts bumps
//!   `classifier_version`.** Adding a work type is such a change. The DB
//!   deliberately carries no `CHECK` on `work_type`, so no migration is needed;
//!   the `CHECK`ed columns (`initiator`, `end_source`, `end_reason`) DO need a
//!   qontinui-web migration when their lists here change.
//!
//! ## Scope — Rust only, deliberately
//!
//! No TypeScript or Python projection ships here, for the reason
//! [`crate::spawn_authorization`]'s "Scope" section gives: generated copies
//! land with their first consumer. The operator page takes its labels from
//! coord's stats response, so none is planned.

use serde::{Deserialize, Serialize};

/// The canonical contract, embedded at compile time.
pub const CONTRACT_JSON: &str = include_str!("session_work_type.json");

/// One work type: a wire name and a one-sentence description of what it
/// covers.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct WorkType {
    pub name: String,
    pub description: String,
}

/// The whole contract.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SessionWorkTypeContract {
    /// The file's own header, routing an editor to this module's rules.
    /// [`validate`](SessionWorkTypeContract::validate) requires it non-empty.
    #[serde(rename = "$comment")]
    pub comment: String,
    /// The version of the classifier that maps run facts to a work type.
    /// Stored beside every verdict; must be at least 1.
    pub classifier_version: u32,
    /// Every legal `work_type` wire value, in display order.
    pub work_types: Vec<WorkType>,
    /// Every legal `initiator` wire value.
    pub initiators: Vec<String>,
    /// Every legal `end_reason` wire value.
    pub end_reasons: Vec<String>,
    /// Every legal `end_source` wire value.
    pub end_sources: Vec<String>,
    /// Every legal `bind_origin` wire value.
    pub bind_origins: Vec<String>,
}

/// The value both mandatory fallback lists must contain.
pub const UNKNOWN: &str = "unknown";

impl SessionWorkTypeContract {
    /// Parse [`CONTRACT_JSON`]. Prefer [`contract`], which parses once per
    /// process and validates.
    pub fn load() -> Result<Self, serde_json::Error> {
        serde_json::from_str(CONTRACT_JSON)
    }

    /// Parse and validate an arbitrary contract body — the door the tests use
    /// to feed hand-built JSON through the same checks as the shipped file.
    pub fn parse_and_validate(json: &str) -> Result<Self, String> {
        let c: Self = serde_json::from_str(json).map_err(|e| format!("parse: {e}"))?;
        c.validate()?;
        Ok(c)
    }

    /// Is `value` a declared `work_type`?
    pub fn is_work_type(&self, value: &str) -> bool {
        self.work_types.iter().any(|w| w.name == value)
    }

    /// Is `value` a declared `initiator`?
    pub fn is_initiator(&self, value: &str) -> bool {
        self.initiators.iter().any(|v| v == value)
    }

    /// Is `value` a declared `end_reason`?
    pub fn is_end_reason(&self, value: &str) -> bool {
        self.end_reasons.iter().any(|v| v == value)
    }

    /// Is `value` a declared `end_source`?
    pub fn is_end_source(&self, value: &str) -> bool {
        self.end_sources.iter().any(|v| v == value)
    }

    /// Is `value` a declared `bind_origin`?
    pub fn is_bind_origin(&self, value: &str) -> bool {
        self.bind_origins.iter().any(|v| v == value)
    }

    /// The description of a declared work type, or `None` for an undeclared
    /// name.
    pub fn work_type_description(&self, name: &str) -> Option<&str> {
        self.work_types
            .iter()
            .find(|w| w.name == name)
            .map(|w| w.description.as_str())
    }

    /// Structural invariants the file must satisfy. Called by [`contract`], so
    /// a malformed contract panics at first use rather than misclassifying
    /// runs later.
    pub fn validate(&self) -> Result<(), String> {
        if self.comment.trim().is_empty() {
            return Err("`$comment` must carry the file's header — it is what \
                        routes an editor to the rules in session_work_type.rs"
                .into());
        }
        if self.classifier_version < 1 {
            return Err(format!(
                "classifier_version must be >= 1, got {}",
                self.classifier_version
            ));
        }
        for w in &self.work_types {
            if w.description.trim().is_empty() {
                return Err(format!("work type `{}` has a blank description", w.name));
            }
        }
        let work_type_names: Vec<String> = self.work_types.iter().map(|w| w.name.clone()).collect();
        // Every vocabulary, checked identically: a blank entry is not a wire
        // value, and a repeated one makes the contract disagree with itself.
        for (label, values) in [
            ("work_types", &work_type_names),
            ("initiators", &self.initiators),
            ("end_reasons", &self.end_reasons),
            ("end_sources", &self.end_sources),
            ("bind_origins", &self.bind_origins),
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
        // The fallback must exist on both classified axes, or an
        // unclassifiable run has nowhere honest to go and would be folded
        // into a real type.
        if !self.is_work_type(UNKNOWN) {
            return Err("work_types must declare `unknown` — an unclassifiable run \
                        is counted as unknown, never folded into another type"
                .into());
        }
        if !self.is_initiator(UNKNOWN) {
            return Err("initiators must declare `unknown` — an unattributable run \
                        is counted as unknown, never folded into another initiator"
                .into());
        }
        Ok(())
    }
}

/// The parsed, validated contract — parsed once per process.
///
/// Panics if the embedded JSON is malformed or violates
/// [`validate`](SessionWorkTypeContract::validate); both are compile-time
/// constant faults this crate's tests catch.
pub fn contract() -> &'static SessionWorkTypeContract {
    static CONTRACT: std::sync::OnceLock<SessionWorkTypeContract> = std::sync::OnceLock::new();
    CONTRACT.get_or_init(|| {
        let c =
            SessionWorkTypeContract::load().expect("embedded session_work_type.json must parse");
        c.validate()
            .expect("embedded session_work_type.json must satisfy its invariants");
        c
    })
}

/// The shipped classifier version (the contract's `classifier_version`).
pub fn classifier_version() -> u32 {
    contract().classifier_version
}

/// Is `value` a declared `work_type` in the shipped contract?
pub fn is_work_type(value: &str) -> bool {
    contract().is_work_type(value)
}

/// Is `value` a declared `initiator` in the shipped contract?
pub fn is_initiator(value: &str) -> bool {
    contract().is_initiator(value)
}

/// Is `value` a declared `end_reason` in the shipped contract?
pub fn is_end_reason(value: &str) -> bool {
    contract().is_end_reason(value)
}

/// Is `value` a declared `end_source` in the shipped contract?
pub fn is_end_source(value: &str) -> bool {
    contract().is_end_source(value)
}

/// Is `value` a declared `bind_origin` in the shipped contract?
pub fn is_bind_origin(value: &str) -> bool {
    contract().is_bind_origin(value)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_embedded_contract_parses_and_validates() {
        let c = contract();
        assert_eq!(c.classifier_version, 1);
        assert_eq!(classifier_version(), 1);
        let names: Vec<&str> = c.work_types.iter().map(|w| w.name.as_str()).collect();
        assert_eq!(
            names,
            vec![
                "post_merge",
                "pr_fix_red_ci",
                "pr_fix_conflict",
                "admission_stall_fix",
                "red_main_fix",
                "merge_shepherd",
                "next_step",
                "unit_dispatch",
                "condition_autodispatch",
                "operator_page_remediation",
                "implement_plan",
                "implement_phase",
                "vet_imp",
                "vet_plan",
                "verify_plan_status",
                "gate_continuation_other",
                "steward",
                "onboarding_audit",
                "operator_interactive",
                "unknown",
            ],
            "the D2 work-type set, in declaration order"
        );
        assert_eq!(
            c.initiators,
            vec!["operator", "coord", "runner_local", "unknown"]
        );
        assert_eq!(
            c.end_reasons,
            vec![
                "exited",
                "poll_dead",
                "superseded",
                "operator_closed",
                "finished_close",
                "runner_stopped",
                "handoff_detached",
                "inferred_lost",
                "other",
            ]
        );
        assert_eq!(c.end_sources, vec!["observed", "inferred"]);
        assert_eq!(c.bind_origins, vec!["authoritative", "reconciled"]);
    }

    /// The accessors discriminate. The negative half is load-bearing: an
    /// `is_*` returning `true` unconditionally would pass every positive check.
    #[test]
    fn the_vocabularies_are_queryable() {
        let c = contract();
        for w in &c.work_types {
            assert!(is_work_type(&w.name));
            assert!(c.work_type_description(&w.name).is_some());
        }
        for v in &c.initiators {
            assert!(is_initiator(v));
        }
        for v in &c.end_reasons {
            assert!(is_end_reason(v));
        }
        for v in &c.end_sources {
            assert!(is_end_source(v));
        }
        for v in &c.bind_origins {
            assert!(is_bind_origin(v));
        }
        assert!(!is_work_type("not_a_work_type"));
        assert!(!is_initiator("not_an_initiator"));
        assert!(!is_end_reason("not_an_end_reason"));
        assert!(!is_end_source("not_an_end_source"));
        assert!(!is_bind_origin("not_a_bind_origin"));
        assert!(c.work_type_description("not_a_work_type").is_none());
        // Separate namespaces, not one pool.
        assert!(!is_work_type("coord"));
        assert!(!is_initiator("vet_imp"));
        assert!(!is_end_reason("observed"));
        assert!(!is_end_source("exited"));
        assert!(!is_bind_origin("operator"));
    }

    // ── Assert the assertions: each invariant must be able to FAIL ──────────
    //
    // Every rejection below feeds HAND-BUILT JSON through the same
    // `parse_and_validate` door, and `the_minimal_fixture_validates` proves
    // the fixture is otherwise valid — so each red is the injected fault.

    fn fixture(work_types: &str, initiators: &str) -> String {
        format!(
            r#"{{
                "$comment": "fixture",
                "classifier_version": 1,
                "work_types": {work_types},
                "initiators": {initiators},
                "end_reasons": ["exited", "other"],
                "end_sources": ["observed", "inferred"],
                "bind_origins": ["authoritative", "reconciled"]
            }}"#
        )
    }

    const WT_OK: &str = r#"[{"name": "vet_imp", "description": "d"},
                            {"name": "unknown", "description": "d"}]"#;
    const INIT_OK: &str = r#"["operator", "unknown"]"#;

    #[test]
    fn the_minimal_fixture_validates() {
        SessionWorkTypeContract::parse_and_validate(&fixture(WT_OK, INIT_OK))
            .expect("the base fixture must be valid, or every rejection is vacuous");
    }

    #[test]
    fn validate_rejects_a_blank_work_type_name() {
        let wt = r#"[{"name": "  ", "description": "d"}, {"name": "unknown", "description": "d"}]"#;
        let err = SessionWorkTypeContract::parse_and_validate(&fixture(wt, INIT_OK)).unwrap_err();
        assert!(err.contains("blank"), "{err}");
    }

    #[test]
    fn validate_rejects_a_blank_initiator() {
        let err =
            SessionWorkTypeContract::parse_and_validate(&fixture(WT_OK, r#"["", "unknown"]"#))
                .unwrap_err();
        assert!(err.contains("blank"), "{err}");
    }

    #[test]
    fn validate_rejects_a_duplicate_work_type() {
        let wt = r#"[{"name": "vet_imp", "description": "d"},
                     {"name": "vet_imp", "description": "e"},
                     {"name": "unknown", "description": "d"}]"#;
        let err = SessionWorkTypeContract::parse_and_validate(&fixture(wt, INIT_OK)).unwrap_err();
        assert!(err.contains("twice"), "{err}");
    }

    #[test]
    fn validate_rejects_a_duplicate_in_every_plain_vocabulary() {
        for field in ["initiators", "end_reasons", "end_sources", "bind_origins"] {
            let mut c = SessionWorkTypeContract::parse_and_validate(&fixture(WT_OK, INIT_OK))
                .expect("fixture valid");
            let list = match field {
                "initiators" => &mut c.initiators,
                "end_reasons" => &mut c.end_reasons,
                "end_sources" => &mut c.end_sources,
                _ => &mut c.bind_origins,
            };
            let first = list[0].clone();
            list.push(first);
            let err = c.validate().unwrap_err();
            assert!(
                err.contains(field) && err.contains("twice"),
                "{field}: {err}"
            );
        }
    }

    #[test]
    fn validate_rejects_a_missing_unknown_work_type() {
        let wt = r#"[{"name": "vet_imp", "description": "d"}]"#;
        let err = SessionWorkTypeContract::parse_and_validate(&fixture(wt, INIT_OK)).unwrap_err();
        assert!(err.contains("work_types must declare `unknown`"), "{err}");
    }

    #[test]
    fn validate_rejects_a_missing_unknown_initiator() {
        let err = SessionWorkTypeContract::parse_and_validate(&fixture(WT_OK, r#"["operator"]"#))
            .unwrap_err();
        assert!(err.contains("initiators must declare `unknown`"), "{err}");
    }

    #[test]
    fn validate_rejects_classifier_version_zero() {
        let json = fixture(WT_OK, INIT_OK)
            .replace("\"classifier_version\": 1", "\"classifier_version\": 0");
        let err = SessionWorkTypeContract::parse_and_validate(&json).unwrap_err();
        assert!(err.contains("classifier_version"), "{err}");
    }

    #[test]
    fn validate_rejects_a_blank_description() {
        let wt =
            r#"[{"name": "vet_imp", "description": " "}, {"name": "unknown", "description": "d"}]"#;
        let err = SessionWorkTypeContract::parse_and_validate(&fixture(wt, INIT_OK)).unwrap_err();
        assert!(err.contains("description"), "{err}");
    }

    #[test]
    fn validate_rejects_an_empty_vocabulary() {
        let json = fixture(WT_OK, INIT_OK).replace(
            r#""end_sources": ["observed", "inferred"]"#,
            r#""end_sources": []"#,
        );
        let err = SessionWorkTypeContract::parse_and_validate(&json).unwrap_err();
        assert!(err.contains("end_sources must not be empty"), "{err}");
    }

    #[test]
    fn an_unknown_field_in_the_contract_is_refused() {
        // A pure ADDITION, so the parse can only fail on deny_unknown_fields
        // rather than on a missing required field.
        let anchor = "\"name\": \"vet_imp\",";
        assert!(
            CONTRACT_JSON.contains(anchor),
            "anchor must exist to inject at"
        );
        let broken = CONTRACT_JSON.replacen(anchor, "\"label\": \"x\", \"name\": \"vet_imp\",", 1);
        assert_ne!(broken, CONTRACT_JSON);
        assert!(serde_json::from_str::<SessionWorkTypeContract>(&broken).is_err());
    }

    #[test]
    fn validate_accepts_the_shipped_contract() {
        SessionWorkTypeContract::parse_and_validate(CONTRACT_JSON)
            .expect("the shipped contract must validate");
    }
}
