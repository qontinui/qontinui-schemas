//! `qontinui-ci import`: GitHub workflow files → a v2 `.qontinui/ci.toml`,
//! and `qontinui-ci import --report`: does a `ci.toml` cover a repo's
//! workflow jobs?
//!
//! Plan `2026-10-04-coord-managed-ci-for-every-tenant-with-an-actions-free-mode`,
//! D2 and Phase 8. The importer is the one-shot onboarding path; the manifest it
//! writes is the single source of truth afterwards (`gen-workflow` regenerates
//! the GitHub leg from it, never the other way round).
//!
//! # Nothing is dropped silently
//!
//! Every construct of every workflow ends up in exactly one of four places:
//!
//! * **translated** — a `[[jobs]]`, `[[jobs.steps]]`, `[[jobs.services]]`,
//!   `[[tools]]` or `[[siblings]]` entry;
//! * **UNTRANSLATED** — not represented, and must be handled by hand (an
//!   arbitrary `uses:` action, a matrix dimension the manifest cannot expand,
//!   an `if:` condition, an expression or secret, a script the argv subset in
//!   [`crate::shell`] refuses, an env key outside the allowlist, …);
//! * **TRANSLATED WITH A CHANGE** — represented, but not literally (a node
//!   major resolved to the registry's pinned release, a job timeout spread over
//!   its steps, a script's message lines dropped, a sibling checkout path that
//!   moves);
//! * **NO-OP** — accounted for and needs nothing (`actions/checkout`, a cache
//!   action, `permissions:`, a service's credentials, which the executor
//!   generates).
//!
//! The last three are written as a header comment block of the generated
//! `ci.toml` and printed by `import --report`.
//!
//! # What a gate job is
//!
//! A manifest job with no `schedule` gates pull requests and merge
//! candidates; one with a `schedule` runs only on it. So a workflow triggered
//! by `pull_request` or a branch `push` imports as gate jobs, a workflow
//! triggered only by `schedule` imports as scheduled jobs (its first cron), and
//! a workflow triggered by neither (`release`, `workflow_call`,
//! `workflow_dispatch` alone, a tag push) is not CI and imports nothing — each
//! of its jobs is listed as untranslated with the reason.
//!
//! # The coverage rule (`import --report`)
//!
//! See [`COVERAGE_RULE`]. In short: a workflow job is COVERED when every one of
//! its *gate commands* has a manifest step running the same command in the
//! same directory. Command identity is a normalized *signature*, so
//! `poetry run pytest` and `pytest`, `npm run lint` and `pnpm lint`,
//! `./scripts/x.sh` and `bash scripts/x.sh`, `python3 -m mypy` and `mypy` are
//! the same command, while arguments after the first flag are ignored.

use std::collections::BTreeMap;

use crate::gha::{self, Job, Matrix, RunsOn, Step, Workflow};
use crate::manifest::{self, CiManifest, EnvKeyClass, Os, MAX_STEP_TIMEOUT_SECS};
use crate::shell;

/// One workflow file handed to the importer.
#[derive(Debug, Clone)]
pub struct WorkflowSource {
    /// How the file is named in output — normally its path as given.
    pub label: String,
    pub text: String,
}

/// How a source construct was carried over.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum ItemKind {
    /// Not represented in the manifest; must be handled by hand.
    Untranslated,
    /// Represented, but not literally.
    Changed,
    /// Accounted for; nothing to declare.
    NoOp,
}

impl ItemKind {
    pub fn heading(self) -> &'static str {
        match self {
            ItemKind::Untranslated => "UNTRANSLATED",
            ItemKind::Changed => "TRANSLATED WITH A CHANGE",
            ItemKind::NoOp => "NO-OPS",
        }
    }
}

/// One reported construct.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ImportItem {
    pub kind: ItemKind,
    /// `file › job › step "name"` (as much of it as applies).
    pub location: String,
    pub detail: String,
}

/// A step the importer produced.
#[derive(Debug, Clone)]
pub struct ImportedStep {
    pub name: String,
    pub command: Vec<String>,
    pub env: BTreeMap<String, String>,
    pub timeout_secs: Option<u64>,
    /// Repo-relative; `None` is the root.
    pub working_dir: Option<String>,
    /// Where it came from, for the comment above it.
    pub source: String,
}

/// A job the importer produced.
#[derive(Debug, Clone)]
pub struct ImportedJob {
    pub name: String,
    pub source: String,
    pub os: Vec<Os>,
    pub needs: Vec<String>,
    pub schedule: Option<String>,
    /// `(registry name, version, digest)`.
    pub services: Vec<(String, String, Option<String>)>,
    pub steps: Vec<ImportedStep>,
}

/// Everything an import produced.
#[derive(Debug, Clone)]
pub struct ImportOutcome {
    pub sources: Vec<String>,
    pub jobs: Vec<ImportedJob>,
    /// `(tool, version)`.
    pub tools: Vec<(String, String)>,
    /// `owner/name`.
    pub siblings: Vec<String>,
    pub items: Vec<ImportItem>,
    /// The generated `ci.toml`, header included.
    pub manifest_toml: String,
    /// The generated manifest, validated — `None` when no job could be
    /// translated (the text is then not a valid manifest).
    pub manifest: Option<CiManifest>,
}

impl ImportOutcome {
    pub fn count(&self, kind: ItemKind) -> usize {
        self.items.iter().filter(|i| i.kind == kind).count()
    }
}

fn basename(label: &str) -> &str {
    label.rsplit(['/', '\\']).next().unwrap_or(label)
}

fn stem(label: &str) -> &str {
    let b = basename(label);
    b.strip_suffix(".yml")
        .or_else(|| b.strip_suffix(".yaml"))
        .unwrap_or(b)
}

/// A workflow job id as a manifest job name: lowercase, `[a-z0-9-]`, at most
/// 48 characters.
pub fn slug(s: &str) -> String {
    let mut out = String::new();
    for c in s.chars() {
        let c = c.to_ascii_lowercase();
        if c.is_ascii_lowercase() || c.is_ascii_digit() {
            out.push(c);
        } else if !out.ends_with('-') && !out.is_empty() {
            out.push('-');
        }
    }
    let mut out: String = out.trim_matches('-').chars().take(48).collect();
    while out.ends_with('-') {
        out.pop();
    }
    if out.is_empty() {
        out.push_str("job");
    }
    out
}

/// `${{ secrets.X }}` names referenced in `text`.
fn secrets_in(text: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut rest = text;
    while let Some(i) = rest.find("secrets.") {
        let name: String = rest[i + 8..]
            .chars()
            .take_while(|c| c.is_ascii_alphanumeric() || *c == '_' || *c == '-')
            .collect();
        if !name.is_empty() && !out.contains(&name) {
            out.push(name);
        }
        rest = &rest[i + 8..];
    }
    out
}

fn with_secrets(detail: String, text: &str) -> String {
    let s = secrets_in(text);
    if s.is_empty() {
        detail
    } else {
        format!("{detail} [secrets referenced: {}]", s.join(", "))
    }
}

/// Normalize a `working-directory` value to repo-relative form. `Err` when it
/// is an expression or leaves the repository.
fn normalize_wd(raw: &str) -> Result<String, String> {
    let s = shell::substitute_workspace_expr(raw.trim());
    if s.contains("${{") {
        return Err(format!("working-directory {raw:?} is an expression"));
    }
    let s = s
        .strip_prefix("$GITHUB_WORKSPACE")
        .or_else(|| s.strip_prefix("${GITHUB_WORKSPACE}"))
        .map(|r| r.trim_start_matches('/').to_string())
        .unwrap_or(s);
    if s.contains('$') {
        return Err(format!("working-directory {raw:?} depends on a variable"));
    }
    shell::join_rel("", &s).ok_or_else(|| format!("working-directory {raw:?} leaves the repository"))
}

fn is_true(s: &Option<gha::Scalar>) -> Option<bool> {
    s.as_ref().map(|v| v.0.trim() == "true" || v.is_expression())
}

/// A step condition that only makes a step a diagnostic (runs on failure or
/// always) rather than part of the gate.
fn diagnostic_condition(expr: &str) -> bool {
    let e: String = expr.chars().filter(|c| !c.is_whitespace()).collect();
    let e = e
        .strip_prefix("${{")
        .and_then(|x| x.strip_suffix("}}"))
        .unwrap_or(&e);
    // Only a step that runs SOLELY when the job already failed or was
    // cancelled is a diagnostic. `always()`, `!cancelled()` and
    // `success() || failure()` steps run on the happy path and gate.
    matches!(
        e,
        "failure()" | "cancelled()" | "failure()||cancelled()" | "cancelled()||failure()"
    )
}

/// `continue-on-error` that is literally `true` (an expression may be false).
fn literally_true(s: &Option<gha::Scalar>) -> bool {
    s.as_ref().is_some_and(|v| v.0.trim() == "true")
}

struct Ctx {
    items: Vec<ImportItem>,
    tools: Vec<(String, String)>,
    siblings: Vec<String>,
}

impl Ctx {
    fn push(&mut self, kind: ItemKind, location: impl Into<String>, detail: impl Into<String>) {
        let item = ImportItem {
            kind,
            location: location.into(),
            detail: detail.into(),
        };
        if !self.items.contains(&item) {
            self.items.push(item);
        }
    }

    fn add_tool(&mut self, loc: &str, name: &str, version: &str) {
        match self.tools.iter().find(|(n, _)| n == name) {
            Some((_, v)) if v == version => {}
            Some((_, v)) => {
                let v = v.clone();
                self.push(
                    ItemKind::Untranslated,
                    loc,
                    format!(
                        "asks for {name} {version}, but {name} {v} is already declared — a \
                         manifest declares one version per tool"
                    ),
                );
            }
            None => self.tools.push((name.to_string(), version.to_string())),
        }
    }

    fn add_sibling(&mut self, loc: &str, repo: &str, how: &str) {
        if manifest::validate_sibling_repo(repo).is_err() {
            self.push(
                ItemKind::Untranslated,
                loc,
                format!("{how} {repo:?}, which is not an owner/name repository"),
            );
            return;
        }
        let dir = crate::local_repo_name(repo);
        if let Some(existing) = self.siblings.iter().find(|s| crate::local_repo_name(s) == dir) {
            let existing = existing.clone();
            if existing == repo {
                self.push(
                    ItemKind::Changed,
                    loc,
                    format!(
                        "{how} {repo} — already declared as [[siblings]] repo = \"{repo}\", \
                         materialised at ../{dir}; a command that used another path for it must \
                         be re-pointed"
                    ),
                );
            } else {
                self.push(
                    ItemKind::Untranslated,
                    loc,
                    format!(
                        "{how} {repo}, whose directory ../{dir} is already taken by sibling \
                         {existing}"
                    ),
                );
            }
            return;
        }
        if self.siblings.len() >= 8 {
            self.push(
                ItemKind::Untranslated,
                loc,
                format!("{how} {repo}: the manifest allows at most 8 [[siblings]]"),
            );
            return;
        }
        self.siblings.push(repo.to_string());
        self.push(
            ItemKind::Changed,
            loc,
            format!(
                "{how} {repo} → [[siblings]] repo = \"{repo}\" (pin = declared-adaptation, the \
                 default). The executor materialises it at ../{dir} beside the checkout; a \
                 command that used another path for it must be re-pointed, and the pin should \
                 be reviewed"
            ),
        );
    }
}

/// How a workflow's triggers classify its jobs.
enum Class {
    Gate,
    Scheduled(String),
    NotCi(String),
}

fn classify(wf: &Workflow, ctx: &mut Ctx) -> Class {
    let t = &wf.triggers;
    let file = basename(&wf.file);
    let gate = t.pull_request.is_some()
        || t.pull_request_target.is_some()
        || t.push.as_ref().is_some_and(|p| p.fires_for_some_branch());
    for ev in &t.events {
        match ev.as_str() {
            "pull_request" | "pull_request_target" | "push" | "schedule" => {}
            "workflow_dispatch" => ctx.push(
                ItemKind::NoOp,
                file,
                "trigger workflow_dispatch — `qontinui-ci run --job <name>` is the manual run",
            ),
            other => ctx.push(
                ItemKind::Untranslated,
                file,
                format!("trigger `{other}` is not carried — a manifest job runs for pull requests, merge candidates and pushes, or on its schedule"),
            ),
        }
    }
    if gate {
        if !t.schedules.is_empty() {
            ctx.push(
                ItemKind::Changed,
                file,
                format!(
                    "also runs on schedule {:?}; imported as gate jobs only (a manifest job is \
                     either a gate or scheduled — add a scheduled copy by hand if the nightly \
                     run matters)",
                    t.schedules
                ),
            );
        }
        let filtered: Vec<&str> = [
            ("pull_request", t.pull_request.as_ref()),
            ("pull_request_target", t.pull_request_target.as_ref()),
            ("push", t.push.as_ref()),
        ]
        .into_iter()
        .filter(|(_, f)| f.is_some_and(|f| f.path_filtered))
        .map(|(n, _)| n)
        .collect();
        if !filtered.is_empty() {
            ctx.push(
                ItemKind::Changed,
                file,
                format!(
                    "{} filtered by paths — a manifest job runs for every change",
                    filtered.join(", ")
                ),
            );
        }
        let narrowed: Vec<String> = [
            ("pull_request", t.pull_request.as_ref()),
            ("pull_request_target", t.pull_request_target.as_ref()),
            ("push", t.push.as_ref()),
        ]
        .into_iter()
        .filter_map(|(n, f)| {
            let f = f?;
            let mut parts = Vec::new();
            if let Some(b) = &f.branches {
                parts.push(format!("branches {b:?}"));
            }
            if let Some(b) = &f.branches_ignore {
                parts.push(format!("branches-ignore {b:?}"));
            }
            if let Some(ty) = &f.types {
                parts.push(format!("types {ty:?}"));
            }
            (!parts.is_empty()).then(|| format!("{n} {}", parts.join(" ")))
        })
        .collect();
        if !narrowed.is_empty() {
            ctx.push(
                ItemKind::Changed,
                file,
                format!(
                    "trigger filters not carried ({}) — a manifest gate job runs for every pull \
                     request and merge candidate",
                    narrowed.join("; ")
                ),
            );
        }
        if t.pull_request_target.is_some() {
            ctx.push(
                ItemKind::Changed,
                file,
                "pull_request_target (runs with the base branch's privileges) imported as an \
                 ordinary gate — a manifest job always runs the dispatched commit's own manifest",
            );
        }
        return Class::Gate;
    }
    if let Some(first) = t.schedules.first() {
        if t.schedules.len() > 1 {
            ctx.push(
                ItemKind::Changed,
                file,
                format!(
                    "has {} cron schedules {:?}; a manifest job carries one — the first is used",
                    t.schedules.len(),
                    t.schedules
                ),
            );
        }
        return Class::Scheduled(first.clone());
    }
    Class::NotCi(format!(
        "the workflow is triggered only by {:?} — not a pull-request/push gate or a schedule, so \
         it is not CI to import",
        t.events
    ))
}

