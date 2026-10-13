//! The git environment a sibling's git runs under: nothing from the node
//! owner's git configuration reaches it.
//!
//! # Why
//!
//! A sibling is cloned over `https://github.com/<owner>/<name>.git`. Left to
//! inherit the box's git configuration, that fetch is answered by whatever the
//! node OWNER configured: a `credential.helper` (`gh auth git-credential`, Git
//! Credential Manager) hands git the owner's GitHub token for a private
//! sibling, and a global `url.<ssh>.insteadOf` rewrite routes the clone over the
//! owner's SSH key. Either way a private sibling "works" with no grant, no
//! audit row and no trust check (plan
//! `2026-10-13-local-lane-ci-reads-a-private-sibling-through-a-dispatch-scoped-door`,
//! D5). This module closes every one of those paths:
//!
//! * `GIT_CONFIG_NOSYSTEM=1` — the system config is not read. **Not on
//!   Windows**: there D5's NAMED FALLBACK is taken (Phase 0 item 2 was not
//!   measured on Windows), because Git for Windows keeps its schannel/CA and
//!   `core.longpaths` settings in the system file and a sibling fetch without
//!   them is a different failure. The global file's `[credential] helper =`
//!   line still clears every system-level helper, URL-scoped ones included
//!   (measured on Linux). **Recorded residual:** a SYSTEM-level
//!   `url.<x>.insteadOf` on Windows is still read — nothing in this module
//!   resets it.
//! * `GIT_CONFIG_GLOBAL=<dispatch root>/git-global.config` — the owner's
//!   `~/.gitconfig` and XDG config are not read. The file holds exactly
//!   `[credential] helper =`, which empties git's helper list (an empty value
//!   resets it), so even a helper that slips in from a config we did not
//!   anticipate is cleared. It lives in the file and NOT in the env overlay,
//!   because an empty `GIT_CONFIG_VALUE_n` may reach a mingw `git.exe` as unset
//!   and git then dies `missing config value` on every invocation (runner
//!   `git_posture.rs`, the `ASKPASS_DISABLED_SENTINEL` doc).
//! * `GIT_CEILING_DIRECTORIES=<parent of the dispatch root>` — git's repository
//!   discovery stops before it leaves the dispatch root. Without it a sibling
//!   git whose cwd is `<root>/.ci-worktrees/<id>` (ls-remote, `git init`)
//!   walks UPWARD, finds whatever repository encloses the CI root, and reads
//!   that repository's `.git/config` — measured: an enclosing repo's
//!   `credential.helper=store` and `url.git@github.com:.insteadOf` both applied
//!   under the rest of this isolation. The ceiling is the dispatch root's
//!   PARENT, not the dispatch root: git always examines its cwd itself, and a
//!   ceiling equal to the cwd was measured NOT to stop the walk. Every cwd a
//!   sibling's git runs in is the dispatch root or a sibling checkout below it,
//!   whose own `.git` is found first.
//! * `HOME` (and `USERPROFILE` on Windows, `XDG_CONFIG_HOME` everywhere) = an
//!   empty per-dispatch directory under the dispatch root. libcurl reads
//!   `$HOME/.netrc` (`_netrc` on Windows) on its own, below git's config, so
//!   an owner's netrc supplied credentials for github.com even with every git
//!   config source isolated (measured). `GIT_CONFIG_GLOBAL` still names the
//!   global file explicitly, so moving `HOME` changes nothing about which
//!   global config git reads.
//! * **Every inherited `GIT_*` variable except an explicit allowlist is
//!   removed** — from the process environment AND from whatever the host's
//!   [`ProcessSpawn::command`] set on the command. Removing only the
//!   `GIT_CONFIG_COUNT` / `KEY_n` / `VALUE_n` overlay is not enough: measured
//!   on git 2.47.3, an inherited `GIT_CONFIG_PARAMETERS="'credential.helper'='store'"`
//!   survives `NOSYSTEM`, an empty `GIT_CONFIG_GLOBAL` and a `KEY_0` reset, and
//!   stays the active helper. An ALLOWLIST rather than a denylist, so a git
//!   variable added later cannot reopen the hole. Trace variables
//!   (`GIT_TRACE*`, `GIT_CURL_VERBOSE`) go with it, so a host that exports them
//!   cannot print the auth header.
//! * The env overlay is then built from index 0 and carries only NON-EMPTY
//!   values: for a granted sibling one `http.<clone url>.extraHeader`, the form
//!   `actions/checkout` uses. git matches `http.<url>.*` on whole path
//!   segments, so the key is spelled from the SAME clone URL the fetch uses,
//!   `.git` suffix included — never re-spelled.
//! * The prompt-proof layer ([`PROMPT_PROOF_GIT_ENV`]) is re-applied on top.
//!
//! The token never appears in argv, a URL, a remote, a file, a log line or a
//! `Debug` rendering: it is held in [`ReadToken`], and the only place its value
//! is written is the git child's environment for the length of one call. That
//! residual — another process of the same OS user can read a child's
//! environment — is the plan's named same-user residual, bounded by revoking
//! the token once the sibling is provisioned.

use std::ffi::{OsStr, OsString};
use std::path::{Path, PathBuf};
use std::time::Duration;

use crate::host::ProcessSpawn;
use crate::standalone::PROMPT_PROOF_GIT_ENV;

