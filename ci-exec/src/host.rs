//! What the executor needs from the machine it runs on.
//!
//! The executor runs on three hosts (plan
//! `2026-10-04-coord-managed-ci-for-every-tenant-with-an-actions-free-mode`,
//! D4): the runner app lending a desktop, the headless CI host agent, and a
//! plain shell running `qontinui-ci`. Everything that differs between them is
//! behind one of the traits below, and a [`Host`] bundles one implementation
//! of each. Nothing else in this crate reads host state: no settings file, no
//! credential store, no global.
//!
//! | Trait | What it answers |
//! |---|---|
//! | [`CiSettings`] | the owner's consent values the executor reads per run |
//! | [`GithubAccess`] | a GitHub token, plus the host's ETag cache and request-budget meter |
//! | [`ProcessSpawn`] | how a child process is built, contained and reaped on this host |
//! | [`VolumeState`] | whether the CI root sits on a volume that may vanish |
//! | [`HostIdentity`] | where CI state lives on this host, and what to call the host |
//! | [`CanonicalConvergence`] | the `[canonical]` gate's measuring and acting halves |
//! | [`ModelInvoker`] | how one model call is made for a `model = true` step (optional: a host with none refuses model access) |
//! | [`SiblingReadAccess`] | the dispatch-scoped door that may mint a read token for a private sibling |
//!
//! Every trait is object-safe, so a host is assembled at runtime from
//! `Arc<dyn …>` parts and the crate never grows a generic parameter per trait.

use std::future::Future;
use std::path::{Path, PathBuf};
use std::pin::Pin;
use std::sync::Arc;

/// A boxed, `Send` future — the return type of every async trait method here,
/// so the traits stay object-safe.
pub type BoxFuture<'a, T> = Pin<Box<dyn Future<Output = T> + Send + 'a>>;

/// The owner's CI consent values that the executor reads per run.
///
/// Read once per dispatch, never cached by the executor: a grant governs the
/// NEXT dispatch, never one already running.
pub trait CiSettings: Send + Sync {
    /// Whether a manifest's `[canonical]` requirement may be satisfied by
    /// converging this box's global toolchains. Default-off on every host; a
    /// manifest can require canonical, but only the box's owner can authorise
    /// rewriting the box to meet it.
    fn canonical_converge(&self) -> bool;

    /// Whether this device's owner lets CI steps call models on this
    /// machine's accounts (plan `2026-10-13-ci-node-runs-model-dependent-eval-steps`
    /// D6, the device scope). Read only when the host offers a
    /// [`ModelInvoker`]; `false` refuses model access with
    /// `device_consent_off`. Defaults to `true`: the capability ships
    /// enabled, with the owner's off-switch.
    fn model_access(&self) -> bool {
        true
    }
}

/// GitHub reads of the dispatched repo during sibling resolution: a token, and
/// the host's metering and conditional-request cache.
///
/// The cache and meter are host state on purpose. The runner keeps one
/// process-wide ETag cache and one request-budget histogram across every
/// subsystem that talks to GitHub, so a dispatched PR's label read must land in
/// the same books; a one-shot CLI process has no cross-run books at all. A
/// sibling's own reads stay out of all of this: they carry the read door's
/// token or none ([`SiblingReadAccess`]).
pub trait GithubAccess: Send + Sync {
    /// A GitHub token for the dispatched repo, or `None` to read unauthenticated
    /// (60 requests/hour per IP). Asked for only when a pull request's labels
    /// will be read, and never sent on a read of a sibling.
    fn token(&self) -> BoxFuture<'_, Option<String>>;
    /// The `ETag` this host holds for `url`, if any. It is echoed as
    /// `If-None-Match` so an unchanged answer costs no request budget.
    fn cached_etag(&self, url: &str) -> Option<String>;
    /// The cached body behind a `304`. `None` means the validator outlived its
    /// body, and the caller treats the read as UNKNOWN.
    fn replay(&self, url: &str) -> Option<Vec<u8>>;
    /// Remember a validated `200`.
    fn store(&self, url: &str, etag: &str, body: &[u8]);
    /// Forget whatever is held for `url`.
    fn invalidate(&self, url: &str);
    /// Meter one completed exchange (any status, including a free `304`).
    fn record_response(&self, url: &str, status: u16, headers: &reqwest::header::HeaderMap);
    /// Meter a request that never produced a response. It cost a socket, not
    /// budget, and dropping it would make a DNS-failure storm look like silence.
    fn record_transport_error(&self, url: &str);
}