/// Map a `runs-on` (and the matrix dimension it may read) to manifest OSes.
/// Returns the OSes and the matrix dimension consumed, if any.
fn map_runs_on(job: &Job, loc: &str, ctx: &mut Ctx) -> (Vec<Os>, Option<String>) {
    fn os_of(label: &str) -> Option<Os> {
        let l = label.to_ascii_lowercase();
        if l.contains("ubuntu") || l.contains("linux") || l.contains("debian") {
            Some(Os::Linux)
        } else if l.contains("windows") || l.starts_with("win") {
            Some(Os::Windows)
        } else if l.contains("macos") || l.contains("mac-") || l == "mac" || l.contains("osx") {
            Some(Os::Macos)
        } else {
            None
        }
    }
    let labels = match &job.runs_on {
        RunsOn::Labels(l) => l.clone(),
        RunsOn::Missing => {
            ctx.push(ItemKind::Changed, loc, "no runs-on; imported as os = any");
            return (vec![Os::Any], None);
        }
        RunsOn::Other(s) => {
            ctx.push(
                ItemKind::Changed,
                loc,
                format!("runs-on {s} (a runner group) imported as os = any"),
            );
            return (vec![Os::Any], None);
        }
    };
    // `${{ matrix.<dim> }}` over a literal dimension expands to an OS matrix.
    if labels.len() == 1 && labels[0].contains("${{") {
        let expr = labels[0].clone();
        let inner = expr
            .trim()
            .trim_start_matches("${{")
            .trim_end_matches("}}")
            .trim()
            .to_string();
        if let (Some(dim), Some(Matrix::Dimensions(dims))) = (inner.strip_prefix("matrix."), &job.matrix) {
            if let Some((_, values)) = dims.iter().find(|(d, _)| d == dim) {
                let mut oses: Vec<Os> = Vec::new();
                let mut unknown: Vec<String> = Vec::new();
                for v in values {
                    match os_of(v) {
                        Some(o) if !oses.contains(&o) => oses.push(o),
                        Some(_) => {}
                        None => unknown.push(v.clone()),
                    }
                }
                if unknown.is_empty() && !oses.is_empty() {
                    if values.len() != oses.len() {
                        ctx.push(
                            ItemKind::Changed,
                            loc,
                            format!("runs-on matrix {values:?} collapsed to one run per OS"),
                        );
                    }
                    return (oses, Some(dim.to_string()));
                }
                ctx.push(
                    ItemKind::Untranslated,
                    loc,
                    format!("runs-on matrix values {unknown:?} name no OS; imported as os = any"),
                );
                return (vec![Os::Any], Some(dim.to_string()));
            }
        }
        ctx.push(
            ItemKind::Untranslated,
            loc,
            format!("runs-on {expr} is an expression; imported as os = any"),
        );
        return (vec![Os::Any], None);
    }
    let oses: Vec<Os> = labels.iter().filter_map(|l| os_of(l)).collect();
    match oses.as_slice() {
        [one] => {
            if labels.iter().any(|l| l == "self-hosted") || labels.len() > 1 {
                ctx.push(
                    ItemKind::Changed,
                    loc,
                    format!("runs-on {labels:?} imported as os = {} (other labels dropped)", one.as_str()),
                );
            }
            (vec![*one], None)
        }
        [] => {
            ctx.push(
                ItemKind::Changed,
                loc,
                format!("runs-on {labels:?} names no OS; imported as os = any"),
            );
            (vec![Os::Any], None)
        }
        _ => {
            ctx.push(
                ItemKind::Changed,
                loc,
                format!("runs-on {labels:?} names several OSes for one runner; imported as os = any"),
            );
            (vec![Os::Any], None)
        }
    }
}

/// Filter env pairs through the manifest's env rules, reporting each key that
/// does not carry over. `kinds` are the service kinds this job declares.
fn filter_env(pairs: &[(String, String)], loc: &str, kinds: &[&str], ctx: &mut Ctx) -> Vec<(String, String)> {
    let mut out = Vec::new();
    for (k, v) in pairs {
        let v_sub = shell::substitute_workspace_expr(v);
        if v_sub.contains("${{") || v_sub.contains("$GITHUB_WORKSPACE") {
            ctx.push(
                ItemKind::Untranslated,
                loc,
                with_secrets(
                    format!("env {k} = {v:?} is an expression — a manifest env value is a literal; not imported"),
                    v,
                ),
            );
            continue;
        }
        match manifest::classify_step_env_key(k) {
            EnvKeyClass::Allowed => out.push((k.clone(), v.clone())),
            EnvKeyClass::ExecutorOwned(owner) => ctx.push(
                ItemKind::Untranslated,
                loc,
                format!("env {k} is set by the executor — express it as {owner}; not imported"),
            ),
            EnvKeyClass::ServiceExported(kind) if kinds.contains(&kind) => ctx.push(
                ItemKind::NoOp,
                loc,
                format!("env {k}: exported by the executor from the job's {kind} service (credentials are generated per dispatch)"),
            ),
            EnvKeyClass::ServiceExported(kind) => ctx.push(
                ItemKind::Untranslated,
                loc,
                format!("env {k} is a {kind} connection, but the job declares no {kind} service; not imported"),
            ),
            EnvKeyClass::NotAllowed => ctx.push(
                ItemKind::Untranslated,
                loc,
                format!("env {k} is outside the manifest's env allowlist; not imported"),
            ),
        }
    }
    out
}

fn timeout_secs(raw: &gha::Scalar, loc: &str, what: &str, ctx: &mut Ctx) -> Option<u64> {
    if raw.is_expression() {
        ctx.push(
            ItemKind::Untranslated,
            loc,
            format!("{what} timeout-minutes {} is an expression; the default applies", raw.0),
        );
        return None;
    }
    let mins: f64 = match raw.0.trim().parse() {
        Ok(m) => m,
        Err(_) => {
            ctx.push(
                ItemKind::Untranslated,
                loc,
                format!("{what} timeout-minutes {:?} is not a number", raw.0),
            );
            return None;
        }
    };
    let secs = (mins * 60.0).round().max(1.0) as u64;
    if secs > MAX_STEP_TIMEOUT_SECS {
        ctx.push(
            ItemKind::Changed,
            loc,
            format!(
                "{what} timeout-minutes {} exceeds the manifest's per-step cap; clamped to \
                 {MAX_STEP_TIMEOUT_SECS}s",
                raw.0
            ),
        );
        return Some(MAX_STEP_TIMEOUT_SECS);
    }
    Some(secs)
}

/// Map a service image to a registry entry.
fn map_service(svc: &gha::Service, loc: &str, kinds: &mut Vec<&'static str>, ctx: &mut Ctx) -> Option<(String, String, Option<String>)> {
    let sloc = format!("{loc} › service {}", svc.key);
    let Some(image) = &svc.image else {
        ctx.push(ItemKind::Untranslated, &sloc, "has no image");
        return None;
    };
    if image.contains("${{") {
        ctx.push(
            ItemKind::Untranslated,
            &sloc,
            format!("image {image} is an expression — a manifest names a pinned registry entry"),
        );
        return None;
    }
    let (rest, digest) = match image.split_once('@') {
        Some((r, d)) => (r, Some(d.to_string())),
        None => (image.as_str(), None),
    };
    let (repo, tag) = match rest.rsplit_once(':') {
        Some((r, t)) if !t.contains('/') => (r, Some(t)),
        _ => (rest, None),
    };
    let repo = repo
        .trim_start_matches("docker.io/")
        .trim_start_matches("index.docker.io/")
        .trim_start_matches("library/");
    let (name, kind) = match repo {
        "postgres" => ("postgres", "postgres"),
        "pgvector/pgvector" => ("postgres-pgvector", "postgres"),
        "redis" => ("redis", "redis"),
        other => {
            ctx.push(
                ItemKind::Untranslated,
                &sloc,
                format!(
                    "image {other} is not in the closed service registry (known: postgres, \
                     postgres-pgvector, redis) — refused; a manifest cannot name an image"
                ),
            );
            return None;
        }
    };
    let Some(tag) = tag else {
        ctx.push(
            ItemKind::Untranslated,
            &sloc,
            format!("image {image} has no tag — a manifest service must pin a version"),
        );
        return None;
    };
    if let Err(e) = manifest::validate_image_tag(tag) {
        ctx.push(ItemKind::Untranslated, &sloc, format!("image {image}: {e}"));
        return None;
    }
    if let Some(d) = &digest {
        if let Err(e) = manifest::validate_image_digest(d) {
            ctx.push(ItemKind::Untranslated, &sloc, format!("image {image}: {e}"));
            return None;
        }
    }
    if kinds.contains(&kind) {
        ctx.push(
            ItemKind::Untranslated,
            &sloc,
            format!("a second {kind} service — a job may declare one service of each kind"),
        );
        return None;
    }
    kinds.push(kind);
    if !svc.other_keys.is_empty() {
        ctx.push(
            ItemKind::NoOp,
            &sloc,
            format!(
                "{} — the executor owns credentials (generated per dispatch), the host port and \
                 the readiness probe",
                svc.other_keys.join(", ")
            ),
        );
    }
    Some((name.to_string(), tag.to_string(), digest))
}

/// Resolve a setup-node `node-version` against the registry's pinned node
/// releases.
fn resolve_node(requested: &str) -> Result<(String, bool), String> {
    let pinned = crate::tools::node_pinned_versions();
    let r = requested.trim().trim_start_matches('v');
    if pinned.contains(&r) {
        return Ok((r.to_string(), false));
    }
    let r = r.trim_end_matches(".x");
    let prefix_ok = !r.is_empty() && r.split('.').all(|p| p.chars().all(|c| c.is_ascii_digit()) && !p.is_empty());
    if prefix_ok {
        if let Some(best) = pinned
            .iter()
            .rev()
            .find(|v| v.starts_with(&format!("{r}.")))
        {
            return Ok((best.to_string(), true));
        }
    }
    Err(format!(
        "node-version {requested:?} resolves to no node release the tool registry pins \
         (pinned: {pinned:?}) — declare a pinned [[tools]] node by hand"
    ))
}

fn with_val<'a>(step: &'a Step, key: &str) -> Option<&'a str> {
    step.with.iter().find(|(k, _)| k == key).map(|(_, v)| v.as_str())
}

/// A known action's effect. Returns `true` when the step was fully accounted
/// for (translated or a no-op).
fn handle_uses(step: &Step, uses: &str, loc: &str, ctx: &mut Ctx) {
    let all_with: String = step.with.iter().map(|(k, v)| format!("{k}={v} ")).collect();
    if uses.starts_with("./") {
        ctx.push(
            ItemKind::Untranslated,
            loc,
            with_secrets(
                format!("local action {uses} — its steps are not expanded; translate what it does by hand"),
                &all_with,
            ),
        );
        return;
    }
    if uses.starts_with("docker://") {
        ctx.push(
            ItemKind::Untranslated,
            loc,
            format!("container action {uses} — a manifest cannot run an image"),
        );
        return;
    }
    let (action, version) = uses.split_once('@').unwrap_or((uses, ""));
    let action_l = action.to_ascii_lowercase();
    let unexpected_with = |allowed: &[&str]| -> Vec<String> {
        step.with
            .iter()
            .filter(|(k, _)| !allowed.contains(&k.as_str()))
            .map(|(k, v)| format!("{k}: {v}"))
            .collect()
    };
    let interpreted: &[&str] = match action_l.as_str() {
        "snok/install-poetry" => &["version"],
        "astral-sh/setup-uv" => &["version"],
        "dtolnay/rust-toolchain" | "actions-rs/toolchain" => &["toolchain", "components", "targets", "target"],
        "actions/setup-python" => &["python-version", "cache", "cache-dependency-path"],
        "taiki-e/install-action" => &["tool"],
        _ => &[],
    };
    if !interpreted.is_empty() {
        let extra = unexpected_with(interpreted);
        if !extra.is_empty() {
            ctx.push(
                ItemKind::Untranslated,
                loc,
                format!("{action} inputs not carried: {}", extra.join(", ")),
            );
        }
    }
    match action_l.as_str() {
        "actions/checkout" => {
            if let Some(repo) = with_val(step, "repository") {
                if repo.contains("${{") {
                    ctx.push(
                        ItemKind::Untranslated,
                        loc,
                        format!("checkout of repository {repo} (an expression)"),
                    );
                } else {
                    let path = with_val(step, "path").unwrap_or("(the workspace)");
                    ctx.add_sibling(loc, repo, &format!("checkout of {repo} at {path}:"));
                }
            } else {
                ctx.push(
                    ItemKind::NoOp,
                    loc,
                    "actions/checkout — the executor checks out the dispatched commit itself",
                );
            }
            let extra = unexpected_with(&[
                "repository", "path", "ref", "fetch-depth", "persist-credentials", "token",
                "clean", "show-progress", "fetch-tags", "filter",
            ]);
            if !extra.is_empty() {
                ctx.push(
                    ItemKind::Untranslated,
                    loc,
                    format!("actions/checkout inputs not carried: {}", extra.join(", ")),
                );
            }
        }
        "actions/setup-node" => {
            match with_val(step, "node-version") {
                Some(v) if v.contains("${{") => ctx.push(
                    ItemKind::Untranslated,
                    loc,
                    format!("setup-node node-version {v} is an expression — declare a pinned [[tools]] node by hand"),
                ),
                Some(v) => match resolve_node(v) {
                    Ok((pinned, approx)) => {
                        ctx.add_tool(loc, "node", &pinned);
                        if approx {
                            ctx.push(
                                ItemKind::Changed,
                                loc,
                                format!(
                                    "setup-node node-version {v:?} → [[tools]] node {pinned}, the \
                                     registry's pinned release of that line (setup-node would \
                                     resolve the newest)"
                                ),
                            );
                        }
                    }
                    Err(e) => ctx.push(ItemKind::Untranslated, loc, e),
                },
                None => ctx.push(
                    ItemKind::Untranslated,
                    loc,
                    "setup-node with no node-version (a version file or the default) — declare a \
                     pinned [[tools]] node by hand",
                ),
            }
            let extra = unexpected_with(&["node-version", "cache", "cache-dependency-path"]);
            if !extra.is_empty() {
                ctx.push(
                    ItemKind::Untranslated,
                    loc,
                    format!("setup-node inputs not carried: {}", extra.join(", ")),
                );
            }
            if with_val(step, "cache").is_some() {
                ctx.push(
                    ItemKind::NoOp,
                    loc,
                    "setup-node cache — a dependency cache, not a gate",
                );
            }
        }
        "actions/setup-python" => ctx.push(
            ItemKind::NoOp,
            loc,
            format!(
                "setup-python {} — the tool registry provisions no python; the host's python \
                 runs the steps (declare [canonical] toolchains = [\"python\"] to require the \
                 host's canonical one)",
                with_val(step, "python-version").unwrap_or("(default)")
            ),
        ),
        "dtolnay/rust-toolchain" | "actions-rs/toolchain" => {
            let toolchain = with_val(step, "toolchain").unwrap_or(if action_l == "dtolnay/rust-toolchain" { version } else { "(default)" });
            let mut detail = format!(
                "{action} {toolchain} — the host's rust toolchain runs the steps (declare \
                 [canonical] toolchains = [\"rustc\"] to require the host's canonical one)"
            );
            if let Some(c) = with_val(step, "components") {
                detail.push_str(&format!("; components {c} must be installed on the host"));
            }
            if let Some(t) = with_val(step, "targets").or_else(|| with_val(step, "target")) {
                ctx.push(
                    ItemKind::Untranslated,
                    loc,
                    format!("{action}: cross-compilation target(s) {t} are not provisioned"),
                );
            }
            ctx.push(ItemKind::NoOp, loc, detail);
        }
        "astral-sh/setup-uv" => ctx.push(
            ItemKind::NoOp,
            loc,
            format!(
                "setup-uv {} — uv is not in the tool registry; the host's uv runs the steps",
                with_val(step, "version").unwrap_or("(default)")
            ),
        ),
        "snok/install-poetry" => match with_val(step, "version") {
            Some(v) if manifest::validate_tool_version(v).is_ok() => ctx.add_tool(loc, "poetry", v),
            Some(v) => ctx.push(
                ItemKind::Untranslated,
                loc,
                format!("install-poetry version {v:?} is not an exact version — declare [[tools]] poetry by hand"),
            ),
            None => ctx.push(
                ItemKind::Untranslated,
                loc,
                "install-poetry with no `version` — a manifest tool must be pinned; declare \
                 [[tools]] poetry by hand",
            ),
        },
        "taiki-e/install-action" => {
            // `uses: taiki-e/install-action@cargo-nextest` names the tool in
            // the ref; `with: tool:` names it otherwise.
            let from_ref = (!version.is_empty() && !version.starts_with('v') && with_val(step, "tool").is_none())
                .then_some(version);
            let tools = with_val(step, "tool").or(from_ref).unwrap_or("");
            if tools.trim().is_empty() {
                ctx.push(
                    ItemKind::Untranslated,
                    loc,
                    format!("{uses} names no tool the importer can read"),
                );
            }
            for t in tools.split([',', '\n']).map(str::trim).filter(|t| !t.is_empty()) {
                let (name, ver) = t.split_once('@').unwrap_or((t, ""));
                let name = if name == "nextest" { "cargo-nextest" } else { name };
                if crate::tools::lookup(name).is_some() && manifest::validate_tool_version(ver).is_ok() {
                    ctx.add_tool(loc, name, ver);
                } else {
                    ctx.push(
                        ItemKind::Untranslated,
                        loc,
                        format!("install-action tool {t:?} is not a pinned entry of the tool registry"),
                    );
                }
            }
        }
        "swatinem/rust-cache" | "actions/cache" | "actions/cache/restore" | "actions/cache/save" => ctx.push(
            ItemKind::NoOp,
            loc,
            format!("{action} — a cache; the executor keeps its own warm target dir and tool cache"),
        ),
        _ => ctx.push(
            ItemKind::Untranslated,
            loc,
            with_secrets(
                format!(
                    "uses {uses} — an arbitrary action; the manifest runs commands only, so \
                     translate its effect by hand"
                ),
                &all_with,
            ),
        ),
    }
}

