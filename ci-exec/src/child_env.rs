//! The environment a provisioning child starts from: an ALLOWLIST, never the
//! host's own environment.
//!
//! # Why
//!
//! [`crate::tools`] runs third-party code on the host. `pip install` fetches a
//! distribution whose transitive dependencies resolve unhashed, and `verify`
//! EXECUTES every freshly installed tool to read its version. It does that on
//! every cache hit too, so the code runs once per dispatch per tool. Those
//! children used to start from the host process's environment minus five
//! prefixes (`PIP_*`, `PYTHON*`, `POETRY_*`, `NPM_CONFIG_*`, `VIRTUAL_ENV`).
//! That environment belongs to the host: the runner keeps plaintext login
//! passwords in its own, a CI box carries a `GITHUB_TOKEN`, and any box carries
//! whatever its owner exported. A denylist protects only the names someone
//! enumerated; an allowlist protects by construction (plan
//! `2026-11-06-ci-node-provisioning-children-inherit-only-an-allowlisted-environment`,
//! coord memory `6bfac12d-5b6f-49d3-8fdf-85c3412fc333`).
//!
//! So a provisioning child starts from `env_clear()` and gets back only:
//!
//! * [`PROVISIONING_BASE_ENV`]. Each name is there because a named consumer
//!   breaks without it.
//! * [`PROVISIONING_NETWORK_ENV`], the proxy and CA names, for the one child
//!   that fetches (`pip install`) and no other. `verify` runs third-party code,
//!   and a proxy URL can carry a credential.
//! * `PIP_CONFIG_FILE`, set explicitly to the null device (see
//!   `PIP_CONFIG_NULL` below for why the spelling matters).
//! * Of the variables the host's [`crate::host::ProcessSpawn::command`] set
//!   explicitly, only those that pass the same allowlist. That is the
//!   production posture of [`crate::sibling_git`], so the property does not
//!   rest on what a host's spawn helper chooses to set.
//! * The `PATH` prepend and the caller's explicit variables, which
//!   `tools::run_capture_env` applies last.
//!
//! Names compare case-insensitively on Windows only, where the environment
//! itself is case-insensitive (`Path`, `SYSTEMROOT`).
//!
//! # What does not take it
//!
//! [`ChildEnv::ContainerRuntime`] (`docker`/`podman`, which [`crate::services`]
//! runs through the same spawn function) keeps inheriting the host
//! environment. The container CLI is host software resolved from the host's
//! `PATH`, its per-host needs (`DOCKER_HOST`, `DOCKER_CONTEXT`,
//! `XDG_RUNTIME_DIR` for a rootless socket, `HOME`) are unmeasured, and a
//! container receives only the `-e NAME` names its argv lists. The five-prefix
//! removal and `PIP_CONFIG_FILE` used to reach it as well. They target
//! ecosystems docker and podman never read, so they stop applying to it, and
//! that is a recorded non-change.
//!
//! # A closed set
//!
//! No runtime knob adds a name: a configured name would reopen exactly what
//! this closes. An addition is a reviewed change that names its consumer, as
//! for the tool registry. The hermetic e2e,
//! `tests::provisioning_children_see_only_allowlisted_names`, which
//! `.github/workflows/rust-ci.yml` runs on Linux and on Windows, is what adds or
//! drops one.
//!
//! # Residual
//!
//! Only the environment channel closes here. A child still runs as the host's
//! OS user, so it can read that user's files (device-JWT files, `.mcp.json`
//! nonces, cloud credential files) and, on Windows, that user's registry.

use std::ffi::{OsStr, OsString};

/// Which environment a child of the shared spawn function
/// (`tools::run_capture_env`) starts from. Every call names one, so no child
/// gets an environment by default.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChildEnv {
    /// A provisioning child that fetches nothing: interpreter discovery, venv
    /// creation, and every run of an installed tool (`verify`). It gets
    /// [`PROVISIONING_BASE_ENV`] and `PIP_CONFIG_FILE`.
    Provisioning,
    /// The one provisioning child that fetches: `pip install`. It gets
    /// everything [`ChildEnv::Provisioning`] gets plus
    /// [`PROVISIONING_NETWORK_ENV`], so a proxied box still provisions.
    /// `download()` fetches in-process and `--only-binary=:all:` runs no
    /// package code, so this child is the only one the proxy names reach.
    ProvisioningNetwork,
    /// A container-runtime CLI (`docker`, `podman`). It inherits the host
    /// environment untouched (see the module docs).
    ContainerRuntime,
}

