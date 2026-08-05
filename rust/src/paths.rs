//! Workspace-root resolution — the **single** answer to "where do the Qontinui
//! repo checkouts live on this box?".
//!
//! Plan: `2026-08-04-remove-hardcoded-machine-paths-from-product-code.md`
//! (slice 1). Qontinui is open source; a shipped binary must not carry the
//! author's machine layout. Before this module the runner carried **four
//! byte-similar copies** of the resolution (`census.rs`, `fleet.rs`,
//! `coord_mcp.rs`, `agent_runtime.rs`), each with a hardcoded
//! `D:/qontinui-root` arm, plus a fifth, differently-shaped answer in
//! `mcp/code_semantics/scope.rs`. This is the one that replaces them.
//!
//! It lives in `qontinui-types` because that crate is a path dependency of
//! **both** Rust consumers (qontinui-runner and qontinui-coord), so one helper
//! genuinely serves both. It deliberately pulls in no new dependency: the home
//! directory is read from the environment rather than via `dirs`, keeping a
//! widely-shared wire-format crate dependency-stable.
//!
//! ## Two things this deliberately is NOT
//!
//! Both rules are quoted from the runner's own `env_agent/collectors.rs`, which
//! already got this right and states the principle authoritatively:
//!
//! - **Deliberately NOT the compile-time `CARGO_MANIFEST_DIR`.** *"That path
//!   measures which source tree the binary was BUILT from, not the box."* A
//!   macro-expanded manifest dir is the hardcoded machine path wearing a
//!   macro's clothes — invisible to any grep for the literal.
//! - **Deliberately NOT the inherited cwd**, and not `current_exe()` either.
//!   The cwd makes resolution a function of how the process was launched; an
//!   installed binary in `Program Files` has no useful ancestry, and
//!   `current_exe()` resolves through symlinks (the workspace `.claude` symlink
//!   trap that the shell `resolve_workspace_root` idiom documents). Callers that
//!   have a *meaningful* anchor pass one explicitly; callers that do not pass
//!   `None` and skip that rung.
//!
//! ## Which anchor should a caller pass?
//!
//! **Passing an anchor is a judgement, not a default.** The table below is the
//! recommendation per consumer, so a caller never reaches for `current_exe()`
//! merely to "make the ancestor-walk rung do something":
//!
//! | Caller | Anchor | Why |
//! |---|---|---|
//! | The runner's process-wide resolver (`workspace_paths.rs`) | `current_exe()` | A dev install runs out of the checkout, so its ancestry genuinely names the root. Resolved **once**, not per call site. |
//! | Runner discovery surfaces (census, fleet publisher, `.mcp.json` reconcile) | *(inherit the above)* | They are library code with no ancestry of their own. They must not synthesize one. |
//! | qontinui-coord | `None` | It runs as a service, not out of a checkout. It configures the root explicitly. |
//! | Anything given a repo path already | that path | Cheapest correct anchor there is. |
//!
//! ## Why a struct and not a `Result`
//!
//! [`WorkspaceRoot`] mirrors `env_agent/collectors.rs`'s
//! `ProbeScope { root, rejected }` shape, field-for-field. An unusable
//! *configured* value **falls through** rather than failing — but it is
//! *reported*, so the fall-through is never invisible. That lets each caller
//! pick its own disposition (discovery surfaces degrade; write/execute surfaces
//! fail closed with an error naming the variable) while making a silent miss
//! impossible at either. A bare `Result` would force every discovery call site
//! into an `.ok()` that discards the reason.
//!
//! ## Naming
//!
//! [`qontinui_workspace_root`], **not** `workspace_root`. Two functions already
//! carry the bare name inside qontinui-runner with *different* meanings —
//! `mcp/code_semantics/scope.rs` (sibling-repo root) and
//! `wrappers/launcher.rs` (cargo workspace root). A third ambiguous
//! `workspace_root` would make that collision worse, and one unambiguous answer
//! is the entire point.

use std::path::{Component, Path, PathBuf};

/// Canonical environment variable naming the workspace root.
pub const QONTINUI_ROOT_ENV: &str = "QONTINUI_ROOT";

/// Recognised alias for [`QONTINUI_ROOT_ENV`], checked second.
///
/// `qontinui-runner`'s code-semantics module shipped this name as its own
/// private answer to the same question. It is honoured here so folding that
/// answer in breaks nobody who set it. One name is canonical; the alias is
/// honoured, documented, and never grows.
pub const QONTINUI_WORKSPACE_ROOT_ENV: &str = "QONTINUI_WORKSPACE_ROOT";

/// Directory name probed under the user's home directory as the last portable
/// fallback.
pub const HOME_WORKSPACE_DIR: &str = "qontinui-root";

/// Human-readable statement of the resolution order, embedded in the
/// fail-closed error so an operator sees every probe without reading the source.
const PROBE_ORDER: &str = "resolution order: $QONTINUI_ROOT, \
                           $QONTINUI_WORKSPACE_ROOT, the application's \
                           configured workspace root, an ancestor walk from a \
                           caller-supplied anchor (skipped when the caller has \
                           none), then <home>/qontinui-root";