/// `git clone https://github.com/<owner>/<name>(.git)` in a loose script.
fn cloned_repos(script: &str) -> Vec<String> {
    shell::extract_loose(script, "")
        .iter()
        .filter_map(|c| github_clone(&c.argv))
        .filter(|r| !r.contains('$'))
        .collect()
}

/// The position of the step that checks THIS repository out into a
/// subdirectory (`actions/checkout` with `path:` and no `repository:`).
fn self_checkout_index(job: &Job) -> Option<usize> {
    job.steps.iter().find_map(|st| {
        let uses = st.uses.as_deref()?;
        let action = uses.split('@').next().unwrap_or(uses).to_ascii_lowercase();
        (action == "actions/checkout"
            && with_val(st, "repository").is_none()
            && with_val(st, "path").is_some())
        .then_some(st.index)
    })
}

/// Where the job checked this repository out, workspace-relative, when it is
/// not the workspace itself.
fn self_checkout_path(job: &Job) -> Option<String> {
    let idx = self_checkout_index(job)?;
    let path = with_val(&job.steps[idx], "path")?;
    normalize_wd(path).ok().filter(|p| !p.is_empty())
}

/// A workspace-relative directory as a repository-relative one, given where
/// the repository was checked out. `None` when it is outside the repository.
fn rebase(wd: &str, self_path: Option<&str>) -> Option<String> {
    let Some(sp) = self_path else {
        return Some(wd.to_string());
    };
    if wd == sp {
        Some(String::new())
    } else {
        wd.strip_prefix(&format!("{sp}/")).map(str::to_string)
    }
}

/// `git clone … https://github.com/<owner>/<name>(.git)` → `owner/name`.
fn github_clone(argv: &[String]) -> Option<String> {
    if argv.first().map(String::as_str) != Some("git") || argv.get(1).map(String::as_str) != Some("clone") {
        return None;
    }
    argv[2..].iter().find_map(|a| {
        let rest = a.strip_prefix("https://github.com/")?;
        let rest = rest.trim_end_matches('/').trim_end_matches(".git");
        (rest.split('/').count() == 2).then(|| rest.to_string())
    })
}

/// Why a command must not run as a manifest step on a CI host: it acts as
/// root or installs into the host's global environment, and a CI host — a
/// user's own machine, on the local lane — must be left as it was found.
fn host_mutating(argv: &[String]) -> Option<&'static str> {
    let a0 = argv.first().map(String::as_str).unwrap_or("");
    let a1 = argv.get(1).map(String::as_str).unwrap_or("");
    let has = |f: &str| argv.iter().any(|t| t == f);
    if a0 == "sudo" || a0 == "doas" {
        return Some("runs as root — a manifest step runs as the host's own user");
    }
    if matches!(
        a0,
        "apt-get" | "apt" | "yum" | "dnf" | "pacman" | "zypper" | "apk" | "brew" | "choco" | "winget" | "scoop" | "snap"
    ) {
        return Some("installs system packages — those are a host prerequisite, not a CI step");
    }
    let global = (a0 == "cargo" && matches!(a1, "install" | "binstall"))
        || (matches!(a0, "pip" | "pip3") && a1 == "install")
        || (matches!(a0, "python" | "python3")
            && a1 == "-m"
            && argv.get(2).map(String::as_str) == Some("pip")
            && argv.get(3).map(String::as_str) == Some("install"))
        || (matches!(a0, "npm" | "pnpm" | "yarn") && (has("-g") || has("--global")))
        || (matches!(a0, "go" | "gem" | "pipx") && a1 == "install")
        || a0 == "rustup"
        || (a0 == "corepack" && a1 == "enable");
    global.then_some(
        "installs into the host's global environment — a CI host must be left as it was found; \
         declare a pinned [[tools]] entry or make it a host prerequisite",
    )
}

/// A script that installs poetry by hand (the installer, pipx or pip).
fn installs_poetry(script: &str) -> bool {
    script.contains("install.python-poetry.org")
        || script.contains("pipx install poetry")
        || script.contains("pip install poetry")
        || script.contains("pip install --user poetry")
}

/// An argv token that names a service connection the executor owns.
fn hardcodes_connection(tok: &str) -> bool {
    tok == "localhost"
        || tok == "127.0.0.1"
        || tok.contains("@localhost")
        || tok.contains("localhost:")
        || tok.contains("127.0.0.1:")
        || tok.ends_with(":5432")
        || tok.ends_with(":6379")
}

fn default_shell_ok(shell: Option<&str>) -> bool {
    match shell {
        None => true,
        Some(s) => {
            let s = s.trim();
            s == "bash" || s == "sh" || s.starts_with("bash ") || s.starts_with("sh ")
        }
    }
}

/// A job's `needs` before resolution: (manifest name, [(workflow job id, its
/// manifest name if it was named)], location).
type RawNeeds = (String, Vec<(String, Option<String>)>, String);

/// Import workflows into a v2 manifest. This is what `qontinui-ci import`
/// runs. `Err` only for a file that is not a workflow, or for an importer
/// defect (its own output failing validation).
pub fn import_workflows(sources: &[WorkflowSource]) -> Result<ImportOutcome, String> {
    let mut workflows = Vec::new();
    for s in sources {
        workflows.push(gha::parse_workflow(&s.label, &s.text)?);
    }
    let mut ctx = Ctx {
        items: Vec::new(),
        tools: Vec::new(),
        siblings: Vec::new(),
    };

    // Pass 1: classify, and name every importable job.
    let mut plan: Vec<(usize, Class)> = Vec::new();
    for (wi, wf) in workflows.iter().enumerate() {
        let file = basename(&wf.file).to_string();
        for k in &wf.other_keys {
            match k.as_str() {
                "permissions" | "concurrency" | "run-name" => ctx.push(
                    ItemKind::NoOp,
                    &file,
                    format!("{k} — GitHub-side setting with no manifest counterpart"),
                ),
                other => ctx.push(ItemKind::Untranslated, &file, format!("top-level key `{other}` is not interpreted")),
            }
        }
        plan.push((wi, classify(wf, &mut ctx)));
    }
    let mut id_count: BTreeMap<String, usize> = BTreeMap::new();
    for (wi, class) in &plan {
        if matches!(class, Class::NotCi(_)) {
            continue;
        }
        for j in &workflows[*wi].jobs {
            *id_count.entry(slug(&j.id)).or_default() += 1;
        }
    }
    // (workflow index, job id) → manifest job name.
    let mut names: BTreeMap<(usize, String), String> = BTreeMap::new();
    let mut used: Vec<String> = Vec::new();
    for (wi, class) in &plan {
        if matches!(class, Class::NotCi(_)) {
            continue;
        }
        let wf = &workflows[*wi];
        for j in &wf.jobs {
            let base = slug(&j.id);
            let mut name = if id_count.get(&base).copied().unwrap_or(0) > 1 {
                slug(&format!("{}-{}", stem(&wf.file), j.id))
            } else {
                base
            };
            if used.contains(&name) {
                let mut n = 2;
                while used.contains(&format!("{}-{n}", name.chars().take(45).collect::<String>())) {
                    n += 1;
                }
                name = format!("{}-{n}", name.chars().take(45).collect::<String>());
            }
            used.push(name.clone());
            names.insert((*wi, j.id.clone()), name);
        }
    }

    // Pass 2: translate.
    let mut jobs: Vec<ImportedJob> = Vec::new();
    let mut raw_needs: Vec<RawNeeds> = Vec::new();
    for (wi, class) in &plan {
        let wf = &workflows[*wi];
        let file = basename(&wf.file).to_string();
        let schedule = match class {
            Class::NotCi(reason) => {
                for j in &wf.jobs {
                    ctx.push(ItemKind::Untranslated, format!("{file} › {}", j.id), format!("not imported: {reason}"));
                }
                continue;
            }
            Class::Gate => None,
            Class::Scheduled(cron) => {
                if let Err(e) = manifest::validate_cron(cron) {
                    for j in &wf.jobs {
                        ctx.push(
                            ItemKind::Untranslated,
                            format!("{file} › {}", j.id),
                            format!("not imported: schedule {cron:?} is not a cron the manifest accepts: {e}"),
                        );
                    }
                    continue;
                }
                Some(cron.clone())
            }
        };
        for job in &wf.jobs {
            let loc = format!("{file} › {}", job.id);
            let name = names[&(*wi, job.id.clone())].clone();
            if let Some(u) = &job.uses {
                ctx.push(
                    ItemKind::Untranslated,
                    &loc,
                    format!("calls the reusable workflow {u} — import that workflow's own file instead; not imported"),
                );
                continue;
            }
            if is_true(&job.continue_on_error) == Some(true) {
                ctx.push(
                    ItemKind::Untranslated,
                    &loc,
                    "continue-on-error job (advisory) — a manifest job always gates; not imported",
                );
                continue;
            }
            if let Some(cond) = &job.if_expr {
                ctx.push(
                    ItemKind::Untranslated,
                    &loc,
                    with_secrets(
                        format!("job condition `if: {cond}` is not carried — the imported job runs unconditionally"),
                        cond,
                    ),
                );
            }
            if let Some(c) = &job.container {
                ctx.push(
                    ItemKind::Untranslated,
                    &loc,
                    format!("runs inside container {c} — the manifest has no job container; the imported steps run on the host"),
                );
            }
            if let Some(e) = &job.environment {
                ctx.push(
                    ItemKind::Untranslated,
                    &loc,
                    format!("deployment environment {e:?} (protection rules, environment secrets) is not carried"),
                );
            }
            for k in &job.other_keys {
                match k.as_str() {
                    "permissions" | "concurrency" => ctx.push(
                        ItemKind::NoOp,
                        &loc,
                        format!("{k} — GitHub-side setting with no manifest counterpart"),
                    ),
                    other => ctx.push(ItemKind::Untranslated, &loc, format!("job key `{other}` is not carried")),
                }
            }
            let (os, consumed) = map_runs_on(job, &loc, &mut ctx);
            match &job.matrix {
                Some(Matrix::Expression(e)) => ctx.push(
                    ItemKind::Untranslated,
                    &loc,
                    format!("matrix {e} is an expression — not expanded; the job is imported once"),
                ),
                Some(Matrix::Dimensions(dims)) => {
                    let rest: Vec<String> = dims
                        .iter()
                        .filter(|(d, _)| Some(d) != consumed.as_ref())
                        .map(|(d, v)| format!("{d} = {v:?}"))
                        .collect();
                    if !rest.is_empty() {
                        ctx.push(
                            ItemKind::Untranslated,
                            &loc,
                            format!(
                                "matrix {} not expanded — the job is imported once; steps that \
                                 read matrix.* are untranslated",
                                rest.join("; ")
                            ),
                        );
                    }
                }
                None => {}
            }
            let mut kinds: Vec<&'static str> = Vec::new();
            let mut services = Vec::new();
            for svc in &job.services {
                if let Some(s) = map_service(svc, &loc, &mut kinds, &mut ctx) {
                    services.push(s);
                }
            }
            let wf_env = filter_env(&wf.env, &format!("{file} › env"), &kinds, &mut ctx);
            let job_env = filter_env(&job.env, &loc, &kinds, &mut ctx);
            let job_timeout = job
                .timeout_minutes
                .as_ref()
                .and_then(|t| timeout_secs(t, &loc, "job", &mut ctx));
            let default_wd_raw = job.default_working_dir.clone().or_else(|| wf.default_working_dir.clone());
            let default_shell = job.default_shell.clone().or_else(|| wf.default_shell.clone());
            let self_path = self_checkout_path(job);
            let mut steps: Vec<ImportedStep> = Vec::new();
            let mut applied_job_timeout = false;
            for step in &job.steps {
                let mut label = step.label();
                if label.trim().is_empty() {
                    label = format!("step {}", step.index + 1);
                }
                let sloc = format!("{loc} › step \"{label}\"");
                for k in &step.other_keys {
                    ctx.push(ItemKind::Untranslated, &sloc, format!("step key `{k}` is not carried"));
                }
                if let Some(cond) = &step.if_expr {
                    ctx.push(
                        ItemKind::Untranslated,
                        &sloc,
                        with_secrets(
                            format!("runs only `if: {cond}` — the manifest has no step conditions; not imported"),
                            cond,
                        ),
                    );
                    continue;
                }
                if is_true(&step.continue_on_error) == Some(true) {
                    ctx.push(
                        ItemKind::Untranslated,
                        &sloc,
                        "continue-on-error step (advisory) — a manifest step always gates; not imported",
                    );
                    continue;
                }
                if let Some(uses) = &step.uses {
                    if self_path.is_some() && Some(step.index) == self_checkout_index(job) {
                        ctx.push(
                            ItemKind::Changed,
                            &sloc,
                            format!(
                                "checks this repository out at {:?} — the executor's checkout \
                                 root IS the repository, so working directories under it are \
                                 re-rooted and any outside it are refused",
                                self_path.clone().unwrap_or_default()
                            ),
                        );
                    }
                    handle_uses(step, uses, &sloc, &mut ctx);
                    continue;
                }
                let Some(script) = &step.run else {
                    ctx.push(ItemKind::Untranslated, &sloc, "has neither `run` nor `uses`");
                    continue;
                };
                let shell_name = step.shell.clone().or_else(|| default_shell.clone());
                if shell_name.is_none() && os.contains(&Os::Windows) {
                    ctx.push(
                        ItemKind::Untranslated,
                        &sloc,
                        with_secrets(
                            "run script not imported — on a Windows runner a `run:` with no \
                             `shell:` is PowerShell, not bash"
                                .to_string(),
                            script,
                        ),
                    );
                    continue;
                }
                if !default_shell_ok(shell_name.as_deref()) {
                    ctx.push(
                        ItemKind::Untranslated,
                        &sloc,
                        with_secrets(
                            format!(
                                "runs under shell {:?} — only bash/sh scripts are read",
                                shell_name.unwrap_or_default()
                            ),
                            script,
                        ),
                    );
                    continue;
                }
                if let Some(sh) = shell_name.as_deref() {
                    let sh = sh.trim();
                    if sh != "bash" && sh != "sh" && !sh.contains("-e") {
                        ctx.push(
                            ItemKind::Changed,
                            &sloc,
                            format!(
                                "runs under `shell: {sh}`, which does not stop at a failing line; \
                                 each manifest command fails the job — the import is stricter"
                            ),
                        );
                    }
                }
                let wd_raw = step.working_dir.clone().or_else(|| default_wd_raw.clone());
                let wd = match wd_raw.as_deref().map(normalize_wd).transpose() {
                    Ok(w) => w.unwrap_or_default(),
                    Err(e) => {
                        ctx.push(ItemKind::Untranslated, &sloc, format!("{e}; not imported"));
                        continue;
                    }
                };
                let mut translation = match shell::translate(script, &wd) {
                    Ok(t) => t,
                    Err(reason) => {
                        let mut detail = format!("run script not imported — {reason}");
                        if installs_poetry(script) {
                            detail.push_str(
                                " (it installs poetry: declare [[tools]] name = \"poetry\" with a \
                                 pinned version instead)",
                            );
                        }
                        ctx.push(ItemKind::Untranslated, &sloc, with_secrets(detail, script));
                        for repo in cloned_repos(script) {
                            ctx.add_sibling(&sloc, &repo, "the script clones");
                        }
                        continue;
                    }
                };
                // Re-root, and refuse what must not run on a CI host as-is.
                let mut refusal: Option<String> = None;
                let mut clones: Vec<String> = Vec::new();
                let mut kept = Vec::new();
                for mut cmd in std::mem::take(&mut translation.commands) {
                    if let Some(repo) = github_clone(&cmd.argv) {
                        clones.push(repo);
                        continue;
                    }
                    if cmd.argv.first().map(String::as_str) == Some("git")
                        && cmd.argv.get(1).map(String::as_str) == Some("clone")
                    {
                        refusal = Some(
                            "clones a repository that is not an owner/name on github.com — a \
                             manifest declares sibling repositories as [[siblings]], never as a \
                             clone step"
                                .to_string(),
                        );
                        break;
                    }
                    if let Some(why) = host_mutating(&cmd.argv) {
                        refusal = Some(format!("`{}` {why}", cmd.argv.join(" ").chars().take(60).collect::<String>()));
                        break;
                    }
                    match rebase(&cmd.working_dir, self_path.as_deref()) {
                        Some(w) => cmd.working_dir = w,
                        None => {
                            refusal = Some(format!(
                                "runs in {:?}, outside this repository's checkout (the workflow \
                                 checked the repository out at {:?}); the manifest's working \
                                 directories are inside the repository",
                                cmd.working_dir,
                                self_path.clone().unwrap_or_default()
                            ));
                            break;
                        }
                    }
                    kept.push(cmd);
                }
                if let Some(why) = refusal {
                    ctx.push(
                        ItemKind::Untranslated,
                        &sloc,
                        with_secrets(format!("run script not imported — {why}"), script),
                    );
                    for repo in cloned_repos(script) {
                        ctx.add_sibling(&sloc, &repo, "the script clones");
                    }
                    continue;
                }
                for repo in &clones {
                    ctx.add_sibling(&sloc, repo, "the script clones");
                }
                if kept.is_empty() {
                    continue;
                }
                translation.commands = kept;
                for note in &translation.notes {
                    ctx.push(ItemKind::Changed, &sloc, note.clone());
                }
                if !kinds.is_empty() {
                    for cmd in &translation.commands {
                        if let Some(tok) = cmd.argv.iter().find(|t| hardcodes_connection(t)) {
                            ctx.push(
                                ItemKind::Changed,
                                &sloc,
                                format!(
                                    "carried literally, but `{}` hardcodes a service connection \
                                     ({tok}) — the executor assigns the host port and credentials \
                                     per dispatch and exports them as DATABASE_URL / PG* / \
                                     REDIS_*; review the command",
                                    cmd.argv.join(" ").chars().take(60).collect::<String>()
                                ),
                            );
                        }
                    }
                }
                let step_env = filter_env(&step.env, &sloc, &kinds, &mut ctx);
                let own_timeout = step
                    .timeout_minutes
                    .as_ref()
                    .and_then(|t| timeout_secs(t, &sloc, "step", &mut ctx));
                let timeout = own_timeout.or_else(|| {
                    if job_timeout.is_some() {
                        applied_job_timeout = true;
                    }
                    job_timeout
                });
                let n = translation.commands.len();
                if n > 1 && timeout.is_some() {
                    ctx.push(
                        ItemKind::Changed,
                        &sloc,
                        format!(
                            "its timeout applies to each of the {n} commands it became, not to \
                             their sum"
                        ),
                    );
                }
                for (ci, cmd) in translation.commands.iter().enumerate() {
                    let mut env: BTreeMap<String, String> = BTreeMap::new();
                    for (k, v) in wf_env.iter().chain(&job_env).chain(&step_env) {
                        env.insert(k.clone(), v.clone());
                    }
                    let cmd_env = filter_env(&cmd.env, &sloc, &kinds, &mut ctx);
                    for (k, v) in cmd_env {
                        env.insert(k, v);
                    }
                    let mut sname = if n > 1 {
                        format!("{label} ({}/{n})", ci + 1)
                    } else {
                        label.clone()
                    };
                    if steps.iter().any(|s| s.name == sname) {
                        let mut k = 2;
                        while steps.iter().any(|s| s.name == format!("{sname} #{k}")) {
                            k += 1;
                        }
                        sname = format!("{sname} #{k}");
                    }
                    steps.push(ImportedStep {
                        name: sname,
                        command: cmd.argv.clone(),
                        env,
                        timeout_secs: timeout,
                        working_dir: (!cmd.working_dir.is_empty()).then(|| cmd.working_dir.clone()),
                        source: format!("step \"{label}\""),
                    });
                }
            }
            if applied_job_timeout {
                ctx.push(
                    ItemKind::Changed,
                    &loc,
                    "job timeout-minutes applied to each imported step without its own — the \
                     manifest has per-step timeouts only",
                );
            }
            if steps.is_empty() {
                ctx.push(
                    ItemKind::Untranslated,
                    &loc,
                    "no step could be translated — the job is not imported",
                );
                continue;
            }
            let needs: Vec<(String, Option<String>)> = job
                .needs
                .iter()
                .map(|n| (n.clone(), names.get(&(*wi, n.clone())).cloned()))
                .collect();
            raw_needs.push((name.clone(), needs, loc.clone()));
            let title = job.name.clone().unwrap_or_default();
            jobs.push(ImportedJob {
                name,
                source: if title.is_empty() || title == job.id {
                    loc.clone()
                } else {
                    format!("{loc} ({title:?})")
                },
                os,
                needs: Vec::new(),
                schedule: schedule.clone(),
                services,
                steps,
            });
        }
    }
    // Resolve needs against what was actually imported.
    for (name, needs, loc) in raw_needs {
        let mut resolved = Vec::new();
        for (id, target) in needs {
            let target_job = target.as_ref().and_then(|t| jobs.iter().find(|j| &j.name == t));
            let me_sched = jobs.iter().find(|j| j.name == name).and_then(|j| j.schedule.clone());
            match target_job {
                Some(t) if t.schedule.is_some() && me_sched.is_none() => ctx.push(
                    ItemKind::Untranslated,
                    &loc,
                    format!("needs {id}, which only runs on a schedule — dependency edge dropped"),
                ),
                Some(t) => {
                    if !resolved.contains(&t.name) {
                        resolved.push(t.name.clone());
                    }
                }
                None => ctx.push(
                    ItemKind::Untranslated,
                    &loc,
                    format!("needs {id}, which was not imported — dependency edge dropped"),
                ),
            }
        }
        if let Some(j) = jobs.iter_mut().find(|j| j.name == name) {
            j.needs = resolved;
        }
    }
    if jobs.len() > 32 {
        for j in jobs.drain(32..) {
            ctx.items.push(ImportItem {
                kind: ItemKind::Untranslated,
                location: j.source.clone(),
                detail: format!("job {} exceeds the manifest's 32-job limit — not imported", j.name),
            });
        }
        let names: Vec<String> = jobs.iter().map(|j| j.name.clone()).collect();
        for j in &mut jobs {
            j.needs.retain(|n| names.contains(n));
        }
    }

    let mut outcome = ImportOutcome {
        sources: sources.iter().map(|s| s.label.clone()).collect(),
        jobs,
        tools: ctx.tools,
        siblings: ctx.siblings,
        items: ctx.items,
        manifest_toml: String::new(),
        manifest: None,
    };
    outcome.items.sort_by_key(|i| i.kind);
    outcome.manifest_toml = render_manifest(&outcome);
    if !outcome.jobs.is_empty() {
        let parsed = manifest::parse_and_validate(&outcome.manifest_toml).map_err(|e| {
            format!("importer defect: the generated manifest does not validate: {e}")
        })?;
        outcome.manifest = Some(parsed);
    }
    Ok(outcome)
}

