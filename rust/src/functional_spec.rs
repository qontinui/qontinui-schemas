//! Functional Spec (v0) — the backend-agnostic contract between **comprehension**
//! (which writes the spec from an observed website) and **generation** (which reads
//! it to regenerate an app + backend).
//!
//! This is **Artifact 1** of the functional-spec-contract keystone
//! (`2026-06-13-functional-spec-contract.md`). It is frozen together with the
//! Completeness Rubric ([`crate::completeness_verdict`]) and the Priorities Profile
//! ([`crate::priorities_profile`]) so every downstream plan (app-gen, backend-gen,
//! comprehension) builds against one stable interface.
//!
//! ## The completeness ceiling this encodes
//!
//! **Completeness = coverage of frontend-*observable* functionality.** Every node
//! carries a [`SpecProvenance`] and a free-form `provenance` evidence string. What
//! the frontend never reveals (server-only validation, hidden business rules) is
//! recorded as an [`SpecProvenance::Assumed`] node in the [`FunctionalSpec::assumptions`]
//! ledger — never silently counted as "covered". The Completeness Rubric scores
//! `observed` + `inferred` coverage separately from `assumed` fill-in (see
//! [`crate::completeness_verdict`]).
//!
//! ## UI states / navigation reuse the typed IR
//!
//! [`FunctionalSpec::ui_states`] is `Vec<`[`crate::ir::IrState`]`>` and
//! [`FunctionalSpec::navigation`] is `Vec<`[`crate::ir::IrTransition`]`>` — the **same
//! types** `IrPageSpec` carries (`states` / `transitions`) and that Spec-Check
//! evaluates. This is a literal *type-level* superset of the UI Bridge IR, not a
//! parallel re-declaration: the app-generator's emitted IR re-parses to the identical
//! spec subset because a shared type cannot drift from itself. We deliberately do
//! **not** reuse `crate::state_machine`'s editor/DB-CRUD DTOs here — that is a parallel
//! representation and binding to it would reintroduce the dual-representation drift the
//! round-trip invariant exists to prevent.
//!
//! ## Wire-format conventions
//!
//! Follows the crate conventions ([`crate`] module docs): `camelCase` wire format,
//! ISO-8601 `String` timestamps, optional fields carry
//! `#[serde(default, skip_serializing_if = "Option::is_none")]`. `deny_unknown_fields`
//! is intentionally **omitted** — the v0 stability contract is additive-only, so a
//! reader must tolerate a forward (newer) document that adds optional fields.

use crate::ir::{IrState, IrTransition};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

// ===========================================================================
// Provenance axis (frozen — three values)
// ===========================================================================

/// How a spec node was established, relative to what the frontend *reveals*.
///
/// Named `SpecProvenance` (not `Confidence`) to stay distinct from the
/// match-confidence axis `crate::spec_check::Confidence`
/// (`high`/`medium`/`low`), which is orthogonal: that one scores how well an
/// observed element matched an assertion; this one scores how the spec node
/// itself was derived from observation.
///
/// The three buckets keep the coverage accounting split crisp: the rubric's
/// denominator is `Observed + Inferred`; `Assumed` is reported separately as
/// an assumption-fill rate and never folded into the headline coverage number.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, JsonSchema, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
pub enum SpecProvenance {
    /// Directly evidenced by the frontend — an input exists, a client-side
    /// validation fires, a route is gated.
    Observed,
    /// Reasonably deduced from multiple observations (a relationship, a role)
    /// but not directly stated. May carry an optional numeric `credibility`.
    Inferred,
    /// The frontend is silent; the generator supplies a best-practice default,
    /// recorded in the [`FunctionalSpec::assumptions`] ledger.
    Assumed,
}

// ===========================================================================
// Top-level spec
// ===========================================================================