/// The names every provisioning child keeps from the host environment, each
/// with the consumer that breaks without it.
///
/// `pub` so a host can test its own credential names against it: none may ever
/// be listed here.
pub const PROVISIONING_BASE_ENV: &[&str] = &[
    // `find_python` probes bare `python3` / `python` / `py`, and npm's Unix
    // `bin/npm` is a `#!/usr/bin/env node` script. `verify` prepends the
    // tool's own bin dir to this.
    "PATH",
    // Python's `expanduser` (pip's and ensurepip's per-user paths) and node's
    // `os.homedir()` (npm's user config): `HOME` on Unix, `USERPROFILE` on
    // Windows.
    "HOME",
    "USERPROFILE",
    // `tempfile` in ensurepip and pip, and node's `os.tmpdir()`. Without them
    // Windows falls back to the current directory for temporary files.
    "TMPDIR",
    "TEMP",
    "TMP",
    // Windows: the owner plan's `python -m venv` failure. CPython's startup
    // and its socket and crypto modules need `SystemRoot`, and pip and npm keep
    // per-user state under the two AppData roots.
    "SystemRoot",
    "APPDATA",
    "LOCALAPPDATA",
    // Windows: `npm.cmd`'s `FOR /F … IN ('…')` runs its inner command through
    // `%ComSpec%`.
    "ComSpec",
    // Windows: cmd.exe resolves a bare command name through it.
    "PATHEXT",
    // Output encoding of every child whose version text `verify` parses.
    "LANG",
    "LC_ALL",
    "LC_CTYPE",
    // Linux: a loader path, not a credential. The CPython that
    // `actions/setup-python` installs is built `--enable-shared` with an rpath
    // naming its build-time tool cache, and setup-python exports
    // `LD_LIBRARY_PATH=<installDir>/lib` so that the interpreter loads from a
    // tool cache anywhere else, as on a self-hosted pool.
    #[cfg(target_os = "linux")]
    "LD_LIBRARY_PATH",
];

/// The proxy and CA names, for [`ChildEnv::ProvisioningNetwork`] only: pip
/// (through requests/urllib3) honours both spellings of each proxy variable,
/// `REQUESTS_CA_BUNDLE` and `CURL_CA_BUNDLE`, and the OpenSSL default paths
/// `SSL_CERT_FILE` / `SSL_CERT_DIR`. Without them a box behind a mandatory
/// proxy or a private CA could no longer provision, which it can today.
/// `PIP_*` stays out: pip's configuration is the registry's, not the host's.
///
/// `pub` for the same reason as [`PROVISIONING_BASE_ENV`].
pub const PROVISIONING_NETWORK_ENV: &[&str] = &[
    "HTTP_PROXY",
    "HTTPS_PROXY",
    "NO_PROXY",
    "ALL_PROXY",
    "http_proxy",
    "https_proxy",
    "no_proxy",
    "all_proxy",
    "SSL_CERT_FILE",
    "SSL_CERT_DIR",
    "REQUESTS_CA_BUNDLE",
    "CURL_CA_BUNDLE",
];

/// Set on every provisioning child, with [`PIP_CONFIG_NULL`] as its value.
const PIP_CONFIG_FILE: &str = "PIP_CONFIG_FILE";

/// Not inheriting `PIP_CONFIG_FILE` does NOT disable pip's config. It makes
/// pip fall back to its defaults (`/etc/pip.conf`, `~/.pip/pip.conf`,
/// `~/.config/pip/pip.conf`, `%APPDATA%\pip\pip.ini`). `--index-url` overrides
/// a config `index-url`, but `extra-index-url`, `find-links`, `trusted-host`
/// and `pre` are ADDITIVE and survive, so a box with an internal mirror
/// configured would still have a dependency-confusion surface. Setting the
/// variable to the null device is the documented way to make pip load no
/// config file at all.
///
/// THE SPELLING IS LOAD-BEARING AND MUST BE LOWERCASE. pip's skip is an exact
/// string comparison, not a path resolution
/// (`_internal/configuration.py::_load_config_files`):
///
/// ```text
/// if config_files[kinds.ENV][0:1] == [os.devnull]:
///     return   # this return is what skips GLOBAL and SITE too
/// ```
///
/// and `os.devnull` on Windows is `'nul'`, lowercase (`ntpath.py`).
/// `"NUL" != "nul"`, so an uppercase value never fires that return: measured
/// with this repo's own Python, `PIP_CONFIG_FILE=NUL` gives `devnull match:
/// False` and still loads the GLOBAL and SITE configs, i.e. exactly the
/// additive `extra-index-url` / `find-links` / `trusted-host` this is here to
/// neutralise. Only the USER config gets suppressed, incidentally, because
/// `iter_config_files` drops USER whenever the variable is set at all. With
/// `nul` the match is True and nothing is loaded. POSIX `/dev/null` matches
/// exactly and is fine.
const PIP_CONFIG_NULL: &str = if cfg!(target_os = "windows") {
    "nul"
} else {
    "/dev/null"
};