fn q(s: &str) -> String {
    toml::Value::String(s.to_string()).to_string()
}

/// Wrap `text` into comment lines of at most ~100 columns.
fn wrap_comment(out: &mut String, first_prefix: &str, cont_prefix: &str, text: &str) {
    const WIDTH: usize = 100;
    let mut line = first_prefix.to_string();
    let mut at_start = true;
    for word in text.split_whitespace() {
        if !at_start && line.chars().count() + 1 + word.chars().count() > WIDTH {
            out.push_str(line.trim_end());
            out.push('\n');
            line = cont_prefix.to_string();
            at_start = true;
        }
        if !at_start {
            line.push(' ');
        }
        line.push_str(word);
        at_start = false;
    }
    out.push_str(line.trim_end());
    out.push('\n');
}

/// The items as a comment block (also used, uncommented, by the report).
pub fn render_items(items: &[ImportItem], comment: bool) -> String {
    let mut out = String::new();
    let c = if comment { "# " } else { "" };
    for kind in [ItemKind::Untranslated, ItemKind::Changed, ItemKind::NoOp] {
        let of: Vec<&ImportItem> = items.iter().filter(|i| i.kind == kind).collect();
        let blurb = match kind {
            ItemKind::Untranslated => "not represented in the manifest — each must be handled by hand",
            ItemKind::Changed => "represented, but not literally",
            ItemKind::NoOp => "accounted for; nothing to declare",
        };
        out.push_str(&format!("{c}{} ({}) — {blurb}:\n", kind.heading(), of.len()));
        if of.is_empty() {
            out.push_str(&format!("{c}  (none)\n"));
        }
        for i in of {
            wrap_comment(&mut out, &format!("{c}  - {}: ", i.location), &format!("{c}      "), &i.detail);
        }
        if comment {
            out.push_str("#\n");
        } else {
            out.push('\n');
        }
    }
    out
}

fn render_manifest(o: &ImportOutcome) -> String {
    let mut s = String::new();
    s.push_str("# GENERATED by `qontinui-ci import` from:\n");
    for src in &o.sources {
        s.push_str(&format!("#   {src}\n"));
    }
    s.push_str(
        "#\n\
         # Review before committing. This manifest replaces those workflows as the repo's CI\n\
         # definition only once every UNTRANSLATED item below is handled by hand; check with\n\
         # `qontinui-ci import --report .qontinui/ci.toml <the workflows>`.\n\
         #\n",
    );
    s.push_str(&render_items(&o.items, true));
    s.push_str("\nversion = 2\n");
    if o.jobs.is_empty() {
        s.push_str(
            "\n# NO JOB COULD BE TRANSLATED. This file is not a valid manifest until a [[jobs]]\n\
             # entry is written by hand.\n",
        );
    }
    for repo in &o.siblings {
        s.push_str(&format!("\n[[siblings]]\nrepo = {}\n", q(repo)));
    }
    for (name, version) in &o.tools {
        s.push_str(&format!("\n[[tools]]\nname = {}\nversion = {}\n", q(name), q(version)));
    }
    for job in &o.jobs {
        s.push_str(&format!("\n[[jobs]]\n# from {}\nname = {}\n", job.source, q(&job.name)));
        match job.os.as_slice() {
            [Os::Any] => {}
            [one] => s.push_str(&format!("os = {}\n", q(one.as_str()))),
            many => s.push_str(&format!(
                "os = [{}]\n",
                many.iter().map(|o| q(o.as_str())).collect::<Vec<_>>().join(", ")
            )),
        }
        if !job.needs.is_empty() {
            s.push_str(&format!(
                "needs = [{}]\n",
                job.needs.iter().map(|n| q(n)).collect::<Vec<_>>().join(", ")
            ));
        }
        if let Some(cron) = &job.schedule {
            s.push_str(&format!("schedule = {}\n", q(cron)));
        }
        for (name, version, digest) in &job.services {
            s.push_str(&format!("\n[[jobs.services]]\nname = {}\nversion = {}\n", q(name), q(version)));
            if let Some(d) = digest {
                s.push_str(&format!("digest = {}\n", q(d)));
            }
        }
        for step in &job.steps {
            s.push_str(&format!("\n[[jobs.steps]]\n# from {}\nname = {}\n", step.source, q(&step.name)));
            s.push_str(&format!(
                "command = [{}]\n",
                step.command.iter().map(|a| q(a)).collect::<Vec<_>>().join(", ")
            ));
            if let Some(wd) = &step.working_dir {
                s.push_str(&format!("working_dir = {}\n", q(wd)));
            }
            if let Some(t) = step.timeout_secs {
                s.push_str(&format!("timeout_secs = {t}\n"));
            }
            if !step.env.is_empty() {
                s.push_str("\n[jobs.steps.env]\n");
                for (k, v) in &step.env {
                    s.push_str(&format!("{k} = {}\n", q(v)));
                }
            }
        }
    }
    s
}

// ───────────────────────────── coverage ─────────────────────────────

/// The rule `import --report` applies, printed with every report.
pub const COVERAGE_RULE: &str = "\
DENY BY DEFAULT: a workflow job is COVERED only when EVERY step is positively understood — every \
gate command matched, with no unresolved directory, toolchain, environment change or unknown \
action. Anything the analyzer cannot fully model makes the job NEEDS-REVIEW, or an opaque gate \
that leaves it PARTIAL/UNCOVERED; it never counts as covered. \
UNDERSTOOD STEPS (an allowlist, `shell::report_grammar`): a `run:` script is understood only \
when every segment is a simple command with literal words (no `$` anywhere), `cd <literal \
relative path>`, `echo`/`printf`/`:`/`true` with no redirect, or exactly `set -e`/`-eu`/\
`-euo pipefail`/`-x` — joined by newlines, `;` or `&&`, optionally ending in `|| true`/`|| :`. \
Anything else in any step (an assignment or `KEY=value` prefix; an env/timeout/nice/xargs/sudo \
wrapper; any redirect, pipe, here-document, subshell, pushd/popd; control flow; `$( )`; \
export/declare/source/eval/set -a; `cd -` or a non-literal `cd`) makes the job NEEDS-REVIEW. A \
`continue-on-error: true` step is not a gate but goes through every one of these checks. \
CONSIDERED JOBS: those whose workflow runs on `pull_request` / `pull_request_target` into the \
default branch, on `merge_group`, or on a `push` to the default branch. Zero considered jobs is \
NOT MET. \
GATE COMMANDS: every command a step runs whose exit status can fail the step (GitHub runs bash \
with -e), read from `run:` scripts through any control flow and inside `$(…)` — installs, tests, \
greps, `jq` assertions and `source` included. Not gates: output, navigation and shell-state \
builtins (echo, cd, set, export, exit, …), a command followed by exactly `|| true` or `|| :` (any \
other fallback leaves it a gate), steps whose condition is only `failure()`/`cancelled()`, steps \
with a literal `continue-on-error: true`, and checkout/cache/upload actions. OPAQUE gates, which \
no manifest step matches: every other `uses:` action, a non-bash script (a `run:` on a Windows \
runner with no `shell: bash` is PowerShell), a command whose program is a variable or expression, \
and a command whose working directory cannot be resolved (an expression in `working-directory`, a \
`cd` to a non-literal). \
MATCHED: a step of a manifest GATE job (scheduled jobs never run for a pull request) runs the same \
command in the same working directory, on a job whose `os` includes every OS the workflow job \
runs on — a manifest `os = any` job counts as Linux only; a workflow job of unknown OS is matched \
by nothing. SAME COMMAND: the whole argv is equal after stripping wrappers (poetry/uv/pipenv/pdm/\
hatch run, npx, pnpm/npm/yarn exec, python -m, env, sudo, timeout N, bash/sh <script>), python3 = \
python, pip3 = pip, and a leading ./ or trailing / per word. Package managers and their verbs are \
NOT unified (`npm ci` is not `npm install`; `pnpm lint` is not `npm run lint`). A manifest step \
with a word containing `$` matches nothing (the executor passes it literally). \
NEEDS-REVIEW, even when everything matched: any workflow/job/step env key, or `KEY=value` prefix, \
the matched step does not set identically (any key; an expression value never matches); a \
`cargo +toolchain` the step lacks; a version pinned by setup-node/setup-python/rust-toolchain or \
`rustup default|override|set` that the manifest's [[tools]] does not pin identically; `export`, \
`declare -x`, `set -a`, `source`/`.`, or any use of $GITHUB_ENV/$GITHUB_PATH/$GITHUB_OUTPUT; a \
self-hosted, ARM or custom runner label, or ANY macOS label (architecture is not modeled); \
`actions/checkout` with inputs other than fetch-depth/persist-credentials/path; a manifest step \
env key the workflow command does not run with identically (env is compared both ways, and a \
`${{ … }}` value never matches); a step or job `timeout-minutes` below the matched step's \
timeout; a non-OS matrix; a container; a service the matched step's own job does not declare. \
Codecov with fail_ci_if_error, upload-artifact with if-no-files-found: error and a cache with \
fail-on-cache-miss are opaque gates. \
VERDICTS: COVERED, NEEDS-REVIEW, NAME-ONLY (no gate command; a manifest gate job carries the \
job's name), PARTIAL, UNCOVERED. Only COVERED counts toward the precondition.";

