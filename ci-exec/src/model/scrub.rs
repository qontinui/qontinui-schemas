//! The environment every CI step loses, and the empty `CLAUDE_CONFIG_DIR` it
//! gets instead (plan `2026-10-13-ci-node-runs-model-dependent-eval-steps`,
//! D11).
//!
//! A step used to inherit the host's whole process environment. The executor
//! now removes, from EVERY step (not just `model = true` ones):
//!
//! - the model-credential class [`MODEL_CREDENTIAL_ENV`];
//! - every inherited key ending in `_MODEL_CMD`, so a host-provided model
//!   command cannot outrank the broker;
//! - the host's own list ([`crate::host::ProcessSpawn::scrubbed_env`]).
//!
//! Removing `CLAUDE_CONFIG_DIR` alone would let a `claude` run inside a step
//! fall back to the default config dir under the inherited `HOME` — the
//! owner's login, with no broker, budget, ledger or trust check. So the
//! executor then SETS it, for every step, to an empty per-dispatch directory
//! ([`fresh_empty_config_dir`]), and an un-brokered `claude` starts with no
//! login.

use std::ffi::OsString;
use std::path::{Path, PathBuf};

use crate::manifest::MODEL_COMMAND_SUFFIX;

/// The model-credential class, removed from every CI step's environment.
///
/// The provider-switch names are spelled literally as runner#2138's
/// `PROVIDER_SWITCH_ENV` spells them: ci-exec is a dependency of the runner,
/// so it cannot import that constant, and the superset test lives on the
/// runner side.
pub const MODEL_CREDENTIAL_ENV: &[&str] = &[
    "ANTHROPIC_API_KEY",
    "ANTHROPIC_AUTH_TOKEN",
    "CLAUDE_CODE_OAUTH_TOKEN",
    "ANTHROPIC_BASE_URL",
    "ANTHROPIC_CUSTOM_HEADERS",
    "CLAUDE_CONFIG_DIR",
    // The provider-switch set.
    "CLAUDE_CODE_USE_BEDROCK",
    "CLAUDE_CODE_USE_VERTEX",
    "CLAUDE_CODE_USE_FOUNDRY",
    "CLAUDE_CODE_SKIP_BEDROCK_AUTH",
    "CLAUDE_CODE_SKIP_VERTEX_AUTH",
    "CLAUDE_CODE_SKIP_FOUNDRY_AUTH",
    "ANTHROPIC_BEDROCK_BASE_URL",
    "ANTHROPIC_VERTEX_BASE_URL",
    "ANTHROPIC_FOUNDRY_BASE_URL",
    "ANTHROPIC_FOUNDRY_RESOURCE",
    // Other providers' keys.
    "OPENAI_API_KEY",
    "GEMINI_API_KEY",
    "GOOGLE_API_KEY",
];

/// The name of the per-dispatch empty config directory, under the dispatch
/// root (which cleanup removes whole).
const EMPTY_CONFIG_DIR_NAME: &str = ".ci-claude-config";

/// Every variable to remove from a step's environment, given the keys the
/// executor's own process holds and the host's extra list. Sorted and
/// de-duplicated.
pub fn step_env_removals(
    inherited_keys: impl IntoIterator<Item = OsString>,
    host_scrubbed: &[&str],
) -> Vec<String> {
    let mut out: Vec<String> = MODEL_CREDENTIAL_ENV
        .iter()
        .chain(host_scrubbed.iter())
        .map(|k| k.to_string())
        .collect();
    for key in inherited_keys {
        // A key that is not valid Unicode cannot end in the ASCII suffix in a
        // way a manifest could name; compare on the lossy spelling anyway so a
        // mangled key is still removed when it carries the suffix.
        let key = key.to_string_lossy().into_owned();
        if key.ends_with(MODEL_COMMAND_SUFFIX) {
            out.push(key);
        }
    }
    out.sort();
    out.dedup();
    out
}

