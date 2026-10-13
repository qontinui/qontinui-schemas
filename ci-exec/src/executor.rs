//! One job end-to-end: checkout → manifest → job selection → canonical gate →
//! provisioning → sequential steps (first failure short-circuits) → report →
//! cleanup.
//!
//! Resource posture (plan §4.6 — load-bearing, not polish):
//! - children run at `BELOW_NORMAL_PRIORITY_CLASS` combined with
//!   `CREATE_NO_WINDOW` on Windows, and are adopted into the host's
//!   [`StepContainment`] for the dispatch — on the runner a per-dispatch Job
//!   Object carrying a job-wide committed-memory limit (the OOM backstop the
//!   plan calls load-bearing) plus kill-on-close, so dropping it at dispatch
//!   end reaps strays;
//! - `CARGO_BUILD_JOBS` **and** the test-concurrency caps are exported by the
//!   executor, sized from the host and bounded by the manifest's `[limits]`
//!   (see [`crate::host_sizing`]) — the manifest cannot raise them, and it no
//!   longer has to smuggle them through argv;
//! - cargo artifacts go to a persistent per-repo CI target dir
//!   (`<root>/.ci-target/<repo basename>`) — warm across dispatches, never
//!   contending with the developer's own `target/`.
//!
//! Provisioning happens between the manifest read and the first step, in this
//! order: tools ([`crate::tools`]), siblings ([`crate::sibling`]), then
//! services ([`crate::services`]). All three are declared IN the manifest, so
//! none can run before it is read and validated; all three abort the dispatch
//! on failure, because a step that silently ran without its declared tool,
//! sibling or database produces a verdict that looks like a code failure and is
//! not. Services come last because they are the only stage that leaves
//! something RUNNING: there is no reason to start a database before finding out
//! a declared tool does not exist for this platform.
//!
//! ## Capture-before-cleanup (type-enforced, not conventional)
//!
//! The steps' JUnit report lives INSIDE the dispatch worktree that
//! [`crate::checkout::cleanup_dispatch`] deletes, and it is coord's Tier-7
//! credibility-gate input. "Clean up, then report" therefore destroys the
//! artifact before anything can send it — which is precisely what happened
//! before this seam existed, and it was invisible because the dispatch itself
//! still went green.
//!
//! So the ordering is expressed in the types rather than in the statement
//! order of one function:
//!
//! 1. [`crate::junit::capture`] returns a value;
//! 2. [`report::report`] files a [`Verdict`] that carries that value, and is
//!    the only minter of [`ResultReported`];
//! 3. [`DispatchWorkspace::cleanup`] CONSUMES a `ResultReported` and is the
//!    only caller of `cleanup_dispatch`.
//!
//! There is no way to reach cleanup without having gone through reporting, and
//! no way to report without having been handed the capture slot. Re-breaking
//! this requires deleting a type, not reordering two lines.

use std::path::{Path, PathBuf};
use std::process::Stdio;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio_util::sync::CancellationToken;
use tracing::{info, warn};

use crate::dispatch::DispatchPayload;
use crate::host::{Host, ProcessSpawn, StepContainment};
use crate::host_sizing;
use crate::manifest::{CiJob, CiManifest, CiStep, Os};
use crate::model::access::{section_before_checks, ModelAccess};
use crate::model::ledger::ModelSection;
use crate::report::{self, Conclusion, LogSink, Reporter, ResultReported, StepSummary, Verdict};
use crate::services::ServiceStack;

/// Everything one dispatch brought into existence — the directory tree AND the
/// service containers — and the ONLY thing that may remove them.
///
/// [`cleanup`](DispatchWorkspace::cleanup) consumes a [`ResultReported`]
/// receipt, which only [`report::report`] can mint. That makes "reported, then
/// cleaned up" the only expressible order — see the module docs for why the
/// reverse order is a silent, green-looking data-loss bug.
///
/// The service stack lives here, rather than beside the provisioning call,
/// precisely so that it rides that single exit path: every early return in
/// [`run_dispatch`] after the checkout goes through `cleanup`, so adding a
/// second thing to remove could not miss one. A leaked container is worse than
/// a failed build.
struct DispatchWorkspace {
    root: PathBuf,
    repo: String,
    dispatch_id: String,
    process: Arc<dyn ProcessSpawn>,
    /// Containers started for this dispatch. Empty (and inert) unless the
    /// job declared services.
    services: ServiceStack,
}

impl DispatchWorkspace {
    fn new(root: &Path, repo: &str, dispatch_id: &str, process: Arc<dyn ProcessSpawn>) -> Self {
        Self {
            root: root.to_path_buf(),
            repo: repo.to_string(),
            dispatch_id: dispatch_id.to_string(),
            services: ServiceStack::new(dispatch_id, process.clone()),
            process,
        }
    }

    /// Destroy the dispatch's service containers, then its worktree and
    /// dispatch root. Consumes both `self` and the reporting receipt, so it can
    /// run at most once per dispatch and never before the result (with its
    /// JUnit artifact) has been filed.
    ///
    /// Containers first: they hold no state the report needs, and taking them
    /// down before a slow filesystem delete shortens the window in which a
    /// crash could leak one.
    async fn cleanup(mut self, _reported: ResultReported) {
        self.services.teardown().await;
        crate::checkout::cleanup_dispatch(
            self.process.as_ref(),
            &self.root,
            &self.repo,
            &self.dispatch_id,
        )
        .await;
    }
}

/// Outcome of one step.
#[derive(Debug, PartialEq, Eq)]
enum StepOutcome {
    Success,
    /// Non-zero exit (carries the human-readable detail already logged).
    Failure,
    Timeout,
    Cancelled,
}

impl StepOutcome {
    fn conclusion(&self) -> Conclusion {
        match self {
            StepOutcome::Success => Conclusion::Success,
            // A timeout is a failure with a log line explaining why.
            StepOutcome::Failure | StepOutcome::Timeout => Conclusion::Failure,
            StepOutcome::Cancelled => Conclusion::Cancelled,
        }
    }
}

fn setup_row(name: &str, conclusion: Conclusion, started: Instant) -> StepSummary {
    StepSummary {
        name: format!("[setup] {name}"),
        conclusion: conclusion.as_str().to_string(),
        duration_secs: started.elapsed().as_secs(),
    }
}