/// A command's identity for coverage matching (see [`COVERAGE_RULE`]).
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct CommandKey {
    /// The signature: normalized words up to the first flag, variable or the
    /// third word — what CLASSIFIES a command (gate, setup, glue). Empty for
    /// an opaque command (an action, a non-bash script).
    pub words: Vec<String>,
    /// The whole normalized argv — what MATCHES a command. Empty for an opaque
    /// command.
    pub argv: Vec<String>,
}

impl CommandKey {
    /// The key of an argv command.
    pub fn of(argv: &[String]) -> CommandKey {
        let words = signature(argv);
        let argv = if words.is_empty() {
            Vec::new()
        } else {
            normalized_argv(argv)
                .into_iter()
                .map(|t| t.strip_prefix("./").unwrap_or(&t).trim_end_matches('/').to_string())
                .collect()
        };
        CommandKey { words, argv }
    }

    /// Two keys name the same command: their whole normalized argv is equal.
    /// Deliberately strict — a looser rule (a prefix, a flag subset) lets
    /// `cargo fmt --all` cover `cargo fmt --all -- --check`, and a false
    /// COVERED is the dangerous direction: it licenses deleting a workflow.
    pub fn matches(&self, other: &CommandKey) -> bool {
        !self.argv.is_empty() && self.argv == other.argv
    }
}

/// Normalize a command to its comparison signature (see [`COVERAGE_RULE`]).
pub fn signature(argv: &[String]) -> Vec<String> {
    let mut out = Vec::new();
    for t in normalized_argv(argv) {
        if t.starts_with('-') || t.contains('$') || t.contains('`') || out.len() == 3 {
            break;
        }
        let t = t.strip_prefix("./").unwrap_or(&t).trim_end_matches('/').to_string();
        if t.is_empty() {
            break;
        }
        out.push(t);
    }
    out
}

/// The argv with env prefixes and wrappers stripped and package-script
/// runners unified — what both [`signature`] and the flag set read.
fn normalized_argv(argv: &[String]) -> Vec<String> {
    let mut a: Vec<String> = argv.to_vec();
    let is_assign = |t: &str| {
        t.split_once('=').is_some_and(|(k, _)| {
            !k.is_empty() && k.chars().all(|c| c.is_ascii_alphanumeric() || c == '_')
        })
    };
    loop {
        let Some(first) = a.first().cloned() else {
            return Vec::new();
        };
        let f = first.as_str();
        if is_assign(f) || matches!(f, "sudo" | "time" | "nohup" | "exec" | "command") {
            a.remove(0);
            continue;
        }
        if f == "env" {
            a.remove(0);
            while a.first().is_some_and(|t| is_assign(t) || t.starts_with('-')) {
                a.remove(0);
            }
            continue;
        }
        if f == "timeout" {
            a.remove(0);
            while a.first().is_some_and(|t| t.starts_with('-')) {
                a.remove(0);
            }
            if !a.is_empty() {
                a.remove(0);
            }
            continue;
        }
        break;
    }
    let norm_prog = |p: &str| -> String {
        let p = p.strip_prefix("./").unwrap_or(p);
        let base = p.rsplit('/').next().unwrap_or(p);
        if base == "python3" || base.starts_with("python3.") || base == "py" {
            "python".to_string()
        } else if base == "pip3" {
            "pip".to_string()
        } else if p.contains('/') {
            p.to_string()
        } else {
            base.to_string()
        }
    };
    if let Some(f) = a.first_mut() {
        *f = norm_prog(f);
    }
    // Wrappers.
    loop {
        let two = |a: &Vec<String>, x: &str, y: &str| a.len() >= 2 && a[0] == x && a[1] == y;
        let strip = if ["poetry", "uv", "pipenv", "pdm", "hatch", "rye"].iter().any(|w| two(&a, w, "run"))
            || ["pnpm", "npm", "yarn"].iter().any(|w| two(&a, w, "exec"))
            || two(&a, "bun", "x")
            || two(&a, "python", "-m")
        {
            2
        } else if a.first().is_some_and(|f| matches!(f.as_str(), "npx" | "bunx" | "pnpx" | "uvx")) {
            1
        } else if a.first().is_some_and(|f| f == "cargo") && a.get(1).is_some_and(|t| t.starts_with('+')) {
            a.remove(1);
            0
        } else {
            break;
        };
        if strip == 0 {
            continue;
        }
        a.drain(..strip);
        while a.first().is_some_and(|t| t.starts_with('-')) {
            a.remove(0);
        }
        if let Some(f) = a.first_mut() {
            *f = norm_prog(f);
        }
    }
    if a.len() >= 2 && (a[0] == "bash" || a[0] == "sh") && !a[1].starts_with('-') {
        a.remove(0);
        a[0] = norm_prog(&a[0]);
    }
    // Package managers are NOT unified: `npm ci`, `npm install`,
    // `pnpm install --frozen-lockfile` and `npm run x` / `pnpm x` are
    // different commands with different lockfile and script semantics.
    a
}

/// Commands whose failure never decides a step in practice: terminal output,
/// navigation and shell-state builtins. EVERY other command a step runs —
/// tests, greps, `jq` assertions, installs, `source` — is a gate, because
/// under GitHub's `bash -e` its exit status fails the step.
const NON_GATE: &[&str] = &[
    "echo", "printf", "cd", "set", "export", "true", ":", "exit", "local", "read", "shift",
    "return", "unset", "trap", "wait", "pushd", "popd", "dirs", "declare", "readonly", "typeset",
    "let", "break", "continue", "mapfile", "readarray", "sleep", "fi", "then", "else", "done",
    "do", "esac", "shopt", "umask", "hash", "clear", "tput",
];

/// Whether a loose command is a gate: anything but [`NON_GATE`], including an
/// opaque command whose program is a variable (`$PYTHON -m mypy`).
fn is_gate_program(argv: &[String]) -> bool {
    let Some(first) = argv.first() else {
        return false;
    };
    !NON_GATE.contains(&first.as_str())
}

/// Actions that never decide a job's verdict on their own (checkout, caches,
/// artifact moves, toolchain setup — the last are checked for version pins
/// separately).
fn known_nongate_action(action_l: &str) -> bool {
    matches!(
        action_l,
        "actions/checkout"
            | "actions/setup-node"
            | "actions/setup-python"
            | "dtolnay/rust-toolchain"
            | "actions-rs/toolchain"
            | "swatinem/rust-cache"
            | "actions/cache"
            | "actions/cache/restore"
            | "actions/cache/save"
            | "actions/upload-artifact"
            | "codecov/codecov-action"
            | "github/codeql-action/upload-sarif"
            | "jlumbroso/free-disk-space"
    )
}

/// A gate command of a workflow job.
#[derive(Debug, Clone)]
pub struct GateCommand {
    /// Human form: `dir: argv…`, or `uses: owner/action`.
    pub display: String,
    pub working_dir: String,
    /// `argv` is empty for an opaque command (an action, a non-bash script, a
    /// command named by a variable, one whose directory cannot be resolved):
    /// no manifest step can match it.
    pub key: CommandKey,
    /// Every env the command runs with: workflow, job and step `env` (any
    /// key) and its own `KEY=value` prefix. A matched manifest step must set
    /// each identically, or the job is NEEDS-REVIEW.
    pub env: Vec<(String, String)>,
    /// A `cargo +<toolchain>` the normalization strips.
    pub toolchain: Option<String>,
    /// The step's (else the job's) `timeout-minutes`, in seconds: a matched
    /// manifest step allowed longer is NEEDS-REVIEW.
    pub timeout_limit: Option<u64>,
}

/// A runtime/toolchain version a workflow job pins through a setup action or
/// a script.
#[derive(Debug, Clone)]
pub struct VersionPin {
    pub what: String,
    /// The `[[tools]]` entry that could pin the same version (`node`,
    /// `poetry`), or `None` when the manifest has no way to.
    pub tool: Option<&'static str>,
    pub version: String,
}

/// Everything the report reads out of one workflow job.
#[derive(Debug, Clone, Default)]
pub struct JobGates {
    pub gates: Vec<GateCommand>,
    /// Things the analyzer cannot model (env exported to later commands, a
    /// write to $GITHUB_ENV, an unusual runner, …): any of them makes a fully
    /// matched job NEEDS-REVIEW.
    pub unmodeled: Vec<String>,
    pub pins: Vec<VersionPin>,
}

/// One manifest step, for matching.
#[derive(Debug, Clone)]
pub struct ManifestCommand {
    /// `job/step`.
    pub label: String,
    pub working_dir: String,
    /// Empty — never a match — when any argv word contains `$`, which the
    /// executor passes literally.
    pub key: CommandKey,
    /// The OSes the step's job runs on.
    pub os: Vec<Os>,
    pub env: BTreeMap<String, String>,
    pub toolchain: Option<String>,
    /// Registry names of the step's own job's services.
    pub services: Vec<String>,
    /// The step's effective timeout, in seconds.
    pub timeout_secs: u64,
}

fn toolchain_of(argv: &[String]) -> Option<String> {
    let i = argv.iter().position(|t| t == "cargo" || t.ends_with("/cargo"))?;
    argv.get(i + 1).filter(|t| t.starts_with('+')).cloned()
}

/// Every gate-job step of a manifest as a matchable command.
pub fn manifest_commands(m: &CiManifest) -> Vec<ManifestCommand> {
    let mut out = Vec::new();
    // A scheduled job never runs for a pull request, so it covers nothing a
    // pull request gates.
    for job in m.gate_jobs() {
        for step in &job.steps {
            let key = if step.command.iter().any(|w| w.contains('$')) {
                CommandKey::default()
            } else {
                CommandKey::of(&step.command)
            };
            out.push(ManifestCommand {
                label: format!("{}/{}", job.name, step.name),
                working_dir: step
                    .working_dir
                    .as_deref()
                    .and_then(|w| shell::join_rel("", w))
                    .unwrap_or_default(),
                key,
                os: job.os.clone(),
                env: step.env.clone(),
                toolchain: toolchain_of(&step.command),
                services: job.services.iter().map(|s| s.name.clone()).collect(),
                timeout_secs: step.effective_timeout_secs(),
            });
        }
    }
    out
}

/// Whether a manifest job on `m_os` runs on every OS in `w_os`. A manifest
/// `any` job is generated onto `ubuntu-latest` and a coord host of unknown
/// OS, so for coverage it counts as LINUX ONLY; a workflow job whose OS is
/// unknown (`any`) is covered by nothing.
fn os_covers(m_os: &[Os], w_os: &[Os]) -> bool {
    w_os.iter().all(|w| match w {
        Os::Any => false,
        Os::Linux => m_os.contains(&Os::Linux) || m_os.contains(&Os::Any),
        other => m_os.contains(other),
    })
}

/// The manifest command that runs `key` in `wd` on a job that runs on every
/// OS in `oses`.
fn best_match<'a>(mcmds: &'a [ManifestCommand], wd: &str, key: &CommandKey, oses: &[Os]) -> Option<&'a ManifestCommand> {
    mcmds
        .iter()
        .find(|m| m.working_dir == wd && m.key.matches(key) && os_covers(&m.os, oses))
}

/// The OSes a workflow job runs on, as the importer maps them.
fn job_oses(job: &Job) -> (Vec<Os>, Option<String>) {
    let mut scratch = Ctx {
        items: Vec::new(),
        tools: Vec::new(),
        siblings: Vec::new(),
    };
    map_runs_on(job, "", &mut scratch)
}

/// GitHub-hosted x86 Linux/Windows runner labels. Anything else — `self-hosted`, an `-arm`
/// image, a larger-runner or custom label — is a host the manifest's `os`
/// cannot be shown to match.
fn standard_runner_label(label: &str) -> bool {
    let l = label.to_ascii_lowercase();
    if l.contains("arm") || l.contains("self-hosted") {
        return false;
    }
    [
        "ubuntu-latest", "ubuntu-24.04", "ubuntu-22.04", "ubuntu-20.04", "windows-latest",
        "windows-2025", "windows-2022", "windows-2019",
    ]
    .contains(&l.as_str())
}

fn runner_unmodeled(job: &Job) -> Vec<String> {
    let mut out = Vec::new();
    let labels: Vec<String> = match &job.runs_on {
        RunsOn::Labels(l) => l.clone(),
        RunsOn::Other(o) => {
            out.push(format!("runs on a runner group ({o})"));
            return out;
        }
        RunsOn::Missing => return out,
    };
    let mut values: Vec<String> = Vec::new();
    for l in &labels {
        let inner = l.trim().trim_start_matches("${{").trim_end_matches("}}").trim();
        match (inner.strip_prefix("matrix."), &job.matrix) {
            (Some(dim), Some(Matrix::Dimensions(dims))) if l.contains("${{") => {
                match dims.iter().find(|(d, _)| d == dim) {
                    Some((_, v)) => values.extend(v.iter().cloned()),
                    None => out.push(format!("runs-on {l} reads a matrix dimension that is not a literal list")),
                }
            }
            _ if l.contains("${{") => out.push(format!("runs-on {l} is an expression")),
            _ => values.push(l.clone()),
        }
    }
    for v in values {
        if v.to_ascii_lowercase().contains("mac") {
            out.push(format!(
                "runs on `{v}`: macOS images differ in CPU architecture (macos-latest/14/15 are \
                 arm64, macos-13 x86) and a manifest `os` does not model architecture"
            ));
            continue;
        }
        if !standard_runner_label(&v) {
            out.push(format!(
                "runs on `{v}`, not a standard GitHub-hosted x86 runner — an `any`/linux manifest \
                 job cannot be shown to match that host"
            ));
        }
    }
    out
}

/// Version pins a setup action declares.
fn action_pins(step: &Step, uses: &str) -> Vec<VersionPin> {
    let (action, ref_) = uses.split_once('@').unwrap_or((uses, ""));
    let action_l = action.to_ascii_lowercase();
    let mut out = Vec::new();
    match action_l.as_str() {
        "actions/setup-node" => {
            if let Some(v) = with_val(step, "node-version").or_else(|| with_val(step, "node-version-file")) {
                out.push(VersionPin {
                    what: format!("setup-node node-version {v}"),
                    tool: Some("node"),
                    version: v.to_string(),
                });
            }
        }
        "actions/setup-python" => {
            if let Some(v) = with_val(step, "python-version").or_else(|| with_val(step, "python-version-file")) {
                out.push(VersionPin {
                    what: format!("setup-python python-version {v}"),
                    tool: None,
                    version: v.to_string(),
                });
            }
        }
        "dtolnay/rust-toolchain" | "actions-rs/toolchain" => {
            let toolchain = with_val(step, "toolchain")
                .map(str::to_string)
                .or_else(|| (action_l == "dtolnay/rust-toolchain").then(|| ref_.to_string()));
            match toolchain.as_deref() {
                Some("stable") => {}
                Some(t) => out.push(VersionPin {
                    what: format!("{action} toolchain {t}"),
                    tool: None,
                    version: t.to_string(),
                }),
                None => out.push(VersionPin {
                    what: format!("{action} with no readable toolchain"),
                    tool: None,
                    version: String::new(),
                }),
            }
        }
        _ => {}
    }
    out
}

