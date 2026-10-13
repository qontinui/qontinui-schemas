//! `qontinui-ci-model` — the shim a `model = true` step runs as
//! `$QONTINUI_CI_MODEL_CMD` (plan
//! `2026-10-13-ci-node-runs-model-dependent-eval-steps`, D4).
//!
//! It mirrors `claude -p`, so a harness that already drives the CLI drives
//! this unchanged:
//!
//! ```text
//! qontinui-ci-model [-p|--print] [--output-format text|json] [--model <name>] [<prompt>]
//! ```
//!
//! - The prompt comes from stdin, or from the one positional argument.
//! - Text output by default (the answer on stdout); `--output-format json`
//!   prints the broker's `claude -p`-compatible envelope.
//! - `--model` defaults to the first model `[model]` declares (the broker
//!   applies the default).
//! - The capability is read from `QONTINUI_CI_MODEL_TOKEN` and sent only as
//!   `Authorization: Bearer`; it never appears on stdout, stderr or argv.
//!
//! A std-only HTTP/1.1 client with a short connect timeout — no async runtime,
//! no HTTP library — following the runner's own `qontinui_shim`.
//!
//! Exit codes: 0 ok; 1 a broker or provider error (the envelope is printed in
//! json mode), or model access refused for this dispatch; 2 a usage error.

use std::io::{Read, Write};
use std::net::{SocketAddr, TcpStream};
use std::time::Duration;

use crate::model::declaration::MAX_MODEL_WALL_SECS;
use crate::model::protocol::{
    CompleteRequest, Envelope, COMPLETE_PATH, MAX_ENVELOPE_BYTES, MAX_PROMPT_BYTES,
    MAX_RESPONSE_HEAD_BYTES,
};
use crate::model::{MODEL_REFUSED_ENV, MODEL_TOKEN_ENV, MODEL_URL_ENV};

/// How long the shim waits to connect. The broker is on loopback: a slow
/// connect means it is not there.
const CONNECT_TIMEOUT: Duration = Duration::from_secs(2);
/// The longest a call can take: a held call waits inside the dispatch's wall
/// budget, which is at most this.
const READ_TIMEOUT: Duration = Duration::from_secs(MAX_MODEL_WALL_SECS + 60);
/// The largest response the shim reads: exactly what the broker may send —
/// a head plus an envelope the broker measured against [`MAX_ENVELOPE_BYTES`].
pub(crate) const MAX_RESPONSE_READ: usize = MAX_ENVELOPE_BYTES + MAX_RESPONSE_HEAD_BYTES;

/// How many times the shim sends a request the broker answered `busy`.
const BUSY_RETRIES: u32 = 8;
/// The pause before retry `n` is `n` times this.
const BUSY_BACKOFF: Duration = Duration::from_millis(250);
/// The broker's subtype for a connection it shed before reading.
const BUSY_SUBTYPE: &str = "busy";

const USAGE: &str = "usage: qontinui-ci-model [-p|--print] [--output-format text|json] \
                     [--model <name>] [<prompt>]  (the prompt is read from stdin when not given)";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Format {
    Text,
    Json,
}

struct Args {
    format: Format,
    model: Option<String>,
    prompt: Option<String>,
}

fn parse_args(args: &[String]) -> Result<Args, String> {
    let mut out = Args {
        format: Format::Text,
        model: None,
        prompt: None,
    };
    let mut it = args.iter();
    while let Some(a) = it.next() {
        match a.as_str() {
            "-p" | "--print" => {}
            "--output-format" => {
                out.format = match it.next().map(String::as_str) {
                    Some("text") => Format::Text,
                    Some("json") => Format::Json,
                    other => {
                        return Err(format!(
                            "--output-format must be text or json, got {other:?}"
                        ))
                    }
                };
            }
            "--model" => {
                out.model = Some(
                    it.next()
                        .filter(|m| !m.is_empty())
                        .ok_or("--model needs a name")?
                        .clone(),
                );
            }
            flag if flag.starts_with('-') && flag.len() > 1 => {
                return Err(format!("unknown option {flag:?}"));
            }
            prompt => {
                if out.prompt.is_some() {
                    return Err("give the prompt once (as one argument, or on stdin)".to_string());
                }
                out.prompt = Some(prompt.to_string());
            }
        }
    }
    Ok(out)
}