/// How child processes are built, contained and reaped on this host.
///
/// Every process the executor starts — git, a tool installer, a container
/// runtime, a manifest step — is built through [`ProcessSpawn::command`] or
/// [`ProcessSpawn::std_command`], so a host's posture (no console window on a
/// GUI app, a prompt-proof git environment) applies to all of them.
pub trait ProcessSpawn: Send + Sync {
    /// An async command for `program`, configured the host's way.
    fn command(&self, program: &str) -> tokio::process::Command;
    /// A blocking command for `program`, configured the host's way. Used where
    /// no runtime is available (a `Drop` reaper).
    fn std_command(&self, program: &str) -> std::process::Command;
    /// Pre-spawn half of the process-TREE reaper: whatever must be set before
    /// the child execs so its descendants can be killed with it (a process
    /// group on Unix).
    fn arm_tree(&self, cmd: &mut tokio::process::Command);
    /// Post-spawn half: dropping the returned guard kills the child's whole
    /// tree unless it is [`TreeGuard::disarm`]ed first. A call that is dropped
    /// mid-flight (a timeout, a cancellation) must not orphan the helpers it
    /// started — a git fetch's `index-pack` holds the repo's locks, not git.
    fn attach_tree(&self, child: &tokio::process::Child) -> Box<dyn TreeGuard>;
    /// A containment unit for one dispatch's step processes: whatever the host
    /// uses to bound their memory and to reap stragglers when the dispatch
    /// ends. Dropped at dispatch end.
    fn step_containment(&self) -> Box<dyn StepContainment>;
    /// Extra variables this host removes from EVERY CI step's environment, on
    /// top of the model-credential class the executor always removes
    /// ([`crate::model::MODEL_CREDENTIAL_ENV`]). A host that keeps credential
    /// values in its own environment (the runner's login passwords) lists
    /// them here. Default: none.
    fn scrubbed_env(&self) -> &'static [&'static str] {
        &[]
    }
    /// Run CPU-bound blocking work (archive extraction) off the async
    /// workers. `Err` means the task did not complete (it panicked or was
    /// cancelled). The default is plain [`tokio::task::spawn_blocking`]; a host
    /// that accounts for its blocking pool (the runner's tracked lanes)
    /// overrides it so this work lands in the same books.
    fn run_blocking(
        &self,
        job: Box<dyn FnOnce() + Send>,
    ) -> BoxFuture<'static, Result<(), String>> {
        Box::pin(async move {
            tokio::task::spawn_blocking(job)
                .await
                .map_err(|e| e.to_string())
        })
    }
}

/// Run `f` through `process`'s [`ProcessSpawn::run_blocking`] and hand back its
/// value.
pub(crate) async fn run_blocking<R: Send + 'static>(
    process: &dyn ProcessSpawn,
    f: impl FnOnce() -> R + Send + 'static,
) -> Result<R, String> {
    let (tx, rx) = tokio::sync::oneshot::channel();
    process
        .run_blocking(Box::new(move || {
            let _ = tx.send(f());
        }))
        .await?;
    rx.await
        .map_err(|_| "the blocking task ended without a result".to_string())
}

/// Kills a child's process tree on drop unless disarmed.
pub trait TreeGuard: Send {
    /// Release the tree WITHOUT killing it — the call completed normally.
    fn disarm(self: Box<Self>);
}

/// One dispatch's containment for step processes.
pub trait StepContainment: Send + Sync {
    /// Bring a freshly spawned step child under this containment. Children it
    /// spawns inherit membership.
    fn adopt(&self, child: &tokio::process::Child);
}

/// Whether the CI root sits on a volume that may not be there.
pub trait VolumeState: Send + Sync {
    /// `Some(reason)` when `root` is on a declared removable volume that is not
    /// provably present and correct right now. Nothing may be written under
    /// `root` then: a build into an unmounted stub fills the very disk the
    /// relocation exists to relieve. `None` when the root is not on such a
    /// volume, or the volume checks out.
    fn refusal_reason(&self, root: &Path) -> Option<String>;
}

/// Where CI state lives on this host, and what to call the host.
pub trait HostIdentity: Send + Sync {
    /// The directory that holds the primary checkouts the executor fetches
    /// into (`<root>/<repo>`), the per-dispatch worktrees
    /// (`<root>/.ci-worktrees`), the warm CI target dirs (`<root>/.ci-target`)
    /// and the tool cache. `Err` names why it could not be resolved.
    fn ci_root(&self) -> Result<PathBuf, String>;
    /// A short human label for this host, printed in the run log so a reader
    /// can tell where a job ran.
    fn label(&self) -> String;
}

