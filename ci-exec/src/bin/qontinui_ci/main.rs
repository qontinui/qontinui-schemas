//! `qontinui-ci` — run a repo's CI from any shell, with no coord at all.
//!
//! ```text
//! qontinui-ci run      [--job <name>] [--repo-dir <dir>] [--manifest <path>] [--root <dir>]
//! qontinui-ci list     [--manifest <file>]
//! qontinui-ci validate [--manifest <file>]
//! qontinui-ci import       [--out <file> [--force]] <workflow.yml>...
//! qontinui-ci import       --report <ci.toml> [--default-branch <name>] <workflow.yml>...
//! qontinui-ci gen-workflow --executor-rev <sha> [--manifest <file>] [--out <file>|-] [--force]
//! qontinui-ci gen-workflow --executor-rev <sha> --check [--manifest <file>] [--out <file>]
//! ```
//!
//! `run` executes one job of `.qontinui/ci.toml` exactly as a CI node would:
//! a fresh worktree of the repo's committed `HEAD` (uncommitted changes are not
//! part of the run), the manifest read from that commit, the declared tools,
//! siblings and services provisioned, the steps run, the worktree removed. It
//! is the same executor the runner and the CI host agent run (plan
//! `2026-10-04-coord-managed-ci-for-every-tenant-with-an-actions-free-mode`, D5:
//! CI keeps running when coord is down).
//!
//! `import` and `gen-workflow` are the onboarding and hybrid-mode halves (plan
//! Phase 8): `import` turns GitHub workflow files into a v2 manifest and lists
//! everything it could not carry over; `import --report` says whether a
//! manifest covers every workflow job that gates pull requests and
//! default-branch pushes; `gen-workflow` writes the generated GitHub leg and
//! `--check` fails a hand-edit of it.
//!
//! Exit status: 0 success, 1 failure, 3 cancelled (a non-verdict: the job did
//! not run on its merits — wrong OS, a checkout that never produced a tree),
//! 2 usage or setup error. For `import`: 1 when no job could be translated.
//! For `import --report`: 1 when any considered job is not covered. For
//! `gen-workflow --check`: 1 when the file is missing, hand-written,
//! hand-edited or stale.

use std::path::{Path, PathBuf};
use std::process::ExitCode;
use std::sync::Arc;

use qontinui_ci_exec::dispatch::DispatchPayload;
use qontinui_ci_exec::host::BoxFuture;
use qontinui_ci_exec::import::{self, ItemKind, WorkflowSource};
use qontinui_ci_exec::manifest::{self, CiManifest};
use qontinui_ci_exec::report::{Conclusion, LogSink, Reporter, Verdict};
use qontinui_ci_exec::{executor, gen_workflow, host_sizing, standalone};

/// `println!` that cannot panic. A run's output goes to a terminal that may
/// close mid-run (SIGHUP, a closed pipe); `println!` panics on a failed write,
/// which would abort the run before its cleanup. Output is best-effort; the
/// cleanup is not.
macro_rules! out {
    ($($arg:tt)*) => {{
        use std::io::Write as _;
        let _ = writeln!(std::io::stdout(), $($arg)*);
    }};
}

/// `eprintln!` that cannot panic — see [`out!`].
macro_rules! err {
    ($($arg:tt)*) => {{
        use std::io::Write as _;
        let _ = writeln!(std::io::stderr(), $($arg)*);
    }};
}

const DEFAULT_MANIFEST: &str = ".qontinui/ci.toml";

const USAGE: &str = "\
usage:
  qontinui-ci run      [--job <name>] [--repo-dir <dir>] [--manifest <path>] [--root <dir>]
  qontinui-ci list     [--manifest <file>]
  qontinui-ci validate [--manifest <file>]
  qontinui-ci import       [--out <file> [--force]] <workflow.yml>...
  qontinui-ci import       --report <ci.toml> [--default-branch <name>] <workflow.yml>...
  qontinui-ci gen-workflow --executor-rev <sha> [--manifest <file>] [--out <file>|-] [--force]
  qontinui-ci gen-workflow --executor-rev <sha> --check [--manifest <file>] [--out <file>]

