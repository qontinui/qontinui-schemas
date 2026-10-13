//! The read door's consumer (plan
//! `2026-10-13-local-lane-ci-reads-a-private-sibling-through-a-dispatch-scoped-door`,
//! Phase 5): each answer's path, the UNKNOWN-then-public-failure message,
//! revocation on success, fetch failure and cancellation, and a granted
//! sibling's API reads carrying the minted token rather than the host's.
//!
//! Driven end to end through [`provision_at`] against three local doubles: a
//! [`SiblingReadAccess`] double, a loopback HTTP server standing in for the
//! GitHub API (it records every request's method, path and `Authorization`),
//! and a `file://` git remote standing in for github.com.

use std::sync::{Arc, Mutex};

use super::*;
use crate::host::BoxFuture;
use crate::sibling_git::ReadToken;
use crate::standalone::{NoSiblingDoor, PlainSpawn};

const MINTED: &str = "ghs_MINTEDsiblingTOKEN0123";
const HOST: &str = "gho_HOSTtokenVALUE";
const CONSUMER: &str = "qontinui/consumer";
const SIB: &str = "qontinui/private-ref";
const EXPIRES: &str = "2026-10-13T19:00:00Z";
const DISPATCH: &str = "d-0198f2b4";

// ── doubles ──────────────────────────────────────────────────────────────

/// A read door that answers from a closure and records every ask.
struct Door<F: Fn(&str) -> SiblingReadAnswer + Send + Sync> {
    answer: F,
    asked: Mutex<Vec<(String, String)>>,
}

impl<F: Fn(&str) -> SiblingReadAnswer + Send + Sync> Door<F> {
    fn new(answer: F) -> Self {
        Self {
            answer,
            asked: Mutex::new(Vec::new()),
        }
    }
}

impl<F: Fn(&str) -> SiblingReadAnswer + Send + Sync> SiblingReadAccess for Door<F> {
    fn read_token<'a>(
        &'a self,
        dispatch_id: &'a str,
        repo: &'a str,
    ) -> BoxFuture<'a, SiblingReadAnswer> {
        self.asked
            .lock()
            .unwrap()
            .push((dispatch_id.to_string(), repo.to_string()));
        let a = (self.answer)(repo);
        Box::pin(async move { a })
    }
}

fn granted(_: &str) -> SiblingReadAnswer {
    SiblingReadAnswer::Granted(ReadToken::new(MINTED), EXPIRES.to_string())
}

/// The host's GitHub access: a host token, and a record of which URLs touched
/// its ETag cache.
#[derive(Default)]
struct HostGithub {
    books: Mutex<Vec<String>>,
}

impl GithubAccess for HostGithub {
    fn token(&self) -> BoxFuture<'_, Option<String>> {
        Box::pin(async { Some(HOST.to_string()) })
    }
    fn cached_etag(&self, url: &str) -> Option<String> {
        self.books.lock().unwrap().push(url.to_string());
        None
    }
    fn replay(&self, url: &str) -> Option<Vec<u8>> {
        self.books.lock().unwrap().push(url.to_string());
        None
    }
    fn store(&self, url: &str, _etag: &str, _body: &[u8]) {
        self.books.lock().unwrap().push(url.to_string());
    }
    fn invalidate(&self, url: &str) {
        self.books.lock().unwrap().push(url.to_string());
    }
    fn record_response(&self, url: &str, _status: u16, _headers: &reqwest::header::HeaderMap) {
        self.books.lock().unwrap().push(url.to_string());
    }
    fn record_transport_error(&self, url: &str) {
        self.books.lock().unwrap().push(url.to_string());
    }
}

#[derive(Debug, Clone)]
struct Seen {
    method: String,
    path: String,
    auth: Option<String>,
}

/// A loopback stand-in for api.github.com: `[]` for every GET, `204` for the
/// revocation `DELETE`. With `hang_pulls`, a sibling PR-list read never
/// answers — the shape of a provision cancelled mid-flight.
struct Api {
    base: String,
    seen: Arc<Mutex<Vec<Seen>>>,
}