/// The broker address from `http://127.0.0.1:<port>`. Anything else is
/// refused: the shim never sends the capability off the loopback.
fn broker_addr(url: &str) -> Result<SocketAddr, String> {
    let rest = url
        .strip_prefix("http://127.0.0.1:")
        .ok_or("QONTINUI_CI_MODEL_URL is not a loopback broker URL")?;
    let port: u16 = rest
        .trim_end_matches('/')
        .parse()
        .map_err(|_| "QONTINUI_CI_MODEL_URL has no valid port".to_string())?;
    Ok(SocketAddr::from(([127, 0, 0, 1], port)))
}

/// Send one request and read the whole answer: `(status, body)`.
fn exchange(addr: SocketAddr, token: &str, body: &[u8]) -> Result<(u16, Vec<u8>), String> {
    let mut s = TcpStream::connect_timeout(&addr, CONNECT_TIMEOUT)
        .map_err(|e| format!("could not reach the model broker: {e}"))?;
    let _ = s.set_read_timeout(Some(READ_TIMEOUT));
    let _ = s.set_nodelay(true);
    let head = format!(
        "POST {COMPLETE_PATH} HTTP/1.1\r\nHost: 127.0.0.1:{}\r\nAuthorization: Bearer {token}\r\n\
         Content-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
        addr.port(),
        body.len()
    );
    s.write_all(head.as_bytes())
        .and_then(|_| s.write_all(body))
        .map_err(|e| format!("could not send the request: {e}"))?;
    // Do NOT half-close the write side: the broker reads a closed client as a
    // disconnect and cancels the call.
    let mut buf = Vec::new();
    s.take(MAX_RESPONSE_READ as u64 + 1)
        .read_to_end(&mut buf)
        .map_err(|e| format!("could not read the answer: {e}"))?;
    if buf.len() > MAX_RESPONSE_READ {
        return Err("the broker's answer is larger than any it may send".to_string());
    }
    let split = buf
        .windows(4)
        .position(|w| w == b"\r\n\r\n")
        .ok_or("the broker's answer has no header end")?;
    let status_line = String::from_utf8_lossy(&buf[..split]).to_string();
    let status = status_line
        .split(' ')
        .nth(1)
        .and_then(|c| c.parse::<u16>().ok())
        .ok_or("the broker's answer has no status")?;
    Ok((status, buf[split + 4..].to_vec()))
}

