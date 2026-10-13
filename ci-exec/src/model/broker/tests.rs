//! The broker against a fake invoker: auth, budgets, exhaustion, requests,
//! cancellation and the ledger it hands back.

use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant, SystemTime};

use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use tokio::sync::Semaphore;

use super::*;
use crate::model::test_support::{declaration, ok_reply, FakeInvoker, Lines};

async fn start(
    inv: FakeInvoker,
    decl: ModelDeclaration,
    in_flight: usize,
) -> (Broker, Lines, Arc<FakeInvoker>) {
    let inv = Arc::new(inv);
    let lines = Lines::default();
    let mut cfg = BrokerConfig::new(decl);
    cfg.max_in_flight = in_flight;
    cfg.hold_log_interval = Duration::from_millis(200);
    let broker = Broker::start(inv.clone(), cfg, Arc::new(lines.clone()))
        .await
        .expect("broker starts");
    (broker, lines, inv)
}

fn http(port: u16, target: &str, auth: Option<&str>, extra: &str, body: &str) -> Vec<u8> {
    let auth = auth
        .map(|t| format!("Authorization: Bearer {t}\r\n"))
        .unwrap_or_default();
    format!(
        "POST {target} HTTP/1.1\r\nHost: 127.0.0.1:{port}\r\n{auth}{extra}Content-Type: application/json\r\nContent-Length: {}\r\n\r\n{body}",
        body.len()
    )
    .into_bytes()
}

fn prompt_body(prompt: &str) -> String {
    serde_json::json!({ "prompt": prompt }).to_string()
}

async fn call(b: &Broker, body: &str) -> (u16, Envelope) {
    let (status, text) = raw_exchange(
        b.port(),
        &http(b.port(), COMPLETE_PATH, Some(b.token()), "", body),
    )
    .await;
    let env: Envelope = serde_json::from_str(&text).unwrap_or_else(|e| panic!("{e}: {text}"));
    (status, env)
}

async fn ask(b: &Broker, prompt: &str) -> (u16, Envelope) {
    call(b, &prompt_body(prompt)).await
}

// ── Auth ──

#[tokio::test]
async fn no_token_is_refused_and_reaches_no_model() {
    let (b, _l, inv) = start(FakeInvoker::answering("hi", 1), declaration(5, 1000, 60), 4).await;
    let (status, text) = raw_exchange(
        b.port(),
        &http(b.port(), COMPLETE_PATH, None, "", &prompt_body("p")),
    )
    .await;
    assert_eq!(status, 401, "{text}");
    assert!(text.contains("\"unauthorized\""), "{text}");
    assert_eq!(inv.calls.load(Ordering::SeqCst), 0);
    let out = b.stop().await;
    assert!(
        out.ledger.calls.is_empty(),
        "an unauthenticated request is not a call"
    );
}

#[tokio::test]
async fn a_wrong_token_is_refused() {
    let (b, _l, inv) = start(FakeInvoker::answering("hi", 1), declaration(5, 1000, 60), 4).await;
    let mut wrong = b.token().to_string();
    let last = wrong.pop().unwrap();
    wrong.push(if last == '0' { '1' } else { '0' });
    for presented in [wrong.as_str(), "", &b.token()[..10]] {
        let (status, text) = raw_exchange(
            b.port(),
            &http(
                b.port(),
                COMPLETE_PATH,
                Some(presented),
                "",
                &prompt_body("p"),
            ),
        )
        .await;
        assert_eq!(status, 401, "{presented:?}: {text}");
    }
    // Not a Bearer scheme.
    let (status, _) = raw_exchange(
        b.port(),
        &http(
            b.port(),
            COMPLETE_PATH,
            None,
            &format!("Authorization: Basic {}\r\n", b.token()),
            &prompt_body("p"),
        ),
    )
    .await;
    assert_eq!(status, 401);
    assert_eq!(inv.calls.load(Ordering::SeqCst), 0);
}

