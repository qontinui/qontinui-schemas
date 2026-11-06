//! The `[model]` manifest declaration: which models a repo's steps may call
//! through the dispatch's model broker, and the per-dispatch budget they share
//! (plan `2026-10-13-ci-node-runs-model-dependent-eval-steps`, D3).
//!
//! ```toml
//! [model]
//! models = ["claude-sonnet"]      # closed registry names, or a pinned id under a registered family
//! max_calls = 400                 # per dispatch
//! max_tokens = 4000000            # per dispatch: input + output + cache tokens, as the provider reports them
//! max_wall_secs = 3600            # per dispatch, counted from the first call
//!
//! [[steps]]
//! name = "b2"
//! command = ["poetry", "run", "docs-eval", "run", "--variant", "b2", "--cases", "all"]
//! model = true                    # only these steps get QONTINUI_CI_MODEL_*
//! ```
//!
//! # Why a closed vocabulary
//!
//! A free model string would select both what runs and what it costs, from a
//! committed file. So a name is either a registered FAMILY ([`MODEL_REGISTRY`],
//! mapped to the CLI's `--model` alias) or a PINNED concrete id under a
//! registered family's prefix — the pinned form exists so a baseline and the
//! engine it is compared against can be held to one concrete model.
//!
//! # Why budgets are required and refused, never clamped
//!
//! The same rule as [`crate::manifest::MAX_STEP_TIMEOUT_SECS`]: a value above
//! a ceiling is a validation error, not a clamp, so an author sees the limit
//! instead of silently losing budget. There are no defaults: a budget nobody
//! wrote down is a budget nobody reasoned about.
//!
//! # Land order: executor first, manifests after
//!
//! `deny_unknown_fields` on the manifest means a `[model]` table, or a step's
//! `model = true`, is a hard parse error on any executor built before this
//! module. Roll the executor out first, exactly as `[canonical]`, `[[tools]]`
//! and `[[siblings]]` each landed.

use serde::Deserialize;

use crate::manifest::CiJob;

/// Hard cap on `max_calls`. A larger value is refused, never clamped.
pub const MAX_MODEL_CALLS: u64 = 2_000;
/// Hard cap on `max_tokens` (input + output + cache tokens). Refused above.
pub const MAX_MODEL_TOKENS: u64 = 20_000_000;
/// Hard cap on `max_wall_secs`. Refused above.
pub const MAX_MODEL_WALL_SECS: u64 = 7_200;

/// Upper bound on the number of names in `models`. A legibility bound: a
/// manifest comparing more models than this in one dispatch has a layout
/// problem.
const MAX_DECLARED_MODELS: usize = 8;

/// The longest model name the grammar admits.
const MAX_MODEL_NAME_LEN: usize = 64;

/// The pointer every model-related refusal carries, so an author who tried to
/// reach a model some other way is told the one supported way.
pub const MODEL_POINTER: &str = "declare [model] and set `model = true` on the step";

/// One registered model family.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ModelFamily {
    /// The manifest spelling (`claude-sonnet`).
    pub name: &'static str,
    /// The CLI's `--model` alias the family maps to (`sonnet`).
    pub cli_alias: &'static str,
}

impl ModelFamily {
    /// The prefix a pinned concrete id under this family starts with.
    fn pinned_prefix(&self) -> String {
        format!("{}-", self.name)
    }
}

/// The closed registry of model families a manifest may name.
///
/// THIS IS A COST AND CAPABILITY BOUNDARY. Adding a family widens what a
/// committed file can make a CI node spend; it is a reviewed code change, never
/// a manifest value.
pub const MODEL_REGISTRY: &[ModelFamily] = &[
    ModelFamily {
        name: "claude-opus",
        cli_alias: "opus",
    },
    ModelFamily {
        name: "claude-sonnet",
        cli_alias: "sonnet",
    },
    ModelFamily {
        name: "claude-haiku",
        cli_alias: "haiku",
    },
];

/// A declared name, resolved against [`MODEL_REGISTRY`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResolvedModel {
    /// Exactly as the manifest (and the step's request) spells it.
    pub name: String,
    /// The family it belongs to.
    pub family: &'static ModelFamily,
    /// What the host's invoker passes as the CLI's `--model`: the family
    /// alias for a family name, the pinned id itself for a pinned id.
    pub cli_model: String,
    /// Whether the name is a pinned concrete id rather than a family.
    pub pinned: bool,
}