run       run one job against a fresh checkout of the repo's committed HEAD
            --job       the [[jobs]] name (default: ci — the one job of a v1 manifest;
                        a v2 manifest with no `ci` job reports cancelled)
            --repo-dir  any directory inside the repo (default: the current directory)
            --manifest  the manifest path inside the repo (default: .qontinui/ci.toml)
            --root      the CI root (default: the directory that contains the repo).
                        run WRITES under it: .ci-worktrees/<run id>/ (the job's
                        checkout and its siblings, removed when the run ends),
                        .ci-target/<repo>/ (a warm cargo target dir, kept), and
                        .ci-tools/ (the version-keyed tool cache, kept).
                        It also registers a git worktree admin entry under
                        <root>/<repo>/.git (and may fetch the dispatched
                        commit there); the entry is removed and pruned when
                        the run ends
list      print the manifest's jobs and the check-run contexts they produce
validate  parse and validate the manifest; exit 0 when it is valid
import    convert GitHub workflow files into a version-2 ci.toml (stdout, or --out;
            --out refuses to overwrite an existing file without --force). Every
            construct it cannot translate (an arbitrary `uses:` action, an `if:`,
            an expression or secret, a matrix it cannot expand, a script that is
            not plain commands, …) is listed in the generated file's header and
            on stderr — nothing is dropped silently. Exit 1 when no job could be
            translated
            --report <ci.toml>  instead of converting, list every workflow job that
                        runs on pull_request or a push to the default branch and
                        whether <ci.toml> covers it (the rule is printed with the
                        report); exit 0 only when every such job is covered
            --default-branch    the branch `push` filters are matched against (main)
gen-workflow  write .github/workflows/qontinui-ci.yml, the hybrid-mode GitHub leg:
            one `qontinui-ci run --job <name>` job per gate job, workflow_dispatch
            only, a GENERATED header with a content hash
            --manifest      the manifest (default: .qontinui/ci.toml)
            --out           the file to write (default: the repo's
                            .github/workflows/qontinui-ci.yml; - for stdout)
            --executor-rev  REQUIRED: the full 40-hex qontinui-schemas commit the
                            executor is installed from
            --force         overwrite a file that is not a generated one
            --check         write nothing; exit 1 when the committed file is missing,
                            hand-written, hand-edited or stale against the
                            --executor-rev given (never the pin the file records).
                            Wire it into the repo's CI so a hand-edit fails";

fn main() -> ExitCode {
    tracing_subscriber::fmt()
        .with_writer(std::io::stderr)
        // A failed log write must never panic: after a SIGHUP or a closed pipe
        // stderr returns EIO/EPIPE, and the default internal-error report is an
        // `eprintln!` that would abort cleanup mid-way.
        .log_internal_errors(false)
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_env("QONTINUI_CI_LOG")
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("warn")),
        )
        .init();
    let args: Vec<String> = std::env::args().skip(1).collect();
    match dispatch(&args) {
        Ok(code) => code,
        Err(e) => {
            err!("qontinui-ci: {e}");
            ExitCode::from(2)
        }
    }
}

/// Parsed `--flag value` pairs, refusing any flag the subcommand does not take.
fn flags(args: &[String], allowed: &[&str]) -> Result<Vec<(String, String)>, String> {
    let mut out = Vec::new();
    let mut it = args.iter();
    while let Some(flag) = it.next() {
        if !allowed.contains(&flag.as_str()) {
            return Err(format!("unknown argument {flag:?}\n\n{USAGE}"));
        }
        let value = it
            .next()
            .ok_or_else(|| format!("{flag} needs a value\n\n{USAGE}"))?;
        out.push((flag.clone(), value.clone()));
    }
    Ok(out)
}

fn flag<'a>(flags: &'a [(String, String)], name: &str) -> Option<&'a str> {
    flags
        .iter()
        .rev()
        .find(|(k, _)| k == name)
        .map(|(_, v)| v.as_str())
}

fn dispatch(args: &[String]) -> Result<ExitCode, String> {
    let Some((sub, rest)) = args.split_first() else {
        return Err(USAGE.to_string());
    };
    match sub.as_str() {
        "run" => run(&flags(
            rest,
            &["--job", "--repo-dir", "--manifest", "--root"],
        )?),
        "list" => list(&flags(rest, &["--manifest"])?),
        "validate" => validate(&flags(rest, &["--manifest"])?),
        "import" => import_cmd(rest),
        "gen-workflow" => gen_workflow_cmd(rest),
        "-h" | "--help" | "help" => {
            out!("{USAGE}");
            Ok(ExitCode::SUCCESS)
        }
        other => Err(format!("unknown subcommand {other:?}\n\n{USAGE}")),
    }
}

/// Parsed arguments of a subcommand that also takes switches and positionals.
struct Args {
    valued: Vec<(String, String)>,
    switches: Vec<String>,
    positionals: Vec<String>,
}