#[tokio::test]
async fn a_token_in_the_query_is_refused() {
    let (b, _l, inv) = start(FakeInvoker::answering("hi", 1), declaration(5, 1000, 60), 4).await;
    let target = format!("{COMPLETE_PATH}?token={}", b.token());
    for auth in [None, Some(b.token().to_string())] {
        let (status, text) = raw_exchange(
            b.port(),
            &http(b.port(), &target, auth.as_deref(), "", &prompt_body("p")),
        )
        .await;
        assert_eq!(status, 401, "{text}");
        assert!(text.contains("query string"), "{text}");
    }
    assert_eq!(inv.calls.load(Ordering::SeqCst), 0);
}

#[tokio::test]
async fn an_origin_header_is_refused_even_with_the_token() {
    let (b, _l, inv) = start(FakeInvoker::answering("hi", 1), declaration(5, 1000, 60), 4).await;
    let (status, text) = raw_exchange(
        b.port(),
        &http(
            b.port(),
            COMPLETE_PATH,
            Some(b.token()),
            "Origin: http://evil.example\r\n",
            &prompt_body("p"),
        ),
    )
    .await;
    assert_eq!(status, 403, "{text}");
    assert!(text.contains("Origin"), "{text}");
    assert_eq!(inv.calls.load(Ordering::SeqCst), 0);
}

#[tokio::test]
async fn a_wrong_host_is_refused_even_with_the_token() {
    let (b, _l, inv) = start(FakeInvoker::answering("hi", 1), declaration(5, 1000, 60), 4).await;
    let port = b.port();
    let body = prompt_body("p");
    for host in [
        format!("localhost:{port}"),
        format!("127.0.0.1:{}", port.wrapping_add(1)),
        "evil.example".to_string(),
    ] {
        let req = format!(
            "POST {COMPLETE_PATH} HTTP/1.1\r\nHost: {host}\r\nAuthorization: Bearer {}\r\nContent-Length: {}\r\n\r\n{body}",
            b.token(),
            body.len()
        );
        let (status, text) = raw_exchange(port, req.as_bytes()).await;
        assert_eq!(status, 403, "{host}: {text}");
    }
    // No Host at all.
    let req = format!(
        "POST {COMPLETE_PATH} HTTP/1.1\r\nAuthorization: Bearer {}\r\nContent-Length: {}\r\n\r\n{body}",
        b.token(),
        body.len()
    );
    let (status, _) = raw_exchange(port, req.as_bytes()).await;
    assert_eq!(status, 403);
    assert_eq!(inv.calls.load(Ordering::SeqCst), 0);
}

#[tokio::test]
async fn a_peer_the_host_does_not_admit_is_refused() {
    let mut inv = FakeInvoker::answering("hi", 1);
    inv.admit_peers = false;
    let (b, _l, inv) = start(inv, declaration(5, 1000, 60), 4).await;
    let (status, _) = ask(&b, "p").await;
    assert_eq!(status, 403);
    assert_eq!(inv.calls.load(Ordering::SeqCst), 0);
}

#[tokio::test]
async fn a_good_request_gets_a_cli_shaped_envelope_and_the_log_carries_no_text() {
    let inv = FakeInvoker::new(Arc::new(|req: crate::host::InvokeRequest| {
        Box::pin(async move {
            assert_eq!(req.cli_model, "sonnet");
            assert_eq!(req.prompt, "SECRET-PROMPT");
            Ok(ok_reply("SECRET-ANSWER", 7))
        })
    }));
    let (b, lines, _inv) = start(inv, declaration(5, 1000, 60), 4).await;
    b.set_step("b2");
    let (status, env) = ask(&b, "SECRET-PROMPT").await;
    assert_eq!(status, 200);
    assert!(!env.is_error);
    assert_eq!(env.subtype, "success");
    assert_eq!(env.result, "SECRET-ANSWER");
    assert_eq!(env.usage.unwrap().input_tokens, 7);
    assert!(env.model_usage.contains_key("claude-sonnet-4-5-20250929"));
    assert_eq!(env.broker.call_index, 1);
    assert_eq!(env.broker.budget_left.calls, 4);
    assert_eq!(env.broker.budget_left.tokens, 993);
    let log = lines.all();
    assert!(
        log.iter()
            .any(|l| l.contains("call 1 step=b2 model=claude-sonnet outcome=ok")),
        "{log:?}"
    );
    assert!(
        !log.iter().any(|l| l.contains("SECRET")),
        "no prompt or answer text: {log:?}"
    );
    assert!(
        !log.iter().any(|l| l.contains(b.token())),
        "the token is never logged"
    );
    let out = b.stop().await;
    assert_eq!(out.ledger.calls[0].step, "b2");
    assert_eq!(out.ledger.calls[0].outcome, "ok");
    assert_eq!(out.calls_used, 1);
}