impl Api {
    async fn start(hang_pulls: bool) -> Self {
        use tokio::io::{AsyncReadExt, AsyncWriteExt};
        let listener = tokio::net::TcpListener::bind(("127.0.0.1", 0))
            .await
            .unwrap();
        let base = format!("http://{}", listener.local_addr().unwrap());
        let seen: Arc<Mutex<Vec<Seen>>> = Arc::default();
        let record = seen.clone();
        tokio::spawn(async move {
            loop {
                let Ok((mut conn, _)) = listener.accept().await else {
                    return;
                };
                let record = record.clone();
                tokio::spawn(async move {
                    let mut buf = Vec::new();
                    let mut chunk = [0u8; 4096];
                    while !buf.windows(4).any(|w| w == b"\r\n\r\n") {
                        match conn.read(&mut chunk).await {
                            Ok(0) | Err(_) => return,
                            Ok(n) => buf.extend_from_slice(&chunk[..n]),
                        }
                    }
                    let head = String::from_utf8_lossy(&buf).into_owned();
                    let mut lines = head.lines();
                    let mut first = lines.next().unwrap_or_default().split(' ');
                    let method = first.next().unwrap_or_default().to_string();
                    let path = first.next().unwrap_or_default().to_string();
                    let auth = lines.find_map(|l| {
                        let (k, v) = l.split_once(':')?;
                        k.eq_ignore_ascii_case("authorization")
                            .then(|| v.trim().to_string())
                    });
                    record.lock().unwrap().push(Seen {
                        method: method.clone(),
                        path: path.clone(),
                        auth,
                    });
                    if hang_pulls && path.contains("/pulls") {
                        std::future::pending::<()>().await;
                    }
                    let resp = if method == "DELETE" {
                        "HTTP/1.1 204 No Content\r\nConnection: close\r\n\r\n".to_string()
                    } else {
                        "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\n\
                         Content-Length: 2\r\nConnection: close\r\n\r\n[]"
                            .to_string()
                    };
                    let _ = conn.write_all(resp.as_bytes()).await;
                    let _ = conn.shutdown().await;
                });
            }
        });
        Self { base, seen }
    }

    fn seen(&self) -> Vec<Seen> {
        self.seen.lock().unwrap().clone()
    }

    fn revocations(&self) -> Vec<Seen> {
        self.seen()
            .into_iter()
            .filter(|s| s.method == "DELETE" && s.path == "/installation/token")
            .collect()
    }

    /// Wait (bounded) for `n` revocations — the cancelled path revokes on a
    /// spawned task.
    async fn await_revocations(&self, n: usize) -> Vec<Seen> {
        for _ in 0..250 {
            let r = self.revocations();
            if r.len() >= n {
                return r;
            }
            tokio::time::sleep(std::time::Duration::from_millis(20)).await;
        }
        self.revocations()
    }
}

/// A `file://` git remote holding `repo` with one commit on `main`.
struct Origin {
    _dir: tempfile::TempDir,
    base: String,
    sha: String,
}