/// The v0 Functional Spec. Five sections, each node confidence- and
/// provenance-tagged. Serializes as an A2A `DataPart` inside a worker's
/// `completion_reports.artifacts`; it is the durable, fully re-derivable
/// hand-off artifact between conductor ticks (orchestration handoff contract §6).
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct FunctionalSpec {
    /// Schema version. Currently always `"0"`. Additive-only until `"1"`.
    pub spec_version: String,

    /// What was observed and when.
    pub target: SpecTarget,

    /// 1. DOMAIN — entities + relationships inferred from rendered data shapes.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub entities: Vec<Entity>,

    /// 2. CAPABILITIES — operations the frontend exposes. Existence is
    /// high-confidence; server-side effect is low-confidence by construction.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub operations: Vec<Operation>,

    /// 3a. UI STATES — a literal superset of the UI Bridge IR. These are the
    /// exact `crate::ir::IrState` values `IrPageSpec.states` carries.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub ui_states: Vec<IrState>,

    /// 3b. NAVIGATION — the exact `crate::ir::IrTransition` values
    /// `IrPageSpec.transitions` carries. Reused (not re-declared) so app-gen's
    /// emitted IR re-parses to the identical spec subset.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub navigation: Vec<IrTransition>,

    /// 4. AUTH / PERMISSION MODEL — from login flows + gated routes.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub auth: Option<AuthModel>,

    /// 5. ASSUMPTIONS LEDGER — every node with provenance `Assumed`, collated so
    /// the operator can review/override the generator's best-practice fills. For
    /// v0 the override surface is a direct edit of this ledger (`overridable`).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub assumptions: Vec<AssumptionEntry>,
}

/// What was observed and when.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct SpecTarget {
    /// The source website URL the spec was synthesized from.
    pub source_url: String,
    /// ISO-8601 UTC timestamp; stamped post-run when the observation completed.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub observed_at: Option<String>,
}

// ===========================================================================
// 1. Domain — entities
// ===========================================================================

/// A domain entity inferred from rendered data shapes.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Entity {
    pub name: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub fields: Vec<EntityField>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub relationships: Vec<Relationship>,
    /// How this entity's existence was established.
    pub confidence: SpecProvenance,
    /// Free-form evidence string (e.g. `"detail view + list view both render it"`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub provenance: Option<String>,
    /// Optional deduction strength ∈ [0,1]. Present only on `Inferred` nodes;
    /// meaningless (and omitted) on `Observed` / `Assumed`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub credibility: Option<f64>,
}

/// A field of an [`Entity`].
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct EntityField {
    pub name: String,
    /// Coarse semantic type: `"string"`, `"money"`, `"enum"`, `"date"`,
    /// `"bool"`, `"number"`, `"reference"`, … Free-form so comprehension can
    /// introduce new types without a schema bump.
    #[serde(rename = "type")]
    pub field_type: String,
    /// Enumerated values when `field_type == "enum"`.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub values: Vec<String>,
    pub confidence: SpecProvenance,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub provenance: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub credibility: Option<f64>,
}

/// A relationship from one entity to another.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Relationship {
    /// Target entity name.
    pub to: String,
    /// `"one-to-one"`, `"one-to-many"`, `"many-to-one"`, `"many-to-many"`.
    pub kind: String,
    pub confidence: SpecProvenance,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub provenance: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub credibility: Option<f64>,
}

// ===========================================================================
// 2. Capabilities — operations
// ===========================================================================

/// An operation the frontend exposes. Existence (`confidence`) is typically
/// `Observed`; the server-side [`OperationEffect`] is `Assumed` by construction
/// (the frontend cannot reveal what persists server-side).
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Operation {
    pub name: String,
    /// `"create"`, `"read"`, `"update"`, `"delete"`, `"custom"`.
    pub verb: String,
    /// Target entity, when the operation acts on one.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub entity: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub inputs: Vec<OperationInput>,
    /// The (largely assumed) server-side effect.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub effect: Option<OperationEffect>,
    pub confidence: SpecProvenance,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub provenance: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub credibility: Option<f64>,
}

/// One input of an [`Operation`].
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct OperationInput {
    /// Field name (usually maps to an [`EntityField::name`]).
    pub field: String,
    #[serde(default)]
    pub required: bool,
    /// Client-side validation rule observed on the input, when any.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub validation: Option<ValidationRule>,
}

/// A client-side validation rule observed on an input.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ValidationRule {
    /// The rule expression as observed (e.g. `"> 0"`, `"email"`, `"maxLength 80"`).
    pub rule: String,
    pub confidence: SpecProvenance,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub provenance: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub credibility: Option<f64>,
}