/// How one declared toolchain stands against the canonical configuration,
/// before any convergence. Produced by the host's measurement
/// ([`CanonicalConvergence::measure`]); every verdict the gate reaches rests on
/// it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Standing {
    /// Both captures carry the key with the same value.
    Agreed { value: String },
    /// Canonical carries a value this box does not match. `local` is `None`
    /// when this box does not report the key at all.
    Drifted {
        local: Option<String>,
        canonical: String,
    },
    /// This box could not READ the key. Not "you do not have it".
    Unmeasured,
    /// This box has a value; canonical does not. Nothing to converge TO.
    NoCanonicalValue { local: String },
    /// Neither capture carries the key at all.
    AbsentBoth,
}

/// One measurement of this box against the canonical configuration.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CanonicalReading {
    /// The canonical machine's name (or id), for the verdict's provenance.
    pub canonical_machine: Option<String>,
    /// This box IS the canonical machine. It is still checked: the question
    /// becomes "is the toolchain present and measured", not "does it match my
    /// own last upload".
    pub is_canonical_self: bool,
    /// What the `versions` section said.
    pub versions: VersionsReading,
}

/// The `versions` section of a [`CanonicalReading`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum VersionsReading {
    /// The canonical configuration carries no `versions` section.
    NoCanonicalSection,
    /// Canonical has the section; this box produced no capture for it.
    LocalSectionAbsent,
    /// One standing per requested key, in the order requested.
    Standings(Vec<(String, Standing)>),
}

/// What a convergence attempt did.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ConvergeReport {
    /// `(key, new value)` for every key the convergence actually moved.
    pub moved: Vec<(String, String)>,
    /// Lines worth logging, in the convergence machinery's own words.
    pub notes: Vec<String>,
    /// Why keys that did not move did not, in the convergence machinery's own
    /// words. Read only when a requested key is missing from `moved`.
    pub failure_reason: String,
}

/// The `[canonical]` gate's two halves: measuring this box against the
/// canonical configuration, and converging it.
pub trait CanonicalConvergence: Send + Sync {
    /// Measure this box for `keys`. `Err` means the canonical configuration
    /// could not be read at all — an UNKNOWN, which the gate refuses on.
    fn measure<'a>(&'a self, keys: &'a [String])
        -> BoxFuture<'a, Result<CanonicalReading, String>>;
    /// Converge exactly `keys` toward canonical (the declared keys, never
    /// more: the blast radius of a convergence is the requirement). `Err`
    /// means the convergence did not complete.
    fn converge<'a>(&'a self, keys: &'a [String]) -> BoxFuture<'a, Result<ConvergeReport, String>>;
}

/// One model call, as the dispatch's broker hands it to the host.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InvokeRequest {
    /// The broker's 1-based index of this call within the dispatch.
    pub call_index: u64,
    /// The model name as the step requested it (a declared registry name or
    /// pinned id).
    pub requested: String,
    /// What to pass as the CLI's `--model`: the family alias, or the pinned
    /// id itself.
    pub cli_model: String,
    /// The prompt. The host passes it on stdin, never in argv.
    pub prompt: String,
}

/// Tokens as the provider reported them for one call.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct TokenUsage {
    pub input: u64,
    pub output: u64,
    pub cache_creation: u64,
    pub cache_read: u64,
}

impl TokenUsage {
    /// Input + output + cache tokens: what `max_tokens` counts.
    pub fn total(&self) -> u64 {
        self.input
            .saturating_add(self.output)
            .saturating_add(self.cache_creation)
            .saturating_add(self.cache_read)
    }
}

/// A completed model call.
#[derive(Debug, Clone, PartialEq)]
pub struct InvokeReply {
    /// The answer text.
    pub result: String,
    /// The concrete model id the provider reports (`modelUsage`). `None` is
    /// UNKNOWN, never a guess.
    pub model_id: Option<String>,
    /// Tokens as the provider reported them. `None` when it reported none —
    /// the ledger then shows the call as uncovered rather than as zero.
    pub usage: Option<TokenUsage>,
    /// The provider-reported cost in USD, when it reports one.
    pub cost_usd: Option<f64>,
    /// How the call was routed (`subscription`, `gateway`).
    pub route: String,
    /// The account label it ran on, when the route has accounts.
    pub account: Option<String>,
    /// How many provider attempts the host made (a provider-limit retry on
    /// the next account is a second attempt).
    pub attempts: u32,
}

/// Why a model call did not complete.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum InvokeError {
    /// Every model account this host could use is exhausted; the earliest
    /// one resets at `reset_at`. The broker holds the call until then when
    /// that falls inside the dispatch's wall budget.
    AllAccountsExhausted { reset_at: std::time::SystemTime },
    /// The provider or the CLI failed. `retryable` says whether the same
    /// request may succeed later.
    Provider { message: String, retryable: bool },
    /// The host's own per-call deadline ran out.
    Timeout,
}

