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
//! ## The session's repository — the single-repository operator's answer
//!
//! A caller that knows which repository an agent session was opened on passes
//! it as `session_repo`. The **folder holding that checkout** (for a linked
//! worktree, the folder holding its main checkout) is then a resolution rung of
//! its own, [`WorkspaceRootKind::SessionRepoParent`], ranked **after** every
//! declaration and the ancestor walk and **before** `<home>/qontinui-root`.
//!
//! This is a decision about a default, not a new setting: an operator who
//! cloned one repository and opened a session on it has a workspace root —
//! the folder that repository sits in — and has never heard of the
//! environment variables above. A box that declares its root, or runs a binary
//! from inside a checkout, is answered by a higher rung first, so this rung
//! changes nothing for it (`session_repo_parent_ranks_below_every_fleet_rung`).
//! The one resolution it DOES change is a box answered today only by
//! `<home>/qontinui-root`, and only for a caller that passes a session
//! repository: that caller now gets the folder holding the session's checkout,
//! which is the more specific answer.
//!
//! ## When nothing resolves — one typed refusal
//!
//! [`WorkspaceRoot::require`] fails with [`WorkspaceRootUnresolved`], which
//! renders through the shared [`Refusal`] envelope (code
//! `workspace_root_unresolved`, discriminator = the [`RootRejection::wire`]
//! value, next action = set the application's workspace-root setting). Every
//! consumer renders that one error instead of composing its own sentence about
//! environment variables the operator has never set.
//!
//! A caller that needs a *sibling* repository next to its own reports
//! [`SiblingCheckoutAbsent`] instead, which says whether the capability is
//! still served from a copy built into the application.
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

use std::fmt;
use std::path::{Component, Path, PathBuf};

use crate::refusal::{NextAction, NextActionKind, Refusal, RefusalCode, RefusalSource};

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

/// The operator-facing name of the application setting that declares the
/// workspace root — the [`NextAction::target`] of every
/// [`WorkspaceRootUnresolved`] refusal.
///
/// Setting it to an existing absolute folder resolves the root even when an
/// environment value is present but unusable: a rejected environment value
/// falls through to it rather than failing (see [`resolve_workspace_root`]).
///
/// It is the label of qontinui-runner's `paths.workspace_root` setting, the only
/// application that holds a workspace-root setting; a service that resolves
/// through this crate with no such setting (coord passes `configured: None`)
/// uses [`WorkspaceRoot::into_root`] and never renders this refusal.
pub const WORKSPACE_ROOT_SETTING: &str = "Workspace root";

/// Statement of the resolution order, carried in the refusal's
/// [`Refusal::detail`] (diagnostic text shown beside the sentence, never inside
/// it) so every probe is visible without reading the source.
const PROBE_ORDER: &str = "resolution order: $QONTINUI_ROOT, \
                           $QONTINUI_WORKSPACE_ROOT, the Workspace root \
                           setting, an ancestor walk from a caller-supplied \
                           anchor (skipped when the caller has none), the \
                           parent of the repository the session was opened on \
                           (skipped when there is none), then \
                           <home>/qontinui-root";

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
            RootSource::Configured => "the Workspace root setting",
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
    /// Nothing explicit was configured and the caller supplied neither an
    /// anchor nor a session repository, so there was nothing to infer from.
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

    /// The stable wire string — the [`Refusal::discriminator`] of a
    /// [`WorkspaceRootUnresolved`] refusal. Mirrors [`WorkspaceRootKind::wire`].
    #[must_use]
    pub fn wire(self) -> &'static str {
        match self {
            RootRejection::Blank => "blank",
            RootRejection::Relative => "relative",
            RootRejection::NotADirectory => "not_a_directory",
            RootRejection::FilesystemRoot => "filesystem_root",
            RootRejection::NoAnchor => "no_anchor",
            RootRejection::NothingConfigured => "nothing_configured",
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

/// WHICH KIND of resolution produced a workspace root.
///
/// The counterpart of `env_agent/collectors.rs`'s `ProbeScopeKind`, and it
/// exists for the same reason: the PATH is not comparable across boxes — a
/// Windows `D:\qontinui-root` and a Linux `/home/dev/src` differ while meaning
/// exactly the same thing, and the path is operator-local — but the KIND is.
///
/// [`RejectedRoot`] is the *failure* story (which input was unusable and why);
/// this is the *success* story (which rung actually answered). A caller that
/// publishes a workspace-root-relative observation needs the latter to say
/// whether two boxes' observations are comparable at all: a box that honoured
/// an explicit declaration and a box that fell through to `<home>/qontinui-root`
/// did not measure the same concept.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WorkspaceRootKind {
    /// An explicit value was honoured — `$QONTINUI_ROOT`, its alias, or the
    /// host application's own setting. The operator said where the checkouts
    /// live and the resolver agreed.
    Declared,
    /// The ancestor walk found the caller's own repo (see
    /// [`is_workspace_root`]) — inferred from where this binary actually sits,
    /// which is a fact about the install rather than a declaration.
    Discovered,
    /// The parent directory of the repository the session was opened on. For
    /// an operator with a single repository this IS the workspace root; ranked
    /// below every declaration and the ancestor walk, so a box answered by
    /// either never reaches it.
    SessionRepoParent,
    /// `<home>/qontinui-root`, the portable last-resort convention. Nothing was
    /// declared and nothing was discovered; this is the default layout, not an
    /// observation about this box.
    HomeDefault,
    /// Nothing resolved. Carried so an unresolved root is a *stated* kind
    /// rather than an absent one — the same absence-is-not-a-value rule the
    /// rest of this module follows.
    Unresolved,
}

impl WorkspaceRootKind {
    /// The stable wire string, for callers that publish this provenance
    /// (e.g. the devenv envelope's `repos.repos_scope_kind`). Mirrors
    /// `ProbeScopeKind::wire`.
    #[must_use]
    pub fn wire(self) -> &'static str {
        match self {
            Self::Declared => "declared",
            Self::Discovered => "discovered",
            Self::SessionRepoParent => "session_repo_parent",
            Self::HomeDefault => "home_default",
            Self::Unresolved => "unresolved",
        }
    }
}