/// The server-side effect of an [`Operation`]. `Assumed` by construction; the
/// `assumption` field records the best-practice default the generator applied.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct OperationEffect {
    pub confidence: SpecProvenance,
    /// The best-practice default applied when `confidence == Assumed`
    /// (e.g. `"persists + returns created row (REST 201 default)"`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub assumption: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub provenance: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub credibility: Option<f64>,
}

// ===========================================================================
// 4. Auth / permission model
// ===========================================================================

/// The auth / permission model inferred from login flows + gated routes.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct AuthModel {
    /// `"none"`, `"session"`, `"jwt"`, `"oauth"`, `"basic"`, …
    pub model: String,
    pub confidence: SpecProvenance,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub roles: Vec<AuthRole>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub provenance: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub credibility: Option<f64>,
}

/// A role surfaced by the auth model (e.g. inferred from nav items only some
/// sessions see).
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct AuthRole {
    pub name: String,
    pub confidence: SpecProvenance,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub provenance: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub credibility: Option<f64>,
}

// ===========================================================================
// 5. Assumptions ledger
// ===========================================================================

/// One collated assumption — a node whose provenance is `Assumed`. The operator
/// reviews/overrides these directly (v0 override surface = a file edit of this
/// ledger).
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct AssumptionEntry {
    /// Dotted ref into the spec the assumption belongs to, e.g.
    /// `"operations.createInvoice.effect"`.
    #[serde(rename = "ref")]
    pub r#ref: String,
    /// The best-practice default the generator applied.
    pub default_applied: String,
    /// Whether the operator may override this fill. Defaults to `true`.
    #[serde(default = "default_overridable")]
    pub overridable: bool,
    /// Optional free-form operator note (e.g. an override rationale).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub note: Option<String>,
}

fn default_overridable() -> bool {
    true
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn provenance_snake_case_round_trip() {
        for (wire, variant) in [
            ("\"observed\"", SpecProvenance::Observed),
            ("\"inferred\"", SpecProvenance::Inferred),
            ("\"assumed\"", SpecProvenance::Assumed),
        ] {
            let parsed: SpecProvenance = serde_json::from_str(wire).unwrap();
            assert_eq!(parsed, variant);
            assert_eq!(serde_json::to_string(&parsed).unwrap(), wire);
        }
    }

    #[test]
    fn assumption_overridable_defaults_true_when_absent() {
        let json =
            r#"{"ref":"operations.createInvoice.effect","defaultApplied":"REST 201 persist"}"#;
        let parsed: AssumptionEntry = serde_json::from_str(json).unwrap();
        assert!(parsed.overridable);
        assert_eq!(parsed.r#ref, "operations.createInvoice.effect");
    }

    #[test]
    fn entity_field_renames_type_keyword() {
        let f = EntityField {
            name: "amount".into(),
            field_type: "money".into(),
            values: vec![],
            confidence: SpecProvenance::Observed,
            provenance: Some("form#invoice input[name=amount]".into()),
            credibility: None,
        };
        let v = serde_json::to_value(&f).unwrap();
        assert_eq!(v["type"], "money");
        assert!(
            v.get("credibility").is_none(),
            "absent credibility must not serialize"
        );
    }

    #[test]
    fn spec_reuses_ir_state_and_transition_types() {
        // Compile-time proof the section types ARE the IR types, not parallel
        // re-declarations. If the IR types change shape, this stops compiling.
        let _ui: Vec<IrState> = Vec::new();
        let _nav: Vec<IrTransition> = Vec::new();
        let spec = FunctionalSpec {
            spec_version: "0".into(),
            target: SpecTarget {
                source_url: "https://example.test".into(),
                observed_at: None,
            },
            entities: vec![],
            operations: vec![],
            ui_states: _ui,
            navigation: _nav,
            auth: None,
            assumptions: vec![],
        };
        let round: FunctionalSpec =
            serde_json::from_str(&serde_json::to_string(&spec).unwrap()).unwrap();
        assert_eq!(round.spec_version, "0");
    }
}