/// Which input produced a rejected value, so an error names the knob the
/// operator actually set wrong.
///
/// Without this, a stale application setting produces "set `$QONTINUI_ROOT`" —
/// pointing at a variable the operator never touched.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RootSource {
    /// [`QONTINUI_ROOT_ENV`].
    Env,
    /// [`QONTINUI_WORKSPACE_ROOT_ENV`], the recognised alias.
    EnvAlias,
    /// The host application's own setting — the runner's `paths.workspace_root`.
    Configured,
    /// No explicit value was at fault; every probe simply missed.
    Probes,
}

impl RootSource {
    /// How to name this input to an operator.
    pub fn describe(self) -> &'static str {
        match self {
            RootSource::Env => "$QONTINUI_ROOT",
            RootSource::EnvAlias => "$QONTINUI_WORKSPACE_ROOT",
            RootSource::Configured => "the configured workspace-root setting",
            RootSource::Probes => "workspace-root resolution",
        }
    }
}

/// Why a candidate workspace root was not usable.
///
/// Variant names mirror `env_agent/collectors.rs`'s `ScopeRootRejection`
/// (`Blank` / `Relative` / `NotADirectory`) so the two rejection vocabularies
/// read the same, plus the three this resolver adds for its extra rungs.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RootRejection {
    /// The value was present but empty or whitespace-only.
    Blank,
    /// The value was present but relative — the inherited cwd wearing a
    /// configured value's clothes.
    Relative,
    /// The value was absolute but is not an existing directory.
    NotADirectory,
    /// The value is a filesystem root (`/`, `C:\`) or a bare UNC share.
    ///
    /// Rejected because the surfaces this feeds **create directories** under the
    /// root: accepting `/` would materialise checkouts at `/qontinui-runner`.
    /// The plan applies the same rule to the analogous web allow-list, and the
    /// resolver must not be laxer than what it feeds.
    FilesystemRoot,
    /// Nothing explicit was configured and the caller supplied no anchor, so
    /// there was no ancestry to walk.
    NoAnchor,
    /// Nothing explicit was configured and every probe missed.
    NothingConfigured,
}

impl RootRejection {
    /// The failure, phrased to compose after a [`RootSource::describe`].
    pub fn reason(self) -> &'static str {
        match self {
            RootRejection::Blank => "is blank",
            RootRejection::Relative => "is not an absolute path",
            RootRejection::NotADirectory => "is not an existing directory",
            RootRejection::FilesystemRoot => {
                "is a filesystem root or bare UNC share, which is never a workspace root"
            }
            RootRejection::NoAnchor => "found no configured workspace root",
            RootRejection::NothingConfigured => {
                "found no configured workspace root, \
                                                 and none of the probes matched"
            }
        }
    }
}

/// A rejected candidate: which input it came from, and why it was unusable.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RejectedRoot {
    /// Which input produced the value.
    pub source: RootSource,
    /// Why it was not usable.
    pub reason: RootRejection,
}

impl RejectedRoot {
    /// One operator-facing sentence fragment: `"$QONTINUI_ROOT is blank"`.
    pub fn describe(self) -> String {
        format!("{} {}", self.source.describe(), self.reason.reason())
    }
}

/// A resolved (or unresolved) workspace root, plus the reason any
/// higher-priority candidate was skipped.
///
/// Same shape and field names as `env_agent/collectors.rs`'s `ProbeScope`.
#[derive(Debug, Clone, PartialEq, Eq)]
#[must_use = "a dropped WorkspaceRoot is exactly the silent miss this type exists to prevent"]
pub struct WorkspaceRoot {
    /// The resolved root, or `None` when nothing resolved.
    pub root: Option<PathBuf>,
    /// The highest-priority input that was present but unusable — reported even
    /// on success, so the fall-through is never invisible. **Always `Some` when
    /// [`root`](Self::root) is `None`**, so a caller that fails closed always
    /// has something to name.
    pub rejected: Option<RejectedRoot>,
}

impl WorkspaceRoot {
    /// The resolved root, discarding the rejection reason. For discovery
    /// surfaces that degrade rather than fail.
    #[must_use]
    pub fn into_root(self) -> Option<PathBuf> {
        self.root
    }

    /// Fail-closed accessor for surfaces that **write or execute** under the
    /// root, where a wrong answer materialises a git worktree at a fabricated
    /// location or runs a script from one.
    ///
    /// The error names the input at fault, states every probe tried, and names
    /// the variable to set — the product-code counterpart of "ask rather than
    /// guess", which a library resolving a path inside a background poller
    /// cannot do.
    pub fn require(self) -> Result<PathBuf, String> {
        match (self.root, self.rejected) {
            (Some(root), _) => Ok(root),
            (None, rejected) => {
                let detail = rejected
                    .unwrap_or(RejectedRoot {
                        source: RootSource::Probes,
                        reason: RootRejection::NothingConfigured,
                    })
                    .describe();
                Err(format!(
                    "cannot resolve the Qontinui workspace root: {detail}. \
                     Set ${QONTINUI_ROOT_ENV} to the absolute path of the \
                     directory holding the Qontinui repo checkouts ({PROBE_ORDER})."
                ))
            }
        }
    }
}