/// Run one job of one dispatch end-to-end. Never panics; files exactly one
/// verdict through `reporter` — with ONE exception: a dispatch whose id fails
/// [`crate::dispatch::dispatch_id_is_safe`] files nothing (the verdict would be
/// addressed by that id) and returns `Cancelled` after a `tracing` warning,
/// without touching the reporter or its sink. Always cleans up the worktree it
/// created. Returns the conclusion it reached.
///
/// A host must therefore gate `dispatch_id_is_safe` BEFORE it builds a
/// reporter addressed by the id (the runner's admission does): this function
/// never pushes a line into such a reporter, but constructing one may already
/// have bound the unsafe id into a URL.
///
/// `capacity` and `max_concurrent` are the host probe and the number of
/// dispatches the host runs side by side (its admission's N, or 1 for a lone
/// CLI run) — the host share below is sized by that same N.
pub async fn run_dispatch(
    host: &Host,
    reporter: Box<dyn Reporter>,
    payload: DispatchPayload,
    cancel: CancellationToken,
    capacity: host_sizing::HostCapacity,
    max_concurrent: u32,
) -> Conclusion {
    // ── Identifiers (before either touches a path or a URL) ──
    //
    // The dispatch id and the repo slug are joined into every path this run
    // writes (`.ci-worktrees/<id>/<repo>`, `.ci-target/<repo>`). A host's
    // admission may already have gated them, but the executor relies on no
    // admission — so an unsafe one is refused here, with nothing written.
    //
    // An unsafe dispatch_id cannot even be REPORTED: a reporter addresses the
    // verdict AND its progress lines by it (coord's
    // `/coord/ci/dispatches/{id}/progress|result`). So it is logged locally and
    // dropped with no verdict — exactly the runner admission's rule — and the
    // reporter is dropped without its sink ever being taken.
    if !crate::dispatch::dispatch_id_is_safe(&payload.dispatch_id) {
        warn!(
            "ci: dropping dispatch with unsafe dispatch_id (len {}) — not reportable",
            payload.dispatch_id.len()
        );
        return Conclusion::Cancelled;
    }
    let sink: Arc<dyn LogSink> = reporter.sink();
    let mut steps_summary: Vec<StepSummary> = Vec::new();
    if !crate::dispatch::repo_slug_is_safe(&payload.repo) {
        sink.push(&format!(
            "[ci-node] refusing dispatch: unsafe repo slug {:?}",
            payload.repo
        ));
        return refuse_before_checkout(
            reporter,
            sink,
            &mut steps_summary,
            Instant::now(),
            crate::checkout::UNSAFE_IDENTIFIER_REASON,
        )
        .await;
    }

    sink.push(&format!(
        "[ci-node] dispatch {} job={} repo={} sha={} check={} on {}",
        payload.dispatch_id,
        payload.job_name(),
        payload.repo,
        payload.head_sha,
        payload.check_name,
        host.identity.label()
    ));

    let started = Instant::now();

    // ── Host (before anything touches disk) ──
    let host_started = Instant::now();
    let root = match host.identity.ci_root() {
        Ok(root) => root,
        Err(e) => {
            sink.push(&format!("[ci-node] no CI root on this host: {e}"));
            return refuse_before_checkout(
                reporter,
                sink,
                &mut steps_summary,
                host_started,
                "ci_root_unresolvable",
            )
            .await;
        }
    };
    // The host's admission may already have checked this, but the executor
    // does not rely on any admission: a CLI run has none, and a queued
    // dispatch can outlive a volume that was there when it was admitted.
    if let Some(reason) = host.volume.refusal_reason(&root) {
        sink.push(&format!(
            "[ci-node] refusing to write under {}: {reason}",
            root.display()
        ));
        return refuse_before_checkout(
            reporter,
            sink,
            &mut steps_summary,
            host_started,
            "ci_root_volume_unavailable",
        )
        .await;
    }

    // Everything on disk this dispatch owns. Only `workspace.cleanup(receipt)`
    // may remove it, and a receipt only exists after `report::report` — so no
    // path out of this function can delete the JUnit before it is sent.
    let mut workspace = DispatchWorkspace::new(
        &root,
        &payload.repo,
        &payload.dispatch_id,
        host.process.clone(),
    );

    // ── Checkout ──
    sink.push(&format!(
        "[ci-node] fetching {} ({})",
        payload.candidate_ref, payload.fetch_url
    ));
    let checkout_started = Instant::now();
    let worktree = match crate::checkout::prepare_worktree(
        host.process.as_ref(),
        &root,
        &payload.repo,
        &payload.dispatch_id,
        &payload.fetch_url,
        &payload.candidate_ref,
        &payload.head_sha,
        &cancel,
        // One line per retry: it tells the operator why the checkout is
        // waiting, and the progress POST it rides is what renews the lease.
        &mut |line: &str| sink.push(line),
    )
    .await
    {
        Ok(p) => p,
        Err(e) => {
            // A checkout that never produced a tree says nothing about the
            // code under test, so it is a non-verdict — `cancelled` — never a
            // `failure`, which would poison shadow parity with a red the code
            // did not earn. The reason says which: `head_sha_unavailable`
            // (the mirror lacks the commit), `fetch_failed` (the fetch failed
            // otherwise), `checkout_deadline` (the deadline ran out in a local
            // step), `unsafe_identifier` (unreachable here: both identifier
            // gates above run first), or none (the dispatch was cancelled). Only a genuine
            // setup fault (`CheckoutError::Failed`) reports `failure`.
            let (conclusion, reason) = e.result_disposition();
            sink.push(&format!("{} {e}", e.log_prefix()));
            steps_summary.push(setup_row("checkout", conclusion, checkout_started));
            // No artifact: the checkout never completed, so no step ever ran.
            // The canonical gate has not run yet — the manifest is not even
            // parsed — so this reports "not evaluated", which is a different
            // statement from "not required".
            let reported = report::report(
                reporter,
                sink,
                Verdict {
                    conclusion,
                    steps: &steps_summary,
                    reason,
                    test_results: None,
                    run_outcomes: None,
                    canonical: None,
                    model: &ModelSection::NotEvaluated,
                    model_ledger: None,
                },
            )
            .await;
            // prepare_worktree may have left a partial worktree behind.
            workspace.cleanup(reported).await;
            return conclusion;
        }
    };
    steps_summary.push(setup_row("checkout", Conclusion::Success, checkout_started));

    // ── Manifest (from the CHECKED-OUT tree, never coord) and the job ──
    let manifest_started = Instant::now();
    let selected = load_manifest(&worktree, &payload.manifest_path)
        .map_err(|e| (e, false))
        .and_then(|m| {
            match job_index(&m, payload.job_name()) {
                Ok(i) => Ok((m, i)),
                // The dispatch named no job, so it asked for the default — and
                // this manifest does not declare one. Nothing about the code was
                // observed: a non-verdict, never a red. (Every coord dispatch
                // omits `job` today, so without this a v2 manifest with no `ci`
                // job would turn every merge candidate red.)
                Err(e) if payload.job.is_none() => Err((e, true)),
                Err(e) => Err((e, false)),
            }
        });
    let (manifest, job_at) = match selected {
        Ok(selected) => selected,
        Err((e, not_declared)) => {
            let (conclusion, reason) = if not_declared {
                sink.push(&format!(
                    "[ci-node] the dispatch named no job and the manifest declares no default: {e}"
                ));
                (Conclusion::Cancelled, Some("job_not_declared"))
            } else {
                sink.push(&format!("[ci-node] manifest rejected: {e}"));
                (Conclusion::Failure, None)
            };
            steps_summary.push(setup_row("manifest", conclusion, manifest_started));
            // No artifact: no step ever ran. Canonical: not evaluated.
            let reported = report::report(
                reporter,
                sink,
                Verdict {
                    conclusion,
                    steps: &steps_summary,
                    reason,
                    test_results: None,
                    run_outcomes: None,
                    canonical: None,
                    model: &ModelSection::NotEvaluated,
                    model_ledger: None,
                },
            )
            .await;
            workspace.cleanup(reported).await;
            return conclusion;
        }
    };
    let job = &manifest.jobs[job_at];
    // A job pinned to an OS this host is not: a non-verdict about the code
    // (whoever sent it to this host chose the wrong host), never a red.
    if !job.runs_on(Os::current()) {
        sink.push(&format!(
            "[ci-node] job '{}' runs on {:?}; this host is {} — not running it here",
            job.name,
            job.os.iter().map(|o| o.as_str()).collect::<Vec<_>>(),
            std::env::consts::OS
        ));
        steps_summary.push(setup_row(
            "manifest",
            Conclusion::Cancelled,
            manifest_started,
        ));
        let reported = report::report(
            reporter,
            sink,
            Verdict {
                conclusion: Conclusion::Cancelled,
                steps: &steps_summary,
                reason: Some("job_os_mismatch"),
                test_results: None,
                run_outcomes: None,
                canonical: None,
                model: &section_before_checks(Some(job)),
                model_ledger: None,
            },
        )
        .await;
        workspace.cleanup(reported).await;
        return Conclusion::Cancelled;
    }

    // Host-derived caps, bounded by the job's limits (which are ceilings,
    // never raises). The host was probed ONCE, at admission — total memory and
    // cpu count cannot change mid-build in any way worth re-reading.
    //
    // Sized against this dispatch's SHARE of the host, not the whole of it: the
    // host runs up to N dispatches side by side, and N dispatches each sized to
    // the whole host oversubscribe it N-fold — see `host_sizing::share`.
    //
    // THE TRADE-OFF, stated so it is a choice and not a surprise: the partition
    // is STATIC. A lone build on a big host still gets only 1/N of it — on
    // 48c/368 GiB at N=12 that is 4 cargo jobs, not 48. Sizing by the LIVE
    // running count instead would be unsafe: a build sized while it ran alone
    // keeps its fat share, and every dispatch admitted after it would then
    // oversubscribe the host — exactly the OOM class this exists to prevent.
    // The operator's lever for fewer, fatter builds is a LOWER explicit
    // concurrency, which widens each share.
    let concurrency = max_concurrent.max(1);
    let sizing = host_sizing::derive(host_sizing::share(capacity, concurrency));
    let limits = manifest.limits_for(job);
    let build_jobs = limits.effective_cargo_build_jobs(sizing);
    let test_threads = limits.effective_test_threads(sizing);
    sink.push(&format!(
        "[ci-node] manifest ok (schema v{}): job '{}' — {} step(s), {} sibling(s), {} tool(s), \
         {} service(s); cargo_build_jobs={build_jobs} test_threads={test_threads} \
         (host sizing: {} / {}, a 1/{concurrency} share of the host)",
        manifest.schema_version,
        job.name,
        job.steps.len(),
        manifest.siblings.len(),
        manifest.tools.len(),
        job.services.len(),
        sizing.cargo_build_jobs,
        sizing.test_threads
    ));

    // ── Canonical-configuration gate ──
    //
    // FIRST, and before anything is downloaded. Two reasons, in order of which
    // one actually bites. A dispatch that will be refused for being off
    // canonical must not first pay to install a node tree into the tool cache;
    // and a convergence that IS authorised can rewrite the very toolchain the
    // later steps compile with, so running it after provisioning would mean
    // provisioning against a toolchain that is about to change underneath it.
    let canonical_started = Instant::now();
    let canonical_outcome = {
        let mut log = |line: String| sink.push(&line);
        // Read per dispatch, like every other consent value — the grant
        // governs the NEXT dispatch, never one already running.
        let authorized = host.settings.canonical_converge();
        crate::canonical::ensure(
            manifest.canonical.as_ref(),
            authorized,
            host.canonical.as_ref(),
            &mut log,
        )
        .await
    };
    sink.push(&canonical_outcome.summary_line());
    if canonical_outcome.is_refusal() {
        steps_summary.push(setup_row(
            "canonical",
            Conclusion::Failure,
            canonical_started,
        ));
        // No artifact, for the same reason a provisioning failure files none:
        // this aborts before the first step, so any JUnit under the warm
        // `.ci-target` cache belongs to a PREVIOUS dispatch.
        let reported = report::report(
            reporter,
            sink,
            Verdict {
                conclusion: Conclusion::Failure,
                steps: &steps_summary,
                reason: canonical_outcome.reason(),
                test_results: None,
                run_outcomes: None,
                canonical: Some(&canonical_outcome),
                model: &section_before_checks(Some(job)),
                model_ledger: None,
            },
        )
        .await;
        workspace.cleanup(reported).await;
        return Conclusion::Failure;
    }
    if manifest.canonical.is_some() {
        steps_summary.push(setup_row(
            "canonical",
            Conclusion::Success,
            canonical_started,
        ));
    }

    // ── Provisioning (tools, siblings, then services) ──
    let provision_started = Instant::now();
    let provisioning = provision(
        host,
        &payload,
        &root,
        &worktree,
        &manifest,
        job,
        sink.as_ref(),
        &cancel,
        &mut workspace.services,
    )
    .await;
    let (tool_dirs, service_env) = match provisioning {
        Ok(provisioned) => {
            steps_summary.push(setup_row(
                "provision",
                Conclusion::Success,
                provision_started,
            ));
            provisioned
        }
        Err(e) => {
            sink.push(&format!("[ci-node] provisioning failed: {e}"));
            steps_summary.push(setup_row(
                "provision",
                Conclusion::Failure,
                provision_started,
            ));
            // No artifact: provisioning aborts BEFORE the first step, so any
            // JUnit under the warm `.ci-target` cache belongs to a PREVIOUS
            // dispatch and must never be attributed to this head.
            let reported = report::report(
                reporter,
                sink,
                Verdict {
                    conclusion: Conclusion::Failure,
                    steps: &steps_summary,
                    reason: None,
                    test_results: None,
                    run_outcomes: None,
                    canonical: Some(&canonical_outcome),
                    model: &section_before_checks(Some(job)),
                    model_ledger: None,
                },
            )
            .await;
            workspace.cleanup(reported).await;
            return Conclusion::Failure;
        }
    };

    // Persistent per-repo CI target dir (warm across dispatches).
    let ci_target_dir = root
        .join(".ci-target")
        .join(crate::local_repo_name(&payload.repo));

    let dispatch_root = crate::checkout::ci_dispatch_root(&root, &payload.dispatch_id);

    // ── The run-outcomes channel (created empty, before the first step) ──
    //
    // Outside the worktree, inside the dispatch's cleanup unit. A creation
    // failure is logged and nothing is exported: the outcome channel is
    // evidence about the run, never a reason to change its verdict.
    let outcomes_path = crate::outcomes::prepare(&dispatch_root);
    if let Err(e) = &outcomes_path {
        sink.push(&format!(
            "[ci-node] run outcomes: could not create the output file ({e}); ${} is not exported",
            crate::outcomes::RUN_OUTCOMES_ENV
        ));
    }

    let dispatch_env = DispatchEnv::build(
        build_jobs,
        test_threads,
        &ci_target_dir,
        &tool_dirs,
        &service_env,
        outcomes_path.as_deref().ok(),
    );

    // ── Model access (after provisioning, before the first step) ──
    //
    // Decided once; from here on there is ONE exit path, and it stops the
    // broker (`model_access.finish`) before the capture, the report and the
    // cleanup. See `crate::model::access` for the ordered refusal checks.
    let model_access = crate::model::access::prepare(
        host,
        payload.model_access.as_ref(),
        manifest.model.as_ref(),
        job,
        sink.clone(),
    )
    .await;

    // Per-dispatch containment (held across all steps; dropping it at
    // dispatch end reaps any stray build processes).
    let containment = host.process.step_containment();

    // ── Steps (sequential; first failure short-circuits) ──
    let mut overall = StepOutcome::Success;
    for step in &job.steps {
        if cancel.is_cancelled() {
            overall = StepOutcome::Cancelled;
            break;
        }
        sink.push(&format!(
            "[ci-node] ── step {} ── {:?} (timeout {}s)",
            step.name,
            step.command,
            step.effective_timeout_secs()
        ));
        let step_started = Instant::now();
        model_access.set_step(&step.name);
        let outcome =
            match step_model_env(&model_access, step, &payload.dispatch_id, &dispatch_root) {
                Ok(model_env) => {
                    run_step(
                        host.process.as_ref(),
                        step,
                        &dispatch_env,
                        &model_env,
                        &worktree,
                        containment.as_ref(),
                        &sink,
                        &cancel,
                    )
                    .await
                }
                Err(e) => {
                    sink.push(&format!("[ci-node] step {}: {e}", step.name));
                    StepOutcome::Failure
                }
            };
        let duration = step_started.elapsed().as_secs();
        sink.push(&format!(
            "[ci-node] step {} → {} in {duration}s",
            step.name,
            outcome.conclusion().as_str()
        ));
        steps_summary.push(StepSummary {
            name: step.name.clone(),
            conclusion: outcome.conclusion().as_str().to_string(),
            duration_secs: duration,
        });
        if outcome != StepOutcome::Success {
            overall = outcome;
            break;
        }
    }
    drop(containment);
    // The broker stops BEFORE capture, report and cleanup: no call can be in
    // flight while the worktree goes, and the ledger it hands back rides the
    // verdict.
    let (model_section, model_ledger) = model_access.finish(sink.as_ref()).await;

    let conclusion = overall.conclusion();
    sink.push(&format!(
        "[ci-node] dispatch {} job {} finished: {} in {}s",
        payload.dispatch_id,
        job.name,
        conclusion.as_str(),
        started.elapsed().as_secs()
    ));
    info!(
        "ci: dispatch {} job {} finished conclusion={}",
        payload.dispatch_id,
        job.name,
        conclusion.as_str()
    );

    // ── Capture the JUnit BEFORE the worktree can be removed ──
    //
    // Ordered here, ahead of the report, so the capture outcome reaches the
    // reporter in this dispatch's own log stream AND its log tail.
    // Unconditional on `conclusion`: a RED suite's report is exactly the
    // evidence the credibility gate needs, and a cancelled/timed-out step may
    // still have written a partial one worth ingesting.
    let capture = crate::junit::capture(&worktree, &ci_target_dir, &job.steps);
    sink.push(&capture.log_line());
    // The run outcomes live inside the same cleanup unit, so they are captured
    // here too, and for the same reason unconditionally: a red run's numbers
    // are a real observation (coord itself refuses a `cancelled` run's).
    let outcomes = crate::outcomes::capture(outcomes_path.as_deref().map_err(String::as_str));
    sink.push(&outcomes.log_line());

    let reported = report::report(
        reporter,
        sink,
        Verdict {
            conclusion,
            steps: &steps_summary,
            reason: None,
            test_results: capture.artifact(),
            run_outcomes: outcomes.entries(),
            canonical: Some(&canonical_outcome),
            model: &model_section,
            model_ledger: model_ledger.as_ref(),
        },
    )
    .await;
    // The receipt is the only key to cleanup; the artifact is already sent.
    workspace.cleanup(reported).await;
    conclusion
}