/// The gates, unmodeled conditions and version pins of one workflow job (see
/// [`COVERAGE_RULE`]).
pub fn gate_commands(wf: &Workflow, job: &Job) -> JobGates {
    let mut jg = JobGates::default();
    let opaque = |display: String| GateCommand {
        display,
        working_dir: String::new(),
        key: CommandKey::default(),
        env: Vec::new(),
        toolchain: None,
        timeout_limit: None,
    };
    let push = |gates: &mut Vec<GateCommand>, g: GateCommand| {
        let dup = gates.iter().any(|o| {
            if g.key.argv.is_empty() {
                o.key.argv.is_empty() && o.display == g.display
            } else {
                o.working_dir == g.working_dir
                    && o.key.argv == g.key.argv
                    && o.env == g.env
                    && o.toolchain == g.toolchain
            }
        });
        if !dup {
            gates.push(g);
        }
    };
    jg.unmodeled.extend(runner_unmodeled(job));
    if let Some(u) = &job.uses {
        push(&mut jg.gates, opaque(format!("uses: {u}")));
        return jg;
    }
    let default_wd = job
        .default_working_dir
        .clone()
        .or_else(|| wf.default_working_dir.clone());
    let default_shell = job.default_shell.clone().or_else(|| wf.default_shell.clone());
    let self_path = self_checkout_path(job);
    let on_windows = job_oses(job).0.contains(&Os::Windows);
    let job_timeout = job.timeout_minutes.clone();
    for step in &job.steps {
        if step.if_expr.as_deref().is_some_and(diagnostic_condition) {
            continue;
        }
        // A `continue-on-error: true` step is not a gate, but it still goes
        // through every check that can downgrade the job: what it runs, uses,
        // sets or writes can change the steps after it.
        let advisory = literally_true(&step.continue_on_error);
        let mut advisory_gates: Vec<GateCommand> = Vec::new();
        let target: &mut Vec<GateCommand> = if advisory {
            &mut advisory_gates
        } else {
            &mut jg.gates
        };
        if advisory && !step.env.is_empty() {
            jg.unmodeled.push(format!(
                "advisory step \"{}\" sets env the manifest does not model",
                step.label()
            ));
        }
        // timeout-minutes on the step (else the job): a matched manifest step
        // must not be allowed longer.
        let timeout_limit = match step.timeout_minutes.as_ref().or(job_timeout.as_ref()) {
            None => None,
            Some(t) => match t.0.trim().parse::<f64>() {
                Ok(m) if !t.is_expression() => Some((m * 60.0).round().max(0.0) as u64),
                _ => {
                    jg.unmodeled.push(format!(
                        "step \"{}\" has timeout-minutes {} the report cannot read",
                        step.label(),
                        t.0
                    ));
                    None
                }
            },
        };
        if let Some(uses) = &step.uses {
            let action_l = uses.split('@').next().unwrap_or(uses).to_ascii_lowercase();
            jg.pins.extend(action_pins(step, uses));
            if action_l == "actions/checkout" {
                let extra: Vec<String> = step
                    .with
                    .iter()
                    .filter(|(k, _)| !matches!(k.as_str(), "fetch-depth" | "persist-credentials" | "path"))
                    .map(|(k, v)| format!("{k}: {v}"))
                    .collect();
                if !extra.is_empty() {
                    jg.unmodeled.push(format!(
                        "actions/checkout with {} — not the dispatched commit's plain checkout",
                        extra.join(", ")
                    ));
                }
            }
            let fails_job = (action_l.starts_with("codecov/") && with_val(step, "fail_ci_if_error").is_some_and(|v| v.trim() == "true"))
                || (action_l == "actions/upload-artifact" && with_val(step, "if-no-files-found").is_some_and(|v| v.trim() == "error"))
                || (action_l.starts_with("actions/cache") && with_val(step, "fail-on-cache-miss").is_some_and(|v| v.trim() == "true"));
            if fails_job || !known_nongate_action(&action_l) {
                if advisory {
                    jg.unmodeled.push(format!("advisory step uses {uses}, which the report does not model"));
                }
                push(&mut *target, opaque(format!("uses: {}", uses.split('@').next().unwrap_or(uses))));
            }
            continue;
        }
        let Some(script) = &step.run else { continue };
        if let Err(why) = shell::report_grammar(script) {
            jg.unmodeled.push(format!(
                "step \"{}\" is outside the report's command grammar: it {why}",
                step.label()
            ));
        }
        for f in shell::github_file_writes(script) {
            jg.unmodeled.push(format!(
                "step \"{}\" touches ${f}, which changes what later steps see",
                step.label()
            ));
        }
        let shell_name = step.shell.clone().or_else(|| default_shell.clone());
        if !default_shell_ok(shell_name.as_deref()) || (on_windows && shell_name.is_none()) {
            push(
                &mut *target,
                opaque(format!(
                    "run ({} script): {}",
                    shell_name.unwrap_or_else(|| "pwsh".to_string()),
                    step.label()
                )),
            );
            continue;
        }
        let wd_raw = step.working_dir.clone().or_else(|| default_wd.clone());
        let wd = match wd_raw.as_deref().map(normalize_wd).transpose() {
            Ok(w) => w.unwrap_or_default(),
            Err(e) => {
                push(&mut *target, opaque(format!("run in an unresolvable directory ({e}): {}", step.label())));
                continue;
            }
        };
        let scoped_env: Vec<(String, String)> = wf
            .env
            .iter()
            .chain(&job.env)
            .chain(&step.env)
            .cloned()
            .collect();
        for cmd in shell::extract_loose(script, &wd) {
            if let Some(effect) = &cmd.env_effect {
                jg.unmodeled.push(format!(
                    "step \"{}\" changes the environment of later commands: {effect}",
                    step.label()
                ));
            }
            if cmd.argv.first().map(String::as_str) == Some("rustup")
                && cmd.argv.iter().any(|a| matches!(a.as_str(), "default" | "override" | "set"))
            {
                jg.pins.push(VersionPin {
                    what: format!("`{}`", cmd.argv.join(" ")),
                    tool: None,
                    version: cmd.argv.last().cloned().unwrap_or_default(),
                });
            }
            if cmd.guarded || !is_gate_program(&cmd.argv) {
                continue;
            }
            let shown: String = cmd.argv.join(" ").chars().take(100).collect();
            if !cmd.wd_known {
                push(&mut *target, opaque(format!("(unresolvable directory) {shown}")));
                continue;
            }
            let working_dir = rebase(&cmd.working_dir, self_path.as_deref())
                .unwrap_or_else(|| format!("<outside the repository>/{}", cmd.working_dir));
            let key = CommandKey::of(&cmd.argv);
            let mut env: BTreeMap<String, String> = scoped_env.iter().cloned().collect();
            for (k, v) in &cmd.assignments {
                env.insert(k.clone(), v.clone());
            }
            push(
                &mut *target,
                GateCommand {
                    display: if working_dir.is_empty() {
                        shown
                    } else {
                        format!("{working_dir}: {shown}")
                    },
                    working_dir,
                    key,
                    env: env.into_iter().collect(),
                    toolchain: toolchain_of(&cmd.argv),
                    timeout_limit,
                },
            );
        }
    }
    jg
}

/// What a matched manifest step does not reproduce of a gate command.
fn command_caveats(g: &GateCommand, m: &ManifestCommand) -> Vec<String> {
    let mut out = Vec::new();
    for (k, v) in &g.env {
        // An expression never matches, even byte-identical: the manifest
        // would pass it literally.
        if v.contains("${{") || m.env.get(k) != Some(v) {
            out.push(format!("`{}` runs with {k}={v}; {} does not set it identically", g.display, m.label));
        }
    }
    for (k, v) in &m.env {
        if !g.env.iter().any(|(gk, gv)| gk == k && gv == v) {
            out.push(format!("{} sets {k}={v}; `{}` does not run with it", m.label, g.display));
        }
    }
    if g.toolchain != m.toolchain {
        out.push(format!(
            "`{}` runs on toolchain {}; {} on {}",
            g.display,
            g.toolchain.as_deref().unwrap_or("(default)"),
            m.label,
            m.toolchain.as_deref().unwrap_or("(default)")
        ));
    }
    if let Some(limit) = g.timeout_limit {
        if m.timeout_secs > limit {
            out.push(format!(
                "`{}` is limited to {limit}s; {} allows {}s",
                g.display, m.label, m.timeout_secs
            ));
        }
    }
    out
}

/// A job's coverage verdict. Only [`Coverage::Covered`] counts as covered.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Coverage {
    Covered,
    /// Every gate command matched, but something about the job is not
    /// positively modeled (see [`COVERAGE_RULE`]).
    NeedsReview,
    /// The job runs no gate command; a manifest gate job carries its name.
    /// Not covered: a name is not evidence.
    NameOnly,
    Partial,
    Uncovered,
}

impl Coverage {
    pub fn as_str(self) -> &'static str {
        match self {
            Coverage::Covered => "COVERED",
            Coverage::NeedsReview => "NEEDS-REVIEW",
            Coverage::NameOnly => "NAME-ONLY",
            Coverage::Partial => "PARTIAL",
            Coverage::Uncovered => "UNCOVERED",
        }
    }

    pub fn is_covered(self) -> bool {
        self == Coverage::Covered
    }
}

/// One considered workflow job.
#[derive(Debug, Clone)]
pub struct JobCoverage {
    pub workflow: String,
    pub job_id: String,
    pub triggers: Vec<String>,
    pub verdict: Coverage,
    /// `(gate command, manifest job/step)`.
    pub matched: Vec<(String, String)>,
    pub missing: Vec<String>,
    /// Why a fully matched job still needs review.
    pub caveats: Vec<String>,
    pub notes: Vec<String>,
}

/// The whole `import --report`.
#[derive(Debug, Clone)]
pub struct CoverageReport {
    pub manifest_label: String,
    pub manifest_summary: String,
    pub default_branch: String,
    pub workflow_count: usize,
    pub jobs: Vec<JobCoverage>,
    /// `(workflow, its jobs, why it is not considered)`.
    pub not_considered: Vec<(String, Vec<String>, String)>,
    /// The import of the same workflows — its items are printed too.
    pub import: ImportOutcome,
}

impl CoverageReport {
    /// At least one job was considered and every considered job is COVERED:
    /// the Phase 9 precondition holds. Zero considered jobs is NOT met — it
    /// means the wrong default branch, or workflows the report could not read
    /// as gates, never "nothing to cover".
    pub fn all_covered(&self) -> bool {
        !self.jobs.is_empty() && self.jobs.iter().all(|j| j.verdict.is_covered())
    }
}

/// Which gate triggers of `wf` fire for the default branch.
fn gate_triggers(wf: &Workflow, default_branch: &str) -> Vec<String> {
    let t = &wf.triggers;
    let mut out = Vec::new();
    for (name, f) in [
        ("pull_request", &t.pull_request),
        ("pull_request_target", &t.pull_request_target),
        ("push", &t.push),
    ] {
        if let Some(f) = f {
            if f.fires_for_branch(default_branch) {
                let mut s = if name == "push" {
                    format!("push:{default_branch}")
                } else {
                    name.to_string()
                };
                if f.path_filtered {
                    s.push_str(" (paths-filtered)");
                }
                out.push(s);
            }
        }
    }
    if t.events.iter().any(|e| e == "merge_group") {
        out.push("merge_group".to_string());
    }
    out
}

/// Job-level conditions the matched manifest steps are not shown to
/// reproduce: services are checked against each matched step's OWN job.
fn job_caveats(job: &Job, consumed_dim: Option<&str>, matched: &[&ManifestCommand]) -> Vec<String> {
    let mut out = Vec::new();
    match &job.matrix {
        Some(Matrix::Expression(e)) => out.push(format!("matrix {e} (not expanded)")),
        Some(Matrix::Dimensions(dims)) => {
            let rest: Vec<String> = dims
                .iter()
                .filter(|(d, _)| Some(d.as_str()) != consumed_dim)
                .map(|(d, v)| format!("{d} = {v:?}"))
                .collect();
            if !rest.is_empty() {
                out.push(format!("matrix legs {} — the manifest runs one leg", rest.join("; ")));
            }
        }
        None => {}
    }
    if let Some(c) = &job.container {
        out.push(format!("runs inside container {c}"));
    }
    let mut scratch = Ctx {
        items: Vec::new(),
        tools: Vec::new(),
        siblings: Vec::new(),
    };
    let mut kinds = Vec::new();
    for svc in &job.services {
        match map_service(svc, "", &mut kinds, &mut scratch) {
            Some((name, _, _)) => {
                for m in matched {
                    if !m.services.contains(&name) {
                        out.push(format!(
                            "uses service {} ({name}); {}'s job does not declare it",
                            svc.key, m.label
                        ));
                    }
                }
            }
            None => out.push(format!(
                "uses service {} ({}), which the manifest cannot declare",
                svc.key,
                svc.image.as_deref().unwrap_or("no image")
            )),
        }
    }
    out
}

/// `import --report`: does `manifest` cover every workflow job that gates
/// pull requests, merge queues or default-branch pushes?
pub fn coverage_report(
    manifest_label: &str,
    manifest: &CiManifest,
    sources: &[WorkflowSource],
    default_branch: &str,
) -> Result<CoverageReport, String> {
    let import = import_workflows(sources)?;
    let mut workflows = Vec::new();
    for s in sources {
        workflows.push(gha::parse_workflow(&s.label, &s.text)?);
    }
    let mcmds = manifest_commands(manifest);
    let job_names: Vec<&str> = manifest.gate_jobs().map(|j| j.name.as_str()).collect();
    let mut jobs = Vec::new();
    let mut not_considered = Vec::new();
    for wf in &workflows {
        let file = basename(&wf.file).to_string();
        let triggers = gate_triggers(wf, default_branch);
        if triggers.is_empty() {
            not_considered.push((
                file,
                wf.jobs.iter().map(|j| j.id.clone()).collect(),
                format!("triggers {:?}", wf.triggers.events),
            ));
            continue;
        }
        for job in &wf.jobs {
            let mut notes = Vec::new();
            if let Some(c) = &job.if_expr {
                notes.push(format!("job condition `if: {c}` — considered as running"));
            }
            let jg = gate_commands(wf, job);
            let (oses, consumed) = job_oses(job);
            if oses.contains(&Os::Any) {
                notes.push("runs-on names no OS — no manifest job can be shown to cover it".to_string());
            }
            let mut matched = Vec::new();
            let mut matched_cmds: Vec<&ManifestCommand> = Vec::new();
            let mut missing = Vec::new();
            let mut caveats = jg.unmodeled.clone();
            for g in &jg.gates {
                match best_match(&mcmds, &g.working_dir, &g.key, &oses) {
                    Some(m) => {
                        matched.push((g.display.clone(), m.label.clone()));
                        caveats.extend(command_caveats(g, m));
                        matched_cmds.push(m);
                    }
                    None => missing.push(g.display.clone()),
                }
            }
            for pin in &jg.pins {
                let pinned = pin.tool.is_some_and(|t| {
                    manifest
                        .tools
                        .iter()
                        .any(|mt| mt.name == t && mt.version == pin.version)
                });
                if !pinned {
                    caveats.push(format!(
                        "pins {} — the manifest's [[tools]] does not pin the same version",
                        pin.what
                    ));
                }
            }
            caveats.extend(job_caveats(job, consumed.as_deref(), &matched_cmds));
            let mut seen = Vec::new();
            caveats.retain(|c| {
                let fresh = !seen.contains(c);
                seen.push(c.clone());
                fresh
            });
            let candidates = [slug(&job.id), slug(&format!("{}-{}", stem(&wf.file), job.id))];
            let name_match = candidates.iter().find(|c| job_names.contains(&c.as_str())).cloned();
            let verdict = if jg.gates.is_empty() {
                match &name_match {
                    Some(n) => {
                        notes.push(format!(
                            "no gate command found; manifest job `{n}` carries its name, which is \
                             not evidence of coverage"
                        ));
                        Coverage::NameOnly
                    }
                    None => {
                        notes.push("no gate command found".to_string());
                        Coverage::Uncovered
                    }
                }
            } else if !missing.is_empty() {
                if matched.is_empty() {
                    Coverage::Uncovered
                } else {
                    Coverage::Partial
                }
            } else if !caveats.is_empty() {
                Coverage::NeedsReview
            } else {
                Coverage::Covered
            };
            jobs.push(JobCoverage {
                workflow: file.clone(),
                job_id: job.id.clone(),
                triggers: triggers.clone(),
                verdict,
                matched,
                missing,
                caveats,
                notes,
            });
        }
    }
    let names: Vec<&str> = manifest.jobs.iter().map(|j| j.name.as_str()).collect();
    Ok(CoverageReport {
        manifest_label: manifest_label.to_string(),
        manifest_summary: format!(
            "schema v{}, {} job(s): {}",
            manifest.schema_version,
            manifest.jobs.len(),
            names.join(", ")
        ),
        default_branch: default_branch.to_string(),
        workflow_count: sources.len(),
        jobs,
        not_considered,
        import,
    })
}