/// The per-dispatch global git config, written under the dispatch root (which
/// cleanup removes with everything else).
pub const GLOBAL_CONFIG_FILE: &str = "git-global.config";

/// Its whole body: an empty `helper =` resets git's credential-helper list.
const GLOBAL_CONFIG_BODY: &str = "[credential]\n\thelper =\n";

/// The per-dispatch, EMPTY directory a sibling's git sees as `HOME`, under the
/// dispatch root (so cleanup removes it with everything else).
pub const ISOLATED_HOME_DIR: &str = "git-home";

/// The `GIT_*` keys the overlay itself SETS. An inherited value of any of them
/// is scrubbed like every other non-allowlisted git variable, then replaced
/// by the overlay's own. (Pinned by the allowlist test.)
#[cfg(test)]
pub(crate) const OVERLAY_GIT_KEYS: &[&str] = &[
    #[cfg(not(windows))]
    "GIT_CONFIG_NOSYSTEM",
    "GIT_CONFIG_GLOBAL",
    "GIT_CEILING_DIRECTORIES",
    "GIT_CONFIG_COUNT",
    "GIT_CONFIG_KEY_0",
    "GIT_CONFIG_VALUE_0",
];

/// The oldest git the isolation holds on. `GIT_CONFIG_GLOBAL` arrived in 2.32
/// and `GIT_CONFIG_COUNT` in 2.31; an older git silently ignores both and reads
/// the owner's global helper and `insteadOf` again, so the isolation would fail
/// OPEN. Below this, provisioning is refused — never attempted publicly.
pub(crate) const MIN_GIT: (u32, u32) = (2, 32);

/// Budget for `git --version`.
const GIT_VERSION_TIMEOUT: Duration = Duration::from_secs(30);

/// The git variables a sibling's git must never inherit, MIRRORED (not
/// imported — ci-exec does not depend on the runner) from qontinui-runner
/// `src-tauri/src/git_posture.rs` `REPO_LOCAL_GIT_ENV` and
/// `COMMAND_SCOPE_GIT_CONFIG_ENV` (read at runner `origin/main` `dfdcab404`).
///
/// It is NOT what the scrub consults — the scrub removes every `GIT_*` outside
/// [`inherited_git_key_allowed`], which is a superset of this list. It exists so
/// a test pins that each of these is in fact removed; the runner KEEPS the two
/// command-scope names for its own git, and a sibling's git must not.
#[cfg(test)]
pub(crate) const MIRRORED_DENIED_GIT_ENV: &[&str] = &[
    // REPO_LOCAL_GIT_ENV
    "GIT_ALTERNATE_OBJECT_DIRECTORIES",
    "GIT_CONFIG",
    "GIT_OBJECT_DIRECTORY",
    "GIT_DIR",
    "GIT_WORK_TREE",
    "GIT_IMPLICIT_WORK_TREE",
    "GIT_GRAFT_FILE",
    "GIT_INDEX_FILE",
    "GIT_NO_REPLACE_OBJECTS",
    "GIT_REPLACE_REF_BASE",
    "GIT_PREFIX",
    "GIT_SHALLOW_FILE",
    "GIT_COMMON_DIR",
    // COMMAND_SCOPE_GIT_CONFIG_ENV
    "GIT_CONFIG_PARAMETERS",
    "GIT_CONFIG_COUNT",
];

/// A minted, short-lived GitHub read token for one sibling.
///
/// Hand-written `Debug` and no `Display`, so the token has no formatting path
/// at all — the [`crate::services`] `ServiceCreds` precedent: closing the
/// CLASS is cheaper than auditing each new format string for the instance.
pub struct ReadToken(String);

impl ReadToken {
    /// Wrap a token a host obtained from its read door.
    pub fn new(token: impl Into<String>) -> Self {
        Self(token.into())
    }

    /// The raw value. Crate-private: the only consumers are the auth header
    /// built here, the sibling's GitHub API reads, and the revocation call.
    pub(crate) fn secret(&self) -> &str {
        &self.0
    }
}

impl std::fmt::Debug for ReadToken {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("ReadToken(<redacted>)")
    }
}

/// Is `key` a git variable? Upper-cased first: Windows environment names are
/// case-insensitive, so `git_dir` reaches git as `GIT_DIR` there.
fn is_git_key(key: &OsStr) -> bool {
    key.to_string_lossy()
        .to_ascii_uppercase()
        .starts_with("GIT_")
}

/// The ONLY `GIT_*` variables a sibling's git may inherit: the prompt-proof
/// keys (re-set here anyway), plus `GIT_EXEC_PATH` on Windows, where Git for
/// Windows' launcher relies on it to find its own helpers.
fn inherited_git_key_allowed(key: &OsStr) -> bool {
    let upper = key.to_string_lossy().to_ascii_uppercase();
    PROMPT_PROOF_GIT_ENV.iter().any(|(k, _)| *k == upper)
        || (cfg!(windows) && upper == "GIT_EXEC_PATH")
}

/// One variable the overlay sets.
struct EnvSet {
    key: String,
    value: OsString,
    /// Rendered `<redacted>` by `Debug`.
    secret: bool,
}

/// The isolated environment for one sibling's git calls: `(removals, sets)`,
/// applied to a command AFTER the host's [`ProcessSpawn::command`], so the host
/// posture comes first and this overrides it.
pub struct SiblingGitEnv {
    removals: Vec<OsString>,
    sets: Vec<EnvSet>,
}