/// File a `cancelled` verdict for a run the host could not even start (no CI
/// root, or a root on a volume that is not there). Nothing exists on disk yet,
/// so there is nothing to clean up.
async fn refuse_before_checkout(
    reporter: Box<dyn Reporter>,
    sink: Arc<dyn LogSink>,
    steps_summary: &mut Vec<StepSummary>,
    started: Instant,
    reason: &str,
) -> Conclusion {
    steps_summary.push(setup_row("host", Conclusion::Cancelled, started));
    let _reported = report::report(
        reporter,
        sink,
        Verdict {
            conclusion: Conclusion::Cancelled,
            steps: steps_summary,
            reason: Some(reason),
            test_results: None,
            run_outcomes: None,
            canonical: None,
            model: &ModelSection::NotEvaluated,
            model_ledger: None,
        },
    )
    .await;
    Conclusion::Cancelled
}

/// The index of the job a dispatch named. An absent job is the manifest's
/// problem (the dispatch named a job this commit does not declare), so it is
/// reported like any other manifest rejection, naming the jobs that exist.
fn job_index(manifest: &CiManifest, name: &str) -> Result<usize, String> {
    manifest
        .jobs
        .iter()
        .position(|j| j.name == name)
        .ok_or_else(|| {
            let names: Vec<&str> = manifest.jobs.iter().map(|j| j.name.as_str()).collect();
            format!("the manifest declares no job '{name}' (jobs: {names:?})")
        })
}

/// Provision what the manifest and the job declare. Returns the tool
/// directories to prepend to the step PATH, and the service connection env to
/// export to every step.
///
/// Tools first, siblings second, services last. A sibling fetch is slower and
/// more failure-prone than a tool install (network, declaration validation),
/// and there is no reason to pay for it before finding out a declared tool does
/// not exist for this platform — and less reason still to have a database
/// RUNNING while finding that out. Services are also the only stage whose
/// failure can leave something behind, which is why the stack is owned by the
/// caller's `DispatchWorkspace` and passed in: a partially-provisioned stack
/// still names every container it started, and the caller's cleanup removes
/// them.
#[allow(clippy::too_many_arguments)]
async fn provision(
    host: &Host,
    payload: &DispatchPayload,
    root: &Path,
    worktree: &Path,
    manifest: &CiManifest,
    job: &CiJob,
    sink: &dyn LogSink,
    cancel: &CancellationToken,
    services: &mut ServiceStack,
) -> Result<(Vec<PathBuf>, Vec<(String, String)>), String> {
    let mut log = |line: String| sink.push(&line);

    let tool_dirs =
        crate::tools::provision(host.process.as_ref(), root, &manifest.tools, &mut log).await?;

    let dispatch_root = crate::checkout::ci_dispatch_root(root, &payload.dispatch_id);
    crate::sibling::provision(
        host.github.as_ref(),
        host.process.as_ref(),
        host.sibling_reads.as_ref(),
        &payload.dispatch_id,
        &dispatch_root,
        worktree,
        &manifest.siblings,
        &payload.repo,
        payload.pr_number,
        &mut log,
    )
    .await?;

    let service_env = services.provision(&job.services, &mut log, cancel).await?;

    Ok((tool_dirs, service_env))
}

/// Read + validate the manifest from the checked-out tree. The path is
/// repo-relative and structurally constrained (no traversal), then
/// canonicalize+prefix-checked against the worktree.
fn load_manifest(worktree: &Path, manifest_path: &str) -> Result<CiManifest, String> {
    let rel = Path::new(manifest_path);
    if rel.is_absolute()
        || manifest_path.contains(':')
        || rel.components().any(|c| {
            !matches!(
                c,
                std::path::Component::Normal(_) | std::path::Component::CurDir
            )
        })
    {
        return Err(format!(
            "manifest_path {manifest_path:?} must be repo-relative"
        ));
    }
    let full = worktree.join(rel);
    let canonical = full.canonicalize().map_err(|e| {
        format!("manifest {manifest_path:?} not readable in the checked-out tree: {e}")
    })?;
    let canonical_wt = worktree
        .canonicalize()
        .map_err(|e| format!("worktree not canonicalizable: {e}"))?;
    if !canonical.starts_with(&canonical_wt) {
        return Err(format!(
            "manifest_path {manifest_path:?} escapes the worktree"
        ));
    }
    let text = std::fs::read_to_string(&canonical)
        .map_err(|e| format!("read {}: {e}", canonical.display()))?;
    crate::manifest::parse_and_validate(&text)
}

/// The model env one step gets on top of the dispatch's exports: the dispatch
/// id, the model variables (or the refusal) for a `model = true` step, and
/// `CLAUDE_CONFIG_DIR` pointed at a freshly emptied per-dispatch directory —
/// for EVERY step, so an un-brokered `claude` inside a step finds no login
/// (D11). `Err` when no empty directory could be guaranteed: the step fails
/// rather than run pointed at something that is not empty.
fn step_model_env(
    access: &ModelAccess,
    step: &CiStep,
    dispatch_id: &str,
    dispatch_root: &Path,
) -> Result<Vec<(String, String)>, String> {
    let mut env = access.step_env(step, dispatch_id);
    let dir = crate::model::scrub::fresh_empty_config_dir(dispatch_root)?;
    env.push((
        crate::model::CLAUDE_CONFIG_DIR_ENV.to_string(),
        dir.to_string_lossy().to_string(),
    ));
    Ok(env)
}

/// Resolve a step's working dir inside the worktree (canonicalize +
/// prefix-check — the runtime enforcement behind the manifest's structural
/// validation).
fn resolve_step_cwd(worktree: &Path, step: &CiStep) -> Result<PathBuf, String> {
    let cwd = match &step.working_dir {
        Some(wd) => worktree.join(wd),
        None => worktree.to_path_buf(),
    };
    let canonical = cwd.canonicalize().map_err(|e| {
        format!(
            "working_dir {:?} not present in the tree: {e}",
            step.working_dir
        )
    })?;
    let canonical_wt = worktree
        .canonicalize()
        .map_err(|e| format!("worktree not canonicalizable: {e}"))?;
    if !canonical.starts_with(&canonical_wt) {
        return Err(format!(
            "working_dir {:?} escapes the worktree",
            step.working_dir
        ));
    }
    Ok(canonical)
}

