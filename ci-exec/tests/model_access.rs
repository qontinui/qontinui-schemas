//! End to end: a dispatch whose job declares `[model]` runs on a host with a
//! (fake) model invoker, and a `model = true` step reaches it through the REAL
//! `qontinui-ci-model` shim and the dispatch's broker — while a plain step of
//! the same job gets none of the model variables. The verdict carries the
//! granted section and the ledger, attributed per step.
//!
//! Unix only: the steps use `sh` to expand `$QONTINUI_CI_MODEL_CMD`, as a
//! harness's `shlex.split` would.
#![cfg(unix)]

use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

use qontinui_ci_exec::dispatch::{DispatchPayload, ModelAccessGrant};
use qontinui_ci_exec::host::{
    BoxFuture, InvokeError, InvokeReply, InvokeRequest, ModelInvoker, TokenUsage,
};
use qontinui_ci_exec::report::{Conclusion, LogSink, Reporter, Verdict};
use qontinui_ci_exec::{executor, host_sizing, standalone};
use tokio_util::sync::CancellationToken;

struct Fake {
    shim: PathBuf,
    prompts: Arc<Mutex<Vec<(String, String)>>>,
}

impl ModelInvoker for Fake {
    fn complete(
        &self,
        request: InvokeRequest,
    ) -> BoxFuture<'static, Result<InvokeReply, InvokeError>> {
        self.prompts
            .lock()
            .unwrap()
            .push((request.cli_model.clone(), request.prompt.clone()));
        Box::pin(async move {
            Ok(InvokeReply {
                result: format!("fake answer to {}", request.prompt.trim()),
                model_id: Some("claude-sonnet-4-5-20250929".to_string()),
                usage: Some(TokenUsage {
                    input: 10,
                    output: 5,
                    cache_creation: 0,
                    cache_read: 2,
                }),
                cost_usd: Some(0.0002),
                route: "subscription".to_string(),
                account: Some("acct-a".to_string()),
                attempts: 1,
            })
        })
    }
    fn ready(&self) -> Result<(), String> {
        Ok(())
    }
    fn shim_argv(&self) -> Option<Vec<String>> {
        Some(vec![self.shim.to_string_lossy().to_string()])
    }
}

/// `(conclusion, summary.model, model_ledger)` of the one verdict filed.
type FiledVerdict = (String, serde_json::Value, Option<serde_json::Value>);

#[derive(Default, Clone)]
struct Filed {
    lines: Arc<Mutex<Vec<String>>>,
    verdict: Arc<Mutex<Option<FiledVerdict>>>,
}

