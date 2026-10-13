//! The executor's model-access hook (plan
//! `2026-10-13-ci-node-runs-model-dependent-eval-steps`, Phase 2 item 6).
//!
//! After provisioning, and only when the dispatched job has a `model = true`
//! step, the executor checks in this order; the first failure names the
//! refusal:
//!
//! 1. the host offers an invoker (`host_has_no_broker`);
//! 2. device consent (`device_consent_off`);
//! 3. coord's grant (its own reason, or `not_offered_by_coord`);
//! 4. the shim is a non-empty regular file (`shim_missing`);
//! 5. the invoker's `ready()` (`cli_missing`, `no_model_account`, …);
//! 6. then it starts the broker (`broker_start_failed`).
//!
//! A refusal never stops a step and never changes the verdict (D12): the
//! `model = true` steps run with `QONTINUI_CI_MODEL_REFUSED=<reason>` instead
//! of the model variables, so a harness can write UNKNOWN rows that say why.

use std::net::{Ipv4Addr, SocketAddr};
use std::sync::Arc;

use crate::dispatch::ModelAccessGrant;
use crate::host::Host;
use crate::manifest::{CiJob, CiStep};
use crate::model::broker::{Broker, BrokerConfig};
use crate::model::declaration::ModelDeclaration;
use crate::model::ledger::{ModelLedger, ModelSection};
use crate::model::{
    COORD_REFUSALS, DISPATCH_ID_ENV, MODEL_CMD_ENV, MODEL_REFUSED_ENV, MODEL_TOKEN_ENV,
    MODEL_URL_ENV, RUNNER_REFUSALS,
};
use crate::report::LogSink;

/// Whether `job` has a step that asks for model access.
pub(crate) fn job_requests_model(job: &CiJob) -> bool {
    job.steps.iter().any(|s| s.model)
}

/// The `summary.model` section for a run that ended BEFORE the model checks:
/// `not_requested` when the job is known and has no model step, else
/// `not_evaluated`.
pub(crate) fn section_before_checks(job: Option<&CiJob>) -> ModelSection {
    match job {
        Some(job) if !job_requests_model(job) => ModelSection::NotRequested,
        _ => ModelSection::NotEvaluated,
    }
}

/// One dispatch's model access, decided once.
pub(crate) enum ModelAccess {
    /// The job has no `model = true` step: no broker, no exports.
    NotRequested,
    /// Withheld for `reason`.
    Refused { reason: String },
    /// Granted: a running broker and the shim's command line.
    Granted { broker: Broker, shim_cmd: String },
}

/// `reason` if it is in the closed vocabulary `allowed`, else `fallback`.
/// D12's reasons form a closed list: a reason outside it — a typo, a newer
/// coord, a host's free text — never reaches a step or the result.
fn closed_reason(reason: &str, allowed: &[&str], fallback: &str) -> String {
    if allowed.contains(&reason) {
        reason.to_string()
    } else {
        fallback.to_string()
    }
}

