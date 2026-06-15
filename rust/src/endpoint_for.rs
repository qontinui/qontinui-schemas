//! Shared operation→endpoint derivation for the website→mobile regeneration program.
//!
//! The program graph keeps the **app generator (#1)** and **backend generator (#2)**
//! independent (both depend only on the frozen contract, no data edge between them).
//! Yet the generated app's data layer must call the exact routes the generated backend
//! serves. The cross-plan reconciliation resolves this with a **single deterministic
//! derivation** — implemented once, here, in the shared contract crate, and called by
//! *both* generators — so they agree without a runtime handoff and cannot drift.
//!
//! ## The rule (v0)
//!
//! `endpoint_for(op, profile)` maps a [`crate::functional_spec::Operation`] to its
//! `{ method, path }`:
//!
//! - **method** from the operation `verb`: `create→POST`, `read→GET`, `update→PUT`,
//!   `delete→DELETE`, anything else (`custom`/unknown) `→POST`.
//! - **path**: `base = /api/v1/{table(entity)}` when the operation targets an entity
//!   (`table` = the entity name under the profile's `entityToTable` convention —
//!   `pluralize` by default — plus the `naming` convention, `snake_case` by default),
//!   else `base = /api/v1`.
//!   - A **bare-CRUD** operation (its `name` is just the verb, or `{verb}{Entity}`,
//!     case-insensitively) addresses the collection: `path = base`.
//!   - A **named custom action** (e.g. `pairConfirm`) addresses a sub-resource:
//!     `path = base/{kebab(name)}` (e.g. `/api/v1/devices/pair-confirm`).
//!
//! This reproduces the real endpoints comprehension observes — e.g. the connect-runner
//! fixture's `pairConfirm` (verb `create`, entity `Device`) derives
//! `POST /api/v1/devices/pair-confirm`, matching the page's observed
//! `POST /api/v1/devices/pair-confirm`.
//!
//! NOTE (v0 scope): the reconciliation reserves an operation's *observed* endpoint
//! (recorded by comprehension) as an override of the derived path. The frozen v0
//! `Operation` carries only a free-form `provenance` string, not a structured endpoint
//! field, so v0 derives deterministically and the structured-override hook lands when
//! the contract gains an `observed_endpoint` field (a future additive bump). Until then
//! the deterministic rule is authoritative; comprehension is expected to name custom
//! operations so the derivation matches the observed route.

use crate::functional_spec::Operation;
use crate::priorities_profile::Profile;

/// A derived HTTP endpoint: the method + path a generated backend serves and a
/// generated app calls. Not a wire type — an internal derivation result shared by the
/// two generators (no `JsonSchema`, so it adds no codegen surface).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Endpoint {
    pub method: String,
    pub path: String,
}

/// Convention keys read from [`Profile::conventions`].
const CONV_NAMING: &str = "naming";
const CONV_ENTITY_TO_TABLE: &str = "entityToTable";

/// Derive the `{ method, path }` for an operation under a profile. The single shared
/// rule both #1 and #2 call — see the module docs for the full specification.
pub fn endpoint_for(op: &Operation, profile: &Profile) -> Endpoint {
    Endpoint {
        method: method_for_verb(&op.verb),
        path: path_for(op, profile),
    }
}

fn method_for_verb(verb: &str) -> String {
    match verb.to_ascii_lowercase().as_str() {
        "create" => "POST",
        "read" | "list" | "get" => "GET",
        "update" => "PUT",
        "delete" => "DELETE",
        _ => "POST", // custom / unknown
    }
    .to_string()
}

fn path_for(op: &Operation, profile: &Profile) -> String {
    let base = match op.entity.as_deref() {
        Some(entity) => format!("/api/v1/{}", table_name(entity, profile)),
        None => "/api/v1".to_string(),
    };
    if is_bare_crud(op) {
        base
    } else {
        format!("{base}/{}", to_kebab(&op.name))
    }
}

/// A bare-CRUD op is one whose name is just the verb (`create`) or `{verb}{Entity}`
/// (`createInvoice`) — it addresses the collection, not a named sub-action.
fn is_bare_crud(op: &Operation) -> bool {
    let name = op.name.to_ascii_lowercase();
    let verb = op.verb.to_ascii_lowercase();
    if name == verb {
        return true;
    }
    if let Some(entity) = op.entity.as_deref() {
        if name == format!("{verb}{}", entity.to_ascii_lowercase()) {
            return true;
        }
    }
    false
}

/// Table/collection segment for an entity under the profile conventions.
/// `entityToTable=pluralize` (default) + `naming=snake_case` (default).
fn table_name(entity: &str, profile: &Profile) -> String {
    let snake = match profile.conventions.get(CONV_NAMING).map(String::as_str) {
        Some("snake_case") | None => to_snake(entity),
        Some(_) => to_snake(entity), // v0: snake is the only supported path segment casing
    };
    match profile
        .conventions
        .get(CONV_ENTITY_TO_TABLE)
        .map(String::as_str)
    {
        Some("as-is") | Some("singular") => snake,
        Some("pluralize") | None | Some(_) => pluralize(&snake),
    }
}