#[tokio::test]
async fn the_token_is_256_bits_per_broker_and_debug_redacts_it() {
    let (a, _l, _) = start(FakeInvoker::answering("hi", 1), declaration(1, 1, 1), 1).await;
    let (b, _l, _) = start(FakeInvoker::answering("hi", 1), declaration(1, 1, 1), 1).await;
    assert_eq!(a.token().len(), 64);
    assert!(a.token().bytes().all(|c| c.is_ascii_hexdigit()));
    assert_ne!(a.token(), b.token());
    assert!(!format!("{a:?}").contains(a.token()));
    assert!(a.url().starts_with("http://127.0.0.1:"));
}

// ── Budgets ──

#[tokio::test]
async fn calls_are_a_hard_limit_at_n_and_n_plus_one() {
    let (b, _l, inv) = start(FakeInvoker::answering("hi", 1), declaration(2, 1000, 60), 4).await;
    assert_eq!(ask(&b, "1").await.0, 200);
    let (status, env) = ask(&b, "2").await;
    assert_eq!(status, 200);
    assert_eq!(env.broker.budget_left.calls, 0);
    let (status, env) = ask(&b, "3").await;
    assert_eq!(status, 429);
    assert_eq!(env.subtype, "budget_exhausted:calls");
    assert_eq!(env.retryable, Some(false));
    assert_eq!(inv.calls.load(Ordering::SeqCst), 2);
    let out = b.stop().await;
    assert_eq!(out.calls_used, 2);
    assert_eq!(out.ledger.calls.len(), 3);
    assert_eq!(out.ledger.calls[2].outcome, "budget_exhausted:calls");
    assert_eq!(out.ledger.calls[2].attempts, 0);
}

#[tokio::test]
async fn token_overshoot_is_one_call_at_concurrency_one() {
    let (b, _l, inv) = start(
        FakeInvoker::answering("hi", 100),
        declaration(10, 50, 60),
        1,
    )
    .await;
    assert_eq!(ask(&b, "1").await.0, 200);
    for _ in 0..2 {
        let (status, env) = ask(&b, "again").await;
        assert_eq!(status, 429);
        assert_eq!(env.subtype, "budget_exhausted:tokens");
    }
    assert_eq!(inv.calls.load(Ordering::SeqCst), 1);
    let out = b.stop().await;
    assert_eq!(out.ledger.budget.tokens_used, 100);
    assert_eq!(out.ledger.budget.tokens_overshoot, 50);
    assert!(
        out.ledger.budget.tokens_overshoot <= 100,
        "bounded by one call in flight"
    );
}

/// A fake whose calls wait on `gate` before answering with `tokens`.
fn gated(
    gate: Arc<Semaphore>,
    tokens: u64,
    in_flight: Arc<AtomicU64>,
    max_seen: Arc<AtomicU64>,
) -> FakeInvoker {
    FakeInvoker::new(Arc::new(move |_req| {
        let gate = gate.clone();
        let in_flight = in_flight.clone();
        let max_seen = max_seen.clone();
        Box::pin(async move {
            let now = in_flight.fetch_add(1, Ordering::SeqCst) + 1;
            max_seen.fetch_max(now, Ordering::SeqCst);
            let _p = gate.acquire().await.unwrap();
            in_flight.fetch_sub(1, Ordering::SeqCst);
            Ok(ok_reply("hi", tokens))
        })
    }))
}

async fn wait_until(what: &str, mut f: impl FnMut() -> bool) {
    let deadline = Instant::now() + Duration::from_secs(10);
    while !f() {
        assert!(Instant::now() < deadline, "timed out waiting for {what}");
        tokio::time::sleep(Duration::from_millis(10)).await;
    }
}