/// Resolve one name: a registered family, or a pinned id under one.
/// `Err` says why the name is not admissible.
pub fn resolve_model(name: &str) -> Result<ResolvedModel, String> {
    if name.is_empty()
        || name.len() > MAX_MODEL_NAME_LEN
        || !name
            .bytes()
            .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'.' || b == b'-')
    {
        return Err(format!(
            "model {name:?} is not a model name (grammar [a-z0-9.-]{{1,{MAX_MODEL_NAME_LEN}}})"
        ));
    }
    if let Some(family) = MODEL_REGISTRY.iter().find(|f| f.name == name) {
        return Ok(ResolvedModel {
            name: name.to_string(),
            family,
            cli_model: family.cli_alias.to_string(),
            pinned: false,
        });
    }
    if let Some(family) = MODEL_REGISTRY.iter().find(|f| {
        let prefix = f.pinned_prefix();
        name.starts_with(&prefix) && name.len() > prefix.len()
    }) {
        return Ok(ResolvedModel {
            name: name.to_string(),
            family,
            cli_model: name.to_string(),
            pinned: true,
        });
    }
    let families: Vec<&str> = MODEL_REGISTRY.iter().map(|f| f.name).collect();
    Err(format!(
        "model {name:?} is not registered — name a family {families:?} or a pinned id under one \
         (e.g. \"claude-sonnet-4-5-20250929\")"
    ))
}

/// `[model]` exactly as written. Every budget is an `Option` so a missing one
/// is refused with a message naming it, rather than a bare serde error.
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct RawModelDeclaration {
    models: Vec<String>,
    #[serde(default)]
    max_calls: Option<u64>,
    #[serde(default)]
    max_tokens: Option<u64>,
    #[serde(default)]
    max_wall_secs: Option<u64>,
}

/// The validated `[model]` declaration.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ModelDeclaration {
    /// Never empty, no duplicates; the first is the shim's default model.
    pub models: Vec<ResolvedModel>,
    /// Per dispatch. A hard limit.
    pub max_calls: u64,
    /// Per dispatch: input + output + cache tokens as the provider reports
    /// them. A call starts only while the tokens used are below this.
    pub max_tokens: u64,
    /// Per dispatch, counted from the first call.
    pub max_wall_secs: u64,
}

impl ModelDeclaration {
    /// The declared model a request names, if it is declared.
    pub fn declared(&self, name: &str) -> Option<&ResolvedModel> {
        self.models.iter().find(|m| m.name == name)
    }

    /// The first declared model: what a request that names none gets.
    pub fn default_model(&self) -> &ResolvedModel {
        &self.models[0]
    }
}

/// One budget: present, non-zero and at most its ceiling.
fn budget(field: &str, value: Option<u64>, ceiling: u64) -> Result<u64, String> {
    match value {
        None => Err(format!(
            "[model]: {field} is required (1..={ceiling}) — there is no default budget"
        )),
        Some(0) => Err(format!(
            "[model]: {field} = 0 would allow nothing (1..={ceiling})"
        )),
        Some(v) if v > ceiling => Err(format!(
            "[model]: {field} = {v} exceeds the ceiling {ceiling} — a larger value is refused, \
             not clamped"
        )),
        Some(v) => Ok(v),
    }
}

/// Validate `[model]` against the jobs it serves. Called from
/// [`crate::manifest::parse_and_validate`] once the jobs are built.
///
/// Refuses: `models = []`; an unregistered name; a pinned id outside a
/// registered family; a duplicate; a missing, zero or over-ceiling budget; a
/// `model = true` step with no `[model]`; and a `[model]` that no step uses —
/// a budget nobody can spend is a declaration that misleads.
pub(crate) fn validate_model(
    raw: Option<RawModelDeclaration>,
    jobs: &[CiJob],
) -> Result<Option<ModelDeclaration>, String> {
    let model_steps: Vec<String> = jobs
        .iter()
        .flat_map(|j| {
            j.steps
                .iter()
                .filter(|s| s.model)
                .map(move |s| format!("{}/{}", j.name, s.name))
        })
        .collect();
    let Some(raw) = raw else {
        if let Some(first) = model_steps.first() {
            return Err(format!(
                "step {first:?} sets `model = true` but the manifest has no [model] table — \
                 {MODEL_POINTER}"
            ));
        }
        return Ok(None);
    };
    if raw.models.is_empty() {
        return Err("[model]: models = [] names no model".to_string());
    }
    if raw.models.len() > MAX_DECLARED_MODELS {
        return Err(format!(
            "[model]: declares {} models (max {MAX_DECLARED_MODELS})",
            raw.models.len()
        ));
    }
    let mut models: Vec<ResolvedModel> = Vec::with_capacity(raw.models.len());
    for name in &raw.models {
        let resolved = resolve_model(name).map_err(|e| format!("[model]: {e}"))?;
        if models.iter().any(|m| m.name == resolved.name) {
            return Err(format!("[model]: model {name:?} is listed twice"));
        }
        models.push(resolved);
    }
    let max_calls = budget("max_calls", raw.max_calls, MAX_MODEL_CALLS)?;
    let max_tokens = budget("max_tokens", raw.max_tokens, MAX_MODEL_TOKENS)?;
    let max_wall_secs = budget("max_wall_secs", raw.max_wall_secs, MAX_MODEL_WALL_SECS)?;
    if model_steps.is_empty() {
        return Err(
            "[model] is declared but no step sets `model = true` — a budget no step can spend \
             is refused; set `model = true` on the steps that call a model, or remove [model]"
                .to_string(),
        );
    }
    Ok(Some(ModelDeclaration {
        models,
        max_calls,
        max_tokens,
        max_wall_secs,
    }))
}