/// A resolved (or unresolved) workspace root, plus which rung answered and the
/// reason any higher-priority candidate was skipped.
///
/// Same shape and field names as `env_agent/collectors.rs`'s `ProbeScope`,
/// including its `kind`.
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
    /// Which KIND of resolution answered — the part that is comparable across
    /// boxes. `Unresolved` exactly when [`root`](Self::root) is `None`.
    pub kind: WorkspaceRootKind,
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
    /// The error is the one typed [`WorkspaceRootUnresolved`]: it names the
    /// input at fault, carries every probe tried as diagnostic detail, and
    /// renders as a [`Refusal`] whose next action is the application setting —
    /// the product-code counterpart of "ask rather than guess", which a library
    /// resolving a path inside a background poller cannot do.
    pub fn require(self) -> Result<PathBuf, WorkspaceRootUnresolved> {
        match (self.root, self.rejected) {
            (Some(root), _) => Ok(root),
            (None, rejected) => Err(WorkspaceRootUnresolved {
                rejected: rejected.unwrap_or(RejectedRoot {
                    source: RootSource::Probes,
                    reason: RootRejection::NothingConfigured,
                }),
            }),
        }
    }
}

/// The workspace root could not be resolved — the typed error of
/// [`WorkspaceRoot::require`], and the ONE sentence every consumer renders for
/// that case.
///
/// It is a [`Refusal`] in waiting: [`refusal`](Self::refusal) builds the
/// envelope (code [`RefusalCode::WorkspaceRootUnresolved`], discriminator
/// [`RootRejection::wire`], next action "set the Workspace root setting"),
/// and [`Display`](fmt::Display) prints the same sentence without needing a
/// source or a timestamp. The next action is always the setting because
/// setting it always works: a rejected environment value falls through to it.
///
/// The display names an environment variable **only** when the operator set
/// that variable and it was at fault. The full probe order — which names
/// variables an operator may never have heard of — is diagnostic detail
/// ([`detail`](Self::detail)), never part of the sentence.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct WorkspaceRootUnresolved {
    /// The highest-priority input that was present but unusable, or
    /// [`RootSource::Probes`] when nothing explicit was at fault.
    pub rejected: RejectedRoot,
}

impl WorkspaceRootUnresolved {
    /// The refusal's discriminator: the [`RootRejection::wire`] value.
    #[must_use]
    pub fn discriminator(&self) -> &'static str {
        self.rejected.reason.wire()
    }

    /// What the operator does next: set the Workspace root setting.
    #[must_use]
    pub fn next_action(&self) -> NextAction {
        NextAction::new(NextActionKind::SetSetting).with_target(WORKSPACE_ROOT_SETTING)
    }

    /// Diagnostic text for [`Refusal::detail`]: the input at fault and the full
    /// resolution order.
    #[must_use]
    pub fn detail(&self) -> String {
        format!("{} ({PROBE_ORDER})", self.rejected.describe())
    }

    /// The input the operator explicitly set that was unusable —
    /// `"$QONTINUI_ROOT is blank"` — or `None` when no explicit input was at
    /// fault (every probe simply missed).
    #[must_use]
    pub fn operator_input_at_fault(&self) -> Option<String> {
        match self.rejected.source {
            RootSource::Env | RootSource::EnvAlias | RootSource::Configured => {
                Some(self.rejected.describe())
            }
            RootSource::Probes => None,
        }
    }

    /// The shared envelope for this failure.
    #[must_use]
    pub fn refusal(&self, source: RefusalSource, observed_at: impl Into<String>) -> Refusal {
        Refusal::new(
            RefusalCode::WorkspaceRootUnresolved,
            self.next_action(),
            source,
            observed_at,
        )
        .with_discriminator(self.discriminator())
        .with_detail(self.detail())
    }
}

/// `<Refusal::render>` — plus, only when the operator set an input that was at
/// fault, which one: `… Cause: the Workspace root setting is not an existing
/// directory.`
impl fmt::Display for WorkspaceRootUnresolved {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // `render()` projects code, discriminator and next action only, so the
        // source and timestamp given here never reach the text.
        let sentence = self.refusal(RefusalSource::Runner, "").render();
        match self.operator_input_at_fault() {
            Some(input) => write!(f, "{sentence} Cause: {input}."),
            None => f.write_str(&sentence),
        }
    }
}

impl std::error::Error for WorkspaceRootUnresolved {}

/// What still serves a capability when a sibling checkout it prefers is
/// absent — the case split of [`SiblingCheckoutAbsent`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SiblingFallback {
    /// A copy compiled into the application serves the capability. Nothing was
    /// refused, so there is no refusal envelope — only the statement of which
    /// copy answered.
    EmbeddedCopy,
    /// Nothing else serves it: the capability is unsupported in this
    /// installation, and the caller refuses.
    Unsupported,
}

impl SiblingFallback {
    /// The statement of which copy (if any) serves the capability.
    #[must_use]
    pub fn describe(self) -> &'static str {
        match self {
            SiblingFallback::EmbeddedCopy => {
                "This capability is served from the copy built into the application"
            }
            SiblingFallback::Unsupported => "This capability is not supported in this installation",
        }
    }
}

/// A sibling repository checkout the caller needs next to its own is not on
/// this machine — the `sibling_checkout_absent { name }` case.
///
/// Distinct from [`WorkspaceRootUnresolved`]: the workspace root may well
/// resolve (a single-repository operator's root is the folder their one
/// repository sits in) while the sibling it would hold was never cloned.
///
/// **Only the [`SiblingFallback::Unsupported`] arm is a refusal.** When a copy
/// built into the application serves the capability the operation SUCCEEDS,
/// and an envelope for it would be rendered by every reader as a failure — so
/// [`refusal`](Self::refusal) is `None` there and the case travels as this
/// type's [`Display`](fmt::Display) (a log line, a provisioning report's
/// detail). The unsupported arm's next action is
/// [`NextActionKind::NoneTerminal`]: the missing checkout is a
/// development-environment artifact that an installed product does not offer,
/// so no operator action within the product changes the outcome.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SiblingCheckoutAbsent {
    /// The sibling repository's directory name — the refusal's discriminator.
    pub name: String,
    /// What serves the capability instead, if anything.
    pub fallback: SiblingFallback,
}