/// A starting point for the ancestor walk, supplied by a caller that has a
/// meaningful one. See the module header's per-caller table.
///
/// `repo_name` is the caller's **own** repo directory name (e.g.
/// `"qontinui-runner"`), used to build the predicate. See [`is_workspace_root`].
#[derive(Debug, Clone, Copy)]
pub struct WorkspaceAnchor<'a> {
    /// Where to start walking. The anchor itself is tested first, then each
    /// ancestor in turn.
    pub start: &'a Path,
    /// The caller's own repo directory name.
    pub repo_name: &'a str,
}

impl<'a> WorkspaceAnchor<'a> {
    /// Build an anchor from a starting path and the caller's own repo directory
    /// name.
    pub fn new(start: &'a Path, repo_name: &'a str) -> Self {
        WorkspaceAnchor { start, repo_name }
    }
}

/// How a candidate directory holds the caller's repo.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum CheckoutKind {
    /// `<dir>/<repo>/.git` is a **directory** — a canonical checkout.
    Canonical,
    /// `<dir>/<repo>/.git` is a **file** — a linked git worktree's marker.
    Worktree,
}

/// True iff `dir` looks like the workspace root for a caller living in
/// `repo_name`.
///
/// The predicate is `dir/<repo_name>/.git` **exists** — `exists`, not
/// `is_dir`, because a linked git worktree's `.git` marker is a *file*, not a
/// directory.
///
/// **This sharpness is load-bearing.** The shell `resolve_workspace_root`
/// idiom's first cut used a loose predicate that was true at
/// `<root>/qontinui-dev-notes` (which tracks a `qontinui-claude-config/docs/`
/// subtree and its own `.claude/`); eight false anchors existed on the
/// operator's machine, each silently anchoring one directory too low. This
/// predicate cannot fire there, because `qontinui-dev-notes` does not contain a
/// `qontinui-runner/.git`.
///
/// `repo_name` must be exactly one ordinary path component: a `repo_name` of
/// `"../qontinui-runner"` would escape the candidate, and an absolute one would
/// *replace* the join outright and make every ancestor match.
///
/// Filesystem roots and bare UNC shares are **never probed** — see
/// [`is_probeable`].
pub fn is_workspace_root(dir: &Path, repo_name: &str) -> bool {
    checkout_kind(dir, repo_name).is_some()
}

/// [`is_workspace_root`] plus *how* the repo is held there, which the ancestor
/// walk needs to prefer a canonical checkout over a worktree parent.
fn checkout_kind(dir: &Path, repo_name: &str) -> Option<CheckoutKind> {
    if !is_single_component(repo_name) || !is_probeable(dir) {
        return None;
    }
    let marker = dir.join(repo_name.trim()).join(".git");
    if marker.is_dir() {
        Some(CheckoutKind::Canonical)
    } else if marker.exists() {
        Some(CheckoutKind::Worktree)
    } else {
        None
    }
}

/// True iff `repo_name` is exactly one ordinary path component — no separator,
/// no `..`, no drive prefix, not absolute.
fn is_single_component(repo_name: &str) -> bool {
    let trimmed = repo_name.trim();
    if trimmed.is_empty() {
        return false;
    }
    let mut components = Path::new(trimmed).components();
    matches!(components.next(), Some(Component::Normal(_))) && components.next().is_none()
}

/// False for a filesystem root (`/`, `C:\`), a bare UNC prefix, and anything
/// else carrying no ordinary path component.
///
/// The motivating cost is Windows-specific: a `//host/.git` existence check is a
/// UNC hostname lookup, measured at 8.3s per probe in the shell idiom's own
/// header. **`Component::Prefix` only exists on Windows**, so `//host/share`
/// parses as `[Prefix(UNC), RootDir]` there and as three ordinary components on
/// Unix — where it is an ordinary path and probing it is correct. The asymmetry
/// is by construction, not an oversight; do not "fix" it.
fn is_probeable(dir: &Path) -> bool {
    dir.components().any(|c| matches!(c, Component::Normal(_)))
}

/// Resolve the Qontinui workspace root, reading the environment for the parts
/// that live there.
///
/// `configured` is an explicitly-configured value the *caller* holds — for
/// qontinui-runner that is the `paths.workspace_root` setting. It is not read
/// here because this crate has no access to any application's settings store.
///
/// Env reading is confined to this wrapper so [`resolve_workspace_root`] stays
/// a pure, deterministically testable core that never mutates process-global
/// environment (which is flaky under parallel tests) — the same split
/// `agent_worktree/canonical_paths.rs`'s `agent_worktree_root` /
/// `agent_worktree_root_inner` already uses.
pub fn qontinui_workspace_root(
    configured: Option<&str>,
    anchor: Option<WorkspaceAnchor<'_>>,
) -> WorkspaceRoot {
    let env_root = std::env::var(QONTINUI_ROOT_ENV).ok();
    let env_alias = std::env::var(QONTINUI_WORKSPACE_ROOT_ENV).ok();
    // Resolved eagerly (one `is_dir` syscall) rather than lazily: the pure core
    // takes its inputs by value so it stays free of closures and of any notion
    // of "the environment", which is the property that makes it testable.
    let home = home_dir();
    resolve_workspace_root(
        env_root.as_deref(),
        env_alias.as_deref(),
        configured,
        anchor,
        home.as_deref(),
    )
}

