//! A sibling's git, entered where a dispatch enters: the public
//! `executor::run_dispatch` on the standalone node, with a manifest that
//! declares a `[[siblings]]` entry (plan
//! `2026-10-13-local-lane-ci-reads-a-private-sibling-through-a-dispatch-scoped-door`,
//! D5).
//!
//! The CI root sits inside an ENCLOSING git repository whose local config
//! rewrites the sibling's `https://github.com/<owner>/` URL to a local
//! `file://` origin — the shape of a node owner's repo-local `insteadOf`. A
//! sibling's git must never read that config: the sibling must NOT resolve to
//! the commit the rewrite points at.
//!
//! Network: this process points `https_proxy` at a closed loopback port, so the
//! un-rewritten `https://github.com/...` ls-remote fails at once instead of
//! leaving the box. That variable is not a git variable, so the sibling
//! isolation passes it through; it is set once, before any test in this binary
//! runs a child (this file holds one test).

use std::path::Path;
use std::sync::{Arc, Mutex};

use qontinui_ci_exec::dispatch::DispatchPayload;
use qontinui_ci_exec::executor::run_dispatch;
use qontinui_ci_exec::host::BoxFuture;
use qontinui_ci_exec::host_sizing::HostCapacity;
use qontinui_ci_exec::report::{Conclusion, LogSink, Reporter, Verdict};
// Aliased so the test reads as what it builds: the standalone node.
use qontinui_ci_exec::standalone::host as standalone_node;
use tokio_util::sync::CancellationToken;

const OWNER: &str = "qontinui-ci-isolation-probe";
const SIB: &str = "qontinui-ci-isolation-probe/sib";

fn sh_git(cwd: &Path, args: &[&str]) -> String {
    let out = std::process::Command::new("git")
        .current_dir(cwd)
        .args([
            "-c",
            "user.name=t",
            "-c",
            "user.email=t@t",
            "-c",
            "commit.gpgsign=false",
            "-c",
            "init.defaultBranch=main",
        ])
        .args(args)
        .output()
        .unwrap();
    assert!(out.status.success(), "git {args:?}: {out:?}");
    String::from_utf8_lossy(&out.stdout).trim().to_string()
}

struct Lines(Arc<Mutex<Vec<String>>>);
impl LogSink for Lines {
    fn push(&self, line: &str) {
        self.0.lock().unwrap().push(line.to_string());
    }
}

struct Recording(Arc<Mutex<Vec<String>>>);
impl Reporter for Recording {
    fn sink(&self) -> Arc<dyn LogSink> {
        Arc::new(Lines(self.0.clone()))
    }
    fn report<'a>(self: Box<Self>, _verdict: Verdict<'a>) -> BoxFuture<'a, ()>
    where
        Self: 'a,
    {
        Box::pin(async {})
    }
}

#[tokio::test]
async fn an_enclosing_repos_insteadof_never_resolves_a_dispatched_sibling() {
    for k in ["https_proxy", "HTTPS_PROXY", "all_proxy", "ALL_PROXY"] {
        std::env::set_var(k, "http://127.0.0.1:1");
    }
    for k in ["no_proxy", "NO_PROXY"] {
        std::env::remove_var(k);
    }

    let tmp = tempfile::tempdir().unwrap();

    // The local origin the owner's rewrite points at, with one commit.
    let origin_root = tmp.path().join("origin");
    let bare = origin_root.join(format!("{SIB}.git"));
    std::fs::create_dir_all(&bare).unwrap();
    sh_git(&bare, &["init", "-q", "--bare"]);
    let work = tmp.path().join("sib-work");
    std::fs::create_dir_all(&work).unwrap();
    sh_git(&work, &["init", "-q"]);
    sh_git(&work, &["commit", "-q", "--allow-empty", "-m", "rewritten"]);
    sh_git(
        &work,
        &[
            "push",
            "-q",
            &bare.to_string_lossy(),
            "HEAD:refs/heads/main",
        ],
    );
    let rewritten_sha = sh_git(&work, &["rev-parse", "HEAD"]);

    // The CI root, inside an enclosing repo whose LOCAL config rewrites the
    // sibling's owner to the local origin.
    let enclosing = tmp.path().join("enclosing");
    std::fs::create_dir_all(&enclosing).unwrap();
    sh_git(&enclosing, &["init", "-q"]);
    sh_git(
        &enclosing,
        &[
            "config",
            &format!(
                "url.file://{}/{OWNER}/.insteadOf",
                origin_root.to_string_lossy()
            ),
            &format!("https://github.com/{OWNER}/"),
        ],
    );
    let ci_root = enclosing.join("ci-root");
    std::fs::create_dir_all(&ci_root).unwrap();

    // The dispatched repo, declaring the sibling. Its primary checkout lives
    // at <ci root>/<name>, as on a node.
    let demo = ci_root.join("demo");
    std::fs::create_dir_all(demo.join(".qontinui")).unwrap();
    std::fs::write(
        demo.join(".qontinui").join("ci.toml"),
        format!(
            "version = 1\n[[siblings]]\nrepo = \"{SIB}\"\npin = \"default-branch\"\n\
             [[steps]]\nname = \"noop\"\ncommand = [\"git\", \"--version\"]\n"
        ),
    )
    .unwrap();
    sh_git(&demo, &["init", "-q"]);
    sh_git(&demo, &["add", "."]);
    sh_git(&demo, &["commit", "-q", "-m", "init"]);
    let head = sh_git(&demo, &["rev-parse", "HEAD"]);

    let dispatch_id = "iso-1";
    let payload = DispatchPayload {
        dispatch_id: dispatch_id.to_string(),
        repo: "local/demo".to_string(),
        head_sha: head,
        fetch_url: demo.to_string_lossy().replace('\\', "/"),
        candidate_ref: format!("refs/ci-dispatch/local/{dispatch_id}"),
        pr_number: None,
        manifest_path: ".qontinui/ci.toml".to_string(),
        job: None,
        check_name: String::new(),
        coord_http_url: String::new(),
        model_access: None,
    };
    let lines = Arc::new(Mutex::new(Vec::new()));
    let node = standalone_node(ci_root, "isolation-test".to_string());
    let conclusion = run_dispatch(
        &node,
        Box::new(Recording(lines.clone())),
        payload,
        CancellationToken::new(),
        HostCapacity {
            mem_bytes: Some(8 * 1024 * 1024 * 1024),
            cpus: 4,
        },
        1,
    )
    .await;
    let lines = lines.lock().unwrap().clone();

    // The enclosing repo's rewrite never resolved the sibling.
    assert!(
        !lines.iter().any(|l| l.contains(&rewritten_sha)),
        "the sibling resolved through the enclosing repo's insteadOf: {lines:#?}"
    );
    // It was attempted against the real URL, and failed there.
    assert!(
        lines
            .iter()
            .any(|l| l.contains(&format!("resolving {SIB}@main"))),
        "{lines:#?}"
    );
    assert_eq!(conclusion, Conclusion::Failure, "{lines:#?}");
}