#[cfg(test)]
mod tests {
    use crate::manifest::parse_and_validate;

    use super::*;

    const STEP: &str = "[[steps]]\nname = \"b2\"\ncommand = [\"true\"]\nmodel = true\n";

    fn manifest(model_table: &str, steps: &str) -> String {
        format!("version = 1\n{model_table}\n{steps}")
    }

    const GOOD_TABLE: &str = "[model]\nmodels = [\"claude-sonnet\"]\nmax_calls = 400\n\
                              max_tokens = 4000000\nmax_wall_secs = 3600\n";

    #[test]
    fn a_full_declaration_parses_and_resolves() {
        let m = parse_and_validate(&manifest(
            "[model]\nmodels = [\"claude-sonnet\", \"claude-opus-4-1-20250805\"]\nmax_calls = 400\n\
             max_tokens = 4000000\nmax_wall_secs = 3600\n",
            &format!("{STEP}[[steps]]\nname = \"plain\"\ncommand = [\"true\"]\n"),
        ))
        .unwrap();
        let decl = m.model.expect("declared");
        assert_eq!(decl.max_calls, 400);
        assert_eq!(decl.max_tokens, 4_000_000);
        assert_eq!(decl.max_wall_secs, 3600);
        assert_eq!(decl.default_model().cli_model, "sonnet");
        let pinned = decl.declared("claude-opus-4-1-20250805").unwrap();
        assert!(pinned.pinned);
        assert_eq!(pinned.family.name, "claude-opus");
        assert_eq!(pinned.cli_model, "claude-opus-4-1-20250805");
        assert!(m.jobs[0].steps[0].model);
        assert!(!m.jobs[0].steps[1].model, "model defaults to false");
    }

    #[test]
    fn the_registry_maps_families_to_cli_aliases() {
        for (name, alias) in [
            ("claude-opus", "opus"),
            ("claude-sonnet", "sonnet"),
            ("claude-haiku", "haiku"),
        ] {
            assert_eq!(resolve_model(name).unwrap().cli_model, alias);
        }
    }

    #[test]
    fn a_v2_job_steps_may_declare_model() {
        let m = parse_and_validate(&format!(
            "version = 2\n{GOOD_TABLE}\n[[jobs]]\nname = \"evals\"\n[[jobs.steps]]\nname = \"b2\"\n\
             command = [\"true\"]\nmodel = true\n"
        ))
        .unwrap();
        assert!(m.model.is_some());
        assert!(m.jobs[0].steps[0].model);
    }

    #[test]
    fn a_job_level_model_table_is_refused() {
        let err = parse_and_validate(
            "version = 2\n[[jobs]]\nname = \"evals\"\n[jobs.model]\nmodels = [\"claude-sonnet\"]\n\
             max_calls = 1\nmax_tokens = 1\nmax_wall_secs = 1\n[[jobs.steps]]\nname = \"b2\"\n\
             command = [\"true\"]\n",
        )
        .unwrap_err();
        assert!(err.contains("unknown field"), "{err}");
    }

    #[test]
    fn an_unknown_key_in_model_is_refused() {
        let err = parse_and_validate(&manifest(&format!("{GOOD_TABLE}max_cost_usd = 5\n"), STEP))
            .unwrap_err();
        assert!(err.contains("unknown field"), "{err}");
    }

    #[test]
    fn an_empty_models_list_is_refused() {
        let err = parse_and_validate(&manifest(
            "[model]\nmodels = []\nmax_calls = 1\nmax_tokens = 1\nmax_wall_secs = 1\n",
            STEP,
        ))
        .unwrap_err();
        assert!(err.contains("names no model"), "{err}");
    }