/// `--flag value` for `valued`, bare `--switch` for `switches`, everything
/// else positional (after `--`, everything is positional).
fn parse_args(args: &[String], valued: &[&str], switches: &[&str]) -> Result<Args, String> {
    let mut out = Args {
        valued: Vec::new(),
        switches: Vec::new(),
        positionals: Vec::new(),
    };
    let mut it = args.iter();
    let mut rest_positional = false;
    while let Some(a) = it.next() {
        if rest_positional {
            out.positionals.push(a.clone());
        } else if a == "--" {
            rest_positional = true;
        } else if valued.contains(&a.as_str()) {
            let v = it
                .next()
                .ok_or_else(|| format!("{a} needs a value\n\n{USAGE}"))?;
            out.valued.push((a.clone(), v.clone()));
        } else if switches.contains(&a.as_str()) {
            out.switches.push(a.clone());
        } else if a.starts_with("--") {
            return Err(format!("unknown argument {a:?}\n\n{USAGE}"));
        } else {
            out.positionals.push(a.clone());
        }
    }
    Ok(out)
}

fn read_workflows(paths: &[String]) -> Result<Vec<WorkflowSource>, String> {
    if paths.is_empty() {
        return Err(format!("name at least one workflow file\n\n{USAGE}"));
    }
    paths
        .iter()
        .map(|p| {
            std::fs::read_to_string(p)
                .map(|text| WorkflowSource {
                    label: p.replace('\\', "/"),
                    text,
                })
                .map_err(|e| format!("read {p}: {e}"))
        })
        .collect()
}

fn import_cmd(rest: &[String]) -> Result<ExitCode, String> {
    let args = parse_args(
        rest,
        &["--out", "--report", "--default-branch"],
        &["--force"],
    )?;
    let sources = read_workflows(&args.positionals)?;
    if let Some(manifest_path) = flag(&args.valued, "--report") {
        if flag(&args.valued, "--out").is_some() || args.switches.iter().any(|s| s == "--force") {
            return Err("--report writes nothing; --out and --force do not apply".to_string());
        }
        let manifest = read_manifest(Path::new(manifest_path))?;
        let branch = flag(&args.valued, "--default-branch").unwrap_or("main");
        let report = import::coverage_report(manifest_path, &manifest, &sources, branch)?;
        out!("{}", import::render_report(&report).trim_end());
        return Ok(if report.all_covered() {
            ExitCode::SUCCESS
        } else {
            ExitCode::from(1)
        });
    }
    if flag(&args.valued, "--default-branch").is_some() {
        return Err("--default-branch applies to --report only".to_string());
    }
    let outcome = import::import_workflows(&sources)?;
    match flag(&args.valued, "--out") {
        Some(path) if path != "-" => {
            let p = Path::new(path);
            if p.exists() && !args.switches.iter().any(|s| s == "--force") {
                return Err(format!(
                    "{path} exists — refusing to overwrite it without --force"
                ));
            }
            if let Some(dir) = p.parent().filter(|d| !d.as_os_str().is_empty()) {
                std::fs::create_dir_all(dir)
                    .map_err(|e| format!("create {}: {e}", dir.display()))?;
            }
            std::fs::write(p, &outcome.manifest_toml).map_err(|e| format!("write {path}: {e}"))?;
            err!("qontinui-ci import: wrote {path}");
        }
        _ => out!("{}", outcome.manifest_toml.trim_end()),
    }
    err!(
        "qontinui-ci import: {} job(s) from {} workflow file(s); {} untranslated, {} translated \
         with a change, {} no-op construct(s) — each listed in the generated header",
        outcome.jobs.len(),
        sources.len(),
        outcome.count(ItemKind::Untranslated),
        outcome.count(ItemKind::Changed),
        outcome.count(ItemKind::NoOp)
    );
    for item in outcome
        .items
        .iter()
        .filter(|i| i.kind == ItemKind::Untranslated)
    {
        err!("  UNTRANSLATED {}: {}", item.location, item.detail);
    }
    if outcome.manifest.is_none() {
        err!("qontinui-ci import: no job could be translated — the output is not a valid manifest");
        return Ok(ExitCode::from(1));
    }
    Ok(ExitCode::SUCCESS)
}