/// Spell the shim's argv as one command line a harness can `shlex.split`:
/// forward slashes (POSIX `shlex` eats a Windows backslash), and single quotes
/// around any token a shell would split or interpret.
pub(crate) fn shim_command_spelling(argv: &[String]) -> String {
    argv.iter()
        .map(|a| {
            let a = a.replace('\\', "/");
            let needs_quotes = a.is_empty()
                || a.chars()
                    .any(|c| c.is_whitespace() || "'\"$`;&|<>()*?[]#~!{}".contains(c));
            if needs_quotes {
                format!("'{}'", a.replace('\'', r"'\''"))
            } else {
                a
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

/// Whether the shim's program is a non-empty regular file. A zero-byte shim
/// "runs" as an empty script and does nothing — the failure dossier
/// `qontinui-pr-shim-zero-byte-opens-no-pr` records.
fn shim_present(argv: &[String]) -> bool {
    argv.first()
        .and_then(|p| std::fs::metadata(p).ok())
        .is_some_and(|m| m.is_file() && m.len() > 0)
}

/// Decide model access for one dispatch. Called after provisioning.
pub(crate) async fn prepare(
    host: &Host,
    grant: Option<&ModelAccessGrant>,
    declaration: Option<&ModelDeclaration>,
    job: &CiJob,
    sink: Arc<dyn LogSink>,
) -> ModelAccess {
    prepare_on(
        SocketAddr::from((Ipv4Addr::LOCALHOST, 0)),
        host,
        grant,
        declaration,
        job,
        sink,
    )
    .await
}

/// [`prepare`] with the broker's bind address as a seam, so a test can make
/// the broker fail to start.
pub(crate) async fn prepare_on(
    bind: SocketAddr,
    host: &Host,
    grant: Option<&ModelAccessGrant>,
    declaration: Option<&ModelDeclaration>,
    job: &CiJob,
    sink: Arc<dyn LogSink>,
) -> ModelAccess {
    if !job_requests_model(job) {
        return ModelAccess::NotRequested;
    }
    let decided = decide(bind, host, grant, declaration, sink.clone()).await;
    match &decided {
        ModelAccess::Granted { broker, .. } => sink.push(&format!(
            "[ci-node] model access: granted — broker on {} for the model = true step(s)",
            broker.url()
        )),
        ModelAccess::Refused { reason } => sink.push(&format!(
            "[ci-node] model access: refused ({reason}) — the model = true step(s) run with \
             {MODEL_REFUSED_ENV}={reason}; the verdict is unaffected"
        )),
        ModelAccess::NotRequested => {}
    }
    decided
}

async fn decide(
    bind: SocketAddr,
    host: &Host,
    grant: Option<&ModelAccessGrant>,
    declaration: Option<&ModelDeclaration>,
    sink: Arc<dyn LogSink>,
) -> ModelAccess {
    let refused = |reason: &str| ModelAccess::Refused {
        reason: reason.to_string(),
    };
    let Some(invoker) = host.model.clone() else {
        return refused("host_has_no_broker");
    };
    if !host.settings.model_access() {
        return refused("device_consent_off");
    }
    match grant {
        None => return refused("not_offered_by_coord"),
        Some(g) if !g.granted => {
            return refused(&closed_reason(
                g.reason.as_deref().unwrap_or(""),
                COORD_REFUSALS,
                "not_offered_by_coord",
            ))
        }
        Some(_) => {}
    }
    let shim_cmd = match invoker.shim_argv() {
        Some(argv) if shim_present(&argv) => shim_command_spelling(&argv),
        _ => return refused("shim_missing"),
    };
    if let Err(reason) = invoker.ready() {
        return refused(&closed_reason(
            &reason,
            RUNNER_REFUSALS,
            "broker_start_failed",
        ));
    }
    // Validation guarantees a `model = true` step implies a declaration; a
    // missing one here is a broken invariant, refused rather than unwrapped.
    let Some(declaration) = declaration else {
        return refused("broker_start_failed");
    };
    match Broker::start_on(
        bind,
        invoker,
        BrokerConfig::new(declaration.clone()),
        sink.clone(),
    )
    .await
    {
        Ok(broker) => ModelAccess::Granted { broker, shim_cmd },
        Err(e) => {
            sink.push(&format!("[ci-node] model broker did not start: {e}"));
            refused("broker_start_failed")
        }
    }
}

impl ModelAccess {
    /// The model env one step gets, beside the dispatch's exports: the
    /// dispatch id for every step; for a `model = true` step the broker's
    /// command, URL and capability on a grant, or the refusal reason.
    pub(crate) fn step_env(&self, step: &CiStep, dispatch_id: &str) -> Vec<(String, String)> {
        let mut env = vec![(DISPATCH_ID_ENV.to_string(), dispatch_id.to_string())];
        if !step.model {
            return env;
        }
        match self {
            ModelAccess::Granted { broker, shim_cmd } => {
                env.push((MODEL_CMD_ENV.to_string(), shim_cmd.clone()));
                env.push((MODEL_URL_ENV.to_string(), broker.url()));
                env.push((MODEL_TOKEN_ENV.to_string(), broker.token().to_string()));
            }
            ModelAccess::Refused { reason } => {
                env.push((MODEL_REFUSED_ENV.to_string(), reason.clone()));
            }
            ModelAccess::NotRequested => {}
        }
        env
    }

    /// Attribute the broker's next calls to `step`.
    pub(crate) fn set_step(&self, step: &str) {
        if let ModelAccess::Granted { broker, .. } = self {
            broker.set_step(step);
        }
    }

    /// Stop the broker (if one runs) and produce the result's model section
    /// and ledger. Must run BEFORE the dispatch's cleanup.
    pub(crate) async fn finish(self) -> (ModelSection, Option<ModelLedger>) {
        match self {
            ModelAccess::NotRequested => (ModelSection::NotRequested, None),
            ModelAccess::Refused { reason } => (ModelSection::Refused { reason }, None),
            ModelAccess::Granted { broker, .. } => {
                let out = broker.stop().await;
                let section = match out.exhausted_reset_at {
                    Some(reset_at) => ModelSection::Exhausted {
                        calls: out.calls_used,
                        reset_at,
                    },
                    None => ModelSection::Granted {
                        calls: out.calls_used,
                    },
                };
                (section, Some(out.ledger))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::host::CiSettings;
    use crate::model::test_support::{declaration, FakeInvoker, Lines};

    struct Consent(bool);
    impl CiSettings for Consent {
        fn canonical_converge(&self) -> bool {
            false
        }
        fn model_access(&self) -> bool {
            self.0
        }
    }

    fn job(model_step: bool) -> CiJob {
        let m = crate::manifest::parse_and_validate(&format!(
            "version = 1\n{}[[steps]]\nname = \"s\"\ncommand = [\"true\"]\nmodel = {model_step}\n",
            if model_step {
                "[model]\nmodels = [\"claude-sonnet\"]\nmax_calls = 1\nmax_tokens = 1\nmax_wall_secs = 1\n"
            } else {
                ""
            }
        ))
        .unwrap();
        m.jobs[0].clone()
    }

    /// A non-empty file to serve as the shim.
    fn shim_file(dir: &std::path::Path, bytes: &[u8]) -> Vec<String> {
        let p = dir.join("qontinui-ci-model");
        std::fs::write(&p, bytes).unwrap();
        vec![p.to_string_lossy().to_string()]
    }

    fn host_with(inv: Option<FakeInvoker>, consent: bool) -> Host {
        let mut h = crate::standalone::host(std::env::temp_dir(), "t".to_string());
        h.settings = Arc::new(Consent(consent));
        h.model = inv.map(|i| Arc::new(i) as Arc<dyn crate::host::ModelInvoker>);
        h
    }

    fn granted() -> ModelAccessGrant {
        ModelAccessGrant {
            granted: true,
            reason: None,
            basis: Some("live".to_string()),
        }
    }

    async fn reason_of(access: ModelAccess) -> String {
        match access {
            ModelAccess::Refused { reason } => reason,
            ModelAccess::Granted { broker, .. } => {
                broker.stop().await;
                "GRANTED".to_string()
            }
            ModelAccess::NotRequested => "NOT_REQUESTED".to_string(),
        }
    }

    /// Each refusal arm, in order, and the step env each one produces.
    #[tokio::test]
    async fn each_refusal_arm_names_its_reason_in_order() {
        let tmp = tempfile::tempdir().unwrap();
        let good_shim = shim_file(tmp.path(), b"#!/bin/sh\n");
        let decl = declaration(5, 100, 60);
        let job = job(true);
        let sink: Arc<dyn LogSink> = Arc::new(Lines::default());
        let loopback = SocketAddr::from((Ipv4Addr::LOCALHOST, 0));
        let ready_inv = |shim: Option<Vec<String>>, ready: Result<(), String>| {
            let mut i = FakeInvoker::answering("x", 1);
            i.shim = shim;
            i.ready = ready;
            i
        };
        let refusal = |reason: &str| ModelAccessGrant {
            granted: false,
            reason: Some(reason.to_string()),
            basis: Some("live".to_string()),
        };
        let zero_dir = tempfile::tempdir().unwrap();
        let zero_shim = shim_file(zero_dir.path(), b"");
        let cases: Vec<(&str, Host, Option<ModelAccessGrant>, SocketAddr)> = vec![
            (
                "host_has_no_broker",
                host_with(None, true),
                Some(granted()),
                loopback,
            ),
            (
                "device_consent_off",
                host_with(Some(ready_inv(Some(good_shim.clone()), Ok(()))), false),
                Some(granted()),
                loopback,
            ),
            (
                "not_offered_by_coord",
                host_with(Some(ready_inv(Some(good_shim.clone()), Ok(()))), true),
                None,
                loopback,
            ),
            (
                "fork_pr",
                host_with(Some(ready_inv(Some(good_shim.clone()), Ok(()))), true),
                Some(refusal("fork_pr")),
                loopback,
            ),
            (
                "shim_missing",
                host_with(Some(ready_inv(None, Ok(()))), true),
                Some(granted()),
                loopback,
            ),
            (
                "shim_missing",
                host_with(Some(ready_inv(Some(zero_shim.clone()), Ok(()))), true),
                Some(granted()),
                loopback,
            ),
            (
                "shim_missing",
                host_with(
                    Some(ready_inv(
                        Some(vec![tmp.path().to_string_lossy().to_string()]),
                        Ok(()),
                    )),
                    true,
                ),
                Some(granted()),
                loopback,
            ),
            (
                "cli_missing",
                host_with(
                    Some(ready_inv(
                        Some(good_shim.clone()),
                        Err("cli_missing".to_string()),
                    )),
                    true,
                ),
                Some(granted()),
                loopback,
            ),
            (
                "broker_start_failed",
                host_with(Some(ready_inv(Some(good_shim.clone()), Ok(()))), true),
                Some(granted()),
                // TEST-NET-1: not an address of this host, so the bind fails.
                SocketAddr::from(([192, 0, 2, 1], 0)),
            ),
            (
                "GRANTED",
                host_with(Some(ready_inv(Some(good_shim.clone()), Ok(()))), true),
                Some(granted()),
                loopback,
            ),
        ];
        for (want, host, grant, bind) in cases {
            let access =
                prepare_on(bind, &host, grant.as_ref(), Some(&decl), &job, sink.clone()).await;
            if want != "GRANTED" {
                let env = access.step_env(&job.steps[0], "d-1");
                assert!(
                    env.contains(&(MODEL_REFUSED_ENV.to_string(), want.to_string())),
                    "{want}: {env:?}"
                );
                assert!(!env.iter().any(|(k, _)| k == MODEL_TOKEN_ENV), "{want}");
            }
            assert_eq!(reason_of(access).await, want);
        }
    }

    /// A job with no `model = true` step starts no broker, whatever the host.
    #[tokio::test]
    async fn a_job_without_model_steps_is_not_requested() {
        let tmp = tempfile::tempdir().unwrap();
        let mut inv = FakeInvoker::answering("x", 1);
        inv.shim = Some(shim_file(tmp.path(), b"x"));
        let host = host_with(Some(inv), true);
        let job = job(false);
        let access = prepare(
            &host,
            Some(&granted()),
            None,
            &job,
            Arc::new(Lines::default()),
        )
        .await;
        assert!(matches!(access, ModelAccess::NotRequested));
        assert_eq!(
            access.step_env(&job.steps[0], "d-1"),
            vec![(DISPATCH_ID_ENV.to_string(), "d-1".to_string())]
        );
        assert_eq!(access.finish().await.0, ModelSection::NotRequested);
    }

    /// On a grant only `model = true` steps get the broker variables; every
    /// step gets the dispatch id.
    #[tokio::test]
    async fn exports_reach_only_model_steps() {
        let tmp = tempfile::tempdir().unwrap();
        let mut inv = FakeInvoker::answering("x", 1);
        let shim_argv = shim_file(tmp.path(), b"x");
        inv.shim = Some(shim_argv.clone());
        let host = host_with(Some(inv), true);
        let m = crate::manifest::parse_and_validate(
            "version = 1\n[model]\nmodels = [\"claude-sonnet\"]\nmax_calls = 1\nmax_tokens = 1\n\
             max_wall_secs = 1\n[[steps]]\nname = \"plain\"\ncommand = [\"true\"]\n[[steps]]\n\
             name = \"llm\"\ncommand = [\"true\"]\nmodel = true\n",
        )
        .unwrap();
        let job = &m.jobs[0];
        let access = prepare(
            &host,
            Some(&granted()),
            m.model.as_ref(),
            job,
            Arc::new(Lines::default()),
        )
        .await;
        let ModelAccess::Granted { broker, .. } = &access else {
            panic!("expected a grant");
        };
        let plain = access.step_env(&job.steps[0], "d-1");
        assert_eq!(
            plain,
            vec![(DISPATCH_ID_ENV.to_string(), "d-1".to_string())]
        );
        let llm = access.step_env(&job.steps[1], "d-1");
        let get = |k: &str| llm.iter().find(|(n, _)| n == k).map(|(_, v)| v.clone());
        assert_eq!(get(MODEL_URL_ENV), Some(broker.url()));
        assert_eq!(get(MODEL_TOKEN_ENV).as_deref(), Some(broker.token()));
        // Compare with what the executor must spell (forward slashes, quoting,
        // `.exe` on Windows) rather than a platform-specific suffix.
        assert_eq!(
            get(MODEL_CMD_ENV).as_deref(),
            Some(shim_command_spelling(&shim_argv).as_str())
        );
        assert_eq!(get(DISPATCH_ID_ENV).as_deref(), Some("d-1"));
        assert!(get(MODEL_REFUSED_ENV).is_none());
        let (section, ledger) = access.finish().await;
        assert_eq!(section, ModelSection::Granted { calls: 0 });
        assert!(ledger.is_some());
    }

    #[test]
    fn the_shim_spelling_survives_a_posix_split() {
        assert_eq!(
            shim_command_spelling(&["/opt/q/qontinui-ci-model".to_string()]),
            "/opt/q/qontinui-ci-model"
        );
        assert_eq!(
            shim_command_spelling(
                &[r"C:\Program Files\Qontinui\qontinui-ci-model.exe".to_string()]
            ),
            "'C:/Program Files/Qontinui/qontinui-ci-model.exe'"
        );
        assert_eq!(
            shim_command_spelling(&["/a b/it's".to_string(), "--x".to_string()]),
            r"'/a b/it'\''s' --x"
        );
    }

    /// D12's closed vocabulary is enforced on both sources: coord's reason
    /// must be a coord reason, and `ready()`'s a runner reason.
    #[tokio::test]
    async fn refusal_reasons_outside_the_closed_vocabulary_are_replaced() {
        let tmp = tempfile::tempdir().unwrap();
        let shim = shim_file(tmp.path(), b"x");
        let decl = declaration(5, 100, 60);
        let job = job(true);
        let sink: Arc<dyn LogSink> = Arc::new(Lines::default());
        for (coord_reason, want) in [
            ("fork_pr", "fork_pr"),
            ("rate_capped", "rate_capped"),
            ("made_up_reason", "not_offered_by_coord"),
            ("cli_missing", "not_offered_by_coord"),
            ("a b\nEVIL=1", "not_offered_by_coord"),
            ("", "not_offered_by_coord"),
        ] {
            let mut inv = FakeInvoker::answering("x", 1);
            inv.shim = Some(shim.clone());
            let host = host_with(Some(inv), true);
            let grant = ModelAccessGrant {
                granted: false,
                reason: Some(coord_reason.to_string()),
                basis: None,
            };
            let access = prepare(&host, Some(&grant), Some(&decl), &job, sink.clone()).await;
            assert_eq!(reason_of(access).await, want, "coord {coord_reason:?}");
        }
        for (ready_err, want) in [
            ("cli_missing", "cli_missing"),
            ("no_model_account", "no_model_account"),
            ("gateway_unresolved", "gateway_unresolved"),
            ("claude not found on PATH", "broker_start_failed"),
            ("fork_pr", "broker_start_failed"),
        ] {
            let mut inv = FakeInvoker::answering("x", 1);
            inv.shim = Some(shim.clone());
            inv.ready = Err(ready_err.to_string());
            let host = host_with(Some(inv), true);
            let access = prepare(&host, Some(&granted()), Some(&decl), &job, sink.clone()).await;
            assert_eq!(reason_of(access).await, want, "ready {ready_err:?}");
        }
    }
}