impl SiblingCheckoutAbsent {
    /// The case for sibling `name` with the given fallback.
    pub fn new(name: impl Into<String>, fallback: SiblingFallback) -> Self {
        Self {
            name: name.into(),
            fallback,
        }
    }

    /// The refusal envelope (code [`RefusalCode::SiblingCheckoutAbsent`],
    /// discriminator = the sibling's name, next action
    /// [`NextActionKind::NoneTerminal`], detail = the unsupported statement) —
    /// or `None` when an embedded copy serves the capability and nothing was
    /// refused.
    #[must_use]
    pub fn refusal(
        &self,
        source: RefusalSource,
        observed_at: impl Into<String>,
    ) -> Option<Refusal> {
        match self.fallback {
            SiblingFallback::EmbeddedCopy => None,
            SiblingFallback::Unsupported => Some(
                Refusal::new(
                    RefusalCode::SiblingCheckoutAbsent,
                    NextAction::new(NextActionKind::NoneTerminal),
                    source,
                    observed_at,
                )
                .with_discriminator(self.name.clone())
                .with_detail(self.fallback.describe()),
            ),
        }
    }
}

/// Unsupported: `<Refusal::render> <unsupported statement>.` Embedded:
/// `<headline> (<name>). <served statement>.` — the same headline, with the
/// statement of which copy answered in place of a next action, because
/// nothing was refused.
impl fmt::Display for SiblingCheckoutAbsent {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.refusal(RefusalSource::Runner, "") {
            Some(refusal) => write!(f, "{} {}.", refusal.render(), self.fallback.describe()),
            None => write!(
                f,
                "{} ({}). {}.",
                RefusalCode::SiblingCheckoutAbsent.headline(),
                self.name,
                self.fallback.describe()
            ),
        }
    }
}

/// An error ONLY in its [`SiblingFallback::Unsupported`] arm. The
/// [`SiblingFallback::EmbeddedCopy`] arm is a success notice: a caller whose
/// embedded copy served the capability must not propagate it with `?` — check
/// [`SiblingCheckoutAbsent::refusal`] (`None` there) first.
impl std::error::Error for SiblingCheckoutAbsent {}

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
/// `session_repo` is the repository the caller's session was opened on, when
/// it has one (module docs, "The session's repository") — the checkout's
/// TOP-LEVEL directory (where `.git` is), not a path inside it; a path inside
/// it skips the rung. A service with no session passes `None`.
///
/// Env reading is confined to this wrapper so [`resolve_workspace_root`] stays
/// a pure, deterministically testable core that never mutates process-global
/// environment (which is flaky under parallel tests) — the same split
/// `agent_worktree/canonical_paths.rs`'s `agent_worktree_root` /
/// `agent_worktree_root_inner` already uses.
pub fn qontinui_workspace_root(
    configured: Option<&str>,
    anchor: Option<WorkspaceAnchor<'_>>,
    session_repo: Option<&Path>,
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
        session_repo,
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
/// 5. **The folder holding `session_repo`'s checkout** — the repository the
///    session was opened on. For a canonical checkout (`.git` is a directory)
///    that is its parent. For a linked git worktree (`.git` is a file) it is
///    the parent of the worktree's MAIN checkout, read through `gitdir:` and
///    `commondir` — the plain parent of a worktree is a worktree container,
///    the same one-directory-too-low answer the ancestor walk refuses. Skipped
///    when `session_repo` is `None`, and when it is not a normalised absolute
///    path to a repository whose main checkout sits in an ordinary directory.
///    Never a rejection: it is an inference the caller offers, not a value the
///    operator configured.
/// 6. **`<home>/qontinui-root`** — portable, and already the POSIX behaviour of
///    every copy this replaces.
/// 7. **Unresolved**, with a typed reason naming the input at fault.
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
    session_repo: Option<&Path>,
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
                    kind: WorkspaceRootKind::Declared,
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
                        kind: WorkspaceRootKind::Discovered,
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
        //
        // Still `Discovered`: the KIND names which RUNG answered, and both arms
        // are the same rung — the ancestor walk. The canonical-over-worktree
        // preference is about picking the right PATH, not about how confident
        // the resolution is, so splitting it into a second kind would publish a
        // distinction that means nothing to a comparing peer.
        if let Some(root) = worktree_fallback {
            return WorkspaceRoot {
                root: Some(root),
                rejected: first_rejection,
                kind: WorkspaceRootKind::Discovered,
            };
        }
    }

    if let Some(parent) = session_repo.and_then(session_repo_parent) {
        return WorkspaceRoot {
            root: Some(parent),
            rejected: first_rejection,
            kind: WorkspaceRootKind::SessionRepoParent,
        };
    }
    if let Some(home) = home {
        let candidate = home.join(HOME_WORKSPACE_DIR);
        if candidate.is_dir() {
            return WorkspaceRoot {
                root: Some(candidate),
                rejected: first_rejection,
                kind: WorkspaceRootKind::HomeDefault,
            };
        }
    }

    let rejected = first_rejection.unwrap_or(RejectedRoot {
        source: RootSource::Probes,
        reason: if anchor.is_none() && session_repo.is_none() {
            RootRejection::NoAnchor
        } else {
            RootRejection::NothingConfigured
        },
    });
    WorkspaceRoot {
        root: None,
        rejected: Some(rejected),
        kind: WorkspaceRootKind::Unresolved,
    }
}