/// The report as text.
pub fn render_report(r: &CoverageReport) -> String {
    let mut s = String::new();
    s.push_str("qontinui-ci import --report\n");
    s.push_str(&format!("  manifest:       {} ({})\n", r.manifest_label, r.manifest_summary));
    s.push_str(&format!("  workflows:      {} file(s)\n", r.workflow_count));
    s.push_str(&format!("  default branch: {}\n", r.default_branch));
    wrap_comment(&mut s, "  rule: ", "        ", COVERAGE_RULE);
    s.push('\n');
    s.push_str(&format!(
        "JOBS ON pull_request / push:{} ({}):\n",
        r.default_branch,
        r.jobs.len()
    ));
    for j in &r.jobs {
        let total = j.matched.len() + j.missing.len();
        s.push_str(&format!(
            "  {:<15} {} › {} [{}] {}/{} gate command(s) matched\n",
            j.verdict.as_str(),
            j.workflow,
            j.job_id,
            j.triggers.join(", "),
            j.matched.len(),
            total
        ));
        for m in &j.missing {
            s.push_str(&format!("      missing: {m}\n"));
        }
        for (g, m) in &j.matched {
            s.push_str(&format!("      matched: {g}  ←  {m}\n"));
        }
        for c in &j.caveats {
            s.push_str(&format!("      review:  {c}\n"));
        }
        for n in &j.notes {
            s.push_str(&format!("      note:    {n}\n"));
        }
    }
    if r.jobs.is_empty() {
        s.push_str(&format!(
            "  (none) — NO workflow job runs on pull_request, merge_group or a push to {}. That \
             is NOT \"nothing to cover\": either --default-branch is wrong, or these workflows \
             have no pull-request/push trigger the report can read. The precondition is not met.\n",
            r.default_branch
        ));
    }
    s.push('\n');
    s.push_str(&format!(
        "NOT CONSIDERED — not on pull_request or a push to {} ({} workflow(s)):\n",
        r.default_branch,
        r.not_considered.len()
    ));
    if r.not_considered.is_empty() {
        s.push_str("  (none)\n");
    }
    for (wf, jobs, why) in &r.not_considered {
        s.push_str(&format!("  {wf} (jobs: {}) — {why}\n", jobs.join(", ")));
    }
    s.push('\n');
    s.push_str("WHAT `qontinui-ci import` WOULD NOT CARRY OVER FROM THESE WORKFLOWS:\n");
    s.push_str(&render_items(&r.import.items, false));
    let count = |v: Coverage| r.jobs.iter().filter(|j| j.verdict == v).count();
    s.push_str(&format!(
        "summary: {} covered, {} needs-review, {} name-only, {} partial, {} uncovered of {} job(s); {} untranslated construct(s) — the Phase 9 precondition is {}\n",
        count(Coverage::Covered),
        count(Coverage::NeedsReview),
        count(Coverage::NameOnly),
        count(Coverage::Partial),
        count(Coverage::Uncovered),
        r.jobs.len(),
        r.import.count(ItemKind::Untranslated),
        if r.all_covered() { "MET" } else { "NOT MET" }
    ));
    s
}

