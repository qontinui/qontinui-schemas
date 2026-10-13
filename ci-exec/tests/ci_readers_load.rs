//! A `[[ci_readers]]` grant in a repo's `.qontinui/ci.toml` must not break the
//! dispatch it rides in: the manifest is read from the checked-out tree by the
//! executor's own loader (`run_dispatch` -> `load_manifest`), and the job runs
//! exactly as it would without the grant.
//!
//! Drives only the crate's pre-existing public entry point, so it builds with
//! the `[[ci_readers]]` parsing absent — and is then red, because the
//! manifest's `deny_unknown_fields` refuses the table (`[setup]
//! manifest=failure`).

use std::path::Path;
use std::process::Command;
use std::sync::{Arc, Mutex};

use qontinui_ci_exec::dispatch::DispatchPayload;
use qontinui_ci_exec::executor::run_dispatch;
use qontinui_ci_exec::host::BoxFuture;
use qontinui_ci_exec::host_sizing::HostCapacity;
use qontinui_ci_exec::report::{Conclusion, LogSink, Reporter, Verdict};
use tokio_util::sync::CancellationToken;

type Filed = Arc<Mutex<Vec<(Conclusion, Vec<String>)>>>;

struct Lines(Arc<Mutex<Vec<String>>>);
impl LogSink for Lines {
    fn push(&self, line: &str) {
        self.0.lock().unwrap().push(line.to_string());
    }
}

struct Recording {
    lines: Arc<Mutex<Vec<String>>>,
    filed: Filed,
}

impl Reporter for Recording {
    fn sink(&self) -> Arc<dyn LogSink> {
        Arc::new(Lines(self.lines.clone()))
    }
    fn report<'a>(self: Box<Self>, verdict: Verdict<'a>) -> BoxFuture<'a, ()>
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
            ));
        })
    }
}

fn git(dir: &Path, args: &[&str]) -> String {
    let out = Command::new("git")
        .args(args)
        .current_dir(dir)
        .env("GIT_AUTHOR_NAME", "t")
        .env("GIT_AUTHOR_EMAIL", "t@t")
        .env("GIT_COMMITTER_NAME", "t")
        .env("GIT_COMMITTER_EMAIL", "t@t")
        .output()
        .unwrap();
    assert!(
        out.status.success(),
        "git {args:?}: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    String::from_utf8_lossy(&out.stdout).trim().to_string()
}

const MANIFEST_WITH_READERS: &str = r#"
version = 2
[[jobs]]
name = "green"
[[jobs.steps]]
name = "git-version"
command = ["git", "--version"]

[[ci_readers]]
repo = "qontinui-docs-evals"
"#;

#[tokio::test]
async fn a_ci_readers_grant_loads_and_the_job_runs_unchanged() {
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path().join("root");
    let repo = root.join("demo");
    std::fs::create_dir_all(repo.join(".qontinui")).unwrap();
    std::fs::write(
        repo.join(".qontinui").join("ci.toml"),
        MANIFEST_WITH_READERS,
    )
    .unwrap();
    git(&repo, &["init", "-q", "-b", "main"]);
    git(&repo, &["add", "."]);
    git(&repo, &["commit", "-q", "-m", "init"]);
    let head = git(&repo, &["rev-parse", "HEAD"]);

    let host = qontinui_ci_exec::standalone::host(root.clone(), "test-host".to_string());
    let lines = Arc::new(Mutex::new(Vec::new()));
    let filed: Filed = Arc::new(Mutex::new(Vec::new()));
    let reporter = Box::new(Recording {
        lines: lines.clone(),
        filed: filed.clone(),
    });
    let payload = DispatchPayload {
        dispatch_id: "ci-readers-1".to_string(),
        repo: "local/demo".to_string(),
        head_sha: head,
        fetch_url: repo.to_string_lossy().replace('\\', "/"),
        candidate_ref: "refs/ci-dispatch/local/ci-readers-1".to_string(),
        pr_number: None,
        manifest_path: ".qontinui/ci.toml".to_string(),
        job: Some("green".to_string()),
        check_name: String::new(),
        coord_http_url: String::new(),
    };
    let capacity = HostCapacity {
        mem_bytes: Some(8 * 1024 * 1024 * 1024),
        cpus: 4,
    };
    let conclusion = run_dispatch(
        &host,
        reporter,
        payload,
        CancellationToken::new(),
        capacity,
        1,
    )
    .await;

    let filed = filed.lock().unwrap().clone();
    assert_eq!(filed.len(), 1, "exactly one verdict");
    assert_eq!(
        filed[0].1,
        [
            "[setup] checkout=success",
            "[setup] provision=success",
            "git-version=success"
        ],
        "a [[ci_readers]] grant must load and leave the job's steps unchanged; log: {:#?}",
        lines.lock().unwrap()
    );
    assert_eq!(conclusion, Conclusion::Success);
}