/// The removals for a step run from this process: its own environment's keys
/// plus the host's list.
pub fn current_step_env_removals(host_scrubbed: &[&str]) -> Vec<String> {
    step_env_removals(std::env::vars_os().map(|(k, _)| k), host_scrubbed)
}

/// (Re)create the empty `CLAUDE_CONFIG_DIR` a step gets, under the dispatch
/// root. Called before EVERY step, so a step that wrote into it cannot hand
/// its files to the next one. `Err` means no empty directory could be
/// guaranteed — the caller fails the step rather than run it with the
/// variable pointing somewhere that is not empty.
pub fn fresh_empty_config_dir(dispatch_root: &Path) -> Result<PathBuf, String> {
    let dir = dispatch_root.join(EMPTY_CONFIG_DIR_NAME);
    if dir.exists() {
        std::fs::remove_dir_all(&dir)
            .map_err(|e| format!("could not empty {}: {e}", dir.display()))?;
    }
    std::fs::create_dir_all(&dir)
        .map_err(|e| format!("could not create {}: {e}", dir.display()))?;
    Ok(dir)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn removals_cover_the_class_the_host_list_and_inherited_model_commands() {
        let inherited = [
            "PATH",
            "DOCS_EVAL_MODEL_CMD",
            "REFERENCE_EVAL_MODEL_CMD",
            "HOME",
        ]
        .map(OsString::from);
        let r = step_env_removals(inherited, &["QONTINUI_TEST_LOGIN_PASSWORD"]);
        for k in MODEL_CREDENTIAL_ENV {
            assert!(r.contains(&k.to_string()), "{k}");
        }
        assert!(r.contains(&"DOCS_EVAL_MODEL_CMD".to_string()));
        assert!(r.contains(&"REFERENCE_EVAL_MODEL_CMD".to_string()));
        assert!(r.contains(&"QONTINUI_TEST_LOGIN_PASSWORD".to_string()));
        assert!(!r.contains(&"PATH".to_string()));
        assert!(!r.contains(&"HOME".to_string()));
    }

    /// D11's list, spelled out once more as literals so an edit to the
    /// constant has to edit this too.
    #[test]
    fn the_credential_class_is_the_planned_list() {
        for k in [
            "ANTHROPIC_API_KEY",
            "ANTHROPIC_AUTH_TOKEN",
            "CLAUDE_CODE_OAUTH_TOKEN",
            "ANTHROPIC_BASE_URL",
            "ANTHROPIC_CUSTOM_HEADERS",
            "CLAUDE_CONFIG_DIR",
            "CLAUDE_CODE_USE_BEDROCK",
            "CLAUDE_CODE_USE_VERTEX",
            "CLAUDE_CODE_USE_FOUNDRY",
            "CLAUDE_CODE_SKIP_BEDROCK_AUTH",
            "CLAUDE_CODE_SKIP_VERTEX_AUTH",
            "CLAUDE_CODE_SKIP_FOUNDRY_AUTH",
            "ANTHROPIC_BEDROCK_BASE_URL",
            "ANTHROPIC_VERTEX_BASE_URL",
            "ANTHROPIC_FOUNDRY_BASE_URL",
            "ANTHROPIC_FOUNDRY_RESOURCE",
            "OPENAI_API_KEY",
            "GEMINI_API_KEY",
            "GOOGLE_API_KEY",
        ] {
            assert!(MODEL_CREDENTIAL_ENV.contains(&k), "{k}");
        }
    }

    #[test]
    fn the_config_dir_is_recreated_empty() {
        let tmp = tempfile::tempdir().unwrap();
        let dir = fresh_empty_config_dir(tmp.path()).unwrap();
        assert!(dir.starts_with(tmp.path()));
        std::fs::write(dir.join(".credentials.json"), "x").unwrap();
        let again = fresh_empty_config_dir(tmp.path()).unwrap();
        assert_eq!(again, dir);
        assert_eq!(std::fs::read_dir(&again).unwrap().count(), 0);
    }
}