/// The folder holding the checkout a session was opened on — rung 5 of
/// [`resolve_workspace_root`].
///
/// `repo` must be absolute and normalised (no `.` or `..` component: a
/// lexical `parent()` of `/a/b/..` is `/a/b`, not `/`). The main checkout is
/// `repo` itself when `repo/.git` is a directory, or the checkout the linked
/// worktree belongs to when it is a file ([`main_checkout_of_worktree`]).
/// The answer is that checkout's parent, when it is an ordinary directory
/// rather than a filesystem root. Anything unreadable skips the rung.
///
/// `repo` is NOT walked upward to find a `.git`: a session opened on a folder
/// that is not itself a checkout (a project folder inside a home directory that
/// is a dotfiles repository, say) would otherwise borrow an unrelated
/// repository's parent. The caller passes the checkout's top level, which a
/// session that knows its repository already has.
///
/// A full clone (`.git` a directory) is taken at face value: one cloned into a
/// container directory answers with that container. This rung is reached only
/// when no declaration and no ancestor walk answered, and for a full clone the
/// parent is the only folder the checkout itself states.
fn session_repo_parent(repo: &Path) -> Option<PathBuf> {
    if !repo.is_absolute()
        || repo
            .components()
            .any(|c| matches!(c, Component::CurDir | Component::ParentDir))
    {
        return None;
    }
    let marker = repo.join(".git");
    let main_checkout = if marker.is_dir() {
        repo.to_path_buf()
    } else if marker.is_file() {
        main_checkout_of_worktree(repo, &marker)?
    } else {
        return None;
    };
    let parent = main_checkout.parent()?;
    (is_probeable(parent) && parent.is_dir()).then(|| parent.to_path_buf())
}

/// The main checkout a linked git worktree at `worktree` belongs to, read from
/// its `.git` file (`gitdir: <main>/.git/worktrees/<name>`) and that git dir's
/// `commondir` (`../..`, naming `<main>/.git`). `None` for anything else — a
/// submodule's `.git` file points at `<super>/.git/modules/<name>`, which has
/// no `commondir`, and guessing a checkout from it would be exactly the
/// fabricated answer this module refuses.
///
/// **Round-trip verified.** Paths are folded lexically (never
/// `canonicalize`, which resolves symlinks and on Windows returns a `\\?\`
/// verbatim path, spelling the root differently from every other rung). A
/// lexical fold of a RELATIVE `gitdir:` read through a symlinked worktree path
/// can land on a different admin directory, so the admin directory must name
/// this worktree back: its own `gitdir` file (which git writes) must fold to
/// this worktree's `.git`. Git writes that back-link with symlinks RESOLVED, so
/// a session repository reached through a symlink (a symlinked home, macOS
/// `/var` -> `/private/var`, a drive letter spelled in the other case) fails the
/// lexical comparison; only then is the IDENTITY re-checked through
/// `canonicalize` of both sides. Canonicalisation is used for that yes/no
/// question only — the root returned is still the lexically folded spelling.
/// A back-link that names another worktree either way skips the rung: a
/// symlinked spelling may cost the inference, it never yields another
/// checkout's folder.
fn main_checkout_of_worktree(worktree: &Path, marker: &Path) -> Option<PathBuf> {
    let contents = std::fs::read_to_string(marker).ok()?;
    let gitdir = contents
        .lines()
        .find_map(|l| l.strip_prefix("gitdir:"))
        .map(str::trim)
        .filter(|g| !g.is_empty())?;
    // An absolute gitdir replaces the join.
    let gitdir = normalize_lexically(&worktree.join(gitdir));
    let back_link = std::fs::read_to_string(gitdir.join("gitdir")).ok()?;
    let back_link = back_link.trim();
    if back_link.is_empty() || !names_same_file(&gitdir.join(back_link), marker) {
        return None;
    }
    let commondir = std::fs::read_to_string(gitdir.join("commondir")).ok()?;
    let commondir = commondir.trim();
    if commondir.is_empty() {
        return None;
    }
    // `commondir` is relative to the (verified) admin dir git wrote.
    let common = normalize_lexically(&gitdir.join(commondir));
    if common.file_name()? != ".git" {
        // A bare common dir has no working checkout to take a parent of.
        return None;
    }
    let main = common.parent()?.to_path_buf();
    main.join(".git").is_dir().then_some(main)
}

/// Whether `a` and `b` name the same file: equal after a lexical fold, or —
/// only when they are not — equal after `canonicalize` (both must exist). The
/// fallback answers a spelling difference git introduced by resolving symlinks;
/// it never changes what [`main_checkout_of_worktree`] returns.
fn names_same_file(a: &Path, b: &Path) -> bool {
    let (a, b) = (normalize_lexically(a), normalize_lexically(b));
    if a == b {
        return true;
    }
    matches!(
        (std::fs::canonicalize(&a), std::fs::canonicalize(&b)),
        (Ok(x), Ok(y)) if x == y
    )
}