/// `PascalCase`/`camelCase` → `snake_case`.
fn to_snake(s: &str) -> String {
    let mut out = String::with_capacity(s.len() + 4);
    for (i, ch) in s.chars().enumerate() {
        if ch == '-' || ch == '_' || ch == ' ' {
            if !out.ends_with('_') {
                out.push('_');
            }
        } else if ch.is_ascii_uppercase() {
            if i != 0 && !out.ends_with('_') {
                out.push('_');
            }
            out.push(ch.to_ascii_lowercase());
        } else {
            out.push(ch);
        }
    }
    out
}

/// `PascalCase`/`camelCase` → `kebab-case`.
fn to_kebab(s: &str) -> String {
    to_snake(s).replace('_', "-")
}

/// Naive English pluralization, sufficient for v0 table names.
fn pluralize(snake: &str) -> String {
    if snake.is_empty() {
        return snake.to_string();
    }
    if snake.ends_with('s')
        || snake.ends_with('x')
        || snake.ends_with('z')
        || snake.ends_with("ch")
        || snake.ends_with("sh")
    {
        return format!("{snake}es");
    }
    if let Some(stem) = snake.strip_suffix('y') {
        if !stem.ends_with(['a', 'e', 'i', 'o', 'u']) {
            return format!("{stem}ies");
        }
    }
    format!("{snake}s")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::functional_spec::{Operation, SpecProvenance};
    use crate::priorities_profile::{ApiStyle, ArchitectureProfile, BackendProfile, Profile};
    use std::collections::BTreeMap;

    fn profile() -> Profile {
        let mut conventions = BTreeMap::new();
        conventions.insert("naming".into(), "snake_case".into());
        conventions.insert("entityToTable".into(), "pluralize".into());
        Profile {
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
                testing_bar: None,
                min_coverage_pct: None,
            },
            conventions,
            enforcement: None,
        }
    }

    fn op(name: &str, verb: &str, entity: Option<&str>) -> Operation {
        Operation {
            name: name.into(),
            verb: verb.into(),
            entity: entity.map(String::from),
            inputs: vec![],
            effect: None,
            confidence: SpecProvenance::Observed,
            provenance: None,
            credibility: None,
        }
    }

    #[test]
    fn named_custom_action_gets_kebab_subpath() {
        // The connect-runner fixture's real observed endpoint.
        let e = endpoint_for(&op("pairConfirm", "create", Some("Device")), &profile());
        assert_eq!(e.method, "POST");
        assert_eq!(e.path, "/api/v1/devices/pair-confirm");
    }

    #[test]
    fn bare_crud_addresses_the_collection() {
        // `createInvoice` == create + Invoice → bare CRUD → collection.
        let e = endpoint_for(&op("createInvoice", "create", Some("Invoice")), &profile());
        assert_eq!(e.method, "POST");
        assert_eq!(e.path, "/api/v1/invoices");
    }

    #[test]
    fn verb_maps_to_method() {
        assert_eq!(
            endpoint_for(&op("read", "read", Some("Invoice")), &profile()).method,
            "GET"
        );
        assert_eq!(
            endpoint_for(&op("update", "update", Some("Invoice")), &profile()).method,
            "PUT"
        );
        assert_eq!(
            endpoint_for(&op("delete", "delete", Some("Invoice")), &profile()).method,
            "DELETE"
        );
    }

    #[test]
    fn pluralization_and_casing() {
        assert_eq!(pluralize("device"), "devices");
        assert_eq!(pluralize("invoice"), "invoices");
        assert_eq!(pluralize("category"), "categories");
        assert_eq!(pluralize("address"), "addresses");
        assert_eq!(to_snake("LineItem"), "line_item");
        assert_eq!(to_kebab("pairConfirm"), "pair-confirm");
    }

    #[test]
    fn multiword_entity_pluralizes_last_segment_style() {
        let e = endpoint_for(
            &op("createLineItem", "create", Some("LineItem")),
            &profile(),
        );
        assert_eq!(e.path, "/api/v1/line_items");
    }

    #[test]
    fn entity_to_table_as_is_skips_pluralization() {
        let mut p = profile();
        p.conventions.insert("entityToTable".into(), "as-is".into());
        let e = endpoint_for(&op("createDevice", "create", Some("Device")), &p);
        assert_eq!(e.path, "/api/v1/device");
    }

    #[test]
    fn custom_op_without_entity_falls_back_to_api_v1() {
        let e = endpoint_for(&op("healthCheck", "custom", None), &profile());
        assert_eq!(e.path, "/api/v1/health-check");
        assert_eq!(e.method, "POST");
    }
}
