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

use crate::manifest::is_model_command_key;

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
    // Credential-bearing names read off the installed CLI (`claude` 2.1.296,
    // `strings` of the binary, 2026-10-13): the Bedrock bearer token, the
    // Foundry key/token, the first-party AWS key, the refresh and session
    // tokens, the gateway token and the identity token.
    "AWS_BEARER_TOKEN_BEDROCK",
    "ANTHROPIC_FOUNDRY_API_KEY",
    "ANTHROPIC_FOUNDRY_AUTH_TOKEN",
    "ANTHROPIC_AWS_API_KEY",
    "ANTHROPIC_IDENTITY_TOKEN",
    "CLAUDE_CODE_OAUTH_REFRESH_TOKEN",
    "CLAUDE_CODE_SESSION_ACCESS_TOKEN",
    "CLAUDE_CODE_GATEWAY_TOKEN",
    // Provider switches the same binary reads beside the ones above: each
    // re-routes a `claude` in a step away from the (absent) login.
    "CLAUDE_CODE_USE_MANTLE",
    "CLAUDE_CODE_USE_GATEWAY",
    "CLAUDE_CODE_SKIP_MANTLE_AUTH",
    "ANTHROPIC_BEDROCK_MANTLE_BASE_URL",
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
/// de-duplicated. An inherited key is returned exactly as inherited, so one
/// that is not valid Unicode is removed under its own name.
pub fn step_env_removals(
    inherited_keys: impl IntoIterator<Item = OsString>,
    host_scrubbed: &[&str],
) -> Vec<OsString> {
    let mut out: Vec<OsString> = MODEL_CREDENTIAL_ENV
        .iter()
        .chain(host_scrubbed.iter())
        .map(OsString::from)
        .collect();
    for key in inherited_keys {
        // Match on the lossy VIEW, but remove the key's OWN spelling. A lossy
        // conversion never alters an ASCII character and the suffix is ASCII,
        // so the view ends in the suffix exactly when the key does. The
        // converted string itself would name a different variable whenever
        // the key is not valid Unicode, and removing it would leave this one
        // inherited.
        if is_model_command_key(&key.to_string_lossy()) {
            out.push(key);
        }
    }
    out.sort();
    out.dedup();
    out
}

/// The removals for a step run from this process: its own environment's keys
/// plus the host's list.
pub fn current_step_env_removals(host_scrubbed: &[&str]) -> Vec<OsString> {
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
    use std::collections::BTreeSet;

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
            assert!(r.contains(&OsString::from(*k)), "{k}");
        }
        assert!(r.contains(&OsString::from("DOCS_EVAL_MODEL_CMD")));
        assert!(r.contains(&OsString::from("REFERENCE_EVAL_MODEL_CMD")));
        assert!(r.contains(&OsString::from("QONTINUI_TEST_LOGIN_PASSWORD")));
        assert!(!r.contains(&OsString::from("PATH")));
        assert!(!r.contains(&OsString::from("HOME")));
    }

    /// An inherited key that is not valid Unicode is removed under its OWN
    /// spelling, byte for byte. Its lossy spelling (`\xff` -> U+FFFD) names a
    /// different variable, so removing that would leave this one inherited.
    #[cfg(unix)]
    #[test]
    fn a_non_unicode_inherited_model_command_is_removed_under_its_own_spelling() {
        use std::os::unix::ffi::{OsStrExt, OsStringExt};
        let inherited = [OsString::from_vec(b"P\xff_MODEL_CMD".to_vec())];
        let r = step_env_removals(inherited, &[]);
        assert!(
            r.iter().any(|k| k.as_bytes() == b"P\xff_MODEL_CMD"),
            "the inherited key is not in the removals byte for byte: {r:?}"
        );
        assert!(
            !r.iter()
                .any(|k| k.as_bytes() == "P\u{FFFD}_MODEL_CMD".as_bytes()),
            "the removals name the lossy spelling, a different variable: {r:?}"
        );
    }

    /// D11's list, spelled out once more as literals so an edit to the
    /// constant has to edit this too. Compared as sets in BOTH directions and
    /// by count, so a name dropped from the constant, a name added to it, and
    /// a repeated name each turn this red.
    #[test]
    fn the_credential_class_is_exactly_the_planned_list() {
        let planned: [&str; 31] = [
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
            // Credential-bearing names read off the installed CLI.
            "AWS_BEARER_TOKEN_BEDROCK",
            "ANTHROPIC_FOUNDRY_API_KEY",
            "ANTHROPIC_FOUNDRY_AUTH_TOKEN",
            "ANTHROPIC_AWS_API_KEY",
            "ANTHROPIC_IDENTITY_TOKEN",
            "CLAUDE_CODE_OAUTH_REFRESH_TOKEN",
            "CLAUDE_CODE_SESSION_ACCESS_TOKEN",
            "CLAUDE_CODE_GATEWAY_TOKEN",
            // Provider switches the same binary reads beside the ones above.
            "CLAUDE_CODE_USE_MANTLE",
            "CLAUDE_CODE_USE_GATEWAY",
            "CLAUDE_CODE_SKIP_MANTLE_AUTH",
            "ANTHROPIC_BEDROCK_MANTLE_BASE_URL",
            // Other providers' keys.
            "OPENAI_API_KEY",
            "GEMINI_API_KEY",
            "GOOGLE_API_KEY",
        ];
        let planned_set: BTreeSet<&str> = planned.iter().copied().collect();
        assert_eq!(planned_set.len(), 31, "the planned list repeats a name");

        let class: BTreeSet<&str> = MODEL_CREDENTIAL_ENV.iter().copied().collect();
        let missing: Vec<&str> = planned_set.difference(&class).copied().collect();
        let unplanned: Vec<&str> = class.difference(&planned_set).copied().collect();
        assert!(
            missing.is_empty(),
            "planned but missing from MODEL_CREDENTIAL_ENV: {missing:?}"
        );
        assert!(
            unplanned.is_empty(),
            "in MODEL_CREDENTIAL_ENV but not planned: {unplanned:?}"
        );

        assert_eq!(
            MODEL_CREDENTIAL_ENV.len(),
            31,
            "MODEL_CREDENTIAL_ENV must hold exactly the 31 planned names"
        );
        assert_eq!(
            class.len(),
            MODEL_CREDENTIAL_ENV.len(),
            "MODEL_CREDENTIAL_ENV repeats a name"
        );
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