/// For each manifest step, the imported step that runs the same command in
/// the same directory (`None` when no imported step does). The arming check
/// of Phase 8: an import over a repo's workflows should yield commands that
/// cover what its hand-written manifest declares.
pub fn manifest_steps_covered_by(m: &CiManifest, imported: &ImportOutcome) -> Vec<(String, Option<String>)> {
    let imported_cmds: Vec<(String, String, CommandKey)> = imported
        .jobs
        .iter()
        .flat_map(|j| {
            j.steps.iter().map(move |s| {
                (
                    format!("{}/{}", j.name, s.name),
                    s.working_dir.clone().unwrap_or_default(),
                    CommandKey::of(&s.command),
                )
            })
        })
        .collect();
    manifest_commands(m)
        .into_iter()
        .map(|mc| {
            let hit = imported_cmds
                .iter()
                .find(|(_, wd, key)| *wd == mc.working_dir && key.matches(&mc.key))
                .map(|(l, _, _)| l.clone());
            (mc.label, hit)
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn v(s: &[&str]) -> Vec<String> {
        s.iter().map(|x| x.to_string()).collect()
    }

    #[test]
    fn signatures_strip_wrappers_and_unify_runners() {
        assert_eq!(signature(&v(&["poetry", "run", "pytest", "-q"])), v(&["pytest"]));
        assert_eq!(signature(&v(&["python3", "-m", "mypy", "app/"])), v(&["mypy", "app"]));
        assert_eq!(signature(&v(&["npm", "run", "lint"])), v(&["npm", "run", "lint"]));
        assert_eq!(signature(&v(&["pnpm", "lint"])), v(&["pnpm", "lint"]));
        assert_eq!(signature(&v(&["bash", "./scripts/x.sh", "--a"])), v(&["scripts/x.sh"]));
        assert_eq!(signature(&v(&["./scripts/x.sh"])), v(&["scripts/x.sh"]));
        assert_eq!(signature(&v(&["cargo", "+stable", "clippy", "--all"])), v(&["cargo", "clippy"]));
        assert_eq!(signature(&v(&["RUST_LOG=x", "timeout", "600", "cargo", "test"])), v(&["cargo", "test"]));
    }

    #[test]
    fn matching_is_whole_argv_after_normalization() {
        let k = |a: &[&str]| CommandKey::of(&v(a));
        assert!(k(&["poetry", "run", "pytest", "-q"]).matches(&k(&["pytest", "-q"])));
        assert!(!k(&["npm", "run", "lint"]).matches(&k(&["pnpm", "lint"])));
        assert!(!k(&["npm", "ci"]).matches(&k(&["npm", "install"])));
        assert!(!k(&["pnpm", "install", "--frozen-lockfile"]).matches(&k(&["pnpm", "add"])));
        assert!(!k(&["pnpm", "audit"]).matches(&k(&["npm", "run", "audit"])));
        assert!(k(&["./scripts/x.sh"]).matches(&k(&["bash", "scripts/x.sh"])));
        assert!(!k(&["cargo", "fmt", "--all"]).matches(&k(&["cargo", "fmt", "--all", "--", "--check"])));
        assert!(!k(&["python", "-u", "a.py"]).matches(&k(&["python", "-u", "b.py"])));
        assert!(!k(&["cargo", "test"]).matches(&k(&["cargo", "test", "smoke_"])));
        assert!(!k(&["pytest"]).matches(&k(&["pytest", "-q"])));
    }

    #[test]
    fn only_pure_failure_conditions_are_diagnostics() {
        assert!(diagnostic_condition("failure()"));
        assert!(diagnostic_condition("${{ cancelled() || failure() }}"));
        assert!(!diagnostic_condition("always()"));
        assert!(!diagnostic_condition("${{ !cancelled() }}"));
        assert!(!diagnostic_condition("success() || failure()"));
    }

    #[test]
    fn slugs_are_manifest_job_names() {
        assert_eq!(slug("lint-and-typecheck"), "lint-and-typecheck");
        assert_eq!(slug("Build_Test"), "build-test");
        assert_eq!(slug("__x__"), "x");
        assert!(manifest::validate_job_name(&slug(&"a".repeat(80))).is_ok());
    }

    #[test]
    fn node_resolution_uses_registry_pins() {
        assert_eq!(resolve_node("22").unwrap(), ("22.11.0".to_string(), true));
        assert_eq!(resolve_node("22.11.0").unwrap(), ("22.11.0".to_string(), false));
        assert!(resolve_node("20").is_err());
        assert!(resolve_node("lts/*").is_err());
    }

    const SMALL: &str = r#"
name: CI
on:
  pull_request:
  push:
    branches: [main]
env:
  CARGO_TERM_COLOR: always
  SOME_TOKEN: ${{ secrets.SOME_TOKEN }}
jobs:
  lint:
    runs-on: ubuntu-latest
    timeout-minutes: 10
    steps:
      - uses: actions/checkout@v4
      - uses: actions/setup-node@v4
        with:
          node-version: "22"
      - name: Install
        working-directory: frontend
        run: npm ci
      - name: Lint
        run: cd frontend && npm run lint
      - name: Upload
        if: failure()
        uses: actions/upload-artifact@v4
  test:
    needs: lint
    runs-on: ${{ matrix.os }}
    strategy:
      matrix:
        os: [ubuntu-latest, windows-latest]
    services:
      db:
        image: postgres:16
        env:
          POSTGRES_PASSWORD: x
      cache:
        image: memcached:1.6
    steps:
      - uses: some/action@v1
        with:
          token: ${{ secrets.DEPLOY_KEY }}
      - name: Test
        shell: bash
        env:
          DATABASE_URL: postgres://x
          TESTING: "1"
        run: |
          set -euo pipefail
          cargo test --workspace
      - name: Pipe
        shell: bash
        run: cargo test | tee out.txt
"#;

    #[test]
    fn import_translates_and_lists_everything_else() {
        let o = import_workflows(&[WorkflowSource {
            label: ".github/workflows/ci.yml".into(),
            text: SMALL.into(),
        }])
        .unwrap();
        let m = o.manifest.as_ref().expect("valid manifest");
        assert_eq!(m.schema_version, 2);
        let lint = m.job("lint").unwrap();
        assert_eq!(lint.os, vec![Os::Linux]);
        assert_eq!(lint.steps.len(), 2);
        assert_eq!(lint.steps[1].command, v(&["npm", "run", "lint"]));
        assert_eq!(lint.steps[1].working_dir.as_deref(), Some("frontend"));
        assert_eq!(lint.steps[1].timeout_secs, Some(600));
        assert_eq!(lint.steps[1].env.get("CARGO_TERM_COLOR").map(String::as_str), Some("always"));
        let test = m.job("test").unwrap();
        assert_eq!(test.os, vec![Os::Linux, Os::Windows]);
        assert_eq!(test.needs, v(&["lint"]));
        assert_eq!(test.services.len(), 1);
        assert_eq!(test.services[0].name, "postgres");
        assert_eq!(test.steps.len(), 1);
        assert_eq!(test.steps[0].env.get("TESTING").map(String::as_str), Some("1"));
        assert!(!test.steps[0].env.contains_key("DATABASE_URL"));
        assert_eq!(m.tools[0].name, "node");
        let un = |needle: &str| {
            o.items
                .iter()
                .any(|i| i.kind == ItemKind::Untranslated && i.detail.contains(needle))
        };
        assert!(un("some/action"), "{:#?}", o.items);
        assert!(un("DEPLOY_KEY"));
        assert!(un("memcached"));
        assert!(un("pipe"));
        assert!(un("if: failure()"));
        assert!(un("SOME_TOKEN"));
        assert!(o.manifest_toml.contains("# UNTRANSLATED ("));
        assert!(o.manifest_toml.contains("some/action"));
    }

    const HOSTILE: &str = r#"
on: pull_request
jobs:
  build:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
        with:
          path: app
      - name: Clone sibling
        run: git clone --depth 1 https://github.com/acme/lib.git
      - name: System deps
        run: sudo apt-get install -y libssl-dev
      - name: Global tool
        run: cargo install cargo-audit
      - name: Inside
        run: make check
        working-directory: app/sub
      - name: Outside
        run: make other
        working-directory: lib
"#;

    #[test]
    fn host_mutation_is_refused_clones_become_siblings_and_paths_rebase() {
        let o = import_workflows(&[WorkflowSource {
            label: "b.yml".into(),
            text: HOSTILE.into(),
        }])
        .unwrap();
        assert_eq!(o.siblings, v(&["acme/lib"]));
        let m = o.manifest.as_ref().unwrap();
        let build = m.job("build").unwrap();
        let cmds: Vec<&Vec<String>> = build.steps.iter().map(|s| &s.command).collect();
        assert_eq!(cmds, vec![&v(&["make", "check"])]);
        assert_eq!(build.steps[0].working_dir.as_deref(), Some("sub"));
        let un = |needle: &str| {
            o.items
                .iter()
                .any(|i| i.kind == ItemKind::Untranslated && i.detail.contains(needle))
        };
        assert!(un("runs as root"), "{:#?}", o.items);
        assert!(un("global environment"));
        assert!(un("outside this repository"));
    }

    /// Report one workflow (`w.yml`) against a manifest text.
    fn report_one(workflow: &str, manifest_text: &str) -> CoverageReport {
        let m = manifest::parse_and_validate(manifest_text).unwrap();
        coverage_report(
            "ci.toml",
            &m,
            &[WorkflowSource {
                label: "w.yml".into(),
                text: workflow.into(),
            }],
            "main",
        )
        .unwrap()
    }

    fn verdict_of(r: &CoverageReport, job: &str) -> Coverage {
        r.jobs.iter().find(|j| j.job_id == job).unwrap().verdict
    }

    const V1_CARGO_TEST: &str = "version = 1\n[[steps]]\nname = \"t\"\ncommand = [\"cargo\", \"test\"]\n";

    #[test]
    fn any_os_manifest_does_not_cover_windows_or_macos() {
        let wf = "on: pull_request\njobs:\n  lin:\n    runs-on: ubuntu-latest\n    steps:\n      - run: cargo test\n  win:\n    runs-on: windows-latest\n    steps:\n      - run: cargo test\n        shell: bash\n  mac:\n    runs-on: macos-latest\n    steps:\n      - run: cargo test\n";
        let r = report_one(wf, V1_CARGO_TEST);
        assert_eq!(verdict_of(&r, "lin"), Coverage::Covered);
        assert_eq!(verdict_of(&r, "win"), Coverage::Uncovered);
        assert_eq!(verdict_of(&r, "mac"), Coverage::Uncovered);
        assert!(!r.all_covered());
    }

    #[test]
    fn variable_programs_are_opaque_gates() {
        let wf = "on: pull_request\njobs:\n  a:\n    runs-on: ubuntu-latest\n    steps:\n      - run: cargo test\n      - run: $PYTHON -m mypy app\n      - run: ${{ matrix.cmd }}\n";
        let r = report_one(wf, V1_CARGO_TEST);
        let a = r.jobs.iter().find(|j| j.job_id == "a").unwrap();
        assert_eq!(a.verdict, Coverage::Partial, "{a:#?}");
        assert_eq!(a.missing.len(), 2, "{a:#?}");
    }

    #[test]
    fn assertion_only_jobs_are_gates_and_names_are_not_coverage() {
        let wf = "on: pull_request\njobs:\n  checks:\n    runs-on: ubuntu-latest\n    steps:\n      - run: |\n          grep -q needle file.txt\n          jq -e .ok out.json\n          poetry check\n  quiet:\n    runs-on: ubuntu-latest\n    steps:\n      - run: echo hello\n      - run: make lint || true\n";
        let manifest = "version = 2\n[[jobs]]\nname = \"checks\"\n[[jobs.steps]]\nname = \"x\"\ncommand = [\"make\"]\n[[jobs]]\nname = \"quiet\"\n[[jobs.steps]]\nname = \"y\"\ncommand = [\"make\"]\n";
        let r = report_one(wf, manifest);
        let checks = r.jobs.iter().find(|j| j.job_id == "checks").unwrap();
        assert_eq!(checks.verdict, Coverage::Uncovered, "{checks:#?}");
        assert_eq!(checks.missing.len(), 3);
        assert_eq!(verdict_of(&r, "quiet"), Coverage::NameOnly);
        assert!(!r.all_covered(), "a name is never coverage");
        assert!(render_report(&r).contains("NOT MET"));
    }

    #[test]
    fn matched_commands_under_other_conditions_need_review() {
        let manifest = "version = 1\n[[steps]]\nname = \"t\"\ncommand = [\"cargo\", \"test\"]\n[[steps]]\nname = \"r\"\ncommand = [\"cargo\", \"test\"]\nworking_dir = \"sub\"\n[steps.env]\nRUSTFLAGS = \"-Dwarnings\"\n";
        let wf = "on: pull_request\njobs:\n  prefix:\n    runs-on: ubuntu-latest\n    steps:\n      - run: RUSTFLAGS=-Dwarnings cargo test\n  prefix-ok:\n    runs-on: ubuntu-latest\n    steps:\n      - run: RUSTFLAGS=-Dwarnings cargo test\n        working-directory: sub\n  toolchain:\n    runs-on: ubuntu-latest\n    steps:\n      - run: cargo +nightly test\n  jobenv:\n    runs-on: ubuntu-latest\n    env:\n      RUST_BACKTRACE: full\n    steps:\n      - run: cargo test\n  matrix:\n    runs-on: ubuntu-latest\n    strategy:\n      matrix:\n        feature: [a, b]\n    steps:\n      - run: cargo test\n  svc:\n    runs-on: ubuntu-latest\n    services:\n      db:\n        image: postgres:16\n    steps:\n      - run: cargo test\n  box:\n    runs-on: ubuntu-latest\n    container: rust:1.80\n    steps:\n      - run: cargo test\n  plain:\n    runs-on: ubuntu-latest\n    steps:\n      - run: cargo test\n";
        let r = report_one(wf, manifest);
        for job in ["prefix", "toolchain", "jobenv", "matrix", "svc", "box"] {
            assert_eq!(verdict_of(&r, job), Coverage::NeedsReview, "{job}: {:#?}", r.jobs);
        }
        // Even an identical env, when set by a `KEY=value` prefix, is outside
        // the report's grammar: NEEDS-REVIEW, never COVERED.
        assert_eq!(verdict_of(&r, "prefix-ok"), Coverage::NeedsReview);
        assert_eq!(verdict_of(&r, "plain"), Coverage::Covered);
        assert!(!r.all_covered());
    }

    #[test]
    fn windows_run_without_shell_is_powershell() {
        let wf = "on: pull_request\njobs:\n  w:\n    runs-on: windows-latest\n    steps:\n      - run: cargo test\n      - run: cargo build\n        shell: bash\n";
        let o = import_workflows(&[WorkflowSource {
            label: "w.yml".into(),
            text: wf.into(),
        }])
        .unwrap();
        let w = o.manifest.as_ref().unwrap().job("w").unwrap();
        assert_eq!(w.steps.len(), 1);
        assert_eq!(w.steps[0].command, v(&["cargo", "build"]));
        assert!(o.items.iter().any(|i| i.kind == ItemKind::Untranslated && i.detail.contains("PowerShell")));
        let manifest = "version = 2\n[[jobs]]\nname = \"w\"\nos = \"windows\"\n[[jobs.steps]]\nname = \"t\"\ncommand = [\"cargo\", \"test\"]\n[[jobs.steps]]\nname = \"b\"\ncommand = [\"cargo\", \"build\"]\n";
        let r = report_one(wf, manifest);
        assert_eq!(verdict_of(&r, "w"), Coverage::Partial, "{:#?}", r.jobs);
    }

    #[test]
    fn gates_are_deduplicated_by_command_not_by_display() {
        let long = "a".repeat(120);
        let wf = format!(
            "on: pull_request\njobs:\n  d:\n    runs-on: ubuntu-latest\n    steps:\n      - run: |\n          python {long} one\n          python {long} two\n          python {long} one\n"
        );
        let parsed = gha::parse_workflow("w.yml", &wf).unwrap();
        let gates = gate_commands(&parsed, &parsed.jobs[0]).gates;
        assert_eq!(gates.len(), 2);
    }

    fn job_yaml(name: &str, body: &str) -> String {
        format!("  {name}:\n    runs-on: ubuntu-latest\n{body}")
    }

    fn wf_of(jobs: &[String]) -> String {
        format!("on: pull_request\njobs:\n{}", jobs.concat())
    }

    const V1_MAKE: &str = "version = 1\n[[steps]]\nname = \"m\"\ncommand = [\"make\", \"check\"]\n";

    #[test]
    fn round2_deny_by_default() {
        // H1: dedup keeps toolchains apart.
        let wf = wf_of(&[job_yaml("t", "    steps:\n      - run: |\n          cargo test\n          cargo +nightly test\n")]);
        let p = gha::parse_workflow("w.yml", &wf).unwrap();
        assert_eq!(gate_commands(&p, &p.jobs[0]).gates.len(), 2);

        let jobs = [
            // H2: unresolvable directories are opaque.
            job_yaml("expr-wd", "    steps:\n      - run: make check\n        working-directory: ${{ matrix.dir }}\n"),
            job_yaml("var-cd", "    steps:\n      - run: |\n          cd \"$DIR\"\n          make check\n"),
            // H3: only `|| true` / `|| :` is harmless.
            job_yaml("rc", "    steps:\n      - run: make other || rc=$?\n"),
            // H4/M6: env effects.
            job_yaml("export", "    steps:\n      - run: |\n          export A=1\n          make check\n"),
            job_yaml("ghenv", "    steps:\n      - run: echo A=1 >> \"$GITHUB_ENV\"\n      - run: make check\n"),
            job_yaml("source", "    steps:\n      - run: |\n          source .env\n          make check\n"),
            // M1: any env key.
            job_yaml("anyenv", "    env:\n      PYTHON_VERSION: \"3.12\"\n    steps:\n      - run: make check\n"),
            // M2: version pins.
            job_yaml("py", "    steps:\n      - uses: actions/setup-python@v5\n        with:\n          python-version: \"3.12\"\n      - run: make check\n"),
            job_yaml("rust", "    steps:\n      - uses: dtolnay/rust-toolchain@1.80.0\n      - run: make check\n"),
            job_yaml("rustup", "    steps:\n      - run: |\n          rustup default nightly\n          make check\n"),
            job_yaml("node-pinned", "    steps:\n      - uses: actions/setup-node@v4\n        with:\n          node-version: 22.11.0\n      - run: make check\n"),
            // Lows: runner labels.
            "  arm:\n    runs-on: ubuntu-24.04-arm\n    steps:\n      - run: make check\n".to_string(),
            "  selfhosted:\n    runs-on: [self-hosted, linux]\n    steps:\n      - run: make check\n".to_string(),
            job_yaml("plain", "    steps:\n      - run: make check\n"),
        ];
        let manifest = "version = 1\n[[tools]]\nname = \"node\"\nversion = \"22.11.0\"\n[[steps]]\nname = \"m\"\ncommand = [\"make\", \"check\"]\n[[steps]]\nname = \"s\"\ncommand = [\"source\", \".env\"]\n";
        let r = report_one(&wf_of(&jobs), manifest);
        assert_eq!(verdict_of(&r, "expr-wd"), Coverage::Uncovered, "{:#?}", r.jobs);
        assert_eq!(verdict_of(&r, "var-cd"), Coverage::Uncovered);
        assert_eq!(verdict_of(&r, "rc"), Coverage::Uncovered, "`|| rc=$?` leaves a gate");
        // `rustup default` is itself an unmatched gate, and it is also a pin.
        let rustup = r.jobs.iter().find(|j| j.job_id == "rustup").unwrap();
        assert_eq!(rustup.verdict, Coverage::Partial);
        assert!(rustup.caveats.iter().any(|c| c.contains("rustup default nightly")));
        for j in ["export", "ghenv", "source", "anyenv", "py", "rust", "arm", "selfhosted"] {
            assert_eq!(verdict_of(&r, j), Coverage::NeedsReview, "{j}: {:#?}", r.jobs.iter().find(|x| x.job_id == j));
        }
        assert_eq!(verdict_of(&r, "node-pinned"), Coverage::Covered);
        assert_eq!(verdict_of(&r, "plain"), Coverage::Covered);
        assert!(!r.all_covered());

        // M3: services are checked per matched job, not pooled.
        let wf = wf_of(&[job_yaml(
            "svc",
            "    services:\n      db:\n        image: postgres:16\n    steps:\n      - run: |\n          make a\n          make b\n",
        )]);
        let manifest = "version = 2\n[[jobs]]\nname = \"with-db\"\n[[jobs.services]]\nname = \"postgres\"\nversion = \"16\"\n[[jobs.steps]]\nname = \"a\"\ncommand = [\"make\", \"a\"]\n[[jobs]]\nname = \"no-db\"\n[[jobs.steps]]\nname = \"b\"\ncommand = [\"make\", \"b\"]\n";
        let r = report_one(&wf, manifest);
        assert_eq!(verdict_of(&r, "svc"), Coverage::NeedsReview, "{:#?}", r.jobs);

        // Lows: a manifest `$VAR` word matches nothing.
        let r = report_one(
            &wf_of(&[job_yaml("v", "    steps:\n      - run: make $TARGET\n")]),
            "version = 1\n[[steps]]\nname = \"m\"\ncommand = [\"make\", \"$TARGET\"]\n",
        );
        assert_eq!(verdict_of(&r, "v"), Coverage::Uncovered);

        // merge_group is considered.
        let r = report_one(
            "on: merge_group\njobs:\n  q:\n    runs-on: ubuntu-latest\n    steps:\n      - run: make check\n",
            V1_MAKE,
        );
        assert_eq!(verdict_of(&r, "q"), Coverage::Covered);
        assert!(r.all_covered());

        // H5: zero considered jobs is NOT MET.
        let r = report_one(
            "on:\n  push:\n    branches: [develop]\njobs:\n  d:\n    runs-on: ubuntu-latest\n    steps:\n      - run: make check\n",
            V1_MAKE,
        );
        assert!(r.jobs.is_empty());
        assert!(!r.all_covered());
        let text = render_report(&r);
        assert!(text.contains("NOT MET") && text.contains("--default-branch is wrong"), "{text}");
    }

    #[test]
    fn round3_allowlist_and_remaining_holes() {
        let jobs = [
            // Grammar: pushd, env wrapper, redirect, assignment, stdin, timeout wrapper.
            job_yaml("pushd", "    steps:\n      - run: |\n          pushd sub\n          make check\n"),
            job_yaml("envwrap", "    steps:\n      - run: |\n          env A=1 true\n          make check\n"),
            job_yaml("redirect", "    steps:\n      - run: |\n          echo x > f.txt\n          make check\n"),
            job_yaml("assign", "    steps:\n      - run: |\n          A=1\n          make check\n"),
            job_yaml("stdin", "    steps:\n      - run: make check < input.txt\n"),
            job_yaml("twrap", "    steps:\n      - run: |\n          timeout 60 true\n          make check\n"),
            // H1: an advisory step still downgrades.
            job_yaml("advisory", "    steps:\n      - run: echo A=1 >> \"$GITHUB_ENV\"\n        continue-on-error: true\n      - run: make check\n"),
            // M4: checkout of another ref.
            job_yaml("ref", "    steps:\n      - uses: actions/checkout@v4\n        with:\n          ref: other\n      - run: make check\n"),
            // L2: an expression env never matches, even identical.
            job_yaml("expr-env", "    env:\n      T: ${{ secrets.T }}\n    steps:\n      - run: make check\n"),
            // L4: a timeout tighter than the manifest's.
            job_yaml("tight", "    timeout-minutes: 5\n    steps:\n      - run: make check\n"),
            // M3: macOS.
            "  mac:\n    runs-on: macos-13\n    steps:\n      - run: make check\n".to_string(),
            job_yaml("plain", "    steps:\n      - run: make check\n"),
        ];
        let manifest = "version = 2\n[[jobs]]\nname = \"all\"\n[[jobs.steps]]\nname = \"m\"\ncommand = [\"make\", \"check\"]\n[[jobs]]\nname = \"mac\"\nos = \"macos\"\n[[jobs.steps]]\nname = \"m\"\ncommand = [\"make\", \"check\"]\n";
        let r = report_one(&wf_of(&jobs), manifest);
        // Each is NEEDS-REVIEW, or stricter (an unmatched wrapper command
        // leaves the job PARTIAL); never COVERED, and always with the reason.
        for j in ["pushd", "envwrap", "redirect", "assign", "stdin", "twrap", "advisory", "ref", "expr-env", "tight", "mac"] {
            let jc = r.jobs.iter().find(|x| x.job_id == j).unwrap();
            assert!(
                matches!(jc.verdict, Coverage::NeedsReview | Coverage::Partial) && !jc.caveats.is_empty(),
                "{j}: {jc:#?}"
            );
        }
        assert_eq!(verdict_of(&r, "plain"), Coverage::Covered);

        // H5: env compared both ways — a manifest-only env key is a difference.
        let r = report_one(
            &wf_of(&[job_yaml("p", "    steps:\n      - run: make check\n")]),
            "version = 1\n[[steps]]\nname = \"m\"\ncommand = [\"make\", \"check\"]\n[steps.env]\nRUST_LOG = \"debug\"\n",
        );
        assert_eq!(verdict_of(&r, "p"), Coverage::NeedsReview);

        // L1: a manifest word CONTAINING `$` matches nothing.
        let r = report_one(
            &wf_of(&[job_yaml("p", "    steps:\n      - run: make out=a$b\n")]),
            "version = 1\n[[steps]]\nname = \"m\"\ncommand = [\"make\", \"out=a$b\"]\n",
        );
        assert_eq!(verdict_of(&r, "p"), Coverage::Uncovered);

        // L3: failing-mode actions are opaque gates.
        let r = report_one(
            &wf_of(&[job_yaml("p", "    steps:\n      - run: make check\n      - uses: codecov/codecov-action@v4\n        with:\n          fail_ci_if_error: true\n")]),
            V1_MAKE,
        );
        assert_eq!(verdict_of(&r, "p"), Coverage::Partial);
    }

    #[test]
    fn report_says_what_is_and_is_not_covered() {
        let manifest = manifest::parse_and_validate(
            r#"
version = 1
[[steps]]
name = "install"
command = ["npm", "ci"]
working_dir = "frontend"
[steps.env]
CARGO_TERM_COLOR = "always"
[[steps]]
name = "lint"
command = ["npm", "run", "lint"]
working_dir = "frontend"
[steps.env]
CARGO_TERM_COLOR = "always"
"#,
        )
        .unwrap();
        let r = coverage_report(
            ".qontinui/ci.toml",
            &manifest,
            &[WorkflowSource {
                label: "ci.yml".into(),
                text: SMALL.into(),
            }],
            "main",
        )
        .unwrap();
        let lint = r.jobs.iter().find(|j| j.job_id == "lint").unwrap();
        // Every gate matched, but the workflow sets SOME_TOKEN (a secret
        // expression) and pins node 22 — neither reproduced by the manifest.
        assert_eq!(lint.verdict, Coverage::NeedsReview, "{lint:#?}");
        assert!(lint.missing.is_empty());
        let test = r.jobs.iter().find(|j| j.job_id == "test").unwrap();
        assert_eq!(test.verdict, Coverage::Uncovered);
        assert!(!r.all_covered());
        let text = render_report(&r);
        assert!(text.contains("NOT MET"));
        assert!(text.contains("UNTRANSLATED ("));
    }
}