/// Run the shim. `env` reads a variable; the streams are the process's.
pub fn run(
    args: &[String],
    env: &dyn Fn(&str) -> Option<String>,
    stdin: &mut dyn Read,
    stdout: &mut dyn Write,
    stderr: &mut dyn Write,
) -> i32 {
    let args = match parse_args(args) {
        Ok(a) => a,
        Err(e) => {
            let _ = writeln!(stderr, "qontinui-ci-model: {e}\n{USAGE}");
            return 2;
        }
    };
    let (url, token) = match (env(MODEL_URL_ENV), env(MODEL_TOKEN_ENV)) {
        (Some(u), Some(t)) if !u.is_empty() && !t.is_empty() => (u, t),
        _ => {
            if let Some(reason) = env(MODEL_REFUSED_ENV).filter(|r| !r.is_empty()) {
                let message = format!("model access was refused for this dispatch: {reason}");
                if args.format == Format::Json {
                    let env = serde_json::json!({
                        "type": "result",
                        "subtype": "refused",
                        "is_error": true,
                        "retryable": false,
                        "result": message,
                        "reason": reason,
                    });
                    let _ = writeln!(stdout, "{env}");
                } else {
                    let _ = writeln!(stderr, "qontinui-ci-model: {message}");
                }
                return 1;
            }
            let _ = writeln!(
                stderr,
                "qontinui-ci-model: {MODEL_URL_ENV} / {MODEL_TOKEN_ENV} are not set — run inside a \
                 CI step that declares `model = true`"
            );
            return 2;
        }
    };
    let addr = match broker_addr(&url) {
        Ok(a) => a,
        Err(e) => {
            let _ = writeln!(stderr, "qontinui-ci-model: {e}");
            return 2;
        }
    };
    let prompt = match args.prompt {
        Some(p) => p,
        None => {
            let mut buf = Vec::new();
            if let Err(e) = stdin
                .take(MAX_PROMPT_BYTES as u64 + 1)
                .read_to_end(&mut buf)
            {
                let _ = writeln!(stderr, "qontinui-ci-model: could not read the prompt: {e}");
                return 2;
            }
            match String::from_utf8(buf) {
                Ok(p) => p,
                Err(_) => {
                    let _ = writeln!(stderr, "qontinui-ci-model: the prompt is not UTF-8");
                    return 2;
                }
            }
        }
    };
    let body = serde_json::to_vec(&CompleteRequest {
        model: args.model,
        prompt,
    })
    .unwrap_or_default();
    // A `busy` answer means the broker shed the connection before reading
    // it: nothing was spent, so retry a few times with a growing pause before
    // handing the envelope to the caller.
    let mut attempt: u32 = 0;
    let (status, answer, envelope) = loop {
        attempt += 1;
        let (status, answer) = match exchange(addr, &token, &body) {
            Ok(r) => r,
            Err(e) => {
                // `e` is built from I/O errors and fixed text only: never the
                // token, which lives in a header this process wrote.
                let _ = writeln!(stderr, "qontinui-ci-model: {e}");
                return 1;
            }
        };
        let envelope: Envelope = match serde_json::from_slice(&answer) {
            Ok(env) => env,
            Err(_) => {
                let _ = writeln!(
                    stderr,
                    "qontinui-ci-model: the broker answered HTTP {status} without an envelope"
                );
                return 1;
            }
        };
        if envelope.subtype == BUSY_SUBTYPE && attempt < BUSY_RETRIES {
            std::thread::sleep(BUSY_BACKOFF * attempt);
            continue;
        }
        break (status, answer, envelope);
    };
    let _ = status;
    match (args.format, envelope.is_error) {
        (Format::Json, is_error) => {
            let _ = stdout.write_all(&answer);
            let _ = writeln!(stdout);
            i32::from(is_error)
        }
        (Format::Text, false) => {
            let _ = writeln!(stdout, "{}", envelope.result);
            0
        }
        (Format::Text, true) => {
            let _ = writeln!(
                stderr,
                "qontinui-ci-model: {}: {}",
                envelope.subtype, envelope.result
            );
            1
        }
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use std::sync::Arc;

    use super::*;
    use crate::model::broker::{Broker, BrokerConfig};
    use crate::model::test_support::{declaration, FakeInvoker, Lines};

    struct Run {
        code: i32,
        stdout: String,
        stderr: String,
    }

    async fn shim(args: &[&str], vars: HashMap<String, String>, stdin: &str) -> Run {
        let args: Vec<String> = args.iter().map(|s| s.to_string()).collect();
        let stdin = stdin.to_string();
        // The shim is blocking std I/O: run it off the async workers, as the
        // real process is a separate process.
        tokio::task::spawn_blocking(move || {
            let mut out = Vec::new();
            let mut err = Vec::new();
            let code = run(
                &args,
                &|k| vars.get(k).cloned(),
                &mut stdin.as_bytes(),
                &mut out,
                &mut err,
            );
            Run {
                code,
                stdout: String::from_utf8_lossy(&out).to_string(),
                stderr: String::from_utf8_lossy(&err).to_string(),
            }
        })
        .await
        .unwrap()
    }

    async fn broker(inv: FakeInvoker, max_calls: u64) -> Broker {
        Broker::start(
            Arc::new(inv),
            BrokerConfig::new(declaration(max_calls, 1000, 60)),
            Arc::new(Lines::default()),
        )
        .await
        .unwrap()
    }

    fn vars(b: &Broker) -> HashMap<String, String> {
        HashMap::from([
            (MODEL_URL_ENV.to_string(), b.url()),
            (MODEL_TOKEN_ENV.to_string(), b.token().to_string()),
        ])
    }

    #[tokio::test]
    async fn text_mode_prints_the_answer_and_never_the_token() {
        let b = broker(FakeInvoker::answering("the answer", 3), 5).await;
        let r = shim(&["-p"], vars(&b), "a prompt on stdin").await;
        assert_eq!(r.code, 0, "{}", r.stderr);
        assert_eq!(r.stdout, "the answer\n");
        assert!(!r.stdout.contains(b.token()) && !r.stderr.contains(b.token()));
        // The positional prompt works too.
        let r = shim(&["-p", "an argv prompt"], vars(&b), "").await;
        assert_eq!(r.code, 0);
    }

    #[tokio::test]
    async fn json_mode_prints_the_envelope() {
        let b = broker(FakeInvoker::answering("json answer", 3), 5).await;
        let r = shim(
            &["--output-format", "json", "--model", "claude-haiku"],
            vars(&b),
            "p",
        )
        .await;
        assert_eq!(r.code, 0, "{}", r.stderr);
        let v: serde_json::Value = serde_json::from_str(r.stdout.trim()).unwrap();
        assert_eq!(v["type"], "result");
        assert_eq!(v["result"], "json answer");
        assert_eq!(v["is_error"], false);
        assert!(v["usage"].is_object());
        assert!(!r.stdout.contains(b.token()));
        let out = b.stop().await;
        assert_eq!(out.ledger.calls[0].requested, "claude-haiku");
    }

    #[tokio::test]
    async fn error_mode_exits_one_and_prints_the_error_envelope_in_json() {
        let b = broker(FakeInvoker::answering("x", 3), 1).await;
        assert_eq!(shim(&[], vars(&b), "p").await.code, 0);
        // The calls budget is spent.
        let r = shim(&["--output-format", "json"], vars(&b), "p").await;
        assert_eq!(r.code, 1);
        let v: serde_json::Value = serde_json::from_str(r.stdout.trim()).unwrap();
        assert_eq!(v["is_error"], true);
        assert_eq!(v["subtype"], "budget_exhausted:calls");
        let r = shim(&[], vars(&b), "p").await;
        assert_eq!(r.code, 1);
        assert!(r.stdout.is_empty());
        assert!(r.stderr.contains("budget_exhausted:calls"), "{}", r.stderr);
        assert!(!r.stderr.contains(b.token()));
        // An undeclared model.
        let r = shim(&["--model", "claude-opus"], vars(&b), "p").await;
        assert_eq!(r.code, 1);
        assert!(r.stderr.contains("model_not_declared"));
    }

    #[tokio::test]
    async fn a_wrong_token_is_an_error_that_does_not_echo_it() {
        let b = broker(FakeInvoker::answering("x", 3), 5).await;
        let mut v = vars(&b);
        v.insert(MODEL_TOKEN_ENV.to_string(), "not-the-token".to_string());
        let r = shim(&["--output-format", "json"], v, "p").await;
        assert_eq!(r.code, 1);
        assert!(r.stdout.contains("unauthorized"));
        assert!(!r.stdout.contains("not-the-token") && !r.stderr.contains("not-the-token"));
    }

    #[tokio::test]
    async fn usage_errors_exit_two() {
        for args in [
            &["--output-format", "yaml"][..],
            &["--model"][..],
            &["--bogus"][..],
            &["one", "two"][..],
        ] {
            let r = shim(args, HashMap::new(), "").await;
            assert_eq!(r.code, 2, "{args:?}");
            assert!(r.stderr.contains("usage:"), "{args:?}: {}", r.stderr);
        }
        // Outside a model step.
        let r = shim(&[], HashMap::new(), "p").await;
        assert_eq!(r.code, 2);
        assert!(r.stderr.contains("model = true"));
        // A non-loopback URL is never sent the capability.
        let v = HashMap::from([
            (
                MODEL_URL_ENV.to_string(),
                "http://evil.example:80".to_string(),
            ),
            (MODEL_TOKEN_ENV.to_string(), "t".to_string()),
        ]);
        assert_eq!(shim(&[], v, "p").await.code, 2);
    }

    /// A broker whose read-phase cap is held by idle peers answers `busy`;
    /// the shim retries and is served once the peers go.
    #[tokio::test]
    async fn the_shim_retries_a_busy_broker() {
        let b = broker(FakeInvoker::answering("served", 1), 5).await;
        let mut idle = Vec::new();
        for _ in 0..crate::model::broker::MAX_CONNECTIONS {
            idle.push(
                tokio::net::TcpStream::connect(("127.0.0.1", b.port()))
                    .await
                    .unwrap(),
            );
        }
        tokio::time::sleep(Duration::from_millis(200)).await;
        let run = tokio::spawn(shim(&[], vars(&b), "p"));
        // Long enough for at least one `busy` answer, then free the cap.
        tokio::time::sleep(Duration::from_millis(1800)).await;
        drop(idle);
        let r = run.await.unwrap();
        assert_eq!(r.code, 0, "{}", r.stderr);
        assert_eq!(r.stdout, "served\n");
    }

    #[tokio::test]
    async fn a_refused_dispatch_says_why() {
        let v = HashMap::from([(MODEL_REFUSED_ENV.to_string(), "fork_pr".to_string())]);
        let r = shim(&["--output-format", "json"], v.clone(), "p").await;
        assert_eq!(r.code, 1);
        let j: serde_json::Value = serde_json::from_str(r.stdout.trim()).unwrap();
        assert_eq!(j["subtype"], "refused");
        assert_eq!(j["reason"], "fork_pr");
        let r = shim(&[], v, "p").await;
        assert_eq!(r.code, 1);
        assert!(r.stderr.contains("fork_pr"));
    }
}
