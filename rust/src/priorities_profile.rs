//! Priorities Profile (v0) — **Artifact 3** of the functional-spec-contract keystone
//! (`2026-06-13-functional-spec-contract.md`).
//!
//! Declarative, versionable, gate-checkable. Parameterizes the generators (backend-gen
//! + app-gen's data layer) and the Completeness Rubric **without re-observing the site**.
//! It shares vocabulary with the [`crate::functional_spec`]: a spec `Entity` maps to a
//! profile-named table; a spec `Operation` maps to a profile-named endpoint. Frozen
//! alongside the spec so the two cannot drift on naming/identity.
//!
//! The [`Profile::enforcement`] section wires the existing claude-config skills (e.g.
//! `code-reviewer`, `security-scan`) into the generation loop, so "best practices" is
//! *enforced by the loop*, not asserted in prose.
//!
//! Wire format: `camelCase`; optional fields skip-serialize when absent;
//! `deny_unknown_fields` omitted for additive-only v0 evolution (see
//! [`crate::functional_spec`] for the rationale).

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// The top-level Priorities Profile.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct Profile {
    /// Schema version. Currently always `"0"`. Additive-only until `"1"`.
    pub profile_version: String,

    /// Target backend stack.
    pub backend: BackendProfile,

    /// Architectural shape + testing bar.
    pub architecture: ArchitectureProfile,

    /// Naming / mapping conventions, as free-form `key -> value` pairs (e.g.
    /// `"naming" -> "snake_case"`, `"entityToTable" -> "pluralize"`). A `BTreeMap`
    /// so JSON serialization is deterministic. Open-ended by design so a profile
    /// can express conventions the schema didn't anticipate.
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub conventions: BTreeMap<String, String>,

    /// How the generation loop enforces quality.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub enforcement: Option<EnforcementProfile>,
}

/// Target backend stack.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct BackendProfile {
    /// e.g. `"python"`, `"typescript"`, `"rust"`, `"go"`.
    pub language: String,
    /// e.g. `"fastapi"`, `"express"`, `"axum"`.
    pub framework: String,
    /// e.g. `"postgres"`, `"sqlite"`, `"mongodb"`.
    pub datastore: String,
    /// REST vs GraphQL.
    pub api_style: ApiStyle,
    /// Backend auth strategy (e.g. `"jwt"`, `"session"`). Free-form; `None` lets
    /// the generator default from the spec's observed [`crate::functional_spec::AuthModel`].
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub auth: Option<String>,
}

/// API style the generated backend exposes.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, JsonSchema, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
pub enum ApiStyle {
    Rest,
    Graphql,
}

/// Architectural shape + testing bar the generators must hold to.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ArchitectureProfile {
    /// e.g. `"layered"`, `"hexagonal"`, `"mvc"`.
    pub pattern: String,
    /// e.g. `"unit"`, `"unit+integration"`, `"unit+integration+e2e"`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub testing_bar: Option<String>,
    /// Minimum line/branch coverage percentage the generated tests must reach.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub min_coverage_pct: Option<u32>,
}

/// How the generation loop enforces quality — names existing claude-config skills
/// to run and which of them are hard blockers.
#[derive(Debug, Clone, Default, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct EnforcementProfile {
    /// Skills to run on generated code (e.g. `["code-reviewer", "security-scan"]`).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub run: Vec<String>,
    /// Which categories block the loop on failure (e.g. `["security"]`). A skill
    /// in `run` but not `block_on` is advisory.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub block_on: Vec<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn api_style_snake_case_round_trip() {
        for (wire, variant) in [
            ("\"rest\"", ApiStyle::Rest),
            ("\"graphql\"", ApiStyle::Graphql),
        ] {
            let parsed: ApiStyle = serde_json::from_str(wire).unwrap();
            assert_eq!(parsed, variant);
            assert_eq!(serde_json::to_string(&parsed).unwrap(), wire);
        }
    }

    #[test]
    fn profile_round_trips_and_orders_conventions() {
        let mut conventions = BTreeMap::new();
        conventions.insert("naming".to_string(), "snake_case".to_string());
        conventions.insert("entityToTable".to_string(), "pluralize".to_string());
        let p = Profile {
            profile_version: "0".into(),
            backend: BackendProfile {
                language: "python".into(),
                framework: "fastapi".into(),
                datastore: "postgres".into(),
                api_style: ApiStyle::Rest,
                auth: None,
            },
            architecture: ArchitectureProfile {
                pattern: "layered".into(),
                testing_bar: Some("unit+integration".into()),
                min_coverage_pct: Some(80),
            },
            conventions,
            enforcement: Some(EnforcementProfile {
                run: vec!["code-reviewer".into(), "security-scan".into()],
                block_on: vec!["security".into()],
            }),
        };
        let json = serde_json::to_string(&p).unwrap();
        // BTreeMap orders keys: "entityToTable" < "naming".
        let etbl = json.find("entityToTable").unwrap();
        let naming = json.find("naming").unwrap();
        assert!(
            etbl < naming,
            "conventions must serialize in deterministic key order"
        );
        let round: Profile = serde_json::from_str(&json).unwrap();
        assert_eq!(round, p);
    }

    #[test]
    fn empty_optional_sections_omitted() {
        let p = Profile {
            profile_version: "0".into(),
            backend: BackendProfile {
                language: "rust".into(),
                framework: "axum".into(),
                datastore: "sqlite".into(),
                api_style: ApiStyle::Rest,
                auth: None,
            },
            architecture: ArchitectureProfile {
                pattern: "hexagonal".into(),
                testing_bar: None,
                min_coverage_pct: None,
            },
            conventions: BTreeMap::new(),
            enforcement: None,
        };
        let v = serde_json::to_value(&p).unwrap();
        assert!(v.get("conventions").is_none());
        assert!(v.get("enforcement").is_none());
        assert!(v["backend"].get("auth").is_none());
    }
}