#[tokio::test]
async fn token_overshoot_is_bounded_by_the_cap_in_flight() {
    let gate = Arc::new(Semaphore::new(0));
    let (in_flight, max_seen) = (Arc::new(AtomicU64::new(0)), Arc::new(AtomicU64::new(0)));
    let (b, _l, inv) = start(
        gated(gate.clone(), 100, in_flight, max_seen),
        declaration(100, 50, 60),
        MAX_IN_FLIGHT,
    )
    .await;
    let b = Arc::new(b);
    let mut handles = Vec::new();
    for i in 0..6 {
        let b = b.clone();
        handles.push(tokio::spawn(async move { ask(&b, &format!("{i}")).await }));
    }
    wait_until("4 calls in flight", || {
        inv.calls.load(Ordering::SeqCst) == 4
    })
    .await;
    gate.add_permits(100);
    let mut subtypes = Vec::new();
    for h in handles {
        subtypes.push(h.await.unwrap().1.subtype);
    }
    subtypes.sort();
    assert_eq!(
        subtypes,
        [
            "budget_exhausted:tokens",
            "budget_exhausted:tokens",
            "success",
            "success",
            "success",
            "success"
        ]
    );
    let b = Arc::try_unwrap(b).unwrap();
    let out = b.stop().await;
    assert_eq!(out.ledger.budget.tokens_used, 400);
    assert_eq!(out.ledger.budget.tokens_overshoot, 350);
    assert!(out.ledger.budget.tokens_overshoot <= MAX_IN_FLIGHT as u64 * 100);
}

#[tokio::test]
async fn concurrency_holds_at_the_cap() {
    let gate = Arc::new(Semaphore::new(0));
    let (in_flight, max_seen) = (Arc::new(AtomicU64::new(0)), Arc::new(AtomicU64::new(0)));
    let (b, _l, inv) = start(
        gated(gate.clone(), 1, in_flight, max_seen.clone()),
        declaration(100, 100_000, 60),
        MAX_IN_FLIGHT,
    )
    .await;
    let b = Arc::new(b);
    let mut handles = Vec::new();
    for i in 0..7 {
        let b = b.clone();
        handles.push(tokio::spawn(async move { ask(&b, &format!("{i}")).await }));
    }
    wait_until("the cap in flight", || {
        inv.calls.load(Ordering::SeqCst) == 4
    })
    .await;
    tokio::time::sleep(Duration::from_millis(200)).await;
    assert_eq!(
        inv.calls.load(Ordering::SeqCst),
        4,
        "a fifth call started past the cap"
    );
    gate.add_permits(100);
    for h in handles {
        assert_eq!(h.await.unwrap().0, 200);
    }
    assert_eq!(max_seen.load(Ordering::SeqCst), MAX_IN_FLIGHT as u64);
    assert_eq!(inv.calls.load(Ordering::SeqCst), 7);
}

#[tokio::test]
async fn the_wall_budget_counts_from_the_first_call() {
    let (b, _l, inv) = start(FakeInvoker::answering("hi", 1), declaration(10, 1000, 1), 4).await;
    // The clock has not started: no call yet.
    tokio::time::sleep(Duration::from_millis(1100)).await;
    assert_eq!(ask(&b, "first").await.0, 200);
    tokio::time::sleep(Duration::from_millis(1100)).await;
    let (status, env) = ask(&b, "late").await;
    assert_eq!(status, 429);
    assert_eq!(env.subtype, "budget_exhausted:wall");
    assert_eq!(inv.calls.load(Ordering::SeqCst), 1);
    let out = b.stop().await;
    assert_eq!(
        out.calls_used, 1,
        "a call refused for wall time is not a call made"
    );
    assert_eq!(out.ledger.calls[1].attempts, 0);
}

// ── Exhaustion ──

/// First call: every account exhausted until `reset_in`; then answers.
fn exhausted_then_ok(reset_in: Duration) -> FakeInvoker {
    let first = Arc::new(AtomicBool::new(true));
    FakeInvoker::new(Arc::new(move |_req| {
        let first = first.swap(false, Ordering::SeqCst);
        Box::pin(async move {
            if first {
                Err(InvokeError::AllAccountsExhausted {
                    reset_at: SystemTime::now() + reset_in,
                })
            } else {
                Ok(ok_reply("after the reset", 3))
            }
        })
    }))
}