impl std::fmt::Debug for SiblingGitEnv {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let sets: Vec<(String, String)> = self
            .sets
            .iter()
            .map(|s| {
                let v = if s.secret {
                    "<redacted>".to_string()
                } else {
                    s.value.to_string_lossy().into_owned()
                };
                (s.key.clone(), v)
            })
            .collect();
        f.debug_struct("SiblingGitEnv")
            .field("removals", &self.removals)
            .field("sets", &sets)
            .finish()
    }
}

impl SiblingGitEnv {
    /// D5's environment for a sibling, reading the inherited environment from
    /// this process. `auth` is `(clone url, token)` for a sibling the read door
    /// granted — the URL must be the one the fetch itself uses.
    ///
    /// Writes [`GLOBAL_CONFIG_FILE`] under `dispatch_root` (idempotent).
    pub(crate) fn isolated(
        dispatch_root: &Path,
        auth: Option<(&str, &ReadToken)>,
    ) -> Result<Self, String> {
        Self::isolated_from(dispatch_root, auth, std::env::vars_os())
    }

    /// [`Self::isolated`] over an explicit inherited environment (the seam the
    /// tests use, since mutating the process environment races other tests).
    pub(crate) fn isolated_from(
        dispatch_root: &Path,
        auth: Option<(&str, &ReadToken)>,
        inherited: impl IntoIterator<Item = (OsString, OsString)>,
    ) -> Result<Self, String> {
        let global = write_global_config(dispatch_root)?;
        let ceiling = ceiling_for(dispatch_root)?;
        let home = fresh_isolated_home(dispatch_root)?;

        let removals: Vec<OsString> = inherited
            .into_iter()
            .map(|(k, _)| k)
            .filter(|k| is_git_key(k) && !inherited_git_key_allowed(k))
            .collect();

        let mut sets = Vec::new();
        // Windows: D5's named fallback — the system config stays readable (see
        // the module docs for why, and for the residual it leaves).
        #[cfg(not(windows))]
        sets.push(EnvSet {
            key: "GIT_CONFIG_NOSYSTEM".into(),
            value: "1".into(),
            secret: false,
        });
        sets.push(EnvSet {
            key: "GIT_CONFIG_GLOBAL".into(),
            value: global.into_os_string(),
            secret: false,
        });
        sets.push(EnvSet {
            key: "GIT_CEILING_DIRECTORIES".into(),
            value: ceiling,
            secret: false,
        });
        let mut home_keys = vec!["HOME", "XDG_CONFIG_HOME"];
        if cfg!(windows) {
            home_keys.push("USERPROFILE");
        }
        for key in home_keys {
            sets.push(EnvSet {
                key: key.into(),
                value: home.clone().into_os_string(),
                secret: false,
            });
        }
        if let Some((url, token)) = auth {
            sets.push(EnvSet {
                key: "GIT_CONFIG_COUNT".into(),
                value: "1".into(),
                secret: false,
            });
            sets.push(EnvSet {
                key: "GIT_CONFIG_KEY_0".into(),
                value: extra_header_key(url).into(),
                secret: false,
            });
            sets.push(EnvSet {
                key: "GIT_CONFIG_VALUE_0".into(),
                value: auth_header(token).into(),
                secret: true,
            });
        }
        for (k, v) in PROMPT_PROOF_GIT_ENV {
            sets.push(EnvSet {
                key: (*k).into(),
                value: (*v).into(),
                secret: false,
            });
        }
        Ok(Self { removals, sets })
    }

    /// Apply to `cmd`: remove every non-allowlisted `GIT_*` the HOST set on it
    /// and every one inherited from this process, then set the overlay.
    pub(crate) fn isolate_command(&self, cmd: &mut tokio::process::Command) {
        let host_set: Vec<OsString> = cmd
            .as_std()
            .get_envs()
            .filter(|(k, v)| v.is_some() && is_git_key(k) && !inherited_git_key_allowed(k))
            .map(|(k, _)| k.to_os_string())
            .collect();
        for k in host_set.iter().chain(self.removals.iter()) {
            cmd.env_remove(k);
        }
        for s in &self.sets {
            cmd.env(&s.key, &s.value);
        }
    }
}

/// `http.<url>.extraHeader` for exactly `url`.
pub(crate) fn extra_header_key(url: &str) -> String {
    format!("http.{url}.extraHeader")
}

/// `Authorization: Basic base64("x-access-token:" + token)` — the form
/// `actions/checkout` sends for an installation token.
fn auth_header(token: &ReadToken) -> String {
    format!(
        "Authorization: Basic {}",
        base64(format!("x-access-token:{}", token.secret()).as_bytes())
    )
}

/// Standard, padded base64. Hand-rolled (it is twenty lines) so the crate's
/// dependency set does not move for one header.
fn base64(input: &[u8]) -> String {
    const ALPHABET: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut out = String::with_capacity(input.len().div_ceil(3) * 4);
    for chunk in input.chunks(3) {
        let b = [
            chunk[0],
            chunk.get(1).copied().unwrap_or(0),
            chunk.get(2).copied().unwrap_or(0),
        ];
        let n = (u32::from(b[0]) << 16) | (u32::from(b[1]) << 8) | u32::from(b[2]);
        out.push(ALPHABET[(n >> 18) as usize & 63] as char);
        out.push(ALPHABET[(n >> 12) as usize & 63] as char);
        out.push(if chunk.len() > 1 {
            ALPHABET[(n >> 6) as usize & 63] as char
        } else {
            '='
        });
        out.push(if chunk.len() > 2 {
            ALPHABET[n as usize & 63] as char
        } else {
            '='
        });
    }
    out
}

