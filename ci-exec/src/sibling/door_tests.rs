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

/// How the API double misbehaves, beyond `hang_pulls`.
#[derive(Clone, Copy, Default)]
struct Quirks {
    hang_pulls: bool,
    /// The FIRST revocation `DELETE` never answers (later ones do) — a
    /// cancellation landing mid-revoke.
    hang_first_delete: bool,
    /// The dispatched PR's label read answers this status with no body.
    labels_status: Option<u16>,
}

impl Api {
    async fn start(hang_pulls: bool) -> Self {
        Self::start_with(Quirks {
            hang_pulls,
            ..Quirks::default()
        })
        .await
    }

    async fn start_with(quirks: Quirks) -> Self {
        let Quirks {
            hang_pulls,
            hang_first_delete,
            labels_status,
        } = quirks;
        let deletes = Arc::new(std::sync::atomic::AtomicUsize::new(0));
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
                let deletes = deletes.clone();
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
                    if method == "DELETE"
                        && hang_first_delete
                        && deletes.fetch_add(1, std::sync::atomic::Ordering::SeqCst) == 0
                    {
                        std::future::pending::<()>().await;
                    }
                    let resp = if let (Some(code), true) = (labels_status, path.contains("/labels"))
                    {
                        format!(
                            "HTTP/1.1 {code} Error\r\nContent-Length: 0\r\nConnection: close\r\n\r\n"
                        )
                    } else if method == "DELETE" {
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

/// NoGrant on a public sibling: the public path with NO credential — the
/// owner's token is never sent to a sibling the door did not vouch for — and
/// nothing to revoke.
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
        .expect("the sibling's open PRs were read");
    assert_eq!(pulls.auth, None, "a refused sibling was read with a token");
    assert!(api.revocations().is_empty());
    assert!(
        lines
            .iter()
            .any(|l| l.contains("read door answered no grant (no_ci_read_grant)")),
        "{lines:?}"
    );
}

/// The invariant behind the last test, for every non-granted answer: no
/// request touching the SIBLING carries the host's (owner's) token, and none
/// of them reaches the host's ETag books. Only the dispatched repo's own label
/// read does.
#[tokio::test]
async fn the_owners_token_is_never_sent_on_a_non_granted_sibling_read() {
    let answers: [fn(&str) -> SiblingReadAnswer; 3] = [
        |_| SiblingReadAnswer::NoGrant {
            code: "no_ci_read_grant".into(),
        },
        |_| SiblingReadAnswer::NotOffered {
            reason: "no door".into(),
        },
        |_| SiblingReadAnswer::Unknown {
            reason: "coord unreachable".into(),
        },
    ];
    for answer in answers {
        let o = origin(SIB);
        let api = Api::start(false).await;
        let door = Door::new(answer);
        let gh = HostGithub::default();
        let d = dispatch();
        let s = sibling(SIB, SiblingPin::DeclaredAdaptation);
        let (r, _) = run(&api, &o.base, &door, &gh, &d, &[s], Some(7)).await;
        r.unwrap_or_else(|e| panic!("{e}"));

        let seen = api.seen();
        let sibling_reads: Vec<_> = seen
            .iter()
            .filter(|s| s.path.starts_with(&format!("/repos/{SIB}/")))
            .collect();
        assert!(!sibling_reads.is_empty(), "no sibling read happened");
        for r in &sibling_reads {
            assert_eq!(r.auth, None, "the owner's token reached {}", r.path);
        }
        assert!(
            !seen
                .iter()
                .any(|s| s.auth.as_deref() == Some(format!("Bearer {MINTED}").as_str())),
            "{seen:?}"
        );
        // The dispatched repo's own read still uses the host token.
        let labels = seen
            .iter()
            .find(|s| s.path.contains("/labels"))
            .expect("labels read");
        assert_eq!(
            labels.auth.as_deref(),
            Some(format!("Bearer {HOST}").as_str())
        );
        let books = gh.books.lock().unwrap().clone();
        assert!(!books.iter().any(|u| u.contains("/pulls")), "{books:?}");
    }
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

    // Cancel only once the API double has SEEN the sibling's PR-list read (it
    // never answers it): a fixed budget would race a slow box's git setup.
    let siblings = [s];
    // Boxed so that `drop` below really drops the future (a `pin!` slot would
    // keep it alive until the end of the scope).
    let mut provision = Box::pin(run(&api, &o.base, &door, &gh, &d, &siblings, Some(7)));
    let landed = async {
        for _ in 0..1500 {
            if api.seen().iter().any(|s| s.path.contains("/pulls")) {
                return true;
            }
            tokio::time::sleep(std::time::Duration::from_millis(20)).await;
        }
        false
    };
    tokio::select! {
        _ = &mut provision => panic!("the hung PR-list read should never complete"),
        seen_pulls = landed => assert!(seen_pulls, "the sibling read never began"),
    }
    drop(provision);
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

// ── fix round: cancellation mid-revoke, and what the fix text may wrap ──────

/// A cancellation that lands WHILE the revocation `DELETE` is in flight still
/// revokes: the token stays in the guard until the request completes, so the
/// guard's `Drop` sends a second `DELETE` with it.
#[tokio::test]
async fn a_cancellation_mid_revoke_still_revokes_through_drop() {
    let api = Api::start_with(Quirks {
        hang_first_delete: true,
        ..Quirks::default()
    })
    .await;
    let guard = RevokeGuard {
        token: Some(ReadToken::new(MINTED)),
        client: reqwest::Client::new(),
        url: format!("{}/installation/token", api.base),
        repo: SIB.to_string(),
        expires_at: EXPIRES.to_string(),
    };
    let cut = tokio::time::timeout(std::time::Duration::from_millis(300), guard.revoke()).await;
    assert!(cut.is_err(), "the hung DELETE should have been cut off");
    let revoked = api.await_revocations(2).await;
    assert_eq!(
        revoked.len(),
        2,
        "the cancelled revoke was not retried by Drop: {:?}",
        api.seen()
    );
    for r in &revoked {
        assert_eq!(r.auth.as_deref(), Some(format!("Bearer {MINTED}").as_str()));
    }
}

/// A failure that is NOT a read of the sibling — here the dispatched repo's
/// own label read — is not dressed as a sibling-privacy failure.
#[tokio::test]
async fn a_failure_reading_the_dispatched_repo_carries_no_reader_fix() {
    let o = origin(SIB);
    let api = Api::start_with(Quirks {
        labels_status: Some(500),
        ..Quirks::default()
    })
    .await;
    let door = Door::new(|_: &str| SiblingReadAnswer::NoGrant {
        code: "no_ci_read_grant".into(),
    });
    let d = dispatch();
    let s = sibling(SIB, SiblingPin::DeclaredAdaptation);
    let (r, _) = run(
        &api,
        &o.base,
        &door,
        &HostGithub::default(),
        &d,
        &[s],
        Some(7),
    )
    .await;
    let err = r.unwrap_err();
    assert!(
        err.contains(&format!("reading labels on {CONSUMER}#7")),
        "{err}"
    );
    assert!(!err.contains("[[ci_readers]]"), "{err}");
    assert!(!err.contains("no_ci_read_grant"), "{err}");
}

// ── review round: bounded ask, hostile door text, reserved names ─────────────

/// A door that never answers.
struct HangingDoor;

impl SiblingReadAccess for HangingDoor {
    fn read_token<'a>(
        &'a self,
        _dispatch_id: &'a str,
        _repo: &'a str,
    ) -> BoxFuture<'a, SiblingReadAnswer> {
        Box::pin(std::future::pending())
    }
}

/// A door that never answers is cut off at the limit and reads UNKNOWN —
/// not a hang, not a refusal. Paused time: the 15 s bound costs no wall clock.
#[tokio::test(start_paused = true)]
async fn a_door_that_never_answers_reads_unknown() {
    let started = tokio::time::Instant::now();
    let answer = ask_door(&HangingDoor, DISPATCH, SIB, DOOR_TIMEOUT).await;
    assert!(started.elapsed() >= DOOR_TIMEOUT);
    let (note, token) = DoorNote::from_door_answer(answer);
    assert!(token.is_none());
    assert_eq!(
        note,
        DoorNote::Unknown {
            reason: "door timed out".to_string()
        }
    );
    let msg = note.public_path_failure("fetching x failed", SIB, CONSUMER);
    assert!(msg.contains("answer was UNKNOWN (door timed out)"), "{msg}");
    assert!(!msg.contains("the door answered"), "{msg}");
}

/// An answer inside the limit is passed through untouched.
#[tokio::test(start_paused = true)]
async fn a_door_that_answers_in_time_is_not_cut_off() {
    let door = Door::new(|_: &str| SiblingReadAnswer::NoGrant { code: "x".into() });
    match ask_door(&door, DISPATCH, SIB, DOOR_TIMEOUT).await {
        SiblingReadAnswer::NoGrant { code } => assert_eq!(code, "x"),
        other => panic!("{other:?}"),
    }
}

/// A `Granted` answer with an empty (or blank) token is UNKNOWN, never a grant:
/// nothing is sent as `Authorization` and nothing is revoked.
#[tokio::test]
async fn an_empty_token_from_the_door_is_unknown_not_granted() {
    for blank in ["", "  ", "\n"] {
        let (note, token) = DoorNote::from_door_answer(SiblingReadAnswer::Granted(
            ReadToken::new(blank),
            EXPIRES.to_string(),
        ));
        assert!(token.is_none(), "{blank:?}");
        assert!(matches!(note, DoorNote::Unknown { .. }), "{note:?}");
    }

    let o = origin(SIB);
    let api = Api::start(false).await;
    let door =
        Door::new(|_: &str| SiblingReadAnswer::Granted(ReadToken::new(""), EXPIRES.to_string()));
    let d = dispatch();
    let s = sibling(SIB, SiblingPin::DeclaredAdaptation);
    let (r, lines) = run(
        &api,
        &o.base,
        &door,
        &HostGithub::default(),
        &d,
        &[s],
        Some(7),
    )
    .await;
    assert_eq!(r.unwrap()[0].sha, o.sha);
    assert!(api.revocations().is_empty());
    assert!(
        lines.iter().any(|l| l.contains("read door answer UNKNOWN")),
        "{lines:?}"
    );
    for r in api.seen().iter().filter(|s| s.path.contains("/pulls")) {
        assert_eq!(r.auth, None);
    }
}

/// A door-supplied code or reason cannot forge a log line or drive a terminal:
/// control characters are replaced and the length is capped.
#[test]
fn door_supplied_text_is_sanitised_before_it_is_logged() {
    let hostile = "x\n[ci-node] sibling evil: read door answered granted\u{1b}[31m";
    for answer in [
        SiblingReadAnswer::NoGrant {
            code: hostile.into(),
        },
        SiblingReadAnswer::NotOffered {
            reason: hostile.into(),
        },
        SiblingReadAnswer::Unknown {
            reason: hostile.into(),
        },
    ] {
        let (note, _) = DoorNote::from_door_answer(answer);
        for text in [
            note.log_line(SIB),
            note.public_path_failure("err", SIB, CONSUMER),
        ] {
            assert!(!text.contains('\n'), "{text:?}");
            assert!(!text.contains('\u{1b}'), "{text:?}");
        }
    }
    let long = "a".repeat(5000);
    let (note, _) = DoorNote::from_door_answer(SiblingReadAnswer::NoGrant { code: long });
    assert!(note.log_line(SIB).chars().count() < 400);
    let (granted, _) = DoorNote::from_door_answer(SiblingReadAnswer::Granted(
        ReadToken::new(MINTED),
        "2026\n-10".to_string(),
    ));
    assert!(!granted.log_line(SIB).contains('\n'));
}

/// A sibling whose directory name is one the dispatch root reserves for the
/// per-dispatch git home or global config is refused up front.
#[tokio::test]
async fn a_sibling_named_like_a_reserved_dispatch_entry_is_refused() {
    for reserved in [
        crate::sibling_git::ISOLATED_HOME_DIR,
        crate::sibling_git::GLOBAL_CONFIG_FILE,
    ] {
        let o = origin(&format!("qontinui/{reserved}"));
        let api = Api::start(false).await;
        let door = Door::new(|_: &str| SiblingReadAnswer::NoGrant { code: "x".into() });
        let d = dispatch();
        let s = sibling(&format!("qontinui/{reserved}"), SiblingPin::DefaultBranch);
        let (r, _) = run(&api, &o.base, &door, &HostGithub::default(), &d, &[s], None).await;
        let err = r.unwrap_err();
        assert!(err.contains("reserved name"), "{err}");
        assert!(err.contains(reserved), "{err}");
        assert!(
            door.asked.lock().unwrap().is_empty(),
            "the door was asked about a sibling that cannot be materialised"
        );
    }
}