    #[test]
    fn an_unregistered_name_is_refused() {
        for name in ["gpt-4o", "opus", "Claude-Sonnet", "claude-sonnet ", ""] {
            let err = parse_and_validate(&manifest(
                &format!(
                    "[model]\nmodels = [{name:?}]\nmax_calls = 1\nmax_tokens = 1\nmax_wall_secs = 1\n"
                ),
                STEP,
            ))
            .unwrap_err();
            assert!(
                err.contains("not registered") || err.contains("not a model name"),
                "{name:?}: {err}"
            );
        }
    }

    #[test]
    fn a_pinned_id_outside_a_registered_family_is_refused() {
        for name in [
            "claude-3-5-sonnet-20241022",
            "claude-sonnetx-4",
            "claude-sonnet-",
            "claude-mythos-1",
        ] {
            let err = parse_and_validate(&manifest(
                &format!(
                    "[model]\nmodels = [{name:?}]\nmax_calls = 1\nmax_tokens = 1\nmax_wall_secs = 1\n"
                ),
                STEP,
            ))
            .unwrap_err();
            assert!(err.contains("not registered"), "{name}: {err}");
        }
        let too_long = format!("claude-sonnet-{}", "9".repeat(60));
        assert!(resolve_model(&too_long)
            .unwrap_err()
            .contains("not a model name"));
    }

    #[test]
    fn a_duplicate_name_is_refused() {
        let err = parse_and_validate(&manifest(
            "[model]\nmodels = [\"claude-haiku\", \"claude-haiku\"]\nmax_calls = 1\nmax_tokens = 1\n\
             max_wall_secs = 1\n",
            STEP,
        ))
        .unwrap_err();
        assert!(err.contains("listed twice"), "{err}");
    }

    #[test]
    fn zero_and_over_ceiling_budgets_are_refused() {
        for (field, ceiling) in [
            ("max_calls", MAX_MODEL_CALLS),
            ("max_tokens", MAX_MODEL_TOKENS),
            ("max_wall_secs", MAX_MODEL_WALL_SECS),
        ] {
            for (value, expect) in [
                (0, "would allow nothing"),
                (ceiling + 1, "exceeds the ceiling"),
            ] {
                let mut fields = vec![("max_calls", 1), ("max_tokens", 1), ("max_wall_secs", 1)];
                for f in &mut fields {
                    if f.0 == field {
                        f.1 = value;
                    }
                }
                let table = format!(
                    "[model]\nmodels = [\"claude-sonnet\"]\n{}\n",
                    fields
                        .iter()
                        .map(|(k, v)| format!("{k} = {v}"))
                        .collect::<Vec<_>>()
                        .join("\n")
                );
                let err = parse_and_validate(&manifest(&table, STEP)).unwrap_err();
                assert!(
                    err.contains(field) && err.contains(expect),
                    "{field}={value}: {err}"
                );
            }
            // The ceiling itself is admitted.
            let mut fields = vec![("max_calls", 1), ("max_tokens", 1), ("max_wall_secs", 1)];
            for f in &mut fields {
                if f.0 == field {
                    f.1 = ceiling;
                }
            }
            let table = format!(
                "[model]\nmodels = [\"claude-sonnet\"]\n{}\n",
                fields
                    .iter()
                    .map(|(k, v)| format!("{k} = {v}"))
                    .collect::<Vec<_>>()
                    .join("\n")
            );
            parse_and_validate(&manifest(&table, STEP))
                .unwrap_or_else(|e| panic!("{field} at the ceiling must parse: {e}"));
        }
    }

    #[test]
    fn a_missing_budget_is_refused() {
        for missing in ["max_calls", "max_tokens", "max_wall_secs"] {
            let table = format!(
                "[model]\nmodels = [\"claude-sonnet\"]\n{}\n",
                ["max_calls", "max_tokens", "max_wall_secs"]
                    .iter()
                    .filter(|k| **k != missing)
                    .map(|k| format!("{k} = 1"))
                    .collect::<Vec<_>>()
                    .join("\n")
            );
            let err = parse_and_validate(&manifest(&table, STEP)).unwrap_err();
            assert!(
                err.contains(missing) && err.contains("is required"),
                "{missing}: {err}"
            );
        }
    }

    #[test]
    fn a_model_step_without_a_model_table_is_refused() {
        let err = parse_and_validate(&manifest("", STEP)).unwrap_err();
        assert!(
            err.contains("no [model] table") && err.contains(MODEL_POINTER),
            "{err}"
        );
    }

    #[test]
    fn a_model_table_no_step_uses_is_refused() {
        let err = parse_and_validate(&manifest(
            GOOD_TABLE,
            "[[steps]]\nname = \"plain\"\ncommand = [\"true\"]\n",
        ))
        .unwrap_err();
        assert!(err.contains("no step sets `model = true`"), "{err}");
    }
}