/// `path` with `.` components dropped and each `..` removing the component
/// before it — text only, no filesystem access. A `..` with nothing ordinary
/// to remove (it would climb past the root or a prefix) is kept, which the
/// caller's existence checks then reject.
fn normalize_lexically(path: &Path) -> PathBuf {
    let mut out: Vec<Component<'_>> = Vec::new();
    for c in path.components() {
        match c {
            Component::CurDir => {}
            Component::ParentDir => match out.last() {
                Some(Component::Normal(_)) => {
                    out.pop();
                }
                _ => out.push(c),
            },
            _ => out.push(c),
        }
    }
    out.iter().collect()
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
            None,
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
        let got = resolve_workspace_root(None, None, None, None, None, Some(&home));
        assert_eq!(
            got.root.as_deref(),
            Some(home.join(HOME_WORKSPACE_DIR).as_path())
        );
    }

    // ---------------------------------------------------------------------
    // The session-repository rung.
    // ---------------------------------------------------------------------

    /// A single-repository operator's layout: `<base>/my-app/.git`, nothing
    /// else — no declaration, no Qontinui checkout, no `<home>/qontinui-root`.
    fn solo_repo(tag: &str) -> (PathBuf, PathBuf, impl Drop) {
        let (base, guard) = other_root(tag);
        let repo = base.join("my-app");
        std::fs::create_dir_all(repo.join(".git")).unwrap();
        (base, repo, guard)
    }

    #[test]
    fn session_repo_parent_answers_for_a_single_repository_operator() {
        let (base, repo, _g) = solo_repo("session_solo");
        let nowhere = temp_path("session_solo_no_home");
        let got = resolve_workspace_root(None, None, None, None, Some(&repo), Some(&nowhere));
        assert_eq!(got.root.as_deref(), Some(base.as_path()));
        assert_eq!(got.kind, WorkspaceRootKind::SessionRepoParent);
        assert_eq!(got.rejected, None);
    }

    /// The ordering this rung must keep so that every fleet box resolves exactly
    /// as before: a DECLARED root and a DISCOVERED root each beat it, and it
    /// beats only `<home>/qontinui-root`. Every input is supplied at once, so
    /// each assertion proves precedence rather than that a rung works alone.
    #[test]
    fn session_repo_parent_ranks_below_every_fleet_rung() {
        let f = Fixture::new("session_order");
        let (session_base, repo, _g) = solo_repo("session_order_repo");
        let home = f.root.join("home");
        std::fs::create_dir_all(home.join(HOME_WORKSPACE_DIR)).unwrap();
        let deep = f.root.join("qontinui-runner").join("src-tauri");
        let anchor = || Some(WorkspaceAnchor::new(&deep, "qontinui-runner"));

        // Declared (each spelling) beats it.
        let (declared, _d) = other_root("session_order_declared");
        let d = declared.to_str().unwrap();
        for (env, alias, cfg) in [
            (Some(d), None, None),
            (None, Some(d), None),
            (None, None, Some(d)),
        ] {
            let got = resolve_workspace_root(env, alias, cfg, anchor(), Some(&repo), Some(&home));
            assert_eq!(got.root.as_deref(), Some(declared.as_path()));
            assert_eq!(got.kind, WorkspaceRootKind::Declared);
        }

        // Discovered beats it.
        let got = resolve_workspace_root(None, None, None, anchor(), Some(&repo), Some(&home));
        assert_eq!(got.root.as_deref(), Some(f.root.as_path()));
        assert_eq!(got.kind, WorkspaceRootKind::Discovered);

        // It beats the home default.
        let got = resolve_workspace_root(None, None, None, None, Some(&repo), Some(&home));
        assert_eq!(got.root.as_deref(), Some(session_base.as_path()));
        assert_eq!(got.kind, WorkspaceRootKind::SessionRepoParent);

        // An anchor that is supplied but finds no workspace falls through to it.
        let missing_anchor = || Some(WorkspaceAnchor::new(&repo, "no-such-repo-for-this-test"));
        let got =
            resolve_workspace_root(None, None, None, missing_anchor(), Some(&repo), Some(&home));
        assert_eq!(got.root.as_deref(), Some(session_base.as_path()));
        assert_eq!(got.kind, WorkspaceRootKind::SessionRepoParent);

        // And a rejected declaration still falls through to it, visibly.
        let got = resolve_workspace_root(Some("   "), None, None, None, Some(&repo), Some(&home));
        assert_eq!(got.kind, WorkspaceRootKind::SessionRepoParent);
        assert_eq!(
            got.rejected,
            Some(RejectedRoot {
                source: RootSource::Env,
                reason: RootRejection::Blank,
            })
        );
    }

    /// A session "repository" that is relative, un-normalised, missing, not a
    /// repository, or a `.git` file that names no worktree
    /// of a main checkout (a submodule's, or garbage) is skipped — the next
    /// rung answers and nothing is fabricated.
    #[test]
    fn session_repo_that_is_not_a_usable_repository_is_skipped() {
        let (base, repo, _g) = solo_repo("session_skip");
        let plain_dir = base.join("not-a-repo");
        std::fs::create_dir_all(&plain_dir).unwrap();
        std::fs::create_dir_all(repo.join("sub")).unwrap();
        // Lexically its parent is `my-app/sub`; it must not be taken at all.
        let dotted = repo.join("sub").join("..");
        let missing = base.join("gone");
        let relative = PathBuf::from("my-app");
        // (A repository directly under a filesystem root — parent `/` — is
        // refused by the `is_probeable` guard. It cannot be built portably in a
        // temp dir, so that arm has no fixture here.)
        // A submodule's `.git` file: its git dir has no `commondir`.
        let submodule = base.join("with-submodule");
        let modules = base.join("super").join(".git").join("modules").join("m");
        std::fs::create_dir_all(&modules).unwrap();
        std::fs::create_dir_all(&submodule).unwrap();
        std::fs::write(
            submodule.join(".git"),
            format!("gitdir: {}", modules.display()),
        )
        .unwrap();
        // A `.git` file naming nothing that exists.
        let dangling = base.join("dangling");
        std::fs::create_dir_all(&dangling).unwrap();
        std::fs::write(dangling.join(".git"), "gitdir: /no/such/gitdir").unwrap();

        let nowhere = temp_path("session_skip_no_home");
        for candidate in [
            &plain_dir, &dotted, &missing, &relative, &submodule, &dangling,
        ] {
            let got =
                resolve_workspace_root(None, None, None, None, Some(candidate), Some(&nowhere));
            assert_eq!(got.root, None, "{candidate:?} must not resolve");
            assert_eq!(got.kind, WorkspaceRootKind::Unresolved);
            assert_eq!(
                got.rejected,
                Some(RejectedRoot {
                    source: RootSource::Probes,
                    reason: RootRejection::NothingConfigured,
                }),
                "an offered session repository means something WAS probed"
            );
        }
    }

    /// A linked worktree answers with the folder holding its MAIN checkout,
    /// read through `gitdir:` and `commondir` — never the worktree's own
    /// container, which is one directory too low. Both an absolute and a
    /// relative `gitdir:` are honoured.
    #[test]
    fn session_repo_in_a_linked_worktree_answers_with_the_main_checkouts_folder() {
        let (base, _g) = other_root("session_worktree");
        let workspace = base.join("workspace");
        let main = workspace.join("my-app");
        for (tag, relative_gitdir) in [("abs", false), ("rel", true)] {
            let admin = main.join(".git").join("worktrees").join(tag);
            std::fs::create_dir_all(&admin).unwrap();
            std::fs::write(admin.join("commondir"), "../..\n").unwrap();
            let worktree = base.join("containers").join(tag).join("my-app");
            std::fs::create_dir_all(&worktree).unwrap();
            let (gitdir, back_link) = if relative_gitdir {
                (
                    format!("../../../workspace/my-app/.git/worktrees/{tag}"),
                    format!("../../../../../containers/{tag}/my-app/.git"),
                )
            } else {
                (
                    admin.display().to_string(),
                    worktree.join(".git").display().to_string(),
                )
            };
            std::fs::write(worktree.join(".git"), format!("gitdir: {gitdir}\n")).unwrap();
            std::fs::write(admin.join("gitdir"), format!("{back_link}\n")).unwrap();

            let got = resolve_workspace_root(None, None, None, None, Some(&worktree), None);
            assert_eq!(
                got.root.as_deref(),
                Some(workspace.as_path()),
                "{tag}: must answer with the main checkout's folder, spelled as the \
                 other rungs spell it (no canonicalisation), not the container"
            );
            assert_eq!(got.kind, WorkspaceRootKind::SessionRepoParent);
        }
    }

    /// Git may write an absolute `commondir`; it is honoured. An admin dir
    /// whose `gitdir` back-link names another worktree is not. A common dir that
    /// is not a `.git` directory (a bare repository) has no working checkout to
    /// take a parent of, so the rung is skipped.
    #[test]
    fn session_worktree_commondir_absolute_is_honoured_and_bare_is_skipped() {
        let (base, _g) = other_root("session_commondir");
        let workspace = base.join("workspace");
        let main_git = workspace.join("my-app").join(".git");
        let admin = main_git.join("worktrees").join("wt");
        std::fs::create_dir_all(&admin).unwrap();
        std::fs::write(admin.join("commondir"), main_git.display().to_string()).unwrap();
        let worktree = base.join("containers").join("wt").join("my-app");
        std::fs::create_dir_all(&worktree).unwrap();
        std::fs::write(
            worktree.join(".git"),
            format!("gitdir: {}", admin.display()),
        )
        .unwrap();
        std::fs::write(
            admin.join("gitdir"),
            worktree.join(".git").display().to_string(),
        )
        .unwrap();
        let got = resolve_workspace_root(None, None, None, None, Some(&worktree), None);
        assert_eq!(got.root.as_deref(), Some(workspace.as_path()));
        assert_eq!(got.kind, WorkspaceRootKind::SessionRepoParent);

        // An admin dir that names a DIFFERENT worktree back (what a lexical
        // fold through a symlinked spelling can land on) is never trusted.
        std::fs::write(admin.join("gitdir"), "/some/other/worktree/.git").unwrap();
        let got = resolve_workspace_root(None, None, None, None, Some(&worktree), None);
        assert_eq!(
            got.root, None,
            "a back-link to another worktree skips the rung"
        );

        let bare = base.join("bare.git");
        let bare_admin = bare.join("worktrees").join("wt");
        std::fs::create_dir_all(&bare_admin).unwrap();
        std::fs::write(bare_admin.join("commondir"), "../..").unwrap();
        let bare_wt = base.join("bare-wt").join("my-app");
        std::fs::create_dir_all(&bare_wt).unwrap();
        std::fs::write(
            bare_wt.join(".git"),
            format!("gitdir: {}", bare_admin.display()),
        )
        .unwrap();
        std::fs::write(
            bare_admin.join("gitdir"),
            bare_wt.join(".git").display().to_string(),
        )
        .unwrap();
        let got = resolve_workspace_root(None, None, None, None, Some(&bare_wt), None);
        assert_eq!(
            got.root, None,
            "a bare repository's worktree has no main checkout"
        );
    }

    /// The hand-written fixtures above pin the format this module expects; this
    /// pins it against what the installed `git` actually writes. Skipped (with a
    /// note) where no `git` is on PATH.
    #[test]
    fn session_repo_in_a_real_git_worktree_answers_with_the_main_checkouts_folder() {
        let git = |dir: &Path, args: &[&str]| {
            let mut cmd = std::process::Command::new("git");
            // A `GIT_DIR` / `GIT_INDEX_FILE` / … inherited from a hook would
            // outrank `-C` and aim these commands at the developer's own
            // repository. Strip every `GIT_*` before setting the two below.
            for (k, _) in std::env::vars_os() {
                if k.to_string_lossy().starts_with("GIT_") {
                    cmd.env_remove(k);
                }
            }
            cmd.arg("-C")
                .arg(dir)
                .args(args)
                .env("GIT_CONFIG_GLOBAL", "/dev/null")
                .env("GIT_CONFIG_NOSYSTEM", "1")
                .output()
        };
        let (base, _g) = other_root("session_real_git");
        let workspace = base.join("workspace");
        let main = workspace.join("my-app");
        std::fs::create_dir_all(&main).unwrap();
        match git(&main, &["init", "-q"]) {
            Ok(o) if o.status.success() => {}
            _ => {
                eprintln!("git unavailable; real-worktree check skipped");
                return;
            }
        }
        let ok = |o: std::io::Result<std::process::Output>| {
            let o = o.expect("git runs");
            assert!(o.status.success(), "{}", String::from_utf8_lossy(&o.stderr));
        };
        ok(git(
            &main,
            &[
                "-c",
                "user.name=t",
                "-c",
                "user.email=t@t",
                "commit",
                "-q",
                "--allow-empty",
                "-m",
                "init",
            ],
        ));
        let worktree = base.join("containers").join("wt").join("my-app");
        std::fs::create_dir_all(worktree.parent().unwrap()).unwrap();
        ok(git(
            &main,
            &[
                "worktree",
                "add",
                "-q",
                "--detach",
                worktree.to_str().unwrap(),
            ],
        ));
        // Git writes resolved paths; compare against the resolved workspace so a
        // symlinked temp dir (macOS `/var`) does not fail the assertion.
        let got = resolve_workspace_root(None, None, None, None, Some(&worktree), None);
        assert_eq!(got.kind, WorkspaceRootKind::SessionRepoParent, "{got:?}");
        assert_eq!(
            std::fs::canonicalize(got.root.unwrap()).unwrap(),
            std::fs::canonicalize(&workspace).unwrap()
        );
    }

    /// A worktree reached through a SYMLINKED spelling: git's back-link names the
    /// resolved path, so the lexical comparison misses and the identity check
    /// falls back to `canonicalize`. The rung still answers.
    #[cfg(unix)]
    #[test]
    fn session_repo_reached_through_a_symlink_still_answers() {
        let (base, _g) = other_root("session_symlink");
        let workspace = base.join("workspace");
        let main_git = workspace.join("my-app").join(".git");
        let admin = main_git.join("worktrees").join("wt");
        std::fs::create_dir_all(&admin).unwrap();
        std::fs::write(admin.join("commondir"), "../..\n").unwrap();
        let real = base.join("real").join("my-app");
        std::fs::create_dir_all(&real).unwrap();
        std::fs::write(real.join(".git"), format!("gitdir: {}\n", admin.display())).unwrap();
        std::fs::write(
            admin.join("gitdir"),
            format!("{}\n", real.join(".git").display()),
        )
        .unwrap();
        let link = base.join("link");
        std::os::unix::fs::symlink(base.join("real"), &link).unwrap();
        let via_link = link.join("my-app");

        let got = resolve_workspace_root(None, None, None, None, Some(&via_link), None);
        assert_eq!(got.root.as_deref(), Some(workspace.as_path()));
        assert_eq!(got.kind, WorkspaceRootKind::SessionRepoParent);

        // A back-link to a DIFFERENT real worktree is still refused through the link.
        let other = base.join("other").join("my-app");
        std::fs::create_dir_all(&other).unwrap();
        std::fs::write(other.join(".git"), "gitdir: x\n").unwrap();
        std::fs::write(
            admin.join("gitdir"),
            other.join(".git").display().to_string(),
        )
        .unwrap();
        let got = resolve_workspace_root(None, None, None, None, Some(&via_link), None);
        assert_eq!(got.root, None);
    }

    #[test]
    fn normalize_lexically_drops_dots_and_folds_parents() {
        assert_eq!(
            normalize_lexically(Path::new("/a/b/./c/../../d")),
            PathBuf::from("/a/d")
        );
        assert_eq!(
            normalize_lexically(Path::new("/../a")),
            PathBuf::from("/../a")
        );
        assert_eq!(
            normalize_lexically(Path::new("a/../../b")),
            PathBuf::from("../b")
        );
    }

    // ---------------------------------------------------------------------
    // Success-side provenance (`kind`). The PATH is not comparable across
    // boxes but the KIND is, so a consumer publishing a workspace-root-relative
    // observation can say whether two readings describe the same concept.
    // ---------------------------------------------------------------------

    /// Each of the three explicit rungs is `Declared` — the operator said where
    /// the checkouts live, regardless of WHICH spelling they used.
    #[test]
    fn every_explicit_rung_reports_declared() {
        let f = Fixture::new("kind_declared");
        for (env, alias, cfg) in [
            (Some(f.s()), None, None),
            (None, Some(f.s()), None),
            (None, None, Some(f.s())),
        ] {
            let got = resolve_workspace_root(
                env.as_deref(),
                alias.as_deref(),
                cfg.as_deref(),
                None,
                None,
                None,
            );
            assert_eq!(got.root.as_deref(), Some(f.root.as_path()));
            assert_eq!(got.kind, WorkspaceRootKind::Declared);
        }
    }

    /// The ancestor walk is an inference about where this binary sits, not a
    /// declaration — and BOTH walk arms report the same rung, because the
    /// canonical-over-worktree preference picks a path, not a confidence.
    #[test]
    fn both_ancestor_walk_arms_report_discovered() {
        // Canonical arm: `.git` is a directory.
        let canonical = Fixture::new("kind_walk_canonical");
        let deep = canonical.root.join("qontinui-runner").join("src-tauri");
        let got = resolve_workspace_root(
            None,
            None,
            None,
            Some(WorkspaceAnchor::new(&deep, "qontinui-runner")),
            None,
            None,
        );
        assert_eq!(got.root.as_deref(), Some(canonical.root.as_path()));
        assert_eq!(got.kind, WorkspaceRootKind::Discovered);

        // Worktree-fallback arm: `.git` is a file and no canonical checkout
        // exists anywhere above it.
        let base = temp_path("kind_walk_worktree");
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
            None,
        );
        assert_eq!(got.root.as_deref(), Some(base.as_path()));
        assert_eq!(got.kind, WorkspaceRootKind::Discovered);
        let _ = std::fs::remove_dir_all(&base);
    }

    #[test]
    fn home_fallback_reports_home_default() {
        let f = Fixture::new("kind_home");
        let home = f.root.join("home");
        std::fs::create_dir_all(home.join(HOME_WORKSPACE_DIR)).unwrap();
        let got = resolve_workspace_root(None, None, None, None, None, Some(&home));
        assert_eq!(got.kind, WorkspaceRootKind::HomeDefault);
    }

    /// `Unresolved` is a STATED kind, not an absent one — and it holds exactly
    /// when `root` is `None`, which is the invariant every consumer branches on.
    #[test]
    fn unresolved_is_a_stated_kind_and_tracks_root_none() {
        let got = resolve_workspace_root(None, None, None, None, None, None);
        assert!(got.root.is_none());
        assert_eq!(got.kind, WorkspaceRootKind::Unresolved);
        assert!(got.rejected.is_some());
    }

    /// A REJECTED explicit value that falls through to a lower rung must report
    /// the rung that actually answered — not the one the operator intended.
    /// Reporting `Declared` here would make the provenance lie about the
    /// measurement in order to preserve intent, the same conflation
    /// `resolve_probe_scope` refuses.
    #[test]
    fn a_rejected_explicit_value_does_not_claim_declared() {
        let f = Fixture::new("kind_rejected_falls_through");
        let deep = f.root.join("qontinui-runner").join("src-tauri");
        let got = resolve_workspace_root(
            Some("   "),
            None,
            None,
            Some(WorkspaceAnchor::new(&deep, "qontinui-runner")),
            None,
            None,
        );
        assert_eq!(got.root.as_deref(), Some(f.root.as_path()));
        assert_eq!(got.kind, WorkspaceRootKind::Discovered);
        assert!(got.rejected.is_some(), "the fall-through must stay visible");
    }

    /// The wire strings are a published contract (the devenv envelope's
    /// `repos.repos_scope_kind`) — a rename silently breaks cross-box
    /// comparison, so pin them.
    #[test]
    fn workspace_root_kind_wire_strings_are_stable() {
        assert_eq!(WorkspaceRootKind::Declared.wire(), "declared");
        assert_eq!(WorkspaceRootKind::Discovered.wire(), "discovered");
        assert_eq!(
            WorkspaceRootKind::SessionRepoParent.wire(),
            "session_repo_parent"
        );
        assert_eq!(WorkspaceRootKind::HomeDefault.wire(), "home_default");
        assert_eq!(WorkspaceRootKind::Unresolved.wire(), "unresolved");
    }

    // ---------------------------------------------------------------------
    // Fall-through: an unusable explicit value never shadows a lower rung,
    // and the reason is always reported.
    // ---------------------------------------------------------------------

    #[test]
    fn unusable_env_falls_through_to_configured_and_reports_why() {
        let f = Fixture::new("fallthrough_cfg");
        let got = resolve_workspace_root(Some("   "), None, Some(&f.s()), None, None, None);
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
        let got = resolve_workspace_root(
            Some("relative/one"),
            Some("   "),
            Some(&f.s()),
            None,
            None,
            None,
        );
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
            resolve_workspace_root(Some("relative/path"), None, None, None, None, None).rejected,
            Some(RejectedRoot {
                source: RootSource::Env,
                reason: RootRejection::Relative
            })
        );
        let missing = temp_path("absent_dir");
        assert_eq!(
            resolve_workspace_root(None, missing.to_str(), None, None, None, None).rejected,
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

        let no_anchor = resolve_workspace_root(None, None, None, None, None, Some(&nowhere));
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
            None,
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

    /// `require()` fails with the ONE typed error, rendered through the shared
    /// refusal envelope: code, the rejection's wire value as discriminator, and
    /// the setting as next action.
    #[test]
    fn require_fails_with_the_typed_refusal() {
        let err = resolve_workspace_root(None, None, Some("relative/path"), None, None, None)
            .require()
            .unwrap_err();
        assert_eq!(
            err.rejected,
            RejectedRoot {
                source: RootSource::Configured,
                reason: RootRejection::Relative,
            }
        );
        let r = err.refusal(RefusalSource::Runner, "2026-09-30T00:00:00Z");
        assert_eq!(r.code, RefusalCode::WorkspaceRootUnresolved);
        assert_eq!(r.discriminator.as_deref(), Some("relative"));
        assert_eq!(r.next_action.kind, NextActionKind::SetSetting);
        assert_eq!(
            r.next_action.target.as_deref(),
            Some(WORKSPACE_ROOT_SETTING)
        );
        assert_eq!(r.source, RefusalSource::Runner);
        // The diagnostic detail keeps the input at fault and every probe.
        let detail = r.detail.as_deref().unwrap();
        assert!(detail.contains("the Workspace root setting is not an absolute path"));
        for probe in [
            "$QONTINUI_ROOT",
            "$QONTINUI_WORKSPACE_ROOT",
            "ancestor walk",
            "the repository the session was opened on",
            "<home>/qontinui-root",
        ] {
            assert!(detail.contains(probe), "detail must list {probe}: {detail}");
        }
    }

    /// The display is the refusal's sentence, plus the input the operator set
    /// when (and only when) one was at fault — never a variable they did not set.
    #[test]
    fn require_display_names_only_the_input_the_operator_set() {
        let setting = resolve_workspace_root(None, None, Some("relative/path"), None, None, None)
            .require()
            .unwrap_err()
            .to_string();
        assert_eq!(
            setting,
            "No workspace folder could be found for this operation (relative). \
             Set the \"Workspace root\" setting, then try again. \
             Cause: the Workspace root setting is not an absolute path."
        );
        assert!(!setting.contains("QONTINUI_ROOT"), "{setting}");

        let env = resolve_workspace_root(Some("relative/path"), None, None, None, None, None)
            .require()
            .unwrap_err()
            .to_string();
        assert!(
            env.ends_with("Cause: $QONTINUI_ROOT is not an absolute path."),
            "the variable the operator set is named: {env}"
        );

        let nothing = resolve_workspace_root(None, None, None, None, None, None)
            .require()
            .unwrap_err()
            .to_string();
        assert_eq!(
            nothing,
            "No workspace folder could be found for this operation (no_anchor). \
             Set the \"Workspace root\" setting, then try again."
        );
    }

    /// Every rejection has a distinct, stable wire value — they are the
    /// refusal's discriminators.
    #[test]
    fn root_rejection_wire_strings_are_stable_and_distinct() {
        let all = [
            (RootRejection::Blank, "blank"),
            (RootRejection::Relative, "relative"),
            (RootRejection::NotADirectory, "not_a_directory"),
            (RootRejection::FilesystemRoot, "filesystem_root"),
            (RootRejection::NoAnchor, "no_anchor"),
            (RootRejection::NothingConfigured, "nothing_configured"),
        ];
        for (r, wire) in all {
            assert_eq!(r.wire(), wire);
        }
        let wires: std::collections::HashSet<_> = all.iter().map(|(r, _)| r.wire()).collect();
        assert_eq!(wires.len(), all.len(), "wire strings must be distinct");
    }

    #[test]
    fn sibling_checkout_absent_refuses_only_when_nothing_serves_the_capability() {
        let embedded = SiblingCheckoutAbsent::new("tooling", SiblingFallback::EmbeddedCopy);
        assert_eq!(
            embedded.refusal(RefusalSource::Runner, "2026-09-30T00:00:00Z"),
            None,
            "an operation an embedded copy served was not refused"
        );
        assert_eq!(
            embedded.to_string(),
            "Source code this operation depends on is not available on this machine \
             (tooling). This capability is served from the copy built into the application."
        );

        let unsupported = SiblingCheckoutAbsent::new("tooling", SiblingFallback::Unsupported);
        let r = unsupported
            .refusal(RefusalSource::Runner, "2026-09-30T00:00:00Z")
            .expect("nothing serves it, so it is a refusal");
        assert_eq!(r.code, RefusalCode::SiblingCheckoutAbsent);
        assert_eq!(r.discriminator.as_deref(), Some("tooling"));
        assert_eq!(r.next_action.kind, NextActionKind::NoneTerminal);
        assert_eq!(
            r.detail.as_deref(),
            Some("This capability is not supported in this installation")
        );
        assert_eq!(
            unsupported.to_string(),
            "Source code this operation depends on is not available on this machine \
             (tooling). Nothing you can do will change this outcome. This capability is \
             not supported in this installation."
        );
    }

    #[test]
    fn require_returns_the_root_when_resolved() {
        let f = Fixture::new("require_ok");
        let got = resolve_workspace_root(Some(&f.s()), None, None, None, None, None)
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