/// Does `name` pass `policy`'s allowlist? Exact on Unix, case-insensitive on
/// Windows. [`ChildEnv::ContainerRuntime`] has no allowlist, because its
/// children inherit and nothing is filtered, so it admits nothing here.
pub(crate) fn admits(policy: ChildEnv, name: &OsStr) -> bool {
    let lists: &[&[&str]] = match policy {
        ChildEnv::Provisioning => &[PROVISIONING_BASE_ENV],
        ChildEnv::ProvisioningNetwork => &[PROVISIONING_BASE_ENV, PROVISIONING_NETWORK_ENV],
        ChildEnv::ContainerRuntime => return false,
    };
    // A name that is not UTF-8 cannot equal an ASCII allowlist entry.
    let Some(name) = name.to_str() else {
        return false;
    };
    lists
        .iter()
        .flat_map(|list| list.iter())
        .any(|allowed| same_name(allowed, name))
}

fn same_name(allowed: &str, name: &str) -> bool {
    if cfg!(target_os = "windows") {
        allowed.eq_ignore_ascii_case(name)
    } else {
        allowed == name
    }
}

/// The environment a child under `policy` starts from, given the host's
/// `inherited` one. Pure, so the allowlist is testable without spawning.
///
/// `None` means "inherit the host environment untouched"
/// ([`ChildEnv::ContainerRuntime`]). Otherwise the result is the WHOLE
/// environment: the inherited names `policy` admits, then `PIP_CONFIG_FILE`
/// with its explicit value. An inherited `PIP_CONFIG_FILE` is not admitted, so
/// the explicit value is the only one.
pub fn provisioning_env(
    policy: ChildEnv,
    inherited: impl IntoIterator<Item = (OsString, OsString)>,
) -> Option<Vec<(OsString, OsString)>> {
    if policy == ChildEnv::ContainerRuntime {
        return None;
    }
    let mut env: Vec<(OsString, OsString)> = inherited
        .into_iter()
        .filter(|(name, _)| admits(policy, name))
        .collect();
    env.push((PIP_CONFIG_FILE.into(), PIP_CONFIG_NULL.into()));
    Some(env)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::host::{ProcessSpawn, StepContainment, TreeGuard};
    use crate::manifest::CiTool;
    use crate::package_index::{overlay, PackageIndex};
    use crate::standalone::PlainSpawn;
    use crate::tools::{Installer, ToolSpec};
    use sha2::Digest;
    use std::io::Write;
    use std::path::{Path, PathBuf};
    use std::time::Duration;

    // ── Literals: what these tests hold the code to ──────────────────────
    //
    // Spelled out, never read from the module under test, so a name added to
    // or dropped from an allowlist is a diff these tests see.

    /// [`PROVISIONING_BASE_ENV`], spelled out.
    const BASE: &[&str] = &[
        "PATH",
        "HOME",
        "USERPROFILE",
        "TMPDIR",
        "TEMP",
        "TMP",
        "SystemRoot",
        "APPDATA",
        "LOCALAPPDATA",
        "ComSpec",
        "PATHEXT",
        "LANG",
        "LC_ALL",
        "LC_CTYPE",
        #[cfg(target_os = "linux")]
        "LD_LIBRARY_PATH",
    ];

    /// [`PROVISIONING_NETWORK_ENV`], spelled out.
    const NETWORK: &[&str] = &[
        "HTTP_PROXY",
        "HTTPS_PROXY",
        "NO_PROXY",
        "ALL_PROXY",
        "http_proxy",
        "https_proxy",
        "no_proxy",
        "all_proxy",
        "SSL_CERT_FILE",
        "SSL_CERT_DIR",
        "REQUESTS_CA_BUNDLE",
        "CURL_CA_BUNDLE",
    ];

    /// What `PIP_CONFIG_FILE` must read on this platform.
    const PIP_CONFIG_NULL_HERE: &str = if cfg!(target_os = "windows") {
        "nul"
    } else {
        "/dev/null"
    };

    /// The environment's own name comparison, written here rather than
    /// borrowed from the module under test: case-insensitive on Windows only.
    fn same(a: &str, b: &str) -> bool {
        if cfg!(target_os = "windows") {
            a.eq_ignore_ascii_case(b)
        } else {
            a == b
        }
    }

    fn listed(names: &[String], name: &str) -> bool {
        names.iter().any(|n| same(n, name))
    }

    /// The names in `names` that `allowed` does not list.
    fn outside(names: &[String], allowed: &[&str]) -> Vec<String> {
        names
            .iter()
            .filter(|n| !allowed.iter().any(|a| same(a, n)))
            .cloned()
            .collect()
    }

    fn os(pairs: &[(&str, &str)]) -> Vec<(OsString, OsString)> {
        pairs
            .iter()
            .map(|(k, v)| (OsString::from(k), OsString::from(v)))
            .collect()
    }

    fn names_of(env: &[(OsString, OsString)]) -> Vec<String> {
        env.iter()
            .map(|(k, _)| k.to_string_lossy().into_owned())
            .collect()
    }

    // ── The pure allowlist ───────────────────────────────────────────────

    /// The two lists are closed sets: adding or dropping a name means
    /// editing this test, which is where the reviewed change shows.
    #[test]
    fn the_allowlists_are_the_reviewed_closed_sets() {
        assert_eq!(PROVISIONING_BASE_ENV, BASE);
        assert_eq!(PROVISIONING_NETWORK_ENV, NETWORK);
    }

    /// The credential class and the ecosystem redirections are gone under
    /// both provisioning policies; the proxy and CA names survive only where
    /// pip fetches; a container runtime inherits.
    #[test]
    fn provisioning_env_keeps_only_the_allowlist() {
        // Each of these must never reach a provisioning child.
        const NEVER: &[&str] = &[
            // qontinui-runner `terminal/mod.rs` `CREDENTIAL_VALUE_ENV_VARS`
            // (runner origin/main 21116fe87): the runner's plaintext logins.
            "QONTINUI_OPERATOR2_PASSWORD",
            "QONTINUI_TEST_LOGIN_PASSWORD",
            "QONTINUI_TEST_AUTO_LOGIN_PASSWORD",
            "GITHUB_TOKEN",
            "AWS_SECRET_ACCESS_KEY",
            // A `--require` hook into npm's verification.
            "NODE_OPTIONS",
            // Redirect where a distribution comes from, or which modules a
            // verification imports.
            "PIP_INDEX_URL",
            "PYTHONPATH",
            "VIRTUAL_ENV",
            "npm_config_prefix",
            "QONTINUI_CANARY_PASSWORD",
        ];
        let mut inherited: Vec<(&str, &str)> = NEVER.iter().map(|name| (*name, "x")).collect();
        inherited.extend([
            // Replaced by the explicit null-device value, never passed on.
            ("PIP_CONFIG_FILE", "/home/owner/.config/pip/pip.conf"),
            ("PATH", "/usr/bin"),
            ("HOME", "/home/owner"),
            ("LANG", "C.UTF-8"),
            ("HTTPS_PROXY", "http://proxy.invalid:3128"),
            ("no_proxy", "localhost"),
            ("SSL_CERT_FILE", "/etc/ssl/certs/ca.pem"),
            // Windows spellings of allowlisted names.
            ("Path", "C:\\Windows\\system32"),
            ("SYSTEMROOT", "C:\\Windows"),
        ]);

        assert!(
            provisioning_env(ChildEnv::ContainerRuntime, os(&inherited)).is_none(),
            "a container runtime inherits the host environment untouched"
        );
        let plain = provisioning_env(ChildEnv::Provisioning, os(&inherited))
            .expect("a provisioning policy builds an environment");
        let network = provisioning_env(ChildEnv::ProvisioningNetwork, os(&inherited))
            .expect("a provisioning policy builds an environment");

        for (label, env) in [("Provisioning", &plain), ("ProvisioningNetwork", &network)] {
            let names = names_of(env);
            for never in NEVER {
                assert!(!listed(&names, never), "{label} kept {never}");
            }
            for kept in ["PATH", "HOME", "LANG"] {
                assert!(listed(&names, kept), "{label} dropped {kept}");
            }
            let pip_config: Vec<&OsString> = env
                .iter()
                .filter(|(name, _)| name == "PIP_CONFIG_FILE")
                .map(|(_, value)| value)
                .collect();
            assert_eq!(
                pip_config,
                vec![&OsString::from(PIP_CONFIG_NULL_HERE)],
                "{label}: PIP_CONFIG_FILE must be set once, to the null device"
            );
            let windows_spellings: Vec<&String> = names
                .iter()
                .filter(|n| *n == "Path" || *n == "SYSTEMROOT")
                .collect();
            if cfg!(target_os = "windows") {
                assert_eq!(windows_spellings.len(), 2, "{label}: {names:?}");
            } else {
                assert!(windows_spellings.is_empty(), "{label}: {names:?}");
            }
        }
        let (plain, network) = (names_of(&plain), names_of(&network));
        for name in ["HTTPS_PROXY", "no_proxy", "SSL_CERT_FILE"] {
            assert!(!listed(&plain, name), "Provisioning kept {name}");
            assert!(listed(&network, name), "ProvisioningNetwork dropped {name}");
        }
    }

    // ── A real child, through the real spawn function ────────────────────

    /// A child that prints the NAMES of its environment, one per line, and
    /// never a value: this runs on boxes whose environment holds plaintext
    /// passwords, and a mutation run spawns it without `env_clear()`. On Unix
    /// awk lists `ENVIRON`'s keys (`env | cut -d= -f1` would print a line of
    /// a multi-line value). On Windows a `.cmd` echoes the text before the
    /// first `=` of each `set` line.
    fn names_printer(dir: &Path) -> (String, Vec<String>) {
        if cfg!(target_os = "windows") {
            let script = dir.join("print-env-names.cmd");
            std::fs::write(
                &script,
                "@echo off\r\nfor /f \"delims==\" %%v in ('set') do @echo %%v\r\n",
            )
            .expect("write the names printer");
            (script.to_string_lossy().into_owned(), Vec::new())
        } else {
            (
                "awk".to_string(),
                vec!["BEGIN { for (k in ENVIRON) print k }".to_string()],
            )
        }
    }

    /// What the names printer itself puts into the environment it lists.
    const PRINTER_INJECTED: &[&str] = &[
        // gawk puts both into `ENVIRON` (per its manual). Not measured here:
        // this box's awk is mawk, which adds nothing (measured 2026-11-07,
        // `env -i awk 'BEGIN { for (k in ENVIRON) print k }'` prints nothing).
        #[cfg(not(target_os = "windows"))]
        "AWKPATH",
        #[cfg(not(target_os = "windows"))]
        "AWKLIBPATH",
        // cmd.exe defines it when it is absent.
        #[cfg(target_os = "windows")]
        "PROMPT",
    ];

    /// The names a [`ChildEnv::Provisioning`] child of `process` sees.
    async fn names_seen_by(process: &dyn ProcessSpawn, envs: &[(String, String)]) -> Vec<String> {
        let dir = tempfile::tempdir().expect("tempdir");
        let (program, args) = names_printer(dir.path());
        let args: Vec<&str> = args.iter().map(String::as_str).collect();
        // The child prints names only, so even the error text carries none.
        let out = crate::tools::run_capture_env(
            process,
            ChildEnv::Provisioning,
            &program,
            &args,
            Duration::from_secs(60),
            None,
            envs,
        )
        .await
        .expect("the names printer must run");
        out.lines()
            .map(str::trim)
            .filter(|line| !line.is_empty())
            .map(str::to_string)
            .collect()
    }

    /// The real spawn function: a provisioning child sees only allowlisted
    /// names, and still receives a variable its caller passed explicitly.
    #[tokio::test]
    async fn a_provisioning_child_sees_only_allowlisted_names() {
        const EXPLICIT: &str = "QONTINUI_EXPLICIT_ENV_PROBE";
        let names = names_seen_by(&PlainSpawn, &[(EXPLICIT.to_string(), "1".to_string())]).await;
        let mut allowed: Vec<&str> = BASE.to_vec();
        allowed.extend(["PIP_CONFIG_FILE", EXPLICIT]);
        allowed.extend(PRINTER_INJECTED);
        let unexpected = outside(&names, &allowed);
        assert!(
            unexpected.is_empty(),
            "a provisioning child saw names outside the allowlist: {unexpected:?}"
        );
        for required in ["PATH", "PIP_CONFIG_FILE", EXPLICIT] {
            assert!(
                listed(&names, required),
                "{required} did not reach the child, which saw {names:?}"
            );
        }
    }

    /// A host whose `command` sets two variables of its own on every command:
    /// one allowlisted name, and one `*_PASSWORD`.
    struct HostSetsVariables {
        allowlisted: &'static str,
    }

    const HOST_SET_PASSWORD: &str = "QONTINUI_HOST_SET_PASSWORD";

    impl ProcessSpawn for HostSetsVariables {
        fn command(&self, program: &str) -> tokio::process::Command {
            let mut cmd = PlainSpawn.command(program);
            cmd.env(self.allowlisted, "set-by-the-host")
                .env(HOST_SET_PASSWORD, "not-a-real-secret");
            cmd
        }
        fn std_command(&self, program: &str) -> std::process::Command {
            PlainSpawn.std_command(program)
        }
        fn arm_tree(&self, cmd: &mut tokio::process::Command) {
            PlainSpawn.arm_tree(cmd)
        }
        fn attach_tree(&self, child: &tokio::process::Child) -> Box<dyn TreeGuard> {
            PlainSpawn.attach_tree(child)
        }
        fn step_containment(&self) -> Box<dyn StepContainment> {
            PlainSpawn.step_containment()
        }
    }

    /// What a host's `ProcessSpawn::command` sets explicitly passes the same
    /// allowlist as what the process inherits: the allowlisted name survives,
    /// the `*_PASSWORD` does not. That holds for every implementer of the
    /// public trait, present and future.
    #[tokio::test]
    async fn a_variable_the_host_set_survives_only_if_allowlisted() {
        // An allowlisted name this process does not carry, so its presence in
        // the child can only be the host's doing.
        let allowlisted = BASE
            .iter()
            .copied()
            .find(|name| std::env::var_os(name).is_none())
            .expect(
                "every allowlisted name is set in this environment, so a host-set one \
                 cannot be told apart from an inherited one",
            );
        let names = names_seen_by(&HostSetsVariables { allowlisted }, &[]).await;
        assert!(
            listed(&names, allowlisted),
            "the host set {allowlisted}, which the allowlist admits, and the child did not see it"
        );
        assert!(
            !listed(&names, HOST_SET_PASSWORD),
            "the host set {HOST_SET_PASSWORD}, which the allowlist refuses, and the child saw it"
        );
    }

    // ── The convention in tools.rs ───────────────────────────────────────

    /// `tools.rs` names only the two provisioning policies, and hands the
    /// network names to the `pip install` child alone: no other provisioning
    /// child, and so no third-party `--version`, ever gets the proxy names.
    #[test]
    fn only_pip_install_is_handed_the_network_names() {
        let source = include_str!("tools.rs").replace("\r\n", "\n");
        let production = source
            .split("\n#[cfg(test)]\nmod tests {")
            .next()
            .unwrap_or_default();
        assert!(
            production.len() < source.len(),
            "tools.rs has no `#[cfg(test)] mod tests` to split off, so this would scan its tests too"
        );
        assert_eq!(
            production.matches("ContainerRuntime").count(),
            0,
            "a container-runtime child is `services`' business; tools.rs must not name one outside its tests"
        );
        let hits: Vec<usize> = production
            .match_indices("ChildEnv::ProvisioningNetwork")
            .map(|(at, _)| at)
            .collect();
        assert_eq!(
            hits.len(),
            1,
            "ChildEnv::ProvisioningNetwork must appear exactly once outside tools.rs's tests, at the pip install"
        );
        let before = &production[..hits[0]];
        let item = [
            "\nfn ",
            "\nasync fn ",
            "\npub(crate) fn ",
            "\npub(crate) async fn ",
        ]
        .iter()
        .filter_map(|start| before.rfind(start))
        .max()
        .expect("the policy is passed inside a function");
        let signature = before[item..]
            .trim_start()
            .lines()
            .next()
            .unwrap_or_default();
        assert!(
            signature.contains("fn install_python_package("),
            "ChildEnv::ProvisioningNetwork is passed in `{signature}`, not in install_python_package"
        );
        let call_start = before.rfind("run_capture(").expect("passed to run_capture");
        let call_end = hits[0]
            + production[hits[0]..]
                .find(".await")
                .expect("the call is awaited");
        assert!(
            production[call_start..call_end].contains("&pip_argv"),
            "the network policy must ride the `pip install` call itself"
        );
        let body = &production[item..call_start];
        assert!(
            body.contains("\"pip\",") && body.contains("\"install\","),
            "pip_argv must be the `-m pip install` argv"
        );
    }

    // ── The end-to-end proof ─────────────────────────────────────────────

    const PROBE: &str = "qontinui-env-probe";
    const PROBE_VERSION: &str = "1.0.0";

    /// The probe's `__main__`. On `--version` it records the NAMES of the
    /// environment it was started with, never a value, in
    /// `<sys.prefix>/seen-env-names.txt`, then prints the version `verify`
    /// checks. `sys.prefix` is the probe's private venv.
    const PROBE_MAIN: &str = r#"import os
import sys

if sys.argv[1:] != ["--version"]:
    sys.exit("usage: qontinui-env-probe --version")
with open(os.path.join(sys.prefix, "seen-env-names.txt"), "w", encoding="utf-8") as seen:
    seen.write("".join(name + "\n" for name in sorted(os.environ)))
print("qontinui-env-probe 1.0.0")
"#;

    /// Unpadded URL-safe base64, the digest encoding a wheel's `RECORD` uses.
    fn urlsafe_b64_nopad(bytes: &[u8]) -> String {
        const ALPHABET: &[u8; 64] =
            b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789-_";
        let mut out = String::new();
        for chunk in bytes.chunks(3) {
            let b = [
                chunk[0],
                chunk.get(1).copied().unwrap_or(0),
                chunk.get(2).copied().unwrap_or(0),
            ];
            let n = (u32::from(b[0]) << 16) | (u32::from(b[1]) << 8) | u32::from(b[2]);
            for i in 0..=chunk.len() {
                out.push(char::from(ALPHABET[((n >> (18 - 6 * i)) & 63) as usize]));
            }
        }
        out
    }

    /// Build the probe as a pure-Python wheel in `dir`, so the e2e installs it
    /// with `--no-index --find-links` and never touches the network.
    fn write_probe_wheel(dir: &Path) {
        let dist_info = "qontinui_env_probe-1.0.0.dist-info";
        let mut files: Vec<(String, Vec<u8>)> = vec![
            ("qontinui_env_probe/__init__.py".to_string(), Vec::new()),
            (
                "qontinui_env_probe/__main__.py".to_string(),
                PROBE_MAIN.as_bytes().to_vec(),
            ),
            (
                format!("{dist_info}/METADATA"),
                b"Metadata-Version: 2.1\nName: qontinui-env-probe\nVersion: 1.0.0\n".to_vec(),
            ),
            (
                format!("{dist_info}/WHEEL"),
                b"Wheel-Version: 1.0\nGenerator: qontinui-ci-exec-tests\nRoot-Is-Purelib: true\nTag: py3-none-any\n"
                    .to_vec(),
            ),
        ];
        let mut record = String::new();
        for (name, body) in &files {
            let digest = sha2::Sha256::digest(body);
            record.push_str(&format!(
                "{name},sha256={},{}\n",
                urlsafe_b64_nopad(&digest),
                body.len()
            ));
        }
        record.push_str(&format!("{dist_info}/RECORD,,\n"));
        files.push((format!("{dist_info}/RECORD"), record.into_bytes()));

        std::fs::create_dir_all(dir).expect("create the wheel dir");
        let wheel = std::fs::File::create(dir.join("qontinui_env_probe-1.0.0-py3-none-any.whl"))
            .expect("create the wheel");
        let mut zip = zip::ZipWriter::new(wheel);
        for (name, body) in &files {
            zip.start_file(name.as_str(), zip::write::SimpleFileOptions::default())
                .expect("start a wheel entry");
            zip.write_all(body).expect("write a wheel entry");
        }
        zip.finish().expect("finish the wheel");
    }

    /// The directory of the first `node` on this process's `PATH`.
    fn host_node_dir() -> Option<PathBuf> {
        let path = std::env::var_os("PATH")?;
        let node = format!("node{}", std::env::consts::EXE_SUFFIX);
        std::env::split_paths(&path).find(|dir| dir.join(&node).is_file())
    }

    /// Names the verify child's own interpreters inject, which no allowlist
    /// can see. Measured, never guessed; names only.
    const VERIFY_INJECTED: &[&str] = &[
        // dash (`/bin/sh` on Debian and Ubuntu) runs the probe's `#!/bin/sh`
        // shim and exports PWD to the python it execs. Measured 2026-11-07 on
        // Debian 13: `env -i /bin/sh -c env` prints only `PWD`, and the shim
        // run under `env -i` hands the probe `LC_CTYPE`, `PATH`,
        // `PIP_CONFIG_FILE` and `PWD`. (`LC_CTYPE` is CPython's PEP 538
        // locale coercion when no locale variable is set, and is already a
        // base name.)
        #[cfg(not(target_os = "windows"))]
        "PWD",
        // Windows: NOT measured, because no Windows box was reachable; the
        // `ci-exec-windows` step's first run is the measurement, and its
        // `--show-output` lines list what the child saw. cmd.exe, which runs
        // the `.cmd` shim, defines PROMPT when it is absent, and a venv's
        // `python.exe` is a redirector that hands the base interpreter
        // `__PYVENV_LAUNCHER__`.
        #[cfg(target_os = "windows")]
        "PROMPT",
        #[cfg(target_os = "windows")]
        "__PYVENV_LAUNCHER__",
    ];

    /// THE END-TO-END PROOF that a provisioning child sees only allowlisted
    /// names. It enters through `tools::provision`, as the executor does
    /// (`executor.rs`, `run_dispatch`), with a probe distribution installed
    /// from a local wheel through the test-only registry overlay. The probe
    /// records the names its verify child saw, on the install (verify runs in
    /// staging, and the record moves into place with the rename) and on a
    /// cache hit (verify runs in place). The record is deleted before each
    /// call, so a cache-hit verify that stopped running the probe cannot pass
    /// on the stale file.
    ///
    /// Two canaries must be set in this test's own environment, and the test
    /// fails without them: `HTTPS_PROXY` (pointing at a closed loopback port,
    /// harmless under `--no-index`) and `QONTINUI_E2E_CANARY_PASSWORD`. Neither
    /// may reach the verify child. `rust-ci.yml` runs it with both, on Linux
    /// and on Windows, and so does the implementer's box.
    ///
    /// The node arm runs `verify_installed` over the host's own node, with its
    /// npm and npx companions, so the allowlist is shown not to break them.
    #[tokio::test]
    #[ignore = "needs a host Python 3 with venv and a host node; rust-ci.yml runs it on Linux and Windows with its two canaries set"]
    async fn provisioning_children_see_only_allowlisted_names() {
        const CANARY_PROXY: &str = "HTTPS_PROXY";
        const CANARY_PASSWORD: &str = "QONTINUI_E2E_CANARY_PASSWORD";
        for canary in [CANARY_PROXY, CANARY_PASSWORD] {
            assert!(
                std::env::var_os(canary).is_some(),
                "{canary} is not set in this test's environment, so its absence from the \
                 child would prove nothing. Set it, as rust-ci.yml's step does"
            );
        }

        let tmp = tempfile::tempdir().expect("tempdir");
        let wheels = tmp.path().join("wheels");
        write_probe_wheel(&wheels);
        let wheels: &'static str = Box::leak(
            wheels
                .to_str()
                .expect("a UTF-8 temp path")
                .to_string()
                .into_boxed_str(),
        );
        overlay::register(ToolSpec {
            name: PROBE,
            verify_args: &["--version"],
            install: Installer::PythonPackage {
                distribution: PROBE,
                module: "qontinui_env_probe",
                command: PROBE,
                index: PackageIndex::LocalWheels(wheels),
            },
        });

        let root = tmp.path().join("ci-root");
        let tool_dir = crate::tools::tool_dir(&root, PROBE, PROBE_VERSION);
        let seen = tool_dir.join("venv").join("seen-env-names.txt");
        let tools = [CiTool {
            name: PROBE.to_string(),
            version: PROBE_VERSION.to_string(),
        }];
        let mut allowed: Vec<&str> = BASE.to_vec();
        allowed.push("PIP_CONFIG_FILE");
        allowed.extend(VERIFY_INJECTED);

        for (call, path_taken) in [("install", "— installed at"), ("cache hit", "— cache hit")]
        {
            match std::fs::remove_file(&seen) {
                Ok(()) => {}
                Err(e) if e.kind() == std::io::ErrorKind::NotFound => {}
                Err(e) => panic!("{call}: could not clear {}: {e}", seen.display()),
            }
            let mut log: Vec<String> = Vec::new();
            let dirs =
                crate::tools::provision(&PlainSpawn, &root, &tools, &mut |line| log.push(line))
                    .await
                    .unwrap_or_else(|e| panic!("{call}: provisioning the probe failed: {e}"));
            assert_eq!(dirs, vec![tool_dir.join("bin")], "{call}");
            assert!(
                log.iter().any(|line| line.contains(path_taken)),
                "{call}: provision did not take the {call} path: {log:?}"
            );
            let names: Vec<String> = std::fs::read_to_string(&seen)
                .unwrap_or_else(|e| {
                    panic!(
                        "{call}: the verify child did not record its names at {} ({e}): \
                         verify did not run the probe",
                        seen.display()
                    )
                })
                .lines()
                .map(str::to_string)
                .collect();
            // Names only: this line is the measurement a CI log carries.
            println!("{call}: the verify child saw {names:?}");

            // The canaries first, so a failure names the one that leaked: the
            // password means no allowlist was applied at all, the proxy means
            // verify was handed the network names.
            for canary in [CANARY_PASSWORD, CANARY_PROXY] {
                assert!(
                    !listed(&names, canary),
                    "{call}: the canary {canary} reached the verify child"
                );
            }
            // Upper bound: nothing outside the literal allowed set.
            let unexpected = outside(&names, &allowed);
            assert!(
                unexpected.is_empty(),
                "{call}: the verify child saw names outside the allowlist: {unexpected:?}"
            );
            // Lower bound: the child came through `build`'s provisioning arm.
            for required in ["PATH", "PIP_CONFIG_FILE"] {
                assert!(
                    listed(&names, required),
                    "{call}: {required} did not reach the verify child, which saw {names:?}"
                );
            }
        }

        // The node arm.
        let node_dir = host_node_dir()
            .expect("no `node` on PATH (rust-ci.yml installs one with actions/setup-node)");
        let node = node_dir.join(format!("node{}", std::env::consts::EXE_SUFFIX));
        let reported = std::process::Command::new(&node)
            .arg("--version")
            .output()
            .expect("run the host's node");
        let version = String::from_utf8_lossy(&reported.stdout)
            .trim()
            .trim_start_matches('v')
            .to_string();
        let reports = crate::tools::verify_host_node(&PlainSpawn, &node_dir, &version)
            .await
            .unwrap_or_else(|e| {
                panic!(
                    "node {version} at {} failed verification under the allowlist: {e}",
                    node_dir.display()
                )
            });
        println!("node {version}: {reports:?}");
        for companion in ["npm ", "npx "] {
            assert!(
                reports.iter().any(|r| r.starts_with(companion)),
                "node's companion {companion}did not verify: {reports:?}"
            );
        }
    }
}