/// The user's home directory, from the environment only.
///
/// `$HOME` first (set on POSIX and by MSYS/Git-Bash), then `%USERPROFILE%`
/// (Windows). Deliberately no `dirs` dependency — see the module header.
fn home_dir() -> Option<PathBuf> {
    home_dir_from(&[
        ("HOME", std::env::var("HOME").ok()),
        ("USERPROFILE", std::env::var("USERPROFILE").ok()),
    ])
}

/// Pure core of [`home_dir`]: the first non-blank value naming an existing
/// directory wins, in the order given.
fn home_dir_from(candidates: &[(&str, Option<String>)]) -> Option<PathBuf> {
    for (_, value) in candidates {
        let Some(raw) = value else { continue };
        let trimmed = raw.trim();
        if trimmed.is_empty() {
            continue;
        }
        let path = PathBuf::from(trimmed);
        if path.is_dir() {
            return Some(path);
        }
    }
    None
}

/// Pure core of [`qontinui_workspace_root`]. Every input is injected, so this is
/// unit-testable against a synthetic tree with no environment mutation and no
/// dependency on the real machine layout.
///
/// Resolution order, first hit wins:
///
/// 1. **`$QONTINUI_ROOT`** — the established name. Explicit user configuration.
/// 2. **`$QONTINUI_WORKSPACE_ROOT`** — recognised alias, checked second.
/// 3. **`configured`** — the host application's own setting. Still explicit
///    configuration, so it outranks any inference; below the env vars, matching
///    the precedence `plans_dir` already uses (env override beats setting).
/// 4. **Ancestor walk** from `anchor`. Skipped entirely when `anchor` is `None`.
/// 5. **`<home>/qontinui-root`** — portable, and already the POSIX behaviour of
///    every copy this replaces.
/// 6. **Unresolved**, with a typed reason naming the input at fault.
///
/// Rungs 1–3 each require the value to be non-blank, **absolute**, an existing
/// directory, and not a filesystem root. An explicit value failing any of those
/// falls through to the next rung and its reason is carried out on
/// [`WorkspaceRoot::rejected`] — the first such reason wins, since that is the
/// highest-priority thing the operator got wrong.
///
/// **The walk prefers a canonical checkout to a worktree parent.** `.git` must be
/// matched by existence (a linked worktree's marker is a file), but that is
/// exactly what makes a worktree *container* — `<root>/_wt/<tag>/`, holding two
/// of the fleet's fourteen repos — satisfy the predicate one directory too low.
/// Every consumer's existence check would then pass against a partial workspace
/// and the wrong answer would propagate silently. So the walk continues past a
/// worktree match looking for a canonical one, and only falls back to the
/// lowest worktree match if no canonical checkout is found anywhere above it.
pub fn resolve_workspace_root(
    env_root: Option<&str>,
    env_alias: Option<&str>,
    configured: Option<&str>,
    anchor: Option<WorkspaceAnchor<'_>>,
    home: Option<&Path>,
) -> WorkspaceRoot {
    let mut first_rejection: Option<RejectedRoot> = None;

    for (source, explicit) in [
        (RootSource::Env, env_root),
        (RootSource::EnvAlias, env_alias),
        (RootSource::Configured, configured),
    ] {
        let Some(raw) = explicit else { continue };
        match check_explicit(raw) {
            Ok(path) => {
                return WorkspaceRoot {
                    root: Some(path),
                    rejected: first_rejection,
                }
            }
            Err(reason) => {
                first_rejection.get_or_insert(RejectedRoot { source, reason });
            }
        }
    }

    if let Some(anchor) = anchor {
        let mut worktree_fallback: Option<PathBuf> = None;
        let mut cur = Some(anchor.start);
        while let Some(dir) = cur {
            match checkout_kind(dir, anchor.repo_name) {
                Some(CheckoutKind::Canonical) => {
                    return WorkspaceRoot {
                        root: Some(dir.to_path_buf()),
                        rejected: first_rejection,
                    }
                }
                Some(CheckoutKind::Worktree) => {
                    worktree_fallback.get_or_insert_with(|| dir.to_path_buf());
                }
                None => {}
            }
            cur = dir.parent();
        }
        // No canonical checkout anywhere up the chain: an install that genuinely
        // IS a worktree still resolves.
        if let Some(root) = worktree_fallback {
            return WorkspaceRoot {
                root: Some(root),
                rejected: first_rejection,
            };
        }
    }

    if let Some(home) = home {
        let candidate = home.join(HOME_WORKSPACE_DIR);
        if candidate.is_dir() {
            return WorkspaceRoot {
                root: Some(candidate),
                rejected: first_rejection,
            };
        }
    }

    let rejected = first_rejection.unwrap_or(RejectedRoot {
        source: RootSource::Probes,
        reason: if anchor.is_none() {
            RootRejection::NoAnchor
        } else {
            RootRejection::NothingConfigured
        },
    });
    WorkspaceRoot {
        root: None,
        rejected: Some(rejected),
    }
}