/// `GIT_CEILING_DIRECTORIES` for a git whose cwd is `dispatch_root` or below:
/// the dispatch root's PARENT, absolute (git ignores a relative entry).
fn ceiling_for(dispatch_root: &Path) -> Result<OsString, String> {
    let abs = std::path::absolute(dispatch_root)
        .map_err(|e| format!("resolve {}: {e}", dispatch_root.display()))?;
    let parent = abs.parent().ok_or_else(|| {
        format!(
            "dispatch root {} has no parent to stop git's repository discovery at",
            abs.display()
        )
    })?;
    // `join_paths` refuses a path containing the list separator, which git
    // would otherwise split into two (wrong) ceilings.
    std::env::join_paths([parent]).map_err(|e| format!("ceiling {}: {e}", parent.display()))
}

/// Create the per-dispatch `HOME` EMPTY — anything a previous sibling's git
/// left there is removed first.
fn fresh_isolated_home(dispatch_root: &Path) -> Result<PathBuf, String> {
    let home = dispatch_root.join(ISOLATED_HOME_DIR);
    match std::fs::remove_dir_all(&home) {
        Ok(()) => {}
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {}
        Err(e) => return Err(format!("clear {}: {e}", home.display())),
    }
    std::fs::create_dir_all(&home).map_err(|e| format!("create {}: {e}", home.display()))?;
    Ok(home)
}

fn write_global_config(dispatch_root: &Path) -> Result<PathBuf, String> {
    std::fs::create_dir_all(dispatch_root)
        .map_err(|e| format!("create {}: {e}", dispatch_root.display()))?;
    let path = dispatch_root.join(GLOBAL_CONFIG_FILE);
    std::fs::write(&path, GLOBAL_CONFIG_BODY)
        .map_err(|e| format!("write {}: {e}", path.display()))?;
    Ok(path)
}

/// A git command for a sibling: the host's `git`, then this isolation on top.
pub(crate) fn sibling_git_command(
    process: &dyn ProcessSpawn,
    env: &SiblingGitEnv,
) -> tokio::process::Command {
    let mut cmd = process.command("git");
    env.isolate_command(&mut cmd);
    cmd
}

/// The ONE door every sibling git call goes through.
pub(crate) async fn run_sibling_git(
    process: &dyn ProcessSpawn,
    cwd: &Path,
    args: &[&str],
    timeout: Duration,
    env: &SiblingGitEnv,
) -> Result<String, String> {
    crate::checkout::run_git_command(
        process,
        sibling_git_command(process, env),
        cwd,
        args,
        timeout,
    )
    .await
}

/// Judge `git --version` output against [`MIN_GIT`].
pub(crate) fn check_git_version(output: &str) -> Result<(), String> {
    let ver = output
        .trim()
        .strip_prefix("git version ")
        .and_then(|rest| rest.split_whitespace().next())
        .unwrap_or_default();
    let mut parts = ver.split('.').map(|p| {
        let digits: String = p.chars().take_while(|c| c.is_ascii_digit()).collect();
        digits.parse::<u32>().ok()
    });
    let (Some(Some(major)), Some(Some(minor))) = (parts.next(), parts.next()) else {
        return Err(format!(
            "could not read git's version for sibling isolation ({:?}); refusing rather than \
             risk a git that ignores the isolation",
            output.trim()
        ));
    };
    if (major, minor) < MIN_GIT {
        return Err(format!(
            "git too old for sibling isolation ({ver} < {}.{})",
            MIN_GIT.0, MIN_GIT.1
        ));
    }
    Ok(())
}