/// Build the tokio Command for a step's argv. On Windows the creation
/// flags combine `CREATE_NO_WINDOW` with `BELOW_NORMAL_PRIORITY_CLASS` so a
/// CI build never steals the foreground from the developer.
///
/// Built through the host's [`ProcessSpawn::command`], like every other
/// process the executor starts, so a step whose argv starts with `git` gets
/// the host's git posture: prompt-proof, and (on the runner) scrubbed of a
/// repository-local env (`GIT_DIR`, …) inherited from whoever launched the
/// host — which would otherwise point the step at the launcher's repo instead
/// of the worktree it runs in.
fn build_step_command(
    process: &dyn ProcessSpawn,
    program: &str,
    args: &[String],
) -> tokio::process::Command {
    let mut cmd = process.command(program);
    cmd.args(args);
    #[cfg(target_os = "windows")]
    {
        #[allow(unused_imports)]
        use std::os::windows::process::CommandExt;
        const CREATE_NO_WINDOW: u32 = 0x0800_0000;
        const BELOW_NORMAL_PRIORITY_CLASS: u32 = 0x0000_4000;
        cmd.creation_flags(CREATE_NO_WINDOW | BELOW_NORMAL_PRIORITY_CLASS);
    }
    cmd
}

/// Spawn a step child, falling back to a `cmd.exe /C` respawn on Windows
/// when the program is a `.cmd`/`.bat` shim (pnpm) that `CreateProcess`
/// cannot launch directly. Argv tokens are metacharacter-validated by the
/// manifest, so the shim path cannot smuggle shell syntax.
///
/// `removals` are taken out of the inherited environment FIRST (the step env
/// scrub, see [`crate::model::scrub`]); `envs` are applied after, so an
/// executor-owned value such as `CLAUDE_CONFIG_DIR` is set, not removed.
fn spawn_step_child(
    process: &dyn ProcessSpawn,
    step: &CiStep,
    cwd: &Path,
    removals: &[String],
    envs: &[(String, String)],
) -> Result<tokio::process::Child, String> {
    let apply = |cmd: &mut tokio::process::Command| {
        cmd.current_dir(cwd)
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());
        for k in removals {
            cmd.env_remove(k);
        }
        for (k, v) in envs {
            cmd.env(k, v);
        }
    };
    let mut direct = build_step_command(process, &step.command[0], &step.command[1..]);
    apply(&mut direct);
    match direct.spawn() {
        Ok(child) => Ok(child),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound && cfg!(target_os = "windows") => {
            let mut argv: Vec<String> = vec!["/C".to_string()];
            argv.extend(step.command.iter().cloned());
            let mut shim = build_step_command(process, "cmd.exe", &argv);
            apply(&mut shim);
            shim.spawn()
                .map_err(|e2| format!("spawn {:?} (direct: {e}; cmd /C: {e2})", step.command))
        }
        Err(e) => Err(format!("spawn {:?}: {e}", step.command)),
    }
}

/// Kill a step's process tree. On Windows `taskkill /T` takes the whole
/// tree (killing only the direct child would orphan cargo/node under a
/// `cmd.exe` shim until Job-Object close); elsewhere `start_kill` suffices.
#[cfg_attr(not(windows), allow(unused_variables))]
async fn kill_step_tree(process: &dyn ProcessSpawn, child: &mut tokio::process::Child) {
    #[cfg(target_os = "windows")]
    {
        if let Some(pid) = child.id() {
            let _ = process
                .command("taskkill")
                .args(["/F", "/T", "/PID", &pid.to_string()])
                .output()
                .await;
        }
    }
    let _ = child.start_kill();
    let _ = child.wait().await;
}

/// The env the executor exports to EVERY step of a dispatch, computed once.
///
/// These win over a step's own `env` — the manifest's validation rejects the
/// keys here outright with a pointer to the manifest key that does control
/// them, so "the step set it and it was ignored" is not a state this can
/// reach.
struct DispatchEnv {
    exports: Vec<(String, String)>,
}

impl DispatchEnv {
    fn build(
        build_jobs: u32,
        test_threads: u32,
        ci_target_dir: &Path,
        tool_dirs: &[PathBuf],
        service_env: &[(String, String)],
        run_outcomes_path: Option<&Path>,
    ) -> Self {
        let mut exports = vec![
            // The build-phase cap.
            ("CARGO_BUILD_JOBS".to_string(), build_jobs.to_string()),
            // The TEST-phase cap, exported for both harnesses. This is the
            // half that used to be missing: the incident behind these caps
            // killed the Actions agent in the test phase as well as the build
            // phase, so bounding only the build leaves half the failure mode
            // open. `RUST_TEST_THREADS` bounds libtest (`cargo test`);
            // `NEXTEST_TEST_THREADS` bounds nextest's process-per-test model.
            // Both are exported unconditionally because a manifest may use
            // either harness, and the unused one is inert.
            ("RUST_TEST_THREADS".to_string(), test_threads.to_string()),
            ("NEXTEST_TEST_THREADS".to_string(), test_threads.to_string()),
            // Keeps cargo out of the developer's own target/.
            (
                "CARGO_TARGET_DIR".to_string(),
                ci_target_dir.to_string_lossy().to_string(),
            ),
            // Mark the environment as CI for tools that branch on it.
            ("CI".to_string(), "true".to_string()),
        ];
        if let Some(path) = tool_path(tool_dirs) {
            // PATH is NOT settable from a manifest — it is the canonical
            // "redirect binary resolution" sink and stays off the allowlist.
            // The executor prepends only directories it provisioned itself,
            // from the closed tool registry, at versions the manifest pinned.
            // The host's own PATH is preserved after them so cargo, git and
            // node still resolve.
            exports.push(("PATH".to_string(), path));
        }
        // Service connections. Executor-owned for a reason the manifest cannot
        // work around: the host port is assigned at dispatch time and the
        // password is generated per dispatch, so a committed file could only
        // ever hold a wrong value. The manifest rejects these keys outright
        // with a pointer to the `[[services]]` entry that provides them.
        exports.extend(service_env.iter().cloned());
        // The run-outcomes output file, when the executor could create it.
        // Executor-owned (the manifest refuses the key): it is created empty
        // per dispatch outside the worktree, so a manifest could only point it
        // at a committed file.
        if let Some(path) = run_outcomes_path {
            exports.push((
                crate::outcomes::RUN_OUTCOMES_ENV.to_string(),
                path.to_string_lossy().to_string(),
            ));
        }
        Self { exports }
    }
}

/// Build the step PATH: provisioned tool directories first, the runner's own
/// PATH after. `None` when nothing was provisioned, so the child simply
/// inherits.
fn tool_path(tool_dirs: &[PathBuf]) -> Option<String> {
    if tool_dirs.is_empty() {
        return None;
    }
    let inherited = std::env::var_os("PATH").unwrap_or_default();
    let mut entries: Vec<PathBuf> = tool_dirs.to_vec();
    entries.extend(std::env::split_paths(&inherited));
    std::env::join_paths(entries)
        .ok()
        .map(|s| s.to_string_lossy().to_string())
}

/// Run one step: spawn, pump output, enforce timeout + cancellation.
#[allow(clippy::too_many_arguments)]
async fn run_step(
    process: &dyn ProcessSpawn,
    step: &CiStep,
    dispatch_env: &DispatchEnv,
    model_env: &[(String, String)],
    worktree: &Path,
    containment: &dyn StepContainment,
    sink: &Arc<dyn LogSink>,
    cancel: &CancellationToken,
) -> StepOutcome {
    let cwd = match resolve_step_cwd(worktree, step) {
        Ok(c) => c,
        Err(e) => {
            sink.push(&format!("[ci-node] step {}: {e}", step.name));
            return StepOutcome::Failure;
        }
    };

    // Env: allowlisted step env first, then the executor's exports, which WIN
    // over the step.
    let mut envs: Vec<(String, String)> = step
        .env
        .iter()
        .map(|(k, v)| (k.clone(), v.clone()))
        .collect();
    envs.extend(dispatch_env.exports.iter().cloned());
    envs.extend(model_env.iter().cloned());

    // The scrub: no step inherits a model credential, a model command or the
    // host's credential values (D11).
    let removals = crate::model::scrub::current_step_env_removals(process.scrubbed_env());
    let mut child = match spawn_step_child(process, step, &cwd, &removals, &envs) {
        Ok(c) => c,
        Err(e) => {
            sink.push(&format!("[ci-node] step {}: {e}", step.name));
            return StepOutcome::Failure;
        }
    };

    // The host's per-dispatch containment (on the runner: the global
    // kill-on-close Job Object every runner child joins, plus this dispatch's
    // memory-backstop job — plan §4.6). Children spawned by the step inherit
    // membership.
    containment.adopt(&child);

    // Pump stdout/stderr per-line into the sink.
    let out_task = child
        .stdout
        .take()
        .map(|s| tokio::spawn(pump_lines(s, sink.clone(), step.name.clone(), "out")));
    let err_task = child
        .stderr
        .take()
        .map(|s| tokio::spawn(pump_lines(s, sink.clone(), step.name.clone(), "err")));

    let timeout = Duration::from_secs(step.effective_timeout_secs());
    let outcome = tokio::select! {
        status = child.wait() => match status {
            Ok(s) if s.success() => StepOutcome::Success,
            Ok(s) => {
                sink.push(&format!("[ci-node] step {} exited {s}", step.name));
                StepOutcome::Failure
            }
            Err(e) => {
                sink.push(&format!("[ci-node] step {} wait error: {e}", step.name));
                StepOutcome::Failure
            }
        },
        _ = tokio::time::sleep(timeout) => {
            sink.push(&format!(
                "[ci-node] step {} timed out after {}s — killing process tree",
                step.name,
                timeout.as_secs()
            ));
            kill_step_tree(process, &mut child).await;
            StepOutcome::Timeout
        }
        _ = cancel.cancelled() => {
            sink.push(&format!(
                "[ci-node] step {} cancelled — killing process tree",
                step.name
            ));
            kill_step_tree(process, &mut child).await;
            StepOutcome::Cancelled
        }
    };

    // The pumps end when the pipes close (child exit or kill).
    if let Some(t) = out_task {
        let _ = t.await;
    }
    if let Some(t) = err_task {
        let _ = t.await;
    }
    outcome
}