#[tokio::test]
async fn an_exhaustion_whose_reset_is_inside_the_wall_budget_is_held() {
    let (b, lines, inv) = start(
        exhausted_then_ok(Duration::from_millis(900)),
        declaration(10, 1000, 30),
        4,
    )
    .await;
    let t0 = Instant::now();
    let (status, env) = ask(&b, "p").await;
    assert_eq!(status, 200, "{env:?}");
    assert_eq!(env.result, "after the reset");
    assert!(
        t0.elapsed() >= Duration::from_millis(500),
        "the call was held"
    );
    assert_eq!(inv.calls.load(Ordering::SeqCst), 2);
    assert!(
        lines.all().iter().any(|l| l.contains("held")),
        "{:?}",
        lines.all()
    );
    let out = b.stop().await;
    assert_eq!(out.ledger.calls[0].attempts, 2);
    assert_eq!(out.exhausted_reset_at, None);
    assert_eq!(out.calls_used, 1, "a held call is one call");
}

#[tokio::test]
async fn an_exhaustion_whose_reset_is_past_the_wall_budget_fails_with_reset_at() {
    let (b, _l, inv) = start(
        exhausted_then_ok(Duration::from_secs(3600)),
        declaration(10, 1000, 60),
        4,
    )
    .await;
    let t0 = Instant::now();
    let (status, env) = ask(&b, "p").await;
    assert_eq!(status, 503);
    assert_eq!(env.subtype, "all_accounts_exhausted");
    assert_eq!(env.retryable, Some(true));
    let reset = env.reset_at.expect("reset_at");
    let now = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap()
        .as_secs();
    assert!(reset >= now + 3500, "{reset} vs {now}");
    assert!(t0.elapsed() < Duration::from_secs(5), "not held");
    assert_eq!(inv.calls.load(Ordering::SeqCst), 1);
    let out = b.stop().await;
    assert_eq!(out.exhausted_reset_at, Some(reset));
    assert_eq!(out.ledger.calls[0].outcome, "all_accounts_exhausted");
}

// ── Requests ──

#[tokio::test]
async fn an_undeclared_model_is_refused_and_recorded() {
    let (b, _l, inv) = start(
        FakeInvoker::answering("hi", 1),
        declaration(10, 1000, 60),
        4,
    )
    .await;
    let (status, env) = call(
        &b,
        &serde_json::json!({"model": "claude-opus", "prompt": "p"}).to_string(),
    )
    .await;
    assert_eq!(status, 400);
    assert_eq!(env.subtype, "model_not_declared");
    // A declared non-default model is accepted.
    let (status, _) = call(
        &b,
        &serde_json::json!({"model": "claude-haiku", "prompt": "p"}).to_string(),
    )
    .await;
    assert_eq!(status, 200);
    assert_eq!(inv.calls.load(Ordering::SeqCst), 1);
    let out = b.stop().await;
    assert_eq!(out.ledger.calls[0].outcome, "model_not_declared");
    assert_eq!(out.ledger.calls[0].requested, "claude-opus");
    assert_eq!(out.ledger.calls[1].requested, "claude-haiku");
}

#[tokio::test]
async fn the_prompt_and_response_caps_hold() {
    let big = Box::leak("x".repeat(MAX_RESPONSE_BYTES + 1).into_boxed_str());
    let (b, _l, inv) = start(FakeInvoker::answering(big, 9), declaration(10, 1000, 60), 4).await;
    // A prompt one byte over the cap.
    let (status, env) = ask(&b, &"p".repeat(MAX_PROMPT_BYTES + 1)).await;
    assert_eq!(status, 413);
    assert_eq!(env.subtype, "prompt_too_large");
    assert_eq!(inv.calls.load(Ordering::SeqCst), 0);
    // A body declared past the read cap is refused without being read.
    let req = format!(
        "POST {COMPLETE_PATH} HTTP/1.1\r\nHost: 127.0.0.1:{}\r\nAuthorization: Bearer {}\r\nContent-Length: {}\r\n\r\n",
        b.port(),
        b.token(),
        MAX_REQUEST_BODY_BYTES + 1
    );
    let (status, text) = raw_exchange(b.port(), req.as_bytes()).await;
    assert_eq!(status, 413, "{text}");
    // An answer over the cap fails, and its tokens still count.
    let (status, env) = ask(&b, "small").await;
    assert_eq!(status, 502);
    assert_eq!(env.subtype, "response_too_large");
    let out = b.stop().await;
    assert_eq!(out.ledger.budget.tokens_used, 9);
    let row = out.ledger.calls.last().unwrap();
    assert_eq!(row.outcome, "response_too_large");
    assert_eq!(row.input_tokens, Some(9));
}