/// The repo root containing `manifest` and the manifest's path relative to
/// it (forward slashes). Outside a git repo: no root, the path as given.
fn repo_layout(manifest: &Path) -> (Option<PathBuf>, String) {
    let given = manifest.to_string_lossy().replace('\\', "/");
    let Ok(canon) = manifest.canonicalize() else {
        return (None, given);
    };
    let dir = canon.parent().map(Path::to_path_buf).unwrap_or_default();
    let Ok(top) = git(&dir, &["rev-parse", "--show-toplevel"]) else {
        return (None, given);
    };
    let Ok(top) = PathBuf::from(top).canonicalize() else {
        return (None, given);
    };
    match canon.strip_prefix(&top) {
        Ok(rel) => (Some(top.clone()), rel.to_string_lossy().replace('\\', "/")),
        Err(_) => (None, given),
    }
}

fn gen_workflow_cmd(rest: &[String]) -> Result<ExitCode, String> {
    let args = parse_args(
        rest,
        &["--manifest", "--out", "--executor-rev"],
        &["--force", "--check"],
    )?;
    if let Some(p) = args.positionals.first() {
        return Err(format!("unexpected argument {p:?}\n\n{USAGE}"));
    }
    let manifest_path = PathBuf::from(flag(&args.valued, "--manifest").unwrap_or(DEFAULT_MANIFEST));
    let manifest = read_manifest(&manifest_path)?;
    let (root, rel_manifest) = repo_layout(&manifest_path);
    if !gen_workflow::plain_token(&rel_manifest) {
        return Err(format!(
            "the manifest path {rel_manifest:?} must be a plain repo-relative path \
             ([A-Za-z0-9._/-]) — it is written into the generated workflow's shell line"
        ));
    }
    let out_path = match flag(&args.valued, "--out") {
        Some(p) => p.to_string(),
        None => match &root {
            Some(r) => r
                .join(gen_workflow::WORKFLOW_PATH)
                .to_string_lossy()
                .to_string(),
            None => gen_workflow::WORKFLOW_PATH.to_string(),
        },
    };
    let check = args.switches.iter().any(|s| s == "--check");
    let force = args.switches.iter().any(|s| s == "--force");
    // The committed file as bytes: a non-UTF-8 file is still a file that must
    // not be overwritten silently.
    let existing_bytes = std::fs::read(&out_path).ok();
    let existing = existing_bytes
        .as_ref()
        .map(|b| String::from_utf8_lossy(b).to_string());
    // The pin is required in BOTH modes, and --check never reads it from the
    // committed file: a hand-edited pin plus a recomputed hash must fail.
    let executor_rev = flag(&args.valued, "--executor-rev")
        .ok_or_else(|| {
            format!(
                "gen-workflow needs --executor-rev <40-hex qontinui-schemas sha>: the GitHub leg \
                 installs the executor from a pinned commit\n\n{USAGE}"
            )
        })?
        .to_string();
    if !gen_workflow::pinned_rev(&executor_rev) {
        return Err(format!(
            "--executor-rev {executor_rev:?} must be a full 40-hex commit sha (a branch or tag can \
             move)"
        ));
    }
    let text = gen_workflow::generate(
        &manifest,
        &gen_workflow::GenOptions {
            manifest_path: rel_manifest,
            executor_rev,
        },
    );
    if check {
        if force || out_path == "-" {
            return Err("--check writes nothing; --force and --out - do not apply".to_string());
        }
        let verdict = gen_workflow::check(existing.as_deref(), &text);
        return Ok(if verdict == gen_workflow::CheckVerdict::UpToDate {
            out!("ok: {out_path} is {}", verdict.explain());
            ExitCode::SUCCESS
        } else {
            err!("{out_path}: {}", verdict.explain());
            ExitCode::from(1)
        });
    }
    if out_path == "-" {
        out!("{}", text.trim_end());
        return Ok(ExitCode::SUCCESS);
    }
    let p = Path::new(&out_path);
    if let Some(existing) = &existing {
        if !gen_workflow::is_generated(existing) && !force {
            return Err(format!(
                "{out_path} exists and is not a generated file (no GENERATED header) — it is a \
                 hand-written workflow; refusing to overwrite it without --force"
            ));
        }
    }
    if let Some(dir) = p.parent().filter(|d| !d.as_os_str().is_empty()) {
        std::fs::create_dir_all(dir).map_err(|e| format!("create {}: {e}", dir.display()))?;
    }
    std::fs::write(p, &text).map_err(|e| format!("write {out_path}: {e}"))?;
    out!(
        "wrote {out_path}: {} gate job(s) from {}",
        manifest.gate_jobs().count(),
        manifest_path.display()
    );
    Ok(ExitCode::SUCCESS)
}