/// Per-line pump for one stream.
async fn pump_lines<R: tokio::io::AsyncRead + Unpin + Send + 'static>(
    stream: R,
    sink: Arc<dyn LogSink>,
    step_name: String,
    stream_tag: &'static str,
) {
    let mut lines = BufReader::new(stream).lines();
    while let Ok(Some(line)) = lines.next_line().await {
        sink.push(&format!("[{step_name}:{stream_tag}] {line}"));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::standalone::PlainSpawn;

    /// A step's command is built by the HOST, so the host's git posture
    /// reaches a step whose argv starts with `git`: `PlainSpawn` sets the
    /// prompt-proof env on git and removes the repository-local env; a bare
    /// `Command::new(program)` does neither.
    #[test]
    fn a_git_step_is_built_through_the_host_spawner() {
        let cmd = build_step_command(&PlainSpawn, "git", &["status".to_string()]);
        let set: Vec<(String, Option<String>)> = cmd
            .as_std()
            .get_envs()
            .map(|(k, v)| {
                (
                    k.to_string_lossy().into_owned(),
                    v.map(|v| v.to_string_lossy().into_owned()),
                )
            })
            .collect();
        for (k, v) in crate::standalone::PROMPT_PROOF_GIT_ENV {
            assert!(
                set.iter()
                    .any(|(sk, sv)| sk == k && sv.as_deref() == Some(*v)),
                "a git CI step must carry the host's {k}={v}; env: {set:?}"
            );
        }
        for var in crate::standalone::REPO_LOCAL_GIT_ENV {
            assert!(
                set.iter().any(|(k, v)| k == var && v.is_none()),
                "a git CI step must not inherit {var}; env: {set:?}"
            );
        }
    }

    #[test]
    fn step_outcome_conclusions() {
        assert_eq!(StepOutcome::Success.conclusion(), Conclusion::Success);
        assert_eq!(StepOutcome::Failure.conclusion(), Conclusion::Failure);
        assert_eq!(StepOutcome::Timeout.conclusion(), Conclusion::Failure);
        assert_eq!(StepOutcome::Cancelled.conclusion(), Conclusion::Cancelled);
    }

    fn exports(env: &DispatchEnv, key: &str) -> Option<String> {
        env.exports
            .iter()
            .find(|(k, _)| k == key)
            .map(|(_, v)| v.clone())
    }

    /// The caps the executor owns are exported for BOTH test harnesses, not
    /// just the build — the failure mode these exist for killed the Actions
    /// agent in the test phase too.
    #[test]
    fn executor_exports_both_phases_of_cap() {
        let env = DispatchEnv::build(3, 7, Path::new("/ci-target"), &[], &[], None);
        assert_eq!(exports(&env, "CARGO_BUILD_JOBS").as_deref(), Some("3"));
        assert_eq!(exports(&env, "RUST_TEST_THREADS").as_deref(), Some("7"));
        assert_eq!(exports(&env, "NEXTEST_TEST_THREADS").as_deref(), Some("7"));
        assert_eq!(exports(&env, "CI").as_deref(), Some("true"));
        assert!(exports(&env, "CARGO_TARGET_DIR").is_some());
        // Nothing provisioned ⇒ the child inherits PATH untouched.
        assert!(exports(&env, "PATH").is_none());
    }

    /// The run-outcomes path is exported only when the executor created the
    /// file; a creation failure exports nothing rather than a dangling path.
    #[test]
    fn run_outcomes_path_is_exported_only_when_created() {
        let bare = DispatchEnv::build(1, 2, Path::new("/ci-target"), &[], &[], None);
        assert!(exports(&bare, crate::outcomes::RUN_OUTCOMES_ENV).is_none());
        let p = Path::new("/root/.ci-worktrees/d-1/.ci-outputs/run_outcomes.jsonl");
        let env = DispatchEnv::build(1, 2, Path::new("/ci-target"), &[], &[], Some(p));
        assert_eq!(
            exports(&env, crate::outcomes::RUN_OUTCOMES_ENV).as_deref(),
            Some(p.to_string_lossy().as_ref())
        );
    }

    /// A provisioned tool goes on the FRONT of PATH, and the host's own PATH
    /// survives behind it (cargo/git/node must still resolve).
    #[test]
    fn tool_dirs_prepend_without_replacing_the_host_path() {
        let dir = PathBuf::from("/tools/cargo-nextest/0.9.98");
        let env = DispatchEnv::build(
            1,
            2,
            Path::new("/ci-target"),
            std::slice::from_ref(&dir),
            &[],
            None,
        );
        let path = exports(&env, "PATH").expect("PATH must be exported when a tool is provisioned");
        let mut entries = std::env::split_paths(&path);
        assert_eq!(entries.next().as_deref(), Some(dir.as_path()));
        let inherited: Vec<_> =
            std::env::split_paths(&std::env::var_os("PATH").unwrap_or_default()).collect();
        let rest: Vec<_> = entries.collect();
        assert_eq!(rest, inherited);
    }

    /// The executor's exports are appended AFTER the step's own env, and
    /// `spawn_step_child` applies them in order, so the executor's value is
    /// the one the child sees. (The manifest also rejects these keys
    /// outright — this asserts the second half of that belt-and-braces.)
    #[test]
    fn executor_exports_are_applied_last() {
        let env = DispatchEnv::build(1, 2, Path::new("/ci-target"), &[], &[], None);
        let step_env = vec![("CARGO_BUILD_JOBS".to_string(), "64".to_string())];
        let mut envs = step_env.clone();
        envs.extend(env.exports.iter().cloned());
        let last = envs
            .iter()
            .rfind(|(k, _)| k == "CARGO_BUILD_JOBS")
            .expect("present");
        assert_eq!(last.1, "1");
    }

    /// Capture-before-cleanup, asserted on the observable consequence rather
    /// than on statement order: cleanup really does destroy the JUnit, so if
    /// the two ever swap back the report is gone.
    ///
    /// (The compile-time half of this guarantee is `DispatchWorkspace::cleanup`
    /// consuming a `ResultReported`, which only `report::report` mints
    /// and which takes the verdict carrying the capture slot. This test covers the runtime half: that the file is genuinely
    /// inside what cleanup removes.)
    #[test]
    fn cleanup_destroys_the_junit_so_capture_must_precede_it() {
        let base = std::env::temp_dir().join(format!("ci-order-test-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&base);
        let dispatch_id = "0198f2b4-1111-7aaa-bbbb-cccccccccccc";
        let repo = "qontinui/qontinui-coord";

        let worktree = crate::checkout::ci_worktree_path(&base, dispatch_id, repo);
        let dispatch_root = crate::checkout::ci_dispatch_root(&base, dispatch_id);
        let profile = worktree.join("target").join("nextest").join("ci-pr");
        std::fs::create_dir_all(&profile).unwrap();
        std::fs::write(
            profile.join("junit.xml"),
            "<testsuites><testcase name=\"t\" classname=\"c\"/></testsuites>",
        )
        .unwrap();

        // The report is INSIDE the tree `cleanup_dispatch` removes — this is
        // the whole reason the ordering is load-bearing.
        assert!(
            profile.join("junit.xml").starts_with(&dispatch_root),
            "the JUnit must live inside the dispatch root cleanup deletes"
        );

        // Capture (the step that must come first) finds it.
        let ci_target = base.join(".ci-target").join("qontinui-coord");
        let captured = crate::junit::capture(&worktree, &ci_target, &[]);
        let artifact = captured
            .artifact()
            .expect("capture must find the report while the worktree exists");
        assert!(artifact.raw.contains("<testcase"));

        // After the dispatch root is gone, capture finds nothing — i.e. the
        // reversed order yields an empty artifact and a silently fail-closed
        // credibility tier.
        std::fs::remove_dir_all(&dispatch_root).unwrap();
        assert!(
            crate::junit::capture(&worktree, &ci_target, &[])
                .artifact()
                .is_none(),
            "post-cleanup capture must find nothing — proving order matters"
        );

        let _ = std::fs::remove_dir_all(&base);
    }

    /// Service connection env reaches the steps, and it is applied AFTER the
    /// step's own env, so the executor's value is the one the child sees.
    #[test]
    fn service_env_is_exported_to_every_step() {
        let service_env = crate::services::exports_for(
            crate::services::ServiceKind::Redis,
            6399,
            &crate::services::ServiceCreds {
                user: "u".to_string(),
                password: "p".to_string(),
                database: "d".to_string(),
            },
        );
        let env = DispatchEnv::build(1, 2, Path::new("/ci-target"), &[], &service_env, None);
        assert_eq!(
            exports(&env, "REDIS_URL").as_deref(),
            Some("redis://127.0.0.1:6399")
        );
        assert_eq!(exports(&env, "REDIS_PORT").as_deref(), Some("6399"));
        // Nothing declared ⇒ nothing exported: a manifest without services
        // must not gain a connection variable pointing nowhere.
        let bare = DispatchEnv::build(1, 2, Path::new("/ci-target"), &[], &[], None);
        assert!(exports(&bare, "REDIS_URL").is_none());
        assert!(exports(&bare, "DATABASE_URL").is_none());
    }

    /// TEARDOWN RIDES THE FAILURE PATH. `cleanup` is the single exit of
    /// `run_dispatch` — every early return (checkout failure, manifest
    /// rejection, provisioning failure) reaches it — so this asserts the two
    /// halves it must do: remove the service containers, and remove EXACTLY
    /// the one dispatch directory.
    ///
    /// The runtime named here does not exist, which is the point: teardown is
    /// best-effort, so a removal that cannot run must still empty the stack and
    /// must never abort the directory cleanup that follows it.
    #[tokio::test]
    async fn cleanup_tears_down_services_and_removes_exactly_one_directory() {
        let base = std::env::temp_dir().join(format!(
            "ci-cleanup-test-{}-{}",
            std::process::id(),
            rand::random::<u32>()
        ));
        let dispatch_id = "0198f2b4-2222-7aaa-bbbb-cccccccccccc";
        let repo = "qontinui/qontinui-coord";

        // Inside the cleanup unit: the worktree and a provisioned sibling.
        let dispatch_root = crate::checkout::ci_dispatch_root(&base, dispatch_id);
        let worktree = crate::checkout::ci_worktree_path(&base, dispatch_id, repo);
        std::fs::create_dir_all(worktree.join("src")).unwrap();
        std::fs::write(worktree.join("src").join("lib.rs"), "fn main() {}").unwrap();
        let sibling = dispatch_root.join("qontinui-schemas");
        std::fs::create_dir_all(&sibling).unwrap();
        std::fs::write(sibling.join("Cargo.toml"), "[package]").unwrap();

        // OUTSIDE it: a peer dispatch, the warm CI target cache, and the
        // primary checkout. None of these may be touched.
        let peer = crate::checkout::ci_dispatch_root(&base, "0198f2b4-3333-7aaa-bbbb-cccccccccccc");
        std::fs::create_dir_all(&peer).unwrap();
        std::fs::write(peer.join("keep.txt"), "peer").unwrap();
        let warm_target = base.join(".ci-target").join("qontinui-coord");
        std::fs::create_dir_all(&warm_target).unwrap();
        std::fs::write(warm_target.join("keep.txt"), "warm").unwrap();
        let primary = base.join("qontinui-coord");
        std::fs::create_dir_all(&primary).unwrap();
        std::fs::write(primary.join("keep.txt"), "primary").unwrap();

        let mut workspace = DispatchWorkspace::new(&base, repo, dispatch_id, Arc::new(PlainSpawn));
        workspace.services = ServiceStack::for_test(
            dispatch_id,
            Some("qontinui-no-such-container-runtime"),
            &["qontinui-ci-0198f2b4-2222-7aaa-bbbb-cccccccccccc-redis"],
        );
        // Held across the move, because `cleanup` consumes the workspace and
        // the point of this test is what cleanup DID, not what it left behind.
        let removals = workspace.services.removal_log();
        workspace.cleanup(ResultReported::for_test()).await;

        // The services half: cleanup must have issued the forced removal. Its
        // name says "tears down services", so it has to assert that — the
        // earlier version asserted only on the directory.
        assert_eq!(
            removals.lock().unwrap().clone(),
            vec![vec![
                "rm".to_string(),
                "-f".to_string(),
                "-v".to_string(),
                "qontinui-ci-0198f2b4-2222-7aaa-bbbb-cccccccccccc-redis".to_string()
            ]],
            "cleanup must tear down the dispatch's service containers"
        );

        assert!(
            !dispatch_root.exists(),
            "the dispatch root must be removed: {}",
            dispatch_root.display()
        );
        assert!(
            peer.join("keep.txt").exists(),
            "a peer dispatch was removed"
        );
        assert!(
            warm_target.join("keep.txt").exists(),
            "the warm CI target cache was removed"
        );
        assert!(
            primary.join("keep.txt").exists(),
            "the primary checkout was removed"
        );

        let _ = std::fs::remove_dir_all(&base);
    }

    /// The manifest-path structural gate (pre-canonicalize half).
    #[test]
    fn manifest_path_structural_rejections() {
        let wt = std::env::temp_dir(); // any existing dir works for the structural half
        for bad in ["../up.toml", "/abs.toml", "C:\\abs.toml", "a/../../b.toml"] {
            let err = load_manifest(&wt, bad).unwrap_err();
            assert!(
                err.contains("repo-relative") || err.contains("not readable"),
                "{bad}: got {err}"
            );
        }
    }

    /// A dispatch naming a job the manifest does not declare is refused with
    /// the jobs that DO exist.
    #[test]
    fn an_unknown_job_names_the_jobs_that_exist() {
        let m = crate::manifest::parse_and_validate(
            "version = 2\n[[jobs]]\nname = \"lint\"\n[[jobs.steps]]\nname = \"s\"\ncommand = [\"true\"]\n",
        )
        .unwrap();
        assert_eq!(job_index(&m, "lint"), Ok(0));
        let err = job_index(&m, "test").unwrap_err();
        assert!(
            err.contains("no job 'test'") && err.contains("lint"),
            "{err}"
        );
    }

    // ── End to end, through the public entry point, on a real git repo ──

    /// `(conclusion, "name=conclusion" per step, reason)` per verdict filed.
    type Filed = Arc<std::sync::Mutex<Vec<(Conclusion, Vec<String>, Option<String>)>>>;

    /// `(summary.model as JSON, the ledger as JSON)` per verdict filed.
    type Models = Arc<std::sync::Mutex<Vec<(serde_json::Value, Option<serde_json::Value>)>>>;

    /// The `run_outcomes` each verdict carried, as wire JSON (`None` = none).
    type Outcomes = Arc<std::sync::Mutex<Vec<Option<serde_json::Value>>>>;

    struct Recording {
        lines: Arc<std::sync::Mutex<Vec<String>>>,
        filed: Filed,
        models: Models,
        outcomes: Outcomes,
    }

    struct Lines(Arc<std::sync::Mutex<Vec<String>>>);
    impl LogSink for Lines {
        fn push(&self, line: &str) {
            self.0.lock().unwrap().push(line.to_string());
        }
    }

    impl Reporter for Recording {
        fn sink(&self) -> Arc<dyn LogSink> {
            Arc::new(Lines(self.lines.clone()))
        }
        fn report<'a>(self: Box<Self>, verdict: Verdict<'a>) -> crate::host::BoxFuture<'a, ()>
        where
            Self: 'a,
        {
            Box::pin(async move {
                self.filed.lock().unwrap().push((
                    verdict.conclusion,
                    verdict
                        .steps
                        .iter()
                        .map(|s| format!("{}={}", s.name, s.conclusion))
                        .collect(),
                    verdict.reason.map(str::to_string),
                ));
                self.models.lock().unwrap().push((
                    serde_json::to_value(verdict.model).unwrap(),
                    verdict
                        .model_ledger
                        .map(|l| serde_json::to_value(l).unwrap()),
                ));
                self.outcomes.lock().unwrap().push(
                    verdict
                        .run_outcomes
                        .map(|e| serde_json::to_value(e).unwrap()),
                );
            })
        }
    }

    fn git(dir: &Path, args: &[&str]) -> String {
        let out = std::process::Command::new("git")
            .current_dir(dir)
            .args(args)
            .env("GIT_AUTHOR_NAME", "t")
            .env("GIT_AUTHOR_EMAIL", "t@t")
            .env("GIT_COMMITTER_NAME", "t")
            .env("GIT_COMMITTER_EMAIL", "t@t")
            .output()
            .expect("git runs");
        assert!(
            out.status.success(),
            "git {args:?}: {}",
            String::from_utf8_lossy(&out.stderr)
        );
        String::from_utf8_lossy(&out.stdout).trim().to_string()
    }

    /// `<tmp>/root/demo` holding one commit whose `.qontinui/ci.toml` is
    /// `manifest`; returns (tempdir, root, head sha).
    fn repo_with(manifest: &str) -> (tempfile::TempDir, PathBuf, String) {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path().join("root");
        let repo = root.join("demo");
        std::fs::create_dir_all(repo.join(".qontinui")).unwrap();
        std::fs::write(repo.join(".qontinui").join("ci.toml"), manifest).unwrap();
        git(&repo, &["init", "-q", "-b", "main"]);
        git(&repo, &["add", "."]);
        git(&repo, &["commit", "-q", "-m", "init"]);
        let head = git(&repo, &["rev-parse", "HEAD"]);
        (tmp, root, head)
    }

    async fn run(root: &Path, head: &str, job: Option<&str>) -> (Conclusion, Recording) {
        run_as(root, head, job, "e2e-1", "local/demo").await
    }

    async fn run_as(
        root: &Path,
        head: &str,
        job: Option<&str>,
        dispatch_id: &str,
        repo: &str,
    ) -> (Conclusion, Recording) {
        let host = crate::standalone::host(root.to_path_buf(), "test-host".to_string());
        run_on(&host, root, head, job, dispatch_id, repo, None).await
    }

    async fn run_on(
        host: &Host,
        root: &Path,
        head: &str,
        job: Option<&str>,
        dispatch_id: &str,
        repo: &str,
        model_access: Option<crate::dispatch::ModelAccessGrant>,
    ) -> (Conclusion, Recording) {
        let lines = Arc::new(std::sync::Mutex::new(Vec::new()));
        let filed = Arc::new(std::sync::Mutex::new(Vec::new()));
        let models = Arc::new(std::sync::Mutex::new(Vec::new()));
        let outcomes = Arc::new(std::sync::Mutex::new(Vec::new()));
        let reporter = Box::new(Recording {
            lines: lines.clone(),
            filed: filed.clone(),
            models: models.clone(),
            outcomes: outcomes.clone(),
        });
        let payload = DispatchPayload {
            dispatch_id: dispatch_id.to_string(),
            repo: repo.to_string(),
            head_sha: head.to_string(),
            fetch_url: root.join("demo").to_string_lossy().replace('\\', "/"),
            candidate_ref: format!("refs/ci-dispatch/local/{dispatch_id}"),
            pr_number: None,
            manifest_path: ".qontinui/ci.toml".to_string(),
            job: job.map(str::to_string),
            check_name: String::new(),
            coord_http_url: String::new(),
            model_access,
        };
        let capacity = host_sizing::HostCapacity {
            mem_bytes: Some(8 * 1024 * 1024 * 1024),
            cpus: 4,
        };
        let conclusion = run_dispatch(
            host,
            reporter,
            payload,
            CancellationToken::new(),
            capacity,
            1,
        )
        .await;
        (
            conclusion,
            Recording {
                lines,
                filed,
                models,
                outcomes,
            },
        )
    }

    const TWO_JOBS: &str = r#"
version = 2
[[jobs]]
name = "green"
[[jobs.steps]]
name = "git-version"
command = ["git", "--version"]
[[jobs]]
name = "red"
[[jobs.steps]]
name = "no-such-revision"
command = ["git", "rev-parse", "--verify", "refs/heads/does-not-exist"]
[[jobs.steps]]
name = "never-reached"
command = ["git", "--version"]
"#;

    /// The named job runs in a fresh checkout of the dispatched commit, files
    /// exactly one verdict, and leaves nothing behind but the warm target dir.
    #[tokio::test]
    async fn a_green_job_runs_reports_once_and_cleans_up() {
        let (_tmp, root, head) = repo_with(TWO_JOBS);
        let (conclusion, rec) = run(&root, &head, Some("green")).await;
        assert_eq!(
            conclusion,
            Conclusion::Success,
            "{:#?}",
            rec.lines.lock().unwrap()
        );
        let filed = rec.filed.lock().unwrap().clone();
        assert_eq!(filed.len(), 1, "exactly one verdict");
        assert_eq!(filed[0].0, Conclusion::Success);
        assert_eq!(
            filed[0].1,
            [
                "[setup] checkout=success",
                "[setup] provision=success",
                "git-version=success"
            ]
        );
        assert!(
            rec.lines
                .lock()
                .unwrap()
                .iter()
                .any(|l| l.contains("on test-host")),
            "the log names the host"
        );
        assert!(
            !crate::checkout::ci_dispatch_root(&root, "e2e-1").exists(),
            "the dispatch root must be cleaned up"
        );
    }

    /// A job whose steps write no run outcomes files a verdict carrying none,
    /// and says so on the dispatch log — the channel exists on every dispatch
    /// but sends nothing it was not given.
    #[tokio::test]
    async fn a_job_that_writes_no_outcomes_sends_none_and_says_so() {
        let (_tmp, root, head) = repo_with(TWO_JOBS);
        let (conclusion, rec) = run(&root, &head, Some("green")).await;
        assert_eq!(conclusion, Conclusion::Success);
        assert_eq!(rec.outcomes.lock().unwrap().clone(), vec![None]);
        assert!(
            rec.lines
                .lock()
                .unwrap()
                .iter()
                .any(|l| l.starts_with("[ci-node] run outcomes: none")),
            "{:#?}",
            rec.lines.lock().unwrap()
        );
    }

    /// The arming test for the outcome channel, entered where the population
    /// enters — `run_dispatch`: a step appends an entry to
    /// `$QONTINUI_CI_RUN_OUTCOMES`, the verdict carries it, the file lived
    /// inside the cleanup unit, and cleanup still ran. A second, invalid entry
    /// sends nothing and leaves the verdict green. Runs on both lanes: the
    /// step copies a committed file to the exported path with the platform's
    /// own shell (`sh -c cp` / `cmd /V:ON /C copy`, delayed expansion because
    /// `%` is a banned argv character).
    #[tokio::test]
    async fn a_step_appended_outcome_reaches_the_verdict_before_cleanup() {
        fn copy_step(file: &str) -> String {
            if cfg!(windows) {
                format!(
                    "[\"cmd\", \"/V:ON\", \"/C\", \"copy /Y {file} !QONTINUI_CI_RUN_OUTCOMES!\"]"
                )
            } else {
                format!("[\"sh\", \"-c\", \"cp {file} $QONTINUI_CI_RUN_OUTCOMES\"]")
            }
        }
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path().join("root");
        let repo = root.join("demo");
        std::fs::create_dir_all(repo.join(".qontinui")).unwrap();
        std::fs::write(
            repo.join("ok.jsonl"),
            "{\"source\":\"reference-eval\",\"subject\":\"oracle\",\"task\":\"s1\",\"metrics\":{\"recall\":98.5},\"executed_case_count\":12}\n",
        )
        .unwrap();
        std::fs::write(repo.join("bad.jsonl"), "{\"repo\":\"forged\"}\n").unwrap();
        std::fs::write(
            repo.join(".qontinui").join("ci.toml"),
            format!(
                "version = 2\n\
                 [[jobs]]\nname = \"ok\"\n[[jobs.steps]]\nname = \"emit\"\ncommand = {}\n\
                 [[jobs]]\nname = \"bad\"\n[[jobs.steps]]\nname = \"emit\"\ncommand = {}\n",
                copy_step("ok.jsonl"),
                copy_step("bad.jsonl")
            ),
        )
        .unwrap();
        git(&repo, &["init", "-q", "-b", "main"]);
        git(&repo, &["add", "."]);
        git(&repo, &["commit", "-q", "-m", "init"]);
        let head = git(&repo, &["rev-parse", "HEAD"]);

        let (c, rec) = run(&root, &head, Some("ok")).await;
        assert_eq!(c, Conclusion::Success, "{:#?}", rec.lines.lock().unwrap());
        let sent = rec.outcomes.lock().unwrap().clone();
        assert_eq!(
            sent,
            vec![Some(serde_json::json!([{
                "source": "reference-eval", "subject": "oracle", "task": "s1",
                "metrics": {"recall": 98.5}, "executed_case_count": 12
            }]))]
        );
        assert!(
            !crate::checkout::ci_dispatch_root(&root, "e2e-1").exists(),
            "the outcome file is inside the cleanup unit and goes with it"
        );

        let (c, rec) = run(&root, &head, Some("bad")).await;
        assert_eq!(
            c,
            Conclusion::Success,
            "an invalid outcome never changes the verdict"
        );
        assert_eq!(rec.outcomes.lock().unwrap().clone(), vec![None]);
        assert!(
            rec.lines
                .lock()
                .unwrap()
                .iter()
                .any(|l| l.contains("run_outcome_attribution_in_body")),
            "{:#?}",
            rec.lines.lock().unwrap()
        );
    }

    /// The first failing step short-circuits the job and the verdict is red.
    #[tokio::test]
    async fn a_failing_step_short_circuits_the_job() {
        let (_tmp, root, head) = repo_with(TWO_JOBS);
        let (conclusion, rec) = run(&root, &head, Some("red")).await;
        assert_eq!(conclusion, Conclusion::Failure);
        let filed = rec.filed.lock().unwrap().clone();
        assert_eq!(filed.len(), 1);
        assert!(
            filed[0].1.contains(&"no-such-revision=failure".to_string()),
            "{:?}",
            filed[0].1
        );
        assert!(
            !filed[0].1.iter().any(|s| s.starts_with("never-reached")),
            "{:?}",
            filed[0].1
        );
        assert!(!crate::checkout::ci_dispatch_root(&root, "e2e-1").exists());
    }

    /// A job the dispatch NAMES that the manifest does not declare is a red
    /// manifest verdict, and a v1 manifest is reachable as job `ci` — named
    /// or defaulted.
    #[tokio::test]
    async fn job_selection_by_name() {
        let (_tmp, root, head) = repo_with(TWO_JOBS);
        let (conclusion, rec) = run(&root, &head, Some("ci")).await;
        assert_eq!(conclusion, Conclusion::Failure);
        let filed = rec.filed.lock().unwrap().clone();
        assert_eq!(
            filed[0].1,
            ["[setup] checkout=success", "[setup] manifest=failure"]
        );
        assert_eq!(filed[0].2, None);
        assert!(rec
            .lines
            .lock()
            .unwrap()
            .iter()
            .any(|l| l.contains("no job 'ci'")));

        let (_tmp1, root1, head1) =
            repo_with("version = 1\n[[steps]]\nname = \"v\"\ncommand = [\"git\", \"--version\"]\n");
        let (conclusion, _) = run(&root1, &head1, None).await;
        assert_eq!(
            conclusion,
            Conclusion::Success,
            "a defaulted job runs a v1 manifest"
        );
        let (_tmp2, root2, head2) =
            repo_with("version = 1\n[[steps]]\nname = \"v\"\ncommand = [\"git\", \"--version\"]\n");
        let (conclusion, _) = run(&root2, &head2, Some("ci")).await;
        assert_eq!(
            conclusion,
            Conclusion::Success,
            "a named `ci` runs a v1 manifest"
        );
    }

    /// A dispatch that names NO job against a v2 manifest with no `ci` job is
    /// a non-verdict (`job_not_declared`), not a red: nothing about the code
    /// was observed, and every coord dispatch omits `job` today.
    #[tokio::test]
    async fn a_defaulted_job_a_v2_manifest_does_not_declare_is_cancelled() {
        let (_tmp, root, head) = repo_with(TWO_JOBS);
        let (conclusion, rec) = run(&root, &head, None).await;
        assert_eq!(conclusion, Conclusion::Cancelled);
        let filed = rec.filed.lock().unwrap().clone();
        assert_eq!(filed.len(), 1);
        assert_eq!(
            filed[0].1,
            ["[setup] checkout=success", "[setup] manifest=cancelled"]
        );
        assert_eq!(filed[0].2.as_deref(), Some("job_not_declared"));
        assert!(!crate::checkout::ci_dispatch_root(&root, "e2e-1").exists());
    }

    /// An unsafe dispatch id or repo slug is refused before anything is
    /// written: no dispatch root, no target dir, no worktree.
    #[tokio::test]
    async fn unsafe_identifiers_are_refused_with_nothing_written() {
        let (_tmp, root, head) = repo_with(TWO_JOBS);
        // An unsafe dispatch id is dropped UNREPORTED: the verdict would be
        // addressed by that very id.
        let (conclusion, rec) =
            run_as(&root, &head, Some("green"), "../../escape", "local/demo").await;
        assert_eq!(conclusion, Conclusion::Cancelled);
        assert!(rec.filed.lock().unwrap().is_empty(), "nothing may be filed");
        assert!(
            rec.lines.lock().unwrap().is_empty(),
            "not one line may reach a sink addressed by an unsafe id"
        );
        // An unsafe repo slug with a safe id is refused WITH a verdict.
        for repo in ["owner/../demo", "/", "/tmp/x/"] {
            let (conclusion, rec) = run_as(&root, &head, Some("green"), "ok-id", repo).await;
            assert_eq!(conclusion, Conclusion::Cancelled, "{repo}");
            let filed = rec.filed.lock().unwrap().clone();
            assert_eq!(filed.len(), 1, "{repo}");
            assert_eq!(filed[0].1, ["[setup] host=cancelled"]);
            assert_eq!(filed[0].2.as_deref(), Some("unsafe_identifier"));
        }
        let entries: Vec<_> = std::fs::read_dir(&root)
            .unwrap()
            .map(|e| e.unwrap().file_name().to_string_lossy().to_string())
            .collect();
        assert_eq!(
            entries,
            ["demo"],
            "nothing but the repo itself may exist under the root"
        );
    }

    /// A job pinned to an OS this host is not is a non-verdict, not a red.
    #[tokio::test]
    async fn a_job_for_another_os_is_cancelled_not_failed() {
        let other = if cfg!(windows) { "linux" } else { "windows" };
        let manifest = format!(
            "version = 2\n[[jobs]]\nname = \"elsewhere\"\nos = \"{other}\"\n[[jobs.steps]]\nname = \"s\"\ncommand = [\"git\", \"--version\"]\n"
        );
        let (_tmp, root, head) = repo_with(&manifest);
        let (conclusion, rec) = run(&root, &head, Some("elsewhere")).await;
        assert_eq!(conclusion, Conclusion::Cancelled);
        assert_eq!(
            rec.filed.lock().unwrap()[0].2.as_deref(),
            Some("job_os_mismatch")
        );
    }

    // ── Model access through the executor (plan
    // `2026-10-13-ci-node-runs-model-dependent-eval-steps`, Phase 2) ──

    fn model_section(rec: &Recording) -> (String, bool) {
        let models = rec.models.lock().unwrap();
        assert_eq!(models.len(), 1, "exactly one verdict");
        let (section, ledger) = &models[0];
        let status = section["status"].as_str().unwrap().to_string();
        let status = match section.get("reason").and_then(|r| r.as_str()) {
            Some(reason) => format!("{status}:{reason}"),
            None => status,
        };
        (status, ledger.is_some())
    }

    const MODEL_TABLE: &str = "[model]\nmodels = [\"claude-sonnet\"]\nmax_calls = 5\n\
                               max_tokens = 1000\nmax_wall_secs = 60\n";

    /// Every exit path files a `summary.model` section: `not_evaluated` before
    /// the job's model steps can be known or checked, `not_requested` for a job
    /// without one, `refused` when checked and withheld.
    #[tokio::test]
    async fn the_verdict_carries_the_model_section_on_every_exit_path() {
        let other = if cfg!(windows) { "linux" } else { "windows" };
        let manifest = format!(
            "version = 2\n{MODEL_TABLE}\
             [[jobs]]\nname = \"plain\"\n[[jobs.steps]]\nname = \"v\"\ncommand = [\"git\", \"--version\"]\n\
             [[jobs]]\nname = \"llm\"\n[[jobs.steps]]\nname = \"m\"\ncommand = [\"git\", \"--version\"]\nmodel = true\n\
             [[jobs]]\nname = \"elsewhere\"\nos = \"{other}\"\n[[jobs.steps]]\nname = \"m\"\ncommand = [\"git\", \"--version\"]\nmodel = true\n\
             [[jobs]]\nname = \"elsewhere-plain\"\nos = \"{other}\"\n[[jobs.steps]]\nname = \"v\"\ncommand = [\"git\", \"--version\"]\n"
        );
        let (_tmp, root, head) = repo_with(&manifest);
        let host = crate::standalone::host(root.clone(), "test-host".to_string());

        // Refused before checkout (unsafe repo slug).
        let (_, rec) = run_on(&host, &root, &head, Some("llm"), "m-0", "owner/../x", None).await;
        assert_eq!(model_section(&rec), ("not_evaluated".to_string(), false));
        // (The checkout-failure exit spells `NotEvaluated` the same way; it is
        // not driven here because a missing head waits out the mirror's
        // retry deadline.)
        // The manifest declares no such job.
        let (_, rec) = run_on(&host, &root, &head, Some("nope"), "m-2", "local/demo", None).await;
        assert_eq!(model_section(&rec), ("not_evaluated".to_string(), false));
        // A job for another OS: known, but never reached the checks.
        let (_, rec) = run_on(
            &host,
            &root,
            &head,
            Some("elsewhere"),
            "m-3",
            "local/demo",
            None,
        )
        .await;
        assert_eq!(model_section(&rec), ("not_evaluated".to_string(), false));
        let (_, rec) = run_on(
            &host,
            &root,
            &head,
            Some("elsewhere-plain"),
            "m-4",
            "local/demo",
            None,
        )
        .await;
        assert_eq!(model_section(&rec), ("not_requested".to_string(), false));
        // A job with no model step runs and starts no broker.
        let (c, rec) = run_on(
            &host,
            &root,
            &head,
            Some("plain"),
            "m-5",
            "local/demo",
            None,
        )
        .await;
        assert_eq!(c, Conclusion::Success);
        assert_eq!(model_section(&rec), ("not_requested".to_string(), false));
        assert!(!rec
            .lines
            .lock()
            .unwrap()
            .iter()
            .any(|l| l.contains("model access")));
        // The standalone host offers no broker: refused, and the verdict stands.
        let (c, rec) = run_on(&host, &root, &head, Some("llm"), "m-6", "local/demo", None).await;
        assert_eq!(c, Conclusion::Success, "a refusal never turns a run red");
        assert_eq!(
            model_section(&rec),
            ("refused:host_has_no_broker".to_string(), false)
        );
    }

    /// The canonical refusal exit files the section too.
    #[tokio::test]
    async fn a_canonical_refusal_files_the_model_section() {
        let (_tmp, root, head) = repo_with(&format!(
            "version = 1\n{MODEL_TABLE}[canonical]\ntoolchains = [\"rustc\"]\n\
             [[steps]]\nname = \"m\"\ncommand = [\"git\", \"--version\"]\nmodel = true\n"
        ));
        let (c, rec) = run(&root, &head, None).await;
        assert_eq!(c, Conclusion::Failure);
        assert_eq!(model_section(&rec), ("not_evaluated".to_string(), false));
    }

    fn step_out<'a>(lines: &'a [String], step: &str) -> Vec<&'a str> {
        let prefix = format!("[{step}:out] ");
        lines
            .iter()
            .filter_map(|l| l.strip_prefix(prefix.as_str()))
            .collect()
    }

    /// The arming sentinel: on a host with no broker a `model = true` step
    /// still runs, with `QONTINUI_CI_MODEL_REFUSED=host_has_no_broker` and no
    /// model variables; a plain step gets neither. Every step gets the
    /// dispatch id.
    #[cfg(unix)]
    #[tokio::test]
    async fn a_model_step_on_a_host_with_no_broker_runs_with_the_refusal() {
        let (_tmp, root, head) = repo_with(&format!(
            "version = 1\n{MODEL_TABLE}[[steps]]\nname = \"plain\"\ncommand = [\"env\"]\n\
             [[steps]]\nname = \"llm\"\ncommand = [\"env\"]\nmodel = true\n"
        ));
        let (c, rec) = run(&root, &head, None).await;
        assert_eq!(c, Conclusion::Success);
        let lines = rec.lines.lock().unwrap().clone();
        let plain = step_out(&lines, "plain");
        let llm = step_out(&lines, "llm");
        assert!(
            llm.contains(&"QONTINUI_CI_MODEL_REFUSED=host_has_no_broker"),
            "{llm:?}"
        );
        assert!(
            !plain.iter().any(|l| l.starts_with("QONTINUI_CI_MODEL_")),
            "{plain:?}"
        );
        for out in [&plain, &llm] {
            assert!(out.contains(&"QONTINUI_CI_DISPATCH_ID=e2e-1"), "{out:?}");
            assert!(!out
                .iter()
                .any(|l| l.starts_with("QONTINUI_CI_MODEL_TOKEN=")));
            assert!(!out.iter().any(|l| l.starts_with("QONTINUI_CI_MODEL_URL=")));
        }
    }

    /// The scrub: a credential or model command the PARENT process holds never
    /// reaches a step, and every step sees `CLAUDE_CONFIG_DIR` pointing at an
    /// empty directory — emptied again before each step, so one step cannot
    /// plant a login for the next.
    #[cfg(unix)]
    #[tokio::test]
    async fn the_scrub_strips_inherited_credentials_and_every_step_gets_an_empty_config_dir() {
        let planted: Vec<String> = crate::model::MODEL_CREDENTIAL_ENV
            .iter()
            .map(|k| k.to_string())
            .chain(["DOCS_EVAL_MODEL_CMD".to_string()])
            .collect();
        for k in &planted {
            std::env::set_var(k, format!("inherited-leak-{k}"));
        }
        let (_tmp, root, head) = repo_with(
            "version = 1\n[[steps]]\nname = \"s1\"\ncommand = [\"env\"]\n\
             [[steps]]\nname = \"s2\"\ncommand = [\"sh\", \"-c\", \"ls -A $CLAUDE_CONFIG_DIR; touch $CLAUDE_CONFIG_DIR/planted; echo S2-DONE\"]\n\
             [[steps]]\nname = \"s3\"\ncommand = [\"sh\", \"-c\", \"ls -A $CLAUDE_CONFIG_DIR; echo S3-DONE\"]\n",
        );
        let (c, rec) = run(&root, &head, None).await;
        for k in &planted {
            std::env::remove_var(k);
        }
        assert_eq!(c, Conclusion::Success, "{:#?}", rec.lines.lock().unwrap());
        let lines = rec.lines.lock().unwrap().clone();
        let s1 = step_out(&lines, "s1");
        assert!(!s1.iter().any(|l| l.contains("inherited-leak")), "{s1:?}");
        let dir = s1
            .iter()
            .find_map(|l| l.strip_prefix("CLAUDE_CONFIG_DIR="))
            .expect("CLAUDE_CONFIG_DIR is set for every step");
        assert!(dir.ends_with(".ci-claude-config"), "{dir}");
        assert!(
            dir.starts_with(&*crate::checkout::ci_dispatch_root(&root, "e2e-1").to_string_lossy()),
            "{dir}"
        );
        assert_eq!(step_out(&lines, "s2"), ["S2-DONE"], "empty for s2");
        assert_eq!(step_out(&lines, "s3"), ["S3-DONE"], "emptied again for s3");
    }

    /// Arming test for the every-step scrub, written against base-era API
    /// only (`run` drives `run_dispatch`; no model symbol is named), so it is
    /// RED on the pre-scrub executor rather than failing to compile: an
    /// inherited `*_MODEL_CMD` reached every step, and no step saw a pinned
    /// `CLAUDE_CONFIG_DIR`.
    #[cfg(unix)]
    #[tokio::test]
    async fn arming_an_inherited_model_command_never_reaches_a_ci_step() {
        const KEY: &str = "ARMING_PROBE_MODEL_CMD";
        std::env::set_var(KEY, "arming-planted-model-cmd");
        let (_tmp, root, head) = repo_with(
            "version = 1\n[[steps]]\nname = \"probe\"\ncommand = [\"sh\", \"-c\", \"echo seen=${ARMING_PROBE_MODEL_CMD:-absent}; echo cfg=${CLAUDE_CONFIG_DIR:-unset}\"]\n",
        );
        let (c, rec) = run(&root, &head, None).await;
        std::env::remove_var(KEY);
        let lines = rec.lines.lock().unwrap().clone();
        assert_eq!(c, Conclusion::Success, "{lines:#?}");
        assert!(
            lines.iter().any(|l| l.ends_with("seen=absent")),
            "an inherited *_MODEL_CMD reached the step: {lines:#?}"
        );
        assert!(
            lines
                .iter()
                .any(|l| l.contains("cfg=") && l.ends_with(".ci-claude-config")),
            "the step did not get the pinned empty CLAUDE_CONFIG_DIR: {lines:#?}"
        );
    }
}