fn git_setup(args: &[&str]) -> String {
    let out = std::process::Command::new("git")
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

fn origin(repo: &str) -> Origin {
    let dir = tempfile::tempdir().unwrap();
    let bare = dir.path().join("origin").join(format!("{repo}.git"));
    let work = dir.path().join("work");
    std::fs::create_dir_all(&bare).unwrap();
    let (bare_s, work_s) = (
        bare.to_string_lossy().into_owned(),
        work.to_string_lossy().into_owned(),
    );
    git_setup(&["init", "-q", "--bare", &bare_s]);
    git_setup(&["init", "-q", &work_s]);
    git_setup(&["-C", &work_s, "commit", "-q", "--allow-empty", "-m", "x"]);
    git_setup(&["-C", &work_s, "push", "-q", &bare_s, "HEAD:refs/heads/main"]);
    let sha = git_setup(&["-C", &work_s, "rev-parse", "HEAD"]);
    Origin {
        base: format!("file://{}", dir.path().join("origin").display()),
        _dir: dir,
        sha,
    }
}

fn sibling(repo: &str, pin: SiblingPin) -> CiSibling {
    let text = format!(
        "version = 1\n[[siblings]]\nrepo = {repo:?}\n\
         [[steps]]\nname = \"x\"\ncommand = [\"true\"]\n"
    );
    let mut s = crate::manifest::parse_and_validate(&text)
        .unwrap()
        .siblings
        .remove(0);
    s.pin = pin;
    s
}

/// One dispatch's layout: a dispatch root holding the consumer's worktree.
struct Dispatch {
    _dir: tempfile::TempDir,
    root: PathBuf,
    worktree: PathBuf,
}

fn dispatch() -> Dispatch {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path().join(".ci-worktrees").join(DISPATCH);
    let worktree = root.join("consumer");
    std::fs::create_dir_all(&worktree).unwrap();
    Dispatch {
        _dir: dir,
        root,
        worktree,
    }
}

async fn run(
    api: &Api,
    origin_base: &str,
    door: &dyn SiblingReadAccess,
    gh: &HostGithub,
    d: &Dispatch,
    siblings: &[CiSibling],
    pr: Option<u64>,
) -> (Result<Vec<ResolvedSibling>, String>, Vec<String>) {
    let endpoints = Endpoints {
        api: api.base.clone(),
        clone: origin_base.to_string(),
    };
    let mut lines = Vec::new();
    let mut log = |l: String| lines.push(l);
    let r = provision_at(
        &endpoints,
        gh,
        &PlainSpawn,
        door,
        DISPATCH,
        &d.root,
        &d.worktree,
        siblings,
        CONSUMER,
        pr,
        &mut log,
    )
    .await;
    (r, lines)
}

fn no_line_carries_the_token(lines: &[String]) {
    for l in lines {
        assert!(!l.contains(MINTED), "a log line carries the token: {l}");
    }
}

// ── the answers ──────────────────────────────────────────────────────────

/// Granted: the sibling's own API reads carry the MINTED token and stay out of
/// the host's ETag books; the dispatched repo's label read keeps the HOST
/// token; the sibling materialises at its resolved sha with no credential in
/// its remote; and the token is revoked, once, with itself.
#[tokio::test]
async fn a_granted_sibling_reads_with_the_minted_token_and_revokes_it() {
    let o = origin(SIB);
    let api = Api::start(false).await;
    let door = Door::new(granted);
    let gh = HostGithub::default();
    let d = dispatch();
    let s = sibling(SIB, SiblingPin::DeclaredAdaptation);

    let (r, lines) = run(
        &api,
        &o.base,
        &door,
        &gh,
        &d,
        std::slice::from_ref(&s),
        Some(7),
    )
    .await;
    let resolved = r.unwrap_or_else(|e| panic!("{e}"));
    assert_eq!(resolved[0].sha, o.sha);

    let seen = api.seen();
    let labels = seen
        .iter()
        .find(|s| {
            s.path
                .starts_with(&format!("/repos/{CONSUMER}/issues/7/labels"))
        })
        .expect("the dispatched PR's labels were read");
    assert_eq!(
        labels.auth.as_deref(),
        Some(format!("Bearer {HOST}").as_str())
    );
    let pulls = seen
        .iter()
        .find(|s| s.path.starts_with(&format!("/repos/{SIB}/pulls")))
        .expect("the sibling's open PRs were read");
    assert_eq!(
        pulls.auth.as_deref(),
        Some(format!("Bearer {MINTED}").as_str())
    );
    // The host's books never saw the sibling's private read.
    let books = gh.books.lock().unwrap().clone();
    assert!(!books.iter().any(|u| u.contains("/pulls")), "{books:?}");
    assert!(books.iter().any(|u| u.contains("/labels")), "{books:?}");

    let revoked = api.revocations();
    assert_eq!(revoked.len(), 1, "{seen:?}");
    assert_eq!(
        revoked[0].auth.as_deref(),
        Some(format!("Bearer {MINTED}").as_str())
    );

    let dest = sibling_dest(&d.root, &s);
    let remote = git_setup(&["-C", &dest.to_string_lossy(), "remote", "get-url", "origin"]);
    assert!(
        !remote.contains(MINTED) && !remote.contains('@'),
        "{remote}"
    );
    let config = std::fs::read_to_string(dest.join(".git").join("config")).unwrap();
    assert!(!config.contains(MINTED), "the token reached .git/config");

    assert_eq!(
        door.asked.lock().unwrap().clone(),
        vec![(DISPATCH.to_string(), SIB.to_string())]
    );
    assert!(
        lines
            .iter()
            .any(|l| l.contains("read door answered granted")
                && l.contains(EXPIRES)
                && l.contains(SIB)),
        "{lines:?}"
    );
    assert!(
        lines.iter().any(|l| l.contains("read token revoked")),
        "{lines:?}"
    );
    no_line_carries_the_token(&lines);
}

/// NoGrant on a public sibling: the public path, with the host's token and
/// books, and nothing to revoke.
#[tokio::test]
async fn a_refused_sibling_reads_on_the_public_path() {
    let o = origin(SIB);
    let api = Api::start(false).await;
    let door = Door::new(|_: &str| SiblingReadAnswer::NoGrant {
        code: "no_ci_read_grant".into(),
    });
    let gh = HostGithub::default();
    let d = dispatch();
    let s = sibling(SIB, SiblingPin::DeclaredAdaptation);

    let (r, lines) = run(&api, &o.base, &door, &gh, &d, &[s], Some(7)).await;
    assert_eq!(r.unwrap()[0].sha, o.sha);
    let pulls = api
        .seen()
        .into_iter()
        .find(|s| s.path.contains("/pulls"))
        .unwrap();
    assert_eq!(
        pulls.auth.as_deref(),
        Some(format!("Bearer {HOST}").as_str())
    );
    assert!(api.revocations().is_empty());
    assert!(
        lines
            .iter()
            .any(|l| l.contains("read door answered no grant (no_ci_read_grant)")),
        "{lines:?}"
    );
}

/// A public-path failure under NoGrant names the fix and the door's code.
#[tokio::test]
async fn a_public_path_failure_names_the_fix() {
    let o = origin(SIB);
    let api = Api::start(false).await;
    let door = Door::new(|_: &str| SiblingReadAnswer::NoGrant {
        code: "no_ci_read_grant".into(),
    });
    let d = dispatch();
    let absent = sibling("qontinui/absent", SiblingPin::DefaultBranch);

    let (r, _) = run(
        &api,
        &o.base,
        &door,
        &HostGithub::default(),
        &d,
        &[absent],
        None,
    )
    .await;
    let err = r.unwrap_err();
    assert!(
        err.contains(
            "if qontinui/absent is private, its default branch must declare [[ci_readers]] \
             repo = \"qontinui/consumer\"; the door answered no_ci_read_grant"
        ),
        "{err}"
    );
}

/// NotOffered (a standalone host): the public path, and a failure says this
/// host has no door rather than blaming a missing grant alone.
#[tokio::test]
async fn not_offered_reads_publicly_and_says_there_is_no_door() {
    let o = origin(SIB);
    let api = Api::start(false).await;
    let d = dispatch();
    let s = sibling(SIB, SiblingPin::DefaultBranch);
    let (r, lines) = run(
        &api,
        &o.base,
        &NoSiblingDoor,
        &HostGithub::default(),
        &d,
        &[s],
        None,
    )
    .await;
    assert_eq!(r.unwrap()[0].sha, o.sha);
    assert!(
        lines
            .iter()
            .any(|l| l.contains("read door not offered (standalone host has no coord door)")),
        "{lines:?}"
    );

    let d = dispatch();
    let absent = sibling("qontinui/absent", SiblingPin::DefaultBranch);
    let (r, _) = run(
        &api,
        &o.base,
        &NoSiblingDoor,
        &HostGithub::default(),
        &d,
        &[absent],
        None,
    )
    .await;
    let err = r.unwrap_err();
    assert!(err.contains("offers no sibling read door"), "{err}");
    assert!(
        err.contains("[[ci_readers]] repo = \"qontinui/consumer\""),
        "{err}"
    );
    assert!(api.revocations().is_empty());
}

/// UNKNOWN, then a public failure: the error says the door's answer was
/// UNKNOWN and why, and never renders it as a refusal.
#[tokio::test]
async fn an_unknown_door_then_a_public_failure_says_unknown_not_no_grant() {
    let o = origin(SIB);
    let api = Api::start(false).await;
    let door = Door::new(|_: &str| SiblingReadAnswer::Unknown {
        reason: "coord unreachable: connection refused".into(),
    });
    let d = dispatch();
    let absent = sibling("qontinui/absent", SiblingPin::DefaultBranch);
    let (r, lines) = run(
        &api,
        &o.base,
        &door,
        &HostGithub::default(),
        &d,
        &[absent],
        None,
    )
    .await;
    let err = r.unwrap_err();
    assert!(
        err.contains("answer was UNKNOWN (coord unreachable: connection refused)"),
        "{err}"
    );
    assert!(
        !err.contains("the door answered"),
        "rendered as a refusal: {err}"
    );
    assert!(!err.contains("no_ci_read_grant"), "{err}");
    assert!(
        lines.iter().any(|l| l.contains("read door answer UNKNOWN")),
        "{lines:?}"
    );
}

// ── revocation on every exit path ────────────────────────────────────────

/// A granted sibling whose fetch FAILS is still revoked.
#[tokio::test]
async fn a_granted_token_is_revoked_when_the_fetch_fails() {
    let o = origin(SIB);
    let api = Api::start(false).await;
    let door = Door::new(granted);
    let d = dispatch();
    // A pin naming a commit the remote does not have: resolution succeeds
    // (no network), the fetch in `materialise` fails.
    let pins = d.worktree.join(SIBLING_PIN_FILE);
    std::fs::create_dir_all(pins.parent().unwrap()).unwrap();
    std::fs::write(&pins, format!("{SIB} {}\n", "ab".repeat(20))).unwrap();
    let s = sibling(SIB, SiblingPin::PinFile);

    let (r, lines) = run(&api, &o.base, &door, &HostGithub::default(), &d, &[s], None).await;
    let err = r.unwrap_err();
    assert!(err.contains("fetching"), "{err}");
    let revoked = api.revocations();
    assert_eq!(revoked.len(), 1);
    assert_eq!(
        revoked[0].auth.as_deref(),
        Some(format!("Bearer {MINTED}").as_str())
    );
    assert!(
        lines.iter().any(|l| l.contains("read token revoked")),
        "{lines:?}"
    );
    // A granted failure is not dressed up as a public-path failure.
    assert!(!err.contains("[[ci_readers]]"), "{err}");
    no_line_carries_the_token(&lines);
}

/// A provision CANCELLED mid-read (its future dropped) still revokes, through
/// the guard's `Drop`.
#[tokio::test]
async fn a_granted_token_is_revoked_when_the_provision_is_cancelled() {
    let o = origin(SIB);
    let api = Api::start(true).await;
    let door = Door::new(granted);
    let d = dispatch();
    let s = sibling(SIB, SiblingPin::DeclaredAdaptation);
    let gh = HostGithub::default();

    let cut = tokio::time::timeout(
        std::time::Duration::from_millis(500),
        run(&api, &o.base, &door, &gh, &d, &[s], Some(7)),
    )
    .await;
    assert!(
        cut.is_err(),
        "the hung PR-list read should have been cut off"
    );
    assert!(
        api.seen().iter().any(|s| s.path.contains("/pulls")),
        "the cancellation landed before the sibling read began"
    );
    let revoked = api.await_revocations(1).await;
    assert_eq!(revoked.len(), 1, "{:?}", api.seen());
    assert_eq!(
        revoked[0].auth.as_deref(),
        Some(format!("Bearer {MINTED}").as_str())
    );
}

/// The door is asked ONCE per declared sibling, each by its own repo.
#[tokio::test]
async fn the_door_is_asked_once_per_declared_sibling() {
    let o = origin(SIB);
    // A second repo in the same origin.
    let second = "qontinui/other-ref";
    {
        let base_dir = o.base.strip_prefix("file://").unwrap();
        let bare = Path::new(base_dir).join(format!("{second}.git"));
        std::fs::create_dir_all(&bare).unwrap();
        git_setup(&["init", "-q", "--bare", &bare.to_string_lossy()]);
        let src = Path::new(base_dir).join(format!("{SIB}.git"));
        git_setup(&[
            "-C",
            &src.to_string_lossy(),
            "push",
            "-q",
            &bare.to_string_lossy(),
            "main",
        ]);
    }
    let api = Api::start(false).await;
    let door = Door::new(|_: &str| SiblingReadAnswer::NoGrant {
        code: "no_ci_read_grant".into(),
    });
    let d = dispatch();
    let siblings = [
        sibling(SIB, SiblingPin::DefaultBranch),
        sibling(second, SiblingPin::DefaultBranch),
    ];
    let (r, _) = run(
        &api,
        &o.base,
        &door,
        &HostGithub::default(),
        &d,
        &siblings,
        None,
    )
    .await;
    assert_eq!(r.unwrap().len(), 2);
    assert_eq!(
        door.asked.lock().unwrap().clone(),
        vec![
            (DISPATCH.to_string(), SIB.to_string()),
            (DISPATCH.to_string(), second.to_string()),
        ]
    );
}

/// A granted answer's `Debug` carries no token.
#[test]
fn a_granted_answer_debug_carries_no_token() {
    let rendered = format!("{:?}", granted(SIB));
    assert!(!rendered.contains(MINTED), "{rendered}");
    assert!(rendered.contains(EXPIRES));
}