/// Validate one explicitly-configured value: non-blank, absolute, an existing
/// directory, and not a filesystem root.
fn check_explicit(raw: &str) -> Result<PathBuf, RootRejection> {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return Err(RootRejection::Blank);
    }
    let path = PathBuf::from(trimmed);
    if !path.is_absolute() {
        return Err(RootRejection::Relative);
    }
    if !is_probeable(&path) {
        return Err(RootRejection::FilesystemRoot);
    }
    if !path.is_dir() {
        return Err(RootRejection::NotADirectory);
    }
    Ok(path)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicU32, Ordering};

    /// Every temp path is pid- AND counter-scoped: this fleet runs `cargo test`
    /// from several worktrees at once, and a shared fixed name would have two
    /// suites deleting each other's tree.
    fn temp_path(tag: &str) -> PathBuf {
        static NEXT: AtomicU32 = AtomicU32::new(0);
        std::env::temp_dir().join(format!(
            "qontinui_paths_{tag}_{}_{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ))
    }

    /// A synthetic workspace holding `qontinui-runner/.git` as a **directory**
    /// (a canonical checkout) and a `qontinui-dev-notes` sibling that must never
    /// anchor.
    ///
    /// Deliberately synthetic — never the operator's real layout — so the suite
    /// passes on a fresh checkout, in CI, and on a non-operator machine.
    struct Fixture {
        root: PathBuf,
    }

    impl Fixture {
        fn new(tag: &str) -> Self {
            let root = temp_path(tag);
            let _ = std::fs::remove_dir_all(&root);
            std::fs::create_dir_all(root.join("qontinui-runner").join("src-tauri")).unwrap();
            std::fs::create_dir_all(root.join("qontinui-runner").join(".git")).unwrap();
            // The historical false anchor: its own `.git` and `.claude`, and it
            // tracks a config-repo subtree — but no `qontinui-runner/.git`.
            std::fs::create_dir_all(root.join("qontinui-dev-notes").join(".claude")).unwrap();
            std::fs::create_dir_all(
                root.join("qontinui-dev-notes")
                    .join("qontinui-claude-config")
                    .join("docs"),
            )
            .unwrap();
            std::fs::create_dir_all(root.join("qontinui-dev-notes").join(".git")).unwrap();
            Fixture { root }
        }

        /// Add `<root>/_wt/<tag>/qontinui-runner/.git` as a FILE — the linked
        /// worktree container that is the live false-anchor hazard.
        fn with_worktree_container(self, tag: &str) -> Self {
            let wt = self.root.join("_wt").join(tag).join("qontinui-runner");
            std::fs::create_dir_all(wt.join("src-tauri")).unwrap();
            std::fs::write(wt.join(".git"), "gitdir: /elsewhere").unwrap();
            self
        }

        fn s(&self) -> String {
            self.root.to_string_lossy().to_string()
        }
    }

    impl Drop for Fixture {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.root);
        }
    }

    /// A second synthetic root, used to prove that a HIGHER-priority input wins
    /// over a lower one rather than merely that each works alone.
    fn other_root(tag: &str) -> (PathBuf, impl Drop) {
        struct Cleanup(PathBuf);
        impl Drop for Cleanup {
            fn drop(&mut self) {
                let _ = std::fs::remove_dir_all(&self.0);
            }
        }
        let root = temp_path(tag);
        std::fs::create_dir_all(&root).unwrap();
        (root.clone(), Cleanup(root))
    }

    // ---------------------------------------------------------------------
    // Rung ordering — every ADJACENT pair, each supplying both sides.
    // ---------------------------------------------------------------------

    #[test]
    fn env_root_beats_the_alias() {
        let f = Fixture::new("order_env_alias");
        let (other, _g) = other_root("order_env_alias_other");
        let got = resolve_workspace_root(
            Some(&f.s()),
            Some(other.to_str().unwrap()),
            None,
            None,
            None,
        );
        assert_eq!(got.root.as_deref(), Some(f.root.as_path()));
    }

    #[test]
    fn alias_beats_the_configured_setting() {
        let f = Fixture::new("order_alias_cfg");
        let (other, _g) = other_root("order_alias_cfg_other");
        let got = resolve_workspace_root(
            None,
            Some(&f.s()),
            Some(other.to_str().unwrap()),
            None,
            None,
        );
        assert_eq!(got.root.as_deref(), Some(f.root.as_path()));
    }

    #[test]
    fn configured_setting_beats_the_ancestor_walk() {
        let f = Fixture::new("order_cfg_walk");
        let (other, _g) = other_root("order_cfg_walk_other");
        let deep = f.root.join("qontinui-runner").join("src-tauri");
        let got = resolve_workspace_root(
            None,
            None,
            Some(other.to_str().unwrap()),
            Some(WorkspaceAnchor::new(&deep, "qontinui-runner")),
            None,
        );
        assert_eq!(
            got.root.as_deref(),
            Some(other.as_path()),
            "an explicit setting must outrank inference from ancestry"
        );
    }

    #[test]
    fn ancestor_walk_beats_the_home_fallback() {
        let f = Fixture::new("order_walk_home");
        let home = f.root.join("home");
        std::fs::create_dir_all(home.join(HOME_WORKSPACE_DIR)).unwrap();
        let deep = f.root.join("qontinui-runner").join("src-tauri");
        let got = resolve_workspace_root(
            None,
            None,
            None,
            Some(WorkspaceAnchor::new(&deep, "qontinui-runner")),
            Some(&home),
        );
        assert_eq!(
            got.root.as_deref(),
            Some(f.root.as_path()),
            "the walk must be tried before <home>/qontinui-root"
        );
    }

    #[test]
    fn home_fallback_is_used_when_nothing_above_it_resolves() {
        let f = Fixture::new("home_last");
        let home = f.root.join("home");
        std::fs::create_dir_all(home.join(HOME_WORKSPACE_DIR)).unwrap();
        let got = resolve_workspace_root(None, None, None, None, Some(&home));
        assert_eq!(
            got.root.as_deref(),
            Some(home.join(HOME_WORKSPACE_DIR).as_path())
        );
    }

    // ---------------------------------------------------------------------
    // Fall-through: an unusable explicit value never shadows a lower rung,
    // and the reason is always reported.
    // ---------------------------------------------------------------------

    #[test]
    fn unusable_env_falls_through_to_configured_and_reports_why() {
        let f = Fixture::new("fallthrough_cfg");
        let got = resolve_workspace_root(Some("   "), None, Some(&f.s()), None, None);
        assert_eq!(got.root.as_deref(), Some(f.root.as_path()));
        assert_eq!(
            got.rejected,
            Some(RejectedRoot {
                source: RootSource::Env,
                reason: RootRejection::Blank
            }),
            "the fall-through must never be invisible"
        );
    }

    #[test]
    fn unusable_configured_falls_through_to_the_ancestor_walk() {
        let f = Fixture::new("fallthrough_walk");
        let deep = f.root.join("qontinui-runner").join("src-tauri");
        let got = resolve_workspace_root(
            None,
            None,
            Some("relative/path"),
            Some(WorkspaceAnchor::new(&deep, "qontinui-runner")),
            None,
        );
        assert_eq!(got.root.as_deref(), Some(f.root.as_path()));
        assert_eq!(
            got.rejected,
            Some(RejectedRoot {
                source: RootSource::Configured,
                reason: RootRejection::Relative
            })
        );
    }

    #[test]
    fn the_first_rejection_wins_when_several_are_bad() {
        let f = Fixture::new("first_rejection");
        let got =
            resolve_workspace_root(Some("relative/one"), Some("   "), Some(&f.s()), None, None);
        assert_eq!(got.root.as_deref(), Some(f.root.as_path()));
        assert_eq!(
            got.rejected,
            Some(RejectedRoot {
                source: RootSource::Env,
                reason: RootRejection::Relative
            }),
            "the highest-priority mistake is the one worth reporting"
        );
    }

    #[test]
    fn explicit_value_failures_are_typed_and_attributed() {
        assert_eq!(
            resolve_workspace_root(Some("relative/path"), None, None, None, None).rejected,
            Some(RejectedRoot {
                source: RootSource::Env,
                reason: RootRejection::Relative
            })
        );
        let missing = temp_path("absent_dir");
        assert_eq!(
            resolve_workspace_root(None, missing.to_str(), None, None, None).rejected,
            Some(RejectedRoot {
                source: RootSource::EnvAlias,
                reason: RootRejection::NotADirectory
            })
        );
    }

    /// A filesystem root passes "absolute" and "is a directory", so without an
    /// explicit guard `$QONTINUI_ROOT=/` would be accepted — after which the
    /// checkout-creating surfaces materialise `/qontinui-runner`.
    ///
    /// It is rejected on every platform; only the *reason* differs, because
    /// Windows does not consider a driveless rooted path absolute (`"/"` is
    /// relative to the current drive there), so it is caught one rung earlier.
    #[test]
    fn a_filesystem_root_is_never_accepted_as_a_configured_value() {
        for raw in ["/", "//"] {
            assert!(
                check_explicit(raw).is_err(),
                "{raw:?} must never be accepted as a workspace root"
            );
        }
        #[cfg(not(target_os = "windows"))]
        for raw in ["/", "//"] {
            assert_eq!(check_explicit(raw), Err(RootRejection::FilesystemRoot));
        }
        #[cfg(target_os = "windows")]
        {
            assert_eq!(check_explicit("/"), Err(RootRejection::Relative));
            assert_eq!(check_explicit("C:\\"), Err(RootRejection::FilesystemRoot));
            assert_eq!(
                check_explicit("\\\\host\\share"),
                Err(RootRejection::FilesystemRoot)
            );
        }
    }

    // ---------------------------------------------------------------------
    // The ancestor walk and its predicate.
    // ---------------------------------------------------------------------

    #[test]
    fn ancestor_walk_finds_the_root_from_a_deep_anchor() {
        let f = Fixture::new("walk");
        let deep = f.root.join("qontinui-runner").join("src-tauri");
        let got = resolve_workspace_root(
            None,
            None,
            None,
            Some(WorkspaceAnchor::new(&deep, "qontinui-runner")),
            None,
        );
        assert_eq!(got.root.as_deref(), Some(f.root.as_path()));
    }

    /// The false-anchor regression the shell idiom paid for in production.
    /// `qontinui-dev-notes` has its own `.git`, its own `.claude`, and a
    /// `qontinui-claude-config/docs` subtree — every loose predicate fires
    /// there. The sharp one must not.
    #[test]
    fn walk_never_anchors_at_the_historical_false_anchor() {
        let f = Fixture::new("false_anchor");
        let inside = f
            .root
            .join("qontinui-dev-notes")
            .join("qontinui-claude-config");
        let got = resolve_workspace_root(
            None,
            None,
            None,
            Some(WorkspaceAnchor::new(&inside, "qontinui-runner")),
            None,
        );
        assert_eq!(
            got.root.as_deref(),
            Some(f.root.as_path()),
            "must walk PAST qontinui-dev-notes to the real root, not stop one dir too low"
        );
        assert!(!is_workspace_root(
            &f.root.join("qontinui-dev-notes"),
            "qontinui-runner"
        ));
    }

    /// The *live* false anchor on this fleet: agent worktrees are materialised
    /// under `<root>/_wt/<tag>/`, so that container holds a
    /// `qontinui-runner/.git` FILE and satisfies the existence predicate one
    /// directory too low. It is worse than the `qontinui-dev-notes` case
    /// because it PARTIALLY exists — it holds two of ~fourteen repos — so every
    /// consumer's existence check passes against a partial workspace and the
    /// wrong answer propagates silently.
    #[test]
    fn walk_prefers_the_canonical_checkout_over_a_worktree_container() {
        let f = Fixture::new("wt_container").with_worktree_container("hp");
        let inside_worktree = f
            .root
            .join("_wt")
            .join("hp")
            .join("qontinui-runner")
            .join("src-tauri");

        let got = resolve_workspace_root(
            None,
            None,
            None,
            Some(WorkspaceAnchor::new(&inside_worktree, "qontinui-runner")),
            None,
        );

        assert_eq!(
            got.root.as_deref(),
            Some(f.root.as_path()),
            "must walk PAST the worktree container to the canonical checkout"
        );
        // The container really does satisfy the bare existence predicate — which
        // is exactly why the walk needs the canonical/worktree distinction.
        assert!(is_workspace_root(
            &f.root.join("_wt").join("hp"),
            "qontinui-runner"
        ));
    }

    /// ...but an install that genuinely IS a worktree, with no canonical
    /// checkout anywhere above it, must still resolve rather than fail.
    #[test]
    fn walk_falls_back_to_a_worktree_when_no_canonical_checkout_exists() {
        let base = temp_path("wt_only");
        let repo = base.join("qontinui-runner");
        std::fs::create_dir_all(repo.join("src-tauri")).unwrap();
        std::fs::write(repo.join(".git"), "gitdir: /elsewhere").unwrap();

        let got = resolve_workspace_root(
            None,
            None,
            None,
            Some(WorkspaceAnchor::new(
                &repo.join("src-tauri"),
                "qontinui-runner",
            )),
            None,
        );
        assert_eq!(got.root.as_deref(), Some(base.as_path()));
        let _ = std::fs::remove_dir_all(&base);
    }

    /// `/` and `//` hold on every platform. The UNC cases are Windows-only by
    /// construction: `Component::Prefix` does not exist on Unix, where
    /// `//host/share` is an ordinary path and probing it is correct.
    #[test]
    fn filesystem_roots_are_never_probed() {
        for p in ["/", "//"] {
            assert!(
                !is_probeable(Path::new(p)),
                "{p:?} must be short-circuited before any existence check"
            );
            assert!(!is_workspace_root(Path::new(p), "qontinui-runner"));
        }
        #[cfg(target_os = "windows")]
        for p in ["C:\\", "\\\\host\\share", "//host/share"] {
            assert!(!is_probeable(Path::new(p)), "{p:?} must be short-circuited");
            assert!(!is_workspace_root(Path::new(p), "qontinui-runner"));
        }
        assert!(is_probeable(Path::new("/qontinui-root")));
    }

    /// `repo_name` is joined onto every candidate, so a traversing or absolute
    /// value would either escape the candidate or replace the join outright —
    /// making every ancestor match and the walk return its own anchor.
    #[test]
    fn repo_name_must_be_exactly_one_ordinary_component() {
        let f = Fixture::new("repo_name");
        for bad in ["", "   ", "../qontinui-runner", "a/b", "."] {
            assert!(
                !is_workspace_root(&f.root, bad),
                "{bad:?} must be rejected as a repo name"
            );
        }
        #[cfg(target_os = "windows")]
        assert!(!is_workspace_root(&f.root, "C:\\qontinui-runner"));
        #[cfg(not(target_os = "windows"))]
        assert!(!is_workspace_root(&f.root, "/qontinui-runner"));

        assert!(is_workspace_root(&f.root, "qontinui-runner"));
    }

    /// The `.git` marker of a LINKED worktree is a file, not a directory; the
    /// predicate must accept both forms.
    #[test]
    fn predicate_accepts_a_git_file_as_well_as_a_git_dir() {
        let f = Fixture::new("gitfile").with_worktree_container("hp");
        assert_eq!(
            checkout_kind(&f.root, "qontinui-runner"),
            Some(CheckoutKind::Canonical)
        );
        assert_eq!(
            checkout_kind(&f.root.join("_wt").join("hp"), "qontinui-runner"),
            Some(CheckoutKind::Worktree)
        );
    }

    // ---------------------------------------------------------------------
    // Unresolved, and the fail-closed door.
    // ---------------------------------------------------------------------

    #[test]
    fn unresolved_always_carries_a_reason() {
        let nowhere = temp_path("no_such_home");

        let no_anchor = resolve_workspace_root(None, None, None, None, Some(&nowhere));
        assert_eq!(no_anchor.root, None);
        assert_eq!(
            no_anchor.rejected,
            Some(RejectedRoot {
                source: RootSource::Probes,
                reason: RootRejection::NoAnchor
            })
        );

        let with_anchor = resolve_workspace_root(
            None,
            None,
            None,
            Some(WorkspaceAnchor::new(&nowhere, "qontinui-runner")),
            Some(&nowhere),
        );
        assert_eq!(with_anchor.root, None);
        assert_eq!(
            with_anchor.rejected,
            Some(RejectedRoot {
                source: RootSource::Probes,
                reason: RootRejection::NothingConfigured
            })
        );
    }

    /// `require()` must name the input AT FAULT — not `$QONTINUI_ROOT` when the
    /// operator's mistake was in the application setting — list every probe, and
    /// say which variable to set.
    #[test]
    fn require_names_the_faulty_input_the_probes_and_the_variable() {
        let err = resolve_workspace_root(None, None, Some("relative/path"), None, None)
            .require()
            .unwrap_err();
        assert!(
            err.contains("configured workspace-root setting"),
            "must blame the setting, not an env var the operator never set: {err}"
        );
        assert!(err.contains("is not an absolute path"), "got {err}");
        assert!(err.contains("QONTINUI_ROOT"), "got {err}");
        assert!(err.contains("QONTINUI_WORKSPACE_ROOT"), "got {err}");
        assert!(err.contains("ancestor walk"), "got {err}");
        assert!(err.contains("<home>/qontinui-root"), "got {err}");
    }

    #[test]
    fn require_blames_the_env_var_when_the_env_var_is_at_fault() {
        let err = resolve_workspace_root(Some("relative/path"), None, None, None, None)
            .require()
            .unwrap_err();
        assert!(
            err.contains("$QONTINUI_ROOT is not an absolute path"),
            "got {err}"
        );
    }

    #[test]
    fn require_returns_the_root_when_resolved() {
        let f = Fixture::new("require_ok");
        let got = resolve_workspace_root(Some(&f.s()), None, None, None, None)
            .require()
            .unwrap();
        assert_eq!(got, f.root);
    }

    // ---------------------------------------------------------------------
    // The home-directory core.
    // ---------------------------------------------------------------------

    #[test]
    fn home_prefers_home_over_userprofile_and_skips_unusable_values() {
        let (first, _g1) = other_root("home_first");
        let (second, _g2) = other_root("home_second");

        assert_eq!(
            home_dir_from(&[
                ("HOME", Some(first.to_string_lossy().into_owned())),
                ("USERPROFILE", Some(second.to_string_lossy().into_owned())),
            ]),
            Some(first.clone()),
            "$HOME must be preferred over %USERPROFILE%"
        );

        let absent = temp_path("home_absent");
        assert_eq!(
            home_dir_from(&[
                ("HOME", Some("   ".to_string())),
                ("USERPROFILE", Some(absent.to_string_lossy().into_owned())),
                ("USERPROFILE", Some(second.to_string_lossy().into_owned())),
            ]),
            Some(second),
            "blank and non-existent values must be skipped, not returned"
        );

        assert_eq!(home_dir_from(&[("HOME", None)]), None);
    }
}