/// How one model call is made on this host (plan
/// `2026-10-13-ci-node-runs-model-dependent-eval-steps`, D1/D5).
///
/// The credential lives behind this trait and never reaches a step: a step
/// gets only the dispatch's broker address and a capability bound to one
/// dispatch and one budget.
pub trait ModelInvoker: Send + Sync {
    /// Make one call. DROPPING the returned future cancels the call — the
    /// broker drops it when the client disconnects or the dispatch ends — so
    /// an implementation that spawns a process must kill it on drop
    /// (`kill_on_drop`), or the spend continues for an answer nobody reads.
    fn complete(
        &self,
        request: InvokeRequest,
    ) -> BoxFuture<'static, Result<InvokeReply, InvokeError>>;

    /// Whether this host can make calls right now, checked once per dispatch.
    /// `Err` carries the refusal reason the dispatch reports, one of the
    /// closed runner reasons (`cli_missing`, `cli_version_unsupported`,
    /// `no_model_account`, `gateway_unresolved`).
    fn ready(&self) -> Result<(), String>;

    /// The argv of the `qontinui-ci-model` shim this host ships, or `None`
    /// when it has none. Its first element must be a non-empty regular file,
    /// or the dispatch refuses model access with `shim_missing`.
    fn shim_argv(&self) -> Option<Vec<String>>;

    /// Whether to serve a broker connection from this peer. A host refuses
    /// peers from another OS user here. Default: admit.
    fn accept_peer(&self, _peer: &tokio::net::TcpStream) -> bool {
        true
    }
}

/// What a host's sibling read door answered for one `(dispatch, sibling)`.
///
/// Four answers, because "absent" and "unanswered" are different answers and
/// only the first is a NO (plan
/// `2026-10-13-local-lane-ci-reads-a-private-sibling-through-a-dispatch-scoped-door`,
/// D3): a sibling the door could not answer for is read on the public path,
/// and if that fails the error says the door's answer was UNKNOWN — never
/// "no grant".
#[derive(Debug)]
pub enum SiblingReadAnswer {
    /// A minted read token for exactly this sibling, and its expiry as the door
    /// reported it (an RFC 3339 string; logged, never parsed).
    Granted(crate::sibling_git::ReadToken, String),
    /// The door answered and refused, with its typed code
    /// (`no_ci_read_grant`, `fork_pr`, `dispatch_not_live`, …).
    NoGrant { code: String },
    /// This host has no such door (a standalone `qontinui-ci` run).
    NotOffered { reason: String },
    /// The door could not be asked or did not answer usably (unreachable,
    /// 5xx, `grant_unestablished`, a missing credential).
    Unknown { reason: String },
}

/// The dispatch-scoped sibling read door.
///
/// The executor asks it ONCE per declared sibling per dispatch, before any
/// read of that sibling. On a coord-dispatched node the host implements it as
/// `POST /coord/ci/dispatches/:dispatch_id/sibling-read-token` with body
/// `{repo}`, answering `{token, expires_at, repository, permissions}`; ci-exec
/// has no coord transport of its own, so the call is the host's. The executor
/// revokes a granted token itself once that sibling is provisioned.
pub trait SiblingReadAccess: Send + Sync {
    /// Ask for a read token for `repo` (`owner/name`) on behalf of
    /// `dispatch_id`.
    fn read_token<'a>(
        &'a self,
        dispatch_id: &'a str,
        repo: &'a str,
    ) -> BoxFuture<'a, SiblingReadAnswer>;
}

/// One implementation of each host trait.
#[derive(Clone)]
pub struct Host {
    pub settings: Arc<dyn CiSettings>,
    pub github: Arc<dyn GithubAccess>,
    pub process: Arc<dyn ProcessSpawn>,
    pub volume: Arc<dyn VolumeState>,
    pub identity: Arc<dyn HostIdentity>,
    pub canonical: Arc<dyn CanonicalConvergence>,
    /// How model calls are made for `model = true` steps. `None` — a host
    /// with no invoker — refuses model access with `host_has_no_broker`; the
    /// steps still run.
    pub model: Option<Arc<dyn ModelInvoker>>,
    /// The dispatch-scoped read door asked once per declared sibling. A host
    /// with no door installs one answering `NotOffered` (see
    /// `standalone::NoSiblingDoor`). A sibling the door does not grant is read
    /// on the public path with NO credential: the host's own GitHub token is
    /// used only for reads of the dispatched repo, never for a sibling.
    pub sibling_reads: Arc<dyn SiblingReadAccess>,
}