struct Rec(Filed);
struct Sink(Arc<Mutex<Vec<String>>>);
impl LogSink for Sink {
    fn push(&self, line: &str) {
        self.0.lock().unwrap().push(line.to_string());
    }
}
impl Reporter for Rec {
    fn sink(&self) -> Arc<dyn LogSink> {
        Arc::new(Sink(self.0.lines.clone()))
    }
    fn report<'a>(self: Box<Self>, verdict: Verdict<'a>) -> BoxFuture<'a, ()>
    where
        Self: 'a,
    {
        Box::pin(async move {
            *self.0.verdict.lock().unwrap() = Some((
                verdict.conclusion.as_str().to_string(),
                serde_json::to_value(verdict.model).unwrap(),
                verdict
                    .model_ledger
                    .map(|l| serde_json::to_value(l).unwrap()),
            ));
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
    assert!(out.status.success(), "git {args:?}");
    String::from_utf8_lossy(&out.stdout).trim().to_string()
}

const MANIFEST: &str = r#"
version = 1

[model]
models = ["claude-sonnet"]
max_calls = 10
max_tokens = 100000
max_wall_secs = 600

[[steps]]
name = "plain"
command = ["env"]

[[steps]]
name = "llm"
command = ["sh", "-c", "$QONTINUI_CI_MODEL_CMD -p hello-from-llm"]
model = true

[[steps]]
name = "llm-json"
command = ["sh", "-c", "$QONTINUI_CI_MODEL_CMD --output-format json hello-json"]
model = true
"#;

#[tokio::test]
async fn a_model_step_reaches_the_invoker_through_the_real_shim_and_broker() {
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path().join("root");
    let repo = root.join("demo");
    std::fs::create_dir_all(repo.join(".qontinui")).unwrap();
    std::fs::write(repo.join(".qontinui").join("ci.toml"), MANIFEST).unwrap();
    git(&repo, &["init", "-q", "-b", "main"]);
    git(&repo, &["add", "."]);
    git(&repo, &["commit", "-q", "-m", "init"]);
    let head = git(&repo, &["rev-parse", "HEAD"]);

    let prompts = Arc::new(Mutex::new(Vec::new()));
    let mut host = standalone::host(root.clone(), "model-e2e".to_string());
    host.model = Some(Arc::new(Fake {
        shim: PathBuf::from(env!("CARGO_BIN_EXE_qontinui-ci-model")),
        prompts: prompts.clone(),
    }));
    let filed = Filed::default();
    let payload = DispatchPayload {
        dispatch_id: "model-e2e-1".to_string(),
        repo: "local/demo".to_string(),
        head_sha: head,
        fetch_url: repo.to_string_lossy().to_string(),
        candidate_ref: "refs/ci-dispatch/local/model-e2e-1".to_string(),
        pr_number: None,
        manifest_path: ".qontinui/ci.toml".to_string(),
        job: None,
        check_name: String::new(),
        coord_http_url: String::new(),
        model_access: Some(ModelAccessGrant {
            granted: true,
            reason: None,
            basis: Some("live".to_string()),
        }),
    };
    let conclusion = executor::run_dispatch(
        &host,
        Box::new(Rec(filed.clone())),
        payload,
        CancellationToken::new(),
        host_sizing::HostCapacity {
            mem_bytes: Some(8 << 30),
            cpus: 4,
        },
        1,
    )
    .await;
    let lines = filed.lines.lock().unwrap().clone();
    assert_eq!(conclusion, Conclusion::Success, "{lines:#?}");

    // The plain step got none of the model variables.
    let plain: Vec<&str> = lines
        .iter()
        .filter_map(|l| l.strip_prefix("[plain:out] "))
        .collect();
    assert!(plain
        .iter()
        .any(|l| l.starts_with("QONTINUI_CI_DISPATCH_ID=")));
    for var in [
        "QONTINUI_CI_MODEL_CMD=",
        "QONTINUI_CI_MODEL_URL=",
        "QONTINUI_CI_MODEL_TOKEN=",
    ] {
        assert!(
            !plain.iter().any(|l| l.starts_with(var)),
            "{var} leaked to a plain step"
        );
    }

    // The model steps reached the invoker through shim → broker.
    assert!(
        lines.contains(&"[llm:out] fake answer to hello-from-llm".to_string()),
        "{lines:#?}"
    );
    let json_line = lines
        .iter()
        .find_map(|l| l.strip_prefix("[llm-json:out] "))
        .expect("json output");
    let env: serde_json::Value = serde_json::from_str(json_line).unwrap();
    assert_eq!(env["result"], "fake answer to hello-json");
    assert_eq!(env["usage"]["cache_read_input_tokens"], 2);
    assert_eq!(
        prompts.lock().unwrap().clone(),
        [
            ("sonnet".to_string(), "hello-from-llm".to_string()),
            ("sonnet".to_string(), "hello-json".to_string()),
        ]
    );
    assert!(lines
        .iter()
        .any(|l| l.contains("[ci-model] call 1 step=llm")));

    // The verdict: granted, with a ledger attributed per step.
    let (_, section, ledger) = filed.verdict.lock().unwrap().clone().unwrap();
    assert_eq!(section["status"], "granted");
    assert_eq!(section["calls"], 2);
    let ledger = ledger.expect("a ledger");
    assert_eq!(ledger["isolation"], "env");
    assert_eq!(ledger["calls"][0]["step"], "llm");
    assert_eq!(ledger["calls"][1]["step"], "llm-json");
    assert_eq!(ledger["budget"]["tokens_used"], 34);
    assert_eq!(ledger["by_model"]["claude-sonnet-4-5-20250929"]["calls"], 2);
    assert_eq!(ledger["cost"]["total_microusd"], 400);
    assert!(
        !root.join(".ci-worktrees").join("model-e2e-1").exists(),
        "cleaned up"
    );
}