fn read_manifest(path: &Path) -> Result<CiManifest, String> {
    let text =
        std::fs::read_to_string(path).map_err(|e| format!("read {}: {e}", path.display()))?;
    manifest::parse_and_validate(&text).map_err(|e| format!("{}: {e}", path.display()))
}

fn validate(flags: &[(String, String)]) -> Result<ExitCode, String> {
    let path = PathBuf::from(flag(flags, "--manifest").unwrap_or(DEFAULT_MANIFEST));
    match read_manifest(&path) {
        Ok(m) => {
            let names: Vec<&str> = m.jobs.iter().map(|j| j.name.as_str()).collect();
            out!(
                "ok: {} — schema v{}, {} job(s): {}",
                path.display(),
                m.schema_version,
                m.jobs.len(),
                names.join(", ")
            );
            Ok(ExitCode::SUCCESS)
        }
        Err(e) => {
            err!("invalid: {e}");
            Ok(ExitCode::from(1))
        }
    }
}

fn list(flags: &[(String, String)]) -> Result<ExitCode, String> {
    let path = PathBuf::from(flag(flags, "--manifest").unwrap_or(DEFAULT_MANIFEST));
    let m = read_manifest(&path)?;
    out!("{} (schema v{})", path.display(), m.schema_version);
    for job in &m.jobs {
        let os: Vec<&str> = job.os.iter().map(|o| o.as_str()).collect();
        let mut line = format!(
            "  {:<24} os={} steps={} services={}",
            job.name,
            os.join(","),
            job.steps.len(),
            job.services.len()
        );
        if !job.needs.is_empty() {
            line.push_str(&format!(" needs={}", job.needs.join(",")));
        }
        match &job.schedule {
            Some(cron) => line.push_str(&format!(
                " schedule=\"{cron}\" (scheduled only, not a gate)"
            )),
            None => line.push_str(&format!(" checks: {}", job.check_contexts().join("; "))),
        }
        out!("{line}");
    }
    out!(
        "  gate jobs: {}",
        m.gate_jobs()
            .map(|j| j.name.as_str())
            .collect::<Vec<_>>()
            .join(", ")
    );
    Ok(ExitCode::SUCCESS)
}

/// `git <args>` in `dir`, trimmed stdout.
fn git(dir: &Path, args: &[&str]) -> Result<String, String> {
    let out = std::process::Command::new("git")
        .current_dir(dir)
        .args(args)
        .output()
        .map_err(|e| format!("git {}: {e}", args.join(" ")))?;
    if !out.status.success() {
        return Err(format!(
            "git {} failed: {}",
            args.join(" "),
            String::from_utf8_lossy(&out.stderr).trim()
        ));
    }
    Ok(String::from_utf8_lossy(&out.stdout).trim().to_string())
}

