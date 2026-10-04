//! `qontinui-ci` — run a repo's CI from any shell, with no coord at all.
//!
//! ```text
//! qontinui-ci run      [--job <name>] [--repo-dir <dir>] [--manifest <path>] [--root <dir>]
//! qontinui-ci list     [--manifest <file>]
//! qontinui-ci validate [--manifest <file>]
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
//! Exit status: 0 success, 1 failure, 3 cancelled (a non-verdict: the job did
//! not run on its merits — wrong OS, a checkout that never produced a tree),
//! 2 usage or setup error.

use std::path::{Path, PathBuf};
use std::process::ExitCode;
use std::sync::Arc;

use qontinui_ci_exec::dispatch::DispatchPayload;
use qontinui_ci_exec::host::BoxFuture;
use qontinui_ci_exec::manifest::{self, CiManifest};
use qontinui_ci_exec::report::{Conclusion, LogSink, Reporter, Verdict};
use qontinui_ci_exec::{executor, host_sizing, standalone};

const DEFAULT_MANIFEST: &str = ".qontinui/ci.toml";

const USAGE: &str = "\
usage:
  qontinui-ci run      [--job <name>] [--repo-dir <dir>] [--manifest <path>] [--root <dir>]
  qontinui-ci list     [--manifest <file>]
  qontinui-ci validate [--manifest <file>]

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
validate  parse and validate the manifest; exit 0 when it is valid";

fn main() -> ExitCode {
    tracing_subscriber::fmt()
        .with_writer(std::io::stderr)
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_env("QONTINUI_CI_LOG")
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("warn")),
        )
        .init();
    let args: Vec<String> = std::env::args().skip(1).collect();
    match dispatch(&args) {
        Ok(code) => code,
        Err(e) => {
            eprintln!("qontinui-ci: {e}");
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
        "-h" | "--help" | "help" => {
            println!("{USAGE}");
            Ok(ExitCode::SUCCESS)
        }
        other => Err(format!("unknown subcommand {other:?}\n\n{USAGE}")),
    }
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
            println!(
                "ok: {} — schema v{}, {} job(s): {}",
                path.display(),
                m.schema_version,
                m.jobs.len(),
                names.join(", ")
            );
            Ok(ExitCode::SUCCESS)
        }
        Err(e) => {
            eprintln!("invalid: {e}");
            Ok(ExitCode::from(1))
        }
    }
}

fn list(flags: &[(String, String)]) -> Result<ExitCode, String> {
    let path = PathBuf::from(flag(flags, "--manifest").unwrap_or(DEFAULT_MANIFEST));
    let m = read_manifest(&path)?;
    println!("{} (schema v{})", path.display(), m.schema_version);
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
        println!("{line}");
    }
    println!(
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
        eprintln!(
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
            eprintln!("qontinui-ci: {signal} — cancelling the job and cleaning up");
            on_signal.cancel();
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
        println!("{line}");
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
            println!();
            println!("── qontinui-ci: {} ──", verdict.conclusion.as_str());
            for step in verdict.steps {
                println!(
                    "  {:<9} {:>6}s  {}",
                    step.conclusion, step.duration_secs, step.name
                );
            }
            if let Some(reason) = verdict.reason {
                println!("  reason: {reason}");
            }
            if let Some(canonical) = verdict.canonical {
                println!("  {}", canonical.summary_line());
            }
            match verdict.test_results {
                Some(artifact) => println!(
                    "  test report captured ({} bytes, {})",
                    artifact.raw.len(),
                    artifact.format
                ),
                None => println!("  no test report captured"),
            }
        })
    }
}