/// Read `git --version` once and refuse below [`MIN_GIT`].
pub(crate) async fn require_git_floor(
    process: &dyn ProcessSpawn,
    cwd: &Path,
) -> Result<(), String> {
    let out = crate::checkout::run_git(process, cwd, &["--version"], GIT_VERSION_TIMEOUT).await?;
    check_git_version(&out)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::standalone::PlainSpawn;

    const TOKEN: &str = "ghs_TESTtokenVALUE0123456789";
    const URL: &str = "https://github.com/qontinui/qontinui-docs-reference.git";

    fn os(pairs: &[(&str, &str)]) -> Vec<(OsString, OsString)> {
        pairs
            .iter()
            .map(|(k, v)| (OsString::from(k), OsString::from(v)))
            .collect()
    }

    /// The child's effective environment when the command starts from
    /// `env_clear()` + `base`, i.e. as if `base` were its whole inheritance.
    fn effective_env(cmd: &tokio::process::Command) -> Vec<(String, String)> {
        cmd.as_std()
            .get_envs()
            .filter_map(|(k, v)| Some((k.to_str()?.to_string(), v?.to_str()?.to_string())))
            .collect()
    }

    /// A host that hands back a command whose whole environment is `base` —
    /// what the host's posture plus the process inheritance would give.
    struct BaseEnvSpawn(Vec<(OsString, OsString)>);
    impl ProcessSpawn for BaseEnvSpawn {
        fn command(&self, program: &str) -> tokio::process::Command {
            let mut cmd = tokio::process::Command::new(program);
            cmd.env_clear().envs(self.0.iter().cloned());
            cmd
        }
        fn std_command(&self, program: &str) -> std::process::Command {
            std::process::Command::new(program)
        }
        fn arm_tree(&self, cmd: &mut tokio::process::Command) {
            PlainSpawn.arm_tree(cmd)
        }
        fn attach_tree(&self, child: &tokio::process::Child) -> Box<dyn crate::host::TreeGuard> {
            PlainSpawn.attach_tree(child)
        }
        fn step_containment(&self) -> Box<dyn crate::host::StepContainment> {
            PlainSpawn.step_containment()
        }
    }

    fn built(
        inherited: &[(&str, &str)],
        auth: Option<(&str, &ReadToken)>,
    ) -> (tempfile::TempDir, SiblingGitEnv, Vec<(String, String)>) {
        let dir = tempfile::tempdir().unwrap();
        let env = SiblingGitEnv::isolated_from(dir.path(), auth, os(inherited)).unwrap();
        let cmd = sibling_git_command(&BaseEnvSpawn(os(inherited)), &env);
        let eff = effective_env(&cmd);
        (dir, env, eff)
    }

    #[test]
    fn an_inherited_config_overlay_is_gone_from_the_child() {
        let inherited = [
            ("GIT_CONFIG_COUNT", "1"),
            ("GIT_CONFIG_KEY_0", "credential.helper"),
            ("GIT_CONFIG_VALUE_0", "store"),
            ("PATH", "/usr/bin"),
        ];
        let (_d, env, eff) = built(&inherited, None);
        for k in ["GIT_CONFIG_COUNT", "GIT_CONFIG_KEY_0", "GIT_CONFIG_VALUE_0"] {
            assert!(!eff.iter().any(|(ek, _)| ek == k), "{k} survived: {eff:?}");
            assert!(env.removals.iter().any(|r| r == k), "{k} not in removals");
        }
        // Not git: untouched.
        assert!(eff.contains(&("PATH".into(), "/usr/bin".into())));
    }

    #[test]
    fn only_allowlisted_git_keys_survive() {
        let mut inherited: Vec<(&str, &str)> = vec![
            ("GIT_CONFIG_PARAMETERS", "'credential.helper'='store'"),
            ("GIT_DIR", "/elsewhere/.git"),
            ("git_work_tree", "/elsewhere"),
            ("GIT_SSH_COMMAND", "ssh -i owner_key"),
            ("GIT_PROXY_COMMAND", "x"),
            ("GIT_TEMPLATE_DIR", "/t"),
            ("GIT_SSL_NO_VERIFY", "1"),
            ("GIT_TERMINAL_PROMPT", "1"),
            ("GIT_ASKPASS", "/usr/lib/ssh/askpass"),
        ];
        inherited.extend(MIRRORED_DENIED_GIT_ENV.iter().map(|k| (*k, "x")));
        let (_d, _env, eff) = built(&inherited, None);
        let allowed: Vec<&str> = PROMPT_PROOF_GIT_ENV.iter().map(|(k, _)| *k).collect();
        for (k, _) in &eff {
            if is_git_key(OsStr::new(k)) {
                assert!(
                    allowed.contains(&k.as_str()) || OVERLAY_GIT_KEYS.contains(&k.as_str()),
                    "{k} survived the scrub"
                );
            }
        }
        for denied in MIRRORED_DENIED_GIT_ENV {
            assert!(!eff.iter().any(|(k, _)| k == denied), "{denied} survived");
        }
        // The prompt-proof layer is re-set, overriding an inherited value.
        for (k, v) in PROMPT_PROOF_GIT_ENV {
            assert!(eff.contains(&(k.to_string(), v.to_string())), "{k}");
        }
    }

    #[test]
    fn the_global_file_resets_the_helper_and_no_env_value_is_empty() {
        let token = ReadToken::new(TOKEN);
        for auth in [None, Some((URL, &token))] {
            let (dir, _env, eff) = built(&[], auth);
            let body = std::fs::read_to_string(dir.path().join(GLOBAL_CONFIG_FILE)).unwrap();
            assert_eq!(body, "[credential]\n\thelper =\n");
            for (k, v) in &eff {
                assert!(!v.is_empty(), "{k} is set EMPTY");
            }
        }
    }

    #[test]
    fn nosystem_and_global_are_set() {
        let (dir, _env, eff) = built(
            &[
                ("GIT_CONFIG_GLOBAL", "/home/owner/.gitconfig"),
                ("GIT_CONFIG_NOSYSTEM", "0"),
            ],
            None,
        );
        // Windows takes D5's named fallback: the system config stays readable.
        let nosystem = eff.iter().find(|(k, _)| k == "GIT_CONFIG_NOSYSTEM");
        if cfg!(windows) {
            assert_eq!(nosystem, None, "{eff:?}");
        } else {
            assert_eq!(
                nosystem,
                Some(&("GIT_CONFIG_NOSYSTEM".to_string(), "1".to_string()))
            );
        }
        let global = dir.path().join(GLOBAL_CONFIG_FILE);
        assert!(
            eff.contains(&(
                "GIT_CONFIG_GLOBAL".into(),
                global.to_string_lossy().into_owned()
            )),
            "{eff:?}"
        );
    }

    #[test]
    fn no_trace_variable_survives() {
        let inherited = [
            ("GIT_TRACE", "1"),
            ("GIT_TRACE_CURL", "1"),
            ("GIT_TRACE_PACKET", "1"),
            ("GIT_TRACE2_EVENT", "/tmp/t"),
            ("GIT_CURL_VERBOSE", "1"),
        ];
        let (_d, _env, eff) = built(&inherited, None);
        for (k, _) in &eff {
            assert!(
                !k.starts_with("GIT_TRACE") && k != "GIT_CURL_VERBOSE",
                "{k} survived"
            );
        }
    }

    #[test]
    fn a_granted_overlay_starts_at_index_zero_with_the_header() {
        let token = ReadToken::new(TOKEN);
        let (_d, _env, eff) = built(&[("GIT_CONFIG_COUNT", "3")], Some((URL, &token)));
        assert!(eff.contains(&("GIT_CONFIG_COUNT".into(), "1".into())));
        assert!(eff.contains(&("GIT_CONFIG_KEY_0".into(), extra_header_key(URL))));
        let expected = format!(
            "Authorization: Basic {}",
            base64(format!("x-access-token:{TOKEN}").as_bytes())
        );
        assert!(eff.contains(&("GIT_CONFIG_VALUE_0".into(), expected)));
    }

    #[test]
    fn no_argv_element_carries_the_token() {
        let token = ReadToken::new(TOKEN);
        let (_d, env, _eff) = built(&[], Some((URL, &token)));
        let mut cmd = sibling_git_command(&PlainSpawn, &env);
        cmd.args(["ls-remote", "--exit-code", URL, "refs/heads/main"]);
        let b64 = base64(format!("x-access-token:{TOKEN}").as_bytes());
        let program = cmd.as_std().get_program().to_string_lossy().into_owned();
        for arg in std::iter::once(program).chain(
            cmd.as_std()
                .get_args()
                .map(|a| a.to_string_lossy().into_owned()),
        ) {
            assert!(!arg.contains(TOKEN) && !arg.contains(&b64), "{arg}");
        }
    }

    #[test]
    fn debug_never_renders_the_token() {
        let token = ReadToken::new(TOKEN);
        let (_d, env, _eff) = built(&[], Some((URL, &token)));
        let b64 = base64(format!("x-access-token:{TOKEN}").as_bytes());
        for rendered in [
            format!("{env:?}"),
            format!("{token:?}"),
            format!("{env:#?}"),
        ] {
            assert!(
                !rendered.contains(TOKEN) && !rendered.contains(&b64),
                "{rendered}"
            );
        }
        assert!(format!("{env:?}").contains("<redacted>"));
    }

    #[test]
    fn base64_matches_the_rfc_vectors() {
        for (input, want) in [
            ("", ""),
            ("f", "Zg=="),
            ("fo", "Zm8="),
            ("foo", "Zm9v"),
            ("foob", "Zm9vYg=="),
            ("fooba", "Zm9vYmE="),
            ("foobar", "Zm9vYmFy"),
            ("x-access-token:ghs_abc", "eC1hY2Nlc3MtdG9rZW46Z2hzX2FiYw=="),
        ] {
            assert_eq!(base64(input.as_bytes()), want, "{input}");
        }
    }

    #[test]
    fn git_below_the_floor_is_refused() {
        let err = check_git_version("git version 2.31.8\n").unwrap_err();
        assert_eq!(err, "git too old for sibling isolation (2.31.8 < 2.32)");
        assert!(check_git_version("git version 1.9.5").is_err());
        assert!(check_git_version("not git").is_err());
        assert!(check_git_version("").is_err());
        for ok in [
            "git version 2.32.0",
            "git version 2.47.3",
            "git version 2.39.2.windows.1",
            "git version 3.0.0",
        ] {
            check_git_version(ok).unwrap_or_else(|e| panic!("{ok}: {e}"));
        }
    }

    /// A host whose `git` is a stub reporting an old version.
    #[cfg(unix)]
    struct OldGitSpawn;
    #[cfg(unix)]
    impl ProcessSpawn for OldGitSpawn {
        fn command(&self, _program: &str) -> tokio::process::Command {
            let mut cmd = tokio::process::Command::new("sh");
            cmd.args(["-c", "echo 'git version 2.25.1'"]);
            cmd
        }
        fn std_command(&self, program: &str) -> std::process::Command {
            std::process::Command::new(program)
        }
        fn arm_tree(&self, cmd: &mut tokio::process::Command) {
            PlainSpawn.arm_tree(cmd)
        }
        fn attach_tree(&self, child: &tokio::process::Child) -> Box<dyn crate::host::TreeGuard> {
            PlainSpawn.attach_tree(child)
        }
        fn step_containment(&self) -> Box<dyn crate::host::StepContainment> {
            PlainSpawn.step_containment()
        }
    }

    #[cfg(unix)]
    #[tokio::test]
    async fn a_stub_git_below_the_floor_is_refused_and_real_git_passes() {
        let dir = tempfile::tempdir().unwrap();
        let err = require_git_floor(&OldGitSpawn, dir.path())
            .await
            .unwrap_err();
        assert!(
            err.contains("git too old for sibling isolation (2.25.1 < 2.32)"),
            "{err}"
        );
        require_git_floor(&PlainSpawn, dir.path()).await.unwrap();
    }

    /// Under the built env, real git sees NO credential helper for github.com —
    /// even with the owner's helper injected through `GIT_CONFIG_PARAMETERS` on
    /// the command, and whatever this box's own global config holds.
    #[tokio::test]
    async fn real_git_sees_no_owner_credential_helper() {
        let dir = tempfile::tempdir().unwrap();
        let env = SiblingGitEnv::isolated(dir.path(), None).unwrap();
        let mut cmd = tokio::process::Command::new("git");
        cmd.env("GIT_CONFIG_PARAMETERS", "'credential.helper'='store'")
            .env("GIT_CONFIG_COUNT", "1")
            .env("GIT_CONFIG_KEY_0", "credential.helper")
            .env("GIT_CONFIG_VALUE_0", "cache");
        env.isolate_command(&mut cmd);
        let out = cmd
            .current_dir(dir.path())
            .args([
                "config",
                "--get-urlmatch",
                "credential.helper",
                "https://github.com/o/n.git",
            ])
            .output()
            .await
            .unwrap();
        // The only helper value git can see is our empty reset.
        assert_eq!(String::from_utf8_lossy(&out.stdout).trim(), "");
        let mut all = tokio::process::Command::new("git");
        env.isolate_command(&mut all);
        let all = all
            .current_dir(dir.path())
            .args(["config", "--get-regexp", "^url\\..*insteadof$"])
            .output()
            .await
            .unwrap();
        assert_eq!(
            String::from_utf8_lossy(&all.stdout).trim(),
            "",
            "an insteadOf reached the sibling"
        );
    }

    /// An inherited ceiling is scrubbed and replaced: the overlay's ceiling is
    /// the dispatch root's absolute PARENT.
    #[test]
    fn the_ceiling_is_the_dispatch_roots_parent() {
        let (dir, _env, eff) = built(&[("GIT_CEILING_DIRECTORIES", "/elsewhere")], None);
        let want = std::path::absolute(dir.path())
            .unwrap()
            .parent()
            .unwrap()
            .to_string_lossy()
            .into_owned();
        let got: Vec<&(String, String)> = eff
            .iter()
            .filter(|(k, _)| k == "GIT_CEILING_DIRECTORIES")
            .collect();
        assert_eq!(got, vec![&("GIT_CEILING_DIRECTORIES".to_string(), want)]);
    }

    /// Run real git under the built env, from `cwd`.
    async fn git_under(env: &SiblingGitEnv, cwd: &Path, args: &[&str]) -> std::process::Output {
        let mut cmd = tokio::process::Command::new("git");
        env.isolate_command(&mut cmd);
        cmd.current_dir(cwd).args(args).output().await.unwrap()
    }

    fn setup_git(cwd: &Path, args: &[&str]) {
        let out = std::process::Command::new("git")
            .current_dir(cwd)
            .args(args)
            .output()
            .unwrap();
        assert!(out.status.success(), "git {args:?}: {out:?}");
    }

    /// A repository ENCLOSING the CI root must not lend its local config to a
    /// sibling's git: with the dispatch root at
    /// `<enclosing>/ci-root/.ci-worktrees/<id>`, git run from the dispatch root
    /// (where ls-remote and `git init` run) sees no `credential.helper` but the
    /// empty reset, and no `insteadOf` rewrite.
    #[tokio::test]
    async fn an_enclosing_repos_local_config_does_not_reach_sibling_git() {
        let tmp = tempfile::tempdir().unwrap();
        let enclosing = tmp.path().join("enclosing");
        std::fs::create_dir_all(&enclosing).unwrap();
        setup_git(&enclosing, &["init", "-q"]);
        setup_git(&enclosing, &["config", "credential.helper", "store"]);
        setup_git(
            &enclosing,
            &[
                "config",
                "url.git@github.com:.insteadOf",
                "https://github.com/",
            ],
        );
        let dispatch_root = enclosing.join("ci-root").join(".ci-worktrees").join("d-1");
        std::fs::create_dir_all(&dispatch_root).unwrap();
        let env = SiblingGitEnv::isolated(&dispatch_root, None).unwrap();

        let helpers = git_under(
            &env,
            &dispatch_root,
            &["config", "--get-all", "credential.helper"],
        )
        .await;
        // Exactly one value — our empty reset — and nothing after it.
        assert_eq!(
            String::from_utf8_lossy(&helpers.stdout),
            "\n",
            "an enclosing repo's helper reached the sibling: {helpers:?}"
        );
        let url = git_under(
            &env,
            &dispatch_root,
            &["ls-remote", "--get-url", "https://github.com/o/n.git"],
        )
        .await;
        assert_eq!(
            String::from_utf8_lossy(&url.stdout).trim(),
            "https://github.com/o/n.git",
            "an enclosing repo's insteadOf rewrote the sibling URL"
        );
        // And the dispatch root is not inside any repository at all.
        let top = git_under(&env, &dispatch_root, &["rev-parse", "--git-dir"]).await;
        assert!(!top.status.success(), "discovery escaped: {top:?}");
    }

    /// The child's HOME is an EMPTY directory inside the dispatch root, never
    /// the HOME the host gave it (whose `.netrc` libcurl would read), and
    /// `GIT_CONFIG_GLOBAL` still names the global file git reads.
    #[cfg(unix)]
    #[tokio::test]
    async fn the_childs_home_is_an_empty_dir_in_the_dispatch_root() {
        let tmp = tempfile::tempdir().unwrap();
        let owner_home = tmp.path().join("owner-home");
        std::fs::create_dir_all(&owner_home).unwrap();
        std::fs::write(
            owner_home.join(".netrc"),
            "machine github.com login owner password ghp_OWNER\n",
        )
        .unwrap();
        std::fs::write(owner_home.join(".gitconfig"), "[user]\n\tname = owner\n").unwrap();
        let dispatch_root = tmp.path().join("ci").join("d-1");
        let env = SiblingGitEnv::isolated(&dispatch_root, None).unwrap();

        let mut cmd = tokio::process::Command::new("sh");
        // What a host's posture would hand the child.
        cmd.env("HOME", &owner_home)
            .env("XDG_CONFIG_HOME", &owner_home);
        env.isolate_command(&mut cmd);
        let out = cmd
            .args(["-c", "printf '%s' \"$HOME\""])
            .output()
            .await
            .unwrap();
        let seen = PathBuf::from(String::from_utf8_lossy(&out.stdout).into_owned());
        assert_ne!(seen, owner_home);
        assert_eq!(seen, dispatch_root.join(ISOLATED_HOME_DIR));
        assert!(seen.starts_with(&dispatch_root));
        assert_eq!(
            std::fs::read_dir(&seen).unwrap().count(),
            0,
            "HOME is not empty"
        );

        // git's global config is still exactly the per-dispatch file.
        let mut git = tokio::process::Command::new("git");
        git.env("HOME", &owner_home);
        env.isolate_command(&mut git);
        let origin = git
            .current_dir(&dispatch_root)
            .args(["config", "--global", "--list", "--show-origin"])
            .output()
            .await
            .unwrap();
        let listed = String::from_utf8_lossy(&origin.stdout).into_owned();
        assert!(
            listed.contains(&format!(
                "file:{}",
                dispatch_root.join(GLOBAL_CONFIG_FILE).display()
            )),
            "{origin:?}"
        );
        assert!(!listed.contains("owner"), "{listed}");
    }

    /// End to end through libcurl: a loopback git-over-HTTP endpoint that
    /// challenges with `401 Basic`. With the owner's `~/.netrc` naming it, git
    /// under the built env must NOT answer the challenge with the netrc
    /// credential.
    #[cfg(unix)]
    #[tokio::test]
    async fn an_owner_netrc_never_answers_a_sibling_fetch() {
        use tokio::io::{AsyncReadExt, AsyncWriteExt};
        let listener = tokio::net::TcpListener::bind(("127.0.0.1", 0))
            .await
            .unwrap();
        let port = listener.local_addr().unwrap().port();
        let auths = std::sync::Arc::new(std::sync::Mutex::new(Vec::<String>::new()));
        let record = auths.clone();
        tokio::spawn(async move {
            while let Ok((mut conn, _)) = listener.accept().await {
                let record = record.clone();
                tokio::spawn(async move {
                    let mut buf = Vec::new();
                    let mut chunk = [0u8; 4096];
                    while !buf.windows(4).any(|w| w == b"\r\n\r\n") {
                        match conn.read(&mut chunk).await {
                            Ok(0) | Err(_) => return,
                            Ok(n) => buf.extend_from_slice(&chunk[..n]),
                        }
                    }
                    let head = String::from_utf8_lossy(&buf).into_owned();
                    for line in head.lines() {
                        if line.to_ascii_lowercase().starts_with("authorization:") {
                            record.lock().unwrap().push(line.to_string());
                        }
                    }
                    let _ = conn
                        .write_all(
                            b"HTTP/1.1 401 Unauthorized\r\nWWW-Authenticate: Basic realm=\"x\"\r\n\
                              Content-Length: 0\r\nConnection: close\r\n\r\n",
                        )
                        .await;
                    let _ = conn.shutdown().await;
                });
            }
        });

        let tmp = tempfile::tempdir().unwrap();
        let owner_home = tmp.path().join("owner-home");
        std::fs::create_dir_all(&owner_home).unwrap();
        std::fs::write(
            owner_home.join(".netrc"),
            "machine 127.0.0.1 login owner password OWNERnetrcSECRET\n",
        )
        .unwrap();
        let dispatch_root = tmp.path().join("ci").join("d-1");
        let env = SiblingGitEnv::isolated(&dispatch_root, None).unwrap();
        let mut cmd = tokio::process::Command::new("git");
        cmd.env("HOME", &owner_home);
        env.isolate_command(&mut cmd);
        let out = cmd
            .current_dir(&dispatch_root)
            .env_remove("http_proxy")
            .env_remove("HTTP_PROXY")
            .env_remove("all_proxy")
            .env_remove("ALL_PROXY")
            .args(["ls-remote", &format!("http://127.0.0.1:{port}/o/n.git")])
            .output()
            .await
            .unwrap();
        assert!(!out.status.success(), "{out:?}");
        let seen = auths.lock().unwrap().clone();
        assert!(
            seen.is_empty(),
            "the owner's netrc answered the challenge: {seen:?}"
        );
    }
}