fn run(flags: &[(String, String)]) -> Result<ExitCode, String> {
    let job = flag(flags, "--job").map(str::to_string);
    let start_dir = match flag(flags, "--repo-dir") {
        Some(d) => PathBuf::from(d),
        None => std::env::current_dir().map_err(|e| format!("current directory: {e}"))?,
    };
    let toplevel = PathBuf::from(git(&start_dir, &["rev-parse", "--show-toplevel"])?);
    let toplevel = toplevel
        .canonicalize()
        .map_err(|e| format!("{}: {e}", toplevel.display()))?;
    let repo_name = toplevel
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .ok_or_else(|| format!("{} has no directory name", toplevel.display()))?;
    // The executor fetches into `<root>/<repo name>`, so by default the root is
    // the directory the repo sits in — where its sibling checkouts also live.
    let root = match flag(flags, "--root") {
        Some(r) => PathBuf::from(r),
        None => toplevel
            .parent()
            .map(Path::to_path_buf)
            .ok_or_else(|| format!("{} has no parent directory", toplevel.display()))?,
    };
    if root.join(&repo_name).canonicalize().ok().as_deref() != Some(toplevel.as_path()) {
        return Err(format!(
            "the executor checks out from <root>/{repo_name}, which is not {} — pass --root as \
             the directory that contains the repo",
            toplevel.display()
        ));
    }
    let head = git(&toplevel, &["rev-parse", "HEAD"])?;
    if !git(&toplevel, &["status", "--porcelain"])?.is_empty() {
        err!(
            "qontinui-ci: note — the working tree has uncommitted changes; this run checks out \
             the committed HEAD {head} and does not include them"
        );
    }
    let stamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or_default();
    let dispatch_id = format!("local-{stamp}-{}", std::process::id());
    let payload = DispatchPayload {
        candidate_ref: format!("refs/ci-dispatch/local/{dispatch_id}"),
        dispatch_id,
        repo: repo_name,
        head_sha: head,
        // Never fetched — HEAD is already in the repo — but validated like any
        // dispatch's, so it is the repo's own absolute path.
        fetch_url: toplevel.to_string_lossy().replace('\\', "/"),
        pr_number: None,
        manifest_path: flag(flags, "--manifest")
            .unwrap_or(DEFAULT_MANIFEST)
            .to_string(),
        job,
        check_name: String::new(),
        coord_http_url: String::new(),
    };
    let label = std::env::var("HOSTNAME")
        .or_else(|_| std::env::var("COMPUTERNAME"))
        .unwrap_or_else(|_| "this machine".to_string());
    let host = standalone::host(root, format!("{label} (qontinui-ci, standalone)"));
    let runtime = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .map_err(|e| format!("start async runtime: {e}"))?;
    let conclusion = runtime.block_on(async move {
        let capacity = tokio::task::spawn_blocking(host_sizing::probe)
            .await
            .unwrap_or(host_sizing::HostCapacity {
                mem_bytes: None,
                cpus: 1,
            });
        let cancel = tokio_util::sync::CancellationToken::new();
        let on_signal = cancel.clone();
        tokio::spawn(async move {
            let signal = interrupted().await;
            // Cancel FIRST: after a SIGHUP the terminal is gone and a stderr
            // write may fail — nothing may stand between the signal and the
            // cancellation that lets the tree guards and cleanup run.
            on_signal.cancel();
            err!("qontinui-ci: {signal} — cancelling the job and cleaning up");
        });
        // A lone CLI run is the only dispatch on this host: N = 1.
        executor::run_dispatch(&host, Box::new(Terminal), payload, cancel, capacity, 1).await
    });
    Ok(match conclusion {
        Conclusion::Success => ExitCode::SUCCESS,
        Conclusion::Failure => ExitCode::from(1),
        Conclusion::Cancelled => ExitCode::from(3),
    })
}

/// Resolves on the first terminating signal, naming it. SIGTERM and SIGHUP
/// matter as much as Ctrl-C: armed git children lead their own process
/// groups, so a closed terminal or a `kill` that ends this process without a
/// cancellation would leave them running.
#[cfg(unix)]
async fn interrupted() -> &'static str {
    use tokio::signal::unix::{signal, SignalKind};
    let (Ok(mut term), Ok(mut hup)) = (
        signal(SignalKind::terminate()),
        signal(SignalKind::hangup()),
    ) else {
        // No handler could be installed: Ctrl-C alone still cancels.
        let _ = tokio::signal::ctrl_c().await;
        return "interrupted";
    };
    tokio::select! {
        _ = tokio::signal::ctrl_c() => "interrupted",
        _ = term.recv() => "terminated (SIGTERM)",
        _ = hup.recv() => "hung up (SIGHUP)",
    }
}

/// Resolves on Ctrl-C (Windows has no SIGTERM/SIGHUP to catch).
#[cfg(not(unix))]
async fn interrupted() -> &'static str {
    let _ = tokio::signal::ctrl_c().await;
    "interrupted"
}

/// Prints the run's log to stdout and its verdict as a summary.
struct Terminal;

struct Stdout;

impl LogSink for Stdout {
    fn push(&self, line: &str) {
        out!("{line}");
    }
}

impl Reporter for Terminal {
    fn sink(&self) -> Arc<dyn LogSink> {
        Arc::new(Stdout)
    }

    fn report<'a>(self: Box<Self>, verdict: Verdict<'a>) -> BoxFuture<'a, ()>
    where
        Self: 'a,
    {
        Box::pin(async move {
            out!();
            out!("── qontinui-ci: {} ──", verdict.conclusion.as_str());
            for step in verdict.steps {
                out!(
                    "  {:<9} {:>6}s  {}",
                    step.conclusion,
                    step.duration_secs,
                    step.name
                );
            }
            if let Some(reason) = verdict.reason {
                out!("  reason: {reason}");
            }
            if let Some(canonical) = verdict.canonical {
                out!("  {}", canonical.summary_line());
            }
            match verdict.test_results {
                Some(artifact) => out!(
                    "  test report captured ({} bytes, {})",
                    artifact.raw.len(),
                    artifact.format
                ),
                None => out!("  no test report captured"),
            }
        })
    }
}