#[tokio::test]
async fn a_provider_error_is_a_ledger_error_row() {
    let inv = FakeInvoker::new(Arc::new(|_req| {
        Box::pin(async {
            Err(InvokeError::Provider {
                message: "overloaded".to_string(),
                retryable: true,
            })
        })
    }));
    let (b, _l, _inv) = start(inv, declaration(10, 1000, 60), 4).await;
    let (status, env) = ask(&b, "p").await;
    assert_eq!(status, 502);
    assert_eq!(env.subtype, "provider_error");
    let out = b.stop().await;
    assert_eq!(out.ledger.calls[0].outcome, "provider_error");
    assert_eq!(
        out.calls_used, 1,
        "a failed call reached the host and counts"
    );
}

/// Sets its flag when dropped: proof a call future was cancelled.
struct DropFlag(Arc<AtomicBool>);
impl Drop for DropFlag {
    fn drop(&mut self) {
        self.0.store(true, Ordering::SeqCst);
    }
}

fn hanging(dropped: Arc<AtomicBool>) -> FakeInvoker {
    FakeInvoker::new(Arc::new(move |_req| {
        let flag = DropFlag(dropped.clone());
        Box::pin(async move {
            let _flag = flag;
            tokio::time::sleep(Duration::from_secs(3600)).await;
            Ok(ok_reply("never", 1))
        })
    }))
}

#[tokio::test]
async fn a_disconnect_cancels_the_call() {
    let dropped = Arc::new(AtomicBool::new(false));
    let (b, lines, inv) = start(hanging(dropped.clone()), declaration(10, 1000, 60), 4).await;
    let mut s = TcpStream::connect(("127.0.0.1", b.port())).await.unwrap();
    s.write_all(&http(
        b.port(),
        COMPLETE_PATH,
        Some(b.token()),
        "",
        &prompt_body("p"),
    ))
    .await
    .unwrap();
    wait_until("the call to start", || {
        inv.calls.load(Ordering::SeqCst) == 1
    })
    .await;
    drop(s);
    wait_until("the call future to be dropped", || {
        dropped.load(Ordering::SeqCst)
    })
    .await;
    wait_until("the ledger row", || {
        lines.all().iter().any(|l| l.contains("outcome=cancelled"))
    })
    .await;
    let out = b.stop().await;
    assert_eq!(out.ledger.calls[0].outcome, "cancelled");
}

#[tokio::test]
async fn stopping_the_broker_cancels_in_flight_calls_and_records_them() {
    let dropped = Arc::new(AtomicBool::new(false));
    let (b, _l, inv) = start(hanging(dropped.clone()), declaration(10, 1000, 60), 4).await;
    let port = b.port();
    let req = http(port, COMPLETE_PATH, Some(b.token()), "", &prompt_body("p"));
    let client = tokio::spawn(async move {
        let mut s = TcpStream::connect(("127.0.0.1", port)).await.unwrap();
        s.write_all(&req).await.unwrap();
        let mut buf = Vec::new();
        let _ = s.read_to_end(&mut buf).await;
        String::from_utf8_lossy(&buf).to_string()
    });
    wait_until("the call to start", || {
        inv.calls.load(Ordering::SeqCst) == 1
    })
    .await;
    let out = tokio::time::timeout(Duration::from_secs(10), b.stop())
        .await
        .expect("stop does not wait for the call");
    assert!(dropped.load(Ordering::SeqCst));
    assert_eq!(out.ledger.calls[0].outcome, "cancelled");
    let answer = client.await.unwrap();
    assert!(answer.contains("\"cancelled\""), "{answer}");
}
