//! The per-dispatch model broker (plan
//! `2026-10-13-ci-node-runs-model-dependent-eval-steps`, D4).
//!
//! - **Listener.** `127.0.0.1:0`, IPv4 loopback only, one per dispatch. The
//!   executor starts it before the first `model = true` step and stops it
//!   before cleanup on every exit path ([`Broker::stop`]); dropping a
//!   [`Broker`] also shuts it down, as a backstop.
//! - **Capability.** 256 bits from a CSPRNG, minted per dispatch, accepted
//!   ONLY as `Authorization: Bearer` and compared in constant time
//!   ([`crate::model::secret_eq`]).
//! - **Request checks.** A request carrying an `Origin`, a `Host` other than
//!   `127.0.0.1:<port>`, or any query string is refused — that shuts out
//!   browsers, DNS rebinding and a capability pasted into a URL. Prompts and
//!   answers have size caps. The host may refuse a peer from another OS user
//!   ([`ModelInvoker::accept_peer`]).
//! - **Budget.** Calls are a hard limit. A call STARTS only while the tokens
//!   used are below `max_tokens`; calls already in flight finish, so the
//!   overshoot is bounded by the in-flight cap ([`MAX_IN_FLIGHT`]) and the
//!   ledger shows it. Wall time counts from the first call.
//! - **Exhaustion.** When the host reports every account exhausted, the call
//!   is HELD until the reported reset if that falls inside the remaining wall
//!   budget (logging each minute, which renews coord's dispatch lease);
//!   otherwise it fails `all_accounts_exhausted` with `reset_at`.
//! - **Cancellation.** A client that disconnects cancels its call (the
//!   invoker's future is dropped); so does the dispatch ending.
//! - **Logging.** One metadata line per call — index, step, model, outcome,
//!   latency. Never prompt or answer text.

use std::net::{Ipv4Addr, SocketAddr};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use tokio::io::{AsyncBufReadExt, AsyncReadExt, AsyncWriteExt, BufReader};
use tokio::net::tcp::OwnedReadHalf;
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::Semaphore;
use tokio::task::{JoinHandle, JoinSet};
use tokio_util::sync::CancellationToken;

use crate::host::{InvokeError, InvokeReply, InvokeRequest, ModelInvoker};
use crate::model::declaration::ModelDeclaration;
use crate::model::ledger::{usd_to_microusd, BudgetReport, CallRow, LedgerBuilder, ModelLedger};
use crate::model::protocol::{
    BrokerBlock, Budget, BudgetLeft, CompleteRequest, Envelope, ErrorSubtype, ModelUsage, Usage,
    COMPLETE_PATH, MAX_ENVELOPE_BYTES, MAX_PROMPT_BYTES, MAX_REQUEST_BODY_BYTES,
    MAX_REQUEST_HEAD_BYTES, MAX_RESPONSE_BYTES,
};
use crate::report::LogSink;

/// At most this many calls in flight per dispatch. A request beyond it waits
/// for a slot (the wait counts toward the wall budget).
pub const MAX_IN_FLIGHT: usize = 4;
/// The broker's own per-call deadline, inside the wall budget.
pub const CALL_TIMEOUT: Duration = Duration::from_secs(900);
/// How often a held call logs that it is still waiting.
pub const HOLD_LOG_INTERVAL: Duration = Duration::from_secs(60);
/// The shortest an exhaustion hold lasts, even when the host reports a reset
/// already in the past — so a host that keeps reporting a stale reset cannot
/// turn the hold into a retry storm.
pub const MIN_EXHAUSTION_HOLD: Duration = Duration::from_secs(1);
/// How many times one call is held for exhaustion before it fails
/// `all_accounts_exhausted` anyway.
pub const MAX_EXHAUSTION_HOLDS: u32 = 5;
/// How long the broker may take to write one response. A client that stops
/// reading cannot hold a connection task — or `stop` — past this.
pub const WRITE_TIMEOUT: Duration = Duration::from_secs(10);
/// At most this many connections are served at once. Past it a new
/// connection is closed unanswered: idle or stalled local peers cannot grow
/// the broker's task set without bound.
pub const MAX_CONNECTIONS: usize = 32;
/// How long a client may take to send its request head and body.
const REQUEST_READ_TIMEOUT: Duration = Duration::from_secs(30);

/// How a broker is configured. Everything but the declaration has a default.
#[derive(Debug, Clone)]
pub struct BrokerConfig {
    pub declaration: ModelDeclaration,
    pub max_in_flight: usize,
    pub call_timeout: Duration,
    pub hold_log_interval: Duration,
    pub min_exhaustion_hold: Duration,
    pub write_timeout: Duration,
}

impl BrokerConfig {
    pub fn new(declaration: ModelDeclaration) -> Self {
        Self {
            declaration,
            max_in_flight: MAX_IN_FLIGHT,
            call_timeout: CALL_TIMEOUT,
            hold_log_interval: HOLD_LOG_INTERVAL,
            min_exhaustion_hold: MIN_EXHAUSTION_HOLD,
            write_timeout: WRITE_TIMEOUT,
        }
    }
}

/// Whether the `n`th occurrence of a repeated log line is written: the first
/// three, then every hundredth. Bounds what a looping client can put in the
/// dispatch log.
fn log_occurrence(n: u64) -> bool {
    n <= 3 || n.is_multiple_of(100)
}

/// What a stopped broker hands back.
#[derive(Debug, Clone)]
pub struct BrokerOutcome {
    pub ledger: ModelLedger,
    /// Calls that reached the host.
    pub calls_used: u64,
    /// The latest reset a call failed `all_accounts_exhausted` with, unix
    /// seconds.
    pub exhausted_reset_at: Option<u64>,
}

struct State {
    /// Requests admitted as calls (ledger rows), refused ones included.
    next_index: u64,
    /// Calls that reached the host: what `max_calls` counts.
    calls_started: u64,
    tokens_used: u64,
    /// When the first call started, monotonic and wall-clock.
    first_call: Option<(Instant, SystemTime)>,
    /// The step currently running (the executor runs steps sequentially).
    step: String,
    ledger: LedgerBuilder,
    exhausted_reset_at: Option<u64>,
    /// Requests refused before admission, by subtype (for the log limit).
    rejections: std::collections::BTreeMap<String, u64>,
    /// Connections closed unanswered at the connection cap.
    connections_shed: u64,
}

struct Shared {
    declaration: ModelDeclaration,
    invoker: Arc<dyn ModelInvoker>,
    sink: Arc<dyn LogSink>,
    token: String,
    host_header: String,
    permits: Arc<Semaphore>,
    call_timeout: Duration,
    hold_log_interval: Duration,
    min_exhaustion_hold: Duration,
    write_timeout: Duration,
    shutdown: CancellationToken,
    state: Mutex<State>,
}

impl Shared {
    fn state(&self) -> std::sync::MutexGuard<'_, State> {
        // A poisoned lock means a connection task panicked mid-update; the
        // counters are still the best record there is.
        self.state.lock().unwrap_or_else(|p| p.into_inner())
    }

    fn wall(&self) -> Duration {
        Duration::from_secs(self.declaration.max_wall_secs)
    }

    fn budget_left(&self, st: &State) -> BudgetLeft {
        let wall_left = match st.first_call {
            Some((t0, _)) => self.wall().saturating_sub(t0.elapsed()),
            None => self.wall(),
        };
        BudgetLeft {
            calls: self.declaration.max_calls.saturating_sub(st.calls_started),
            tokens: self.declaration.max_tokens.saturating_sub(st.tokens_used),
            wall_secs: wall_left.as_secs(),
        }
    }
}

/// One dispatch's running broker.
pub struct Broker {
    shared: Arc<Shared>,
    port: u16,
    accept: Option<JoinHandle<()>>,
}

impl std::fmt::Debug for Broker {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Broker")
            .field("port", &self.port)
            .field("token", &"<redacted>")
            .finish()
    }
}

/// A fresh 256-bit capability, hex-encoded.
fn mint_token() -> String {
    hex::encode(rand::random::<[u8; 32]>())
}

impl Broker {
    /// Start a broker on `127.0.0.1:0`.
    pub async fn start(
        invoker: Arc<dyn ModelInvoker>,
        config: BrokerConfig,
        sink: Arc<dyn LogSink>,
    ) -> std::io::Result<Self> {
        Self::start_on(
            SocketAddr::from((Ipv4Addr::LOCALHOST, 0)),
            invoker,
            config,
            sink,
        )
        .await
    }

    /// Start on `addr`. Production always passes the IPv4 loopback; the seam
    /// exists so a test can make the bind fail.
    pub(crate) async fn start_on(
        addr: SocketAddr,
        invoker: Arc<dyn ModelInvoker>,
        config: BrokerConfig,
        sink: Arc<dyn LogSink>,
    ) -> std::io::Result<Self> {
        let listener = TcpListener::bind(addr).await?;
        let port = listener.local_addr()?.port();
        let shared = Arc::new(Shared {
            declaration: config.declaration,
            invoker,
            sink,
            token: mint_token(),
            host_header: format!("127.0.0.1:{port}"),
            permits: Arc::new(Semaphore::new(config.max_in_flight.max(1))),
            call_timeout: config.call_timeout,
            hold_log_interval: config.hold_log_interval,
            min_exhaustion_hold: config.min_exhaustion_hold,
            write_timeout: config.write_timeout,
            shutdown: CancellationToken::new(),
            state: Mutex::new(State {
                next_index: 0,
                calls_started: 0,
                tokens_used: 0,
                first_call: None,
                step: String::new(),
                ledger: LedgerBuilder::default(),
                exhausted_reset_at: None,
                rejections: std::collections::BTreeMap::new(),
                connections_shed: 0,
            }),
        });
        let accept = tokio::spawn(accept_loop(listener, shared.clone()));
        Ok(Self {
            shared,
            port,
            accept: Some(accept),
        })
    }

    /// `http://127.0.0.1:<port>`.
    pub fn url(&self) -> String {
        format!("http://127.0.0.1:{}", self.port)
    }

    pub fn port(&self) -> u16 {
        self.port
    }

    /// The dispatch's capability. A secret: it is exported only to
    /// `model = true` steps and never logged.
    pub fn token(&self) -> &str {
        &self.shared.token
    }

    /// Attribute the calls that follow to `step`.
    pub fn set_step(&self, step: &str) {
        self.shared.state().step = step.to_string();
    }

    /// Stop accepting, cancel every in-flight call (each is recorded as
    /// `cancelled`), wait for the connection tasks to finish, and hand back
    /// the ledger. Call this BEFORE the dispatch's cleanup.
    pub async fn stop(mut self) -> BrokerOutcome {
        self.shared.shutdown.cancel();
        if let Some(accept) = self.accept.take() {
            let _ = accept.await;
        }
        let mut st = self.shared.state();
        let wall_used = st
            .first_call
            .map(|(t0, _)| t0.elapsed().as_secs())
            .unwrap_or(0);
        let d = &self.shared.declaration;
        let budget = BudgetReport {
            max_calls: d.max_calls,
            max_tokens: d.max_tokens,
            max_wall_secs: d.max_wall_secs,
            calls_used: st.calls_started,
            tokens_used: st.tokens_used,
            wall_secs_used: wall_used,
            tokens_overshoot: st.tokens_used.saturating_sub(d.max_tokens),
        };
        let ledger = std::mem::take(&mut st.ledger).finish(budget);
        BrokerOutcome {
            ledger,
            calls_used: st.calls_started,
            exhausted_reset_at: st.exhausted_reset_at,
        }
    }
}

impl Drop for Broker {
    fn drop(&mut self) {
        self.shared.shutdown.cancel();
        if let Some(accept) = self.accept.take() {
            accept.abort();
        }
    }
}

async fn accept_loop(listener: TcpListener, shared: Arc<Shared>) {
    let mut conns: JoinSet<()> = JoinSet::new();
    loop {
        tokio::select! {
            _ = shared.shutdown.cancelled() => break,
            accepted = listener.accept() => match accepted {
                Ok((stream, _)) => {
                    while conns.try_join_next().is_some() {}
                    if conns.len() >= MAX_CONNECTIONS {
                        drop(stream);
                        let shed = {
                            let mut st = shared.state();
                            st.connections_shed += 1;
                            st.connections_shed
                        };
                        if log_occurrence(shed) {
                            shared.sink.push(&format!(
                                "[ci-model] closed a connection unanswered: {MAX_CONNECTIONS} \
                                 already open ({shed} so far)"
                            ));
                        }
                    } else {
                        conns.spawn(serve(shared.clone(), stream));
                    }
                }
                Err(e) => {
                    shared.sink.push(&format!("[ci-model] broker accept failed: {e}"));
                    tokio::time::sleep(Duration::from_millis(50)).await;
                }
            },
        }
        while conns.try_join_next().is_some() {}
    }
    drop(listener);
    // Every connection task watches the shutdown token, so this ends
    // promptly; it is awaited so every cancelled call is in the ledger before
    // `stop` reads it.
    while conns.join_next().await.is_some() {}
}

/// A parsed request head.
struct Head {
    method: String,
    target: String,
    headers: Vec<(String, String)>,
}

impl Head {
    fn all(&self, name: &str) -> Vec<&str> {
        self.headers
            .iter()
            .filter(|(k, _)| k == name)
            .map(|(_, v)| v.as_str())
            .collect()
    }
}

/// A refusal before the request is admitted as a call: no ledger row.
struct Rejection {
    status: (u16, &'static str),
    subtype: ErrorSubtype,
    message: String,
}

impl Rejection {
    fn new(status: (u16, &'static str), subtype: ErrorSubtype, message: &str) -> Self {
        Self {
            status,
            subtype,
            message: message.to_string(),
        }
    }
}

const FORBIDDEN: (u16, &str) = (403, "Forbidden");

async fn read_head(rd: &mut BufReader<OwnedReadHalf>) -> Result<Head, Rejection> {
    let bad = |m: &str| Rejection::new((400, "Bad Request"), ErrorSubtype::BadRequest, m);
    let mut total = 0usize;
    let mut lines: Vec<String> = Vec::new();
    loop {
        let mut line = Vec::new();
        let n = rd
            .read_until(b'\n', &mut line)
            .await
            .map_err(|_| bad("the request could not be read"))?;
        if n == 0 {
            return Err(bad("the connection closed before the request head ended"));
        }
        total += n;
        if total > MAX_REQUEST_HEAD_BYTES {
            return Err(bad("the request head is too large"));
        }
        let text = String::from_utf8(line).map_err(|_| bad("the request head is not UTF-8"))?;
        let text = text.trim_end_matches(['\r', '\n']).to_string();
        if text.is_empty() {
            break;
        }
        lines.push(text);
    }
    let mut it = lines.into_iter();
    let request_line = it.next().ok_or_else(|| bad("no request line"))?;
    let mut parts = request_line.split(' ');
    let (Some(method), Some(target), Some(version), None) =
        (parts.next(), parts.next(), parts.next(), parts.next())
    else {
        return Err(bad("malformed request line"));
    };
    if !version.starts_with("HTTP/1.") {
        return Err(bad("only HTTP/1.x is served"));
    }
    let mut headers = Vec::new();
    for line in it {
        let Some((k, v)) = line.split_once(':') else {
            return Err(bad("malformed header line"));
        };
        headers.push((k.trim().to_ascii_lowercase(), v.trim().to_string()));
    }
    Ok(Head {
        method: method.to_string(),
        target: target.to_string(),
        headers,
    })
}

/// The request checks, in order. `Ok` carries the declared body length.
fn check(shared: &Shared, head: &Head) -> Result<usize, Rejection> {
    let hosts = head.all("host");
    if hosts.len() != 1 || hosts[0] != shared.host_header {
        return Err(Rejection::new(
            FORBIDDEN,
            ErrorSubtype::Unauthorized,
            "the Host header must be exactly the broker's 127.0.0.1:<port>",
        ));
    }
    if !head.all("origin").is_empty() {
        return Err(Rejection::new(
            FORBIDDEN,
            ErrorSubtype::Unauthorized,
            "a request carrying an Origin header is refused (browsers are not clients)",
        ));
    }
    if head.target.contains('?') {
        return Err(Rejection::new(
            (401, "Unauthorized"),
            ErrorSubtype::Unauthorized,
            "a query string is refused: the capability travels only in Authorization: Bearer",
        ));
    }
    let auth = head.all("authorization");
    let presented = match auth.as_slice() {
        [one] => one.strip_prefix("Bearer ").unwrap_or(""),
        _ => "",
    };
    if presented.is_empty()
        || !crate::model::secret_eq(presented.as_bytes(), shared.token.as_bytes())
    {
        return Err(Rejection::new(
            (401, "Unauthorized"),
            ErrorSubtype::Unauthorized,
            "missing or wrong capability",
        ));
    }
    if head.target != COMPLETE_PATH {
        return Err(Rejection::new(
            (404, "Not Found"),
            ErrorSubtype::BadRequest,
            "the broker serves POST /v1/complete only",
        ));
    }
    if head.method != "POST" {
        return Err(Rejection::new(
            (405, "Method Not Allowed"),
            ErrorSubtype::BadRequest,
            "the broker serves POST /v1/complete only",
        ));
    }
    let len = match head.all("content-length").as_slice() {
        [one] => one.parse::<usize>().map_err(|_| {
            Rejection::new(
                (400, "Bad Request"),
                ErrorSubtype::BadRequest,
                "Content-Length is not a number",
            )
        })?,
        _ => {
            return Err(Rejection::new(
                (411, "Length Required"),
                ErrorSubtype::BadRequest,
                "exactly one Content-Length is required",
            ))
        }
    };
    if len > MAX_REQUEST_BODY_BYTES {
        return Err(Rejection::new(
            ErrorSubtype::PromptTooLarge.http_status(),
            ErrorSubtype::PromptTooLarge,
            "the request body exceeds the broker's cap",
        ));
    }
    Ok(len)
}

/// Write one response, bounded by `write_timeout`: a client that never reads
/// cannot hold the connection task (or `stop`) past it.
async fn respond(
    wr: &mut tokio::net::tcp::OwnedWriteHalf,
    write_timeout: Duration,
    status: (u16, &str),
    env: &Envelope,
) {
    let _ = tokio::time::timeout(write_timeout, write_response(wr, status, env)).await;
}

async fn write_response(
    wr: &mut tokio::net::tcp::OwnedWriteHalf,
    status: (u16, &str),
    env: &Envelope,
) {
    let body = serde_json::to_vec(env).unwrap_or_default();
    let head = format!(
        "HTTP/1.1 {} {}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
        status.0,
        status.1,
        body.len()
    );
    let _ = wr.write_all(head.as_bytes()).await;
    let _ = wr.write_all(&body).await;
    let _ = wr.shutdown().await;
}

/// Resolves when the client closes its side (or the socket errors). Any bytes
/// it sends after its request are discarded.
async fn disconnected(rd: &mut BufReader<OwnedReadHalf>) {
    let mut buf = [0u8; 256];
    loop {
        match rd.read(&mut buf).await {
            Ok(0) | Err(_) => return,
            Ok(_) => continue,
        }
    }
}

fn unix_secs(t: SystemTime) -> u64 {
    t.duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

/// How one admitted call ended.
enum CallEnd {
    Ok(InvokeReply),
    Failed {
        subtype: ErrorSubtype,
        message: String,
        reset_at: Option<u64>,
        /// The reply of a call that completed but cannot be returned (too
        /// large): its tokens were spent and are counted.
        spent: Option<InvokeReply>,
    },
}

fn failed(subtype: ErrorSubtype, message: impl Into<String>) -> CallEnd {
    CallEnd::Failed {
        subtype,
        message: message.into(),
        reset_at: None,
        spent: None,
    }
}

async fn serve(shared: Arc<Shared>, stream: TcpStream) {
    let peer_ok = shared.invoker.accept_peer(&stream);
    let (rd, mut wr) = stream.into_split();
    let mut rd = BufReader::new(rd);
    if !peer_ok {
        shared
            .sink
            .push("[ci-model] refused a connection: the host does not admit this peer");
        let env = Envelope::error(
            ErrorSubtype::Unauthorized,
            "this peer is not admitted".to_string(),
            None,
            BrokerBlock::default(),
        );
        respond(&mut wr, shared.write_timeout, FORBIDDEN, &env).await;
        return;
    }

    // ── Head, checks, body ──
    let admitted = tokio::time::timeout(REQUEST_READ_TIMEOUT, async {
        let head = read_head(&mut rd).await?;
        let len = check(&shared, &head)?;
        let mut body = vec![0u8; len];
        rd.read_exact(&mut body).await.map_err(|_| {
            Rejection::new(
                (400, "Bad Request"),
                ErrorSubtype::BadRequest,
                "the body ended before Content-Length",
            )
        })?;
        // The serde error is NOT echoed: it can quote body content, and the
        // rejection message reaches the dispatch log.
        serde_json::from_slice::<CompleteRequest>(&body).map_err(|_| {
            Rejection::new(
                (400, "Bad Request"),
                ErrorSubtype::BadRequest,
                "the body is not a JSON {model, prompt} request",
            )
        })
    });
    // Reading the request observes the dispatch ending: a stalled client
    // cannot hold `stop` for the read timeout.
    let admitted = tokio::select! {
        r = admitted => r.unwrap_or_else(|_| {
            Err(Rejection::new(
                (408, "Request Timeout"),
                ErrorSubtype::BadRequest,
                "the request was not sent in time",
            ))
        }),
        _ = shared.shutdown.cancelled() => return,
    };
    let request = match admitted {
        Ok(r) => r,
        Err(rej) => {
            // A fixed category per subtype, rate-limited: a looping or
            // hostile local client cannot fill the dispatch log.
            let n = {
                let mut st = shared.state();
                let n = st.rejections.entry(rej.subtype.as_str()).or_insert(0);
                *n += 1;
                *n
            };
            if log_occurrence(n) {
                shared.sink.push(&format!(
                    "[ci-model] refused a request: {} ({n} so far)",
                    rej.subtype.as_str()
                ));
            }
            // No `budget_left`: a request refused before admission learns
            // nothing about the dispatch.
            let env = Envelope::error(rej.subtype, rej.message, None, BrokerBlock::default());
            respond(&mut wr, shared.write_timeout, rej.status, &env).await;
            return;
        }
    };

    // ── Admitted as a call: it gets an index and a ledger row ──
    let (index, step) = {
        let mut st = shared.state();
        st.next_index += 1;
        (st.next_index, st.step.clone())
    };
    let requested = request
        .model
        .clone()
        .unwrap_or_else(|| shared.declaration.default_model().name.clone());
    let started = Instant::now();
    let end = run_call(&shared, &mut rd, index, &requested, request).await;

    // ── Account, record, log, answer ──
    let latency_ms = started.elapsed().as_millis() as u64;
    let (row, env, status) = {
        let mut st = shared.state();
        let (row, env, status) = match end {
            CallEnd::Ok(reply) => {
                if let Some(u) = reply.usage {
                    st.tokens_used = st.tokens_used.saturating_add(u.total());
                }
                let row = reply_row(index, &step, &requested, &reply, latency_ms, "ok");
                let usage = reply.usage.map(|u| Usage {
                    input_tokens: u.input,
                    output_tokens: u.output,
                    cache_creation_input_tokens: u.cache_creation,
                    cache_read_input_tokens: u.cache_read,
                });
                let mut model_usage = std::collections::BTreeMap::new();
                if let Some(id) = &reply.model_id {
                    let u = reply.usage.unwrap_or_default();
                    model_usage.insert(
                        id.clone(),
                        ModelUsage {
                            input_tokens: u.input,
                            output_tokens: u.output,
                            cache_read_input_tokens: u.cache_read,
                            cache_creation_input_tokens: u.cache_creation,
                            cost_usd: reply.cost_usd,
                        },
                    );
                }
                let env = Envelope::success(
                    reply.result,
                    usage,
                    model_usage,
                    reply.cost_usd,
                    BrokerBlock::default(),
                );
                // Measure the SERIALIZED envelope (JSON escaping can inflate
                // the answer), with room for the broker block filled below,
                // so nothing the broker sends exceeds what the shim reads.
                let size = serde_json::to_vec(&env)
                    .map(|v| v.len())
                    .unwrap_or(usize::MAX);
                if size.saturating_add(BROKER_BLOCK_ROOM) > MAX_ENVELOPE_BYTES {
                    let mut row = row;
                    row.outcome = ErrorSubtype::ResponseTooLarge.as_str();
                    let env = Envelope::error(
                        ErrorSubtype::ResponseTooLarge,
                        format!("the answer's envelope exceeds {MAX_ENVELOPE_BYTES} bytes"),
                        None,
                        BrokerBlock::default(),
                    );
                    (row, env, ErrorSubtype::ResponseTooLarge.http_status())
                } else {
                    (row, env, (200, "OK"))
                }
            }
            CallEnd::Failed {
                subtype,
                message,
                reset_at,
                spent,
            } => {
                let outcome = subtype.as_str();
                let row = match &spent {
                    Some(reply) => {
                        if let Some(u) = reply.usage {
                            st.tokens_used = st.tokens_used.saturating_add(u.total());
                        }
                        reply_row(index, &step, &requested, reply, latency_ms, &outcome)
                    }
                    None => {
                        let mut row = CallRow::refused(index, &step, &requested, outcome);
                        row.latency_ms = latency_ms;
                        row
                    }
                };
                if subtype == ErrorSubtype::AllAccountsExhausted {
                    st.exhausted_reset_at = reset_at.max(st.exhausted_reset_at);
                }
                let env = Envelope::error(subtype, message, reset_at, BrokerBlock::default());
                (row, env, subtype.http_status())
            }
        };
        (row, env, status)
    };
    let mut env = env;
    let refusals_so_far = {
        let mut st = shared.state();
        let n = st.ledger.push(row.clone());
        env.broker = BrokerBlock {
            call_index: index,
            budget_left: Some(shared.budget_left(&st)),
        };
        n
    };
    // A call that reached the host is always logged; a refusal before the
    // host (a harness looping on a spent budget) is rate-limited.
    if refusals_so_far == 0 || log_occurrence(refusals_so_far) {
        let tally = if refusals_so_far > 0 {
            format!(" ({refusals_so_far} so far)")
        } else {
            String::new()
        };
        shared.sink.push(&format!(
            "[ci-model] call {index} step={} model={} outcome={} latency_ms={latency_ms}{tally}",
            row.step, row.requested, row.outcome
        ));
    }
    respond(&mut wr, shared.write_timeout, status, &env).await;
}

/// Room left in [`MAX_ENVELOPE_BYTES`] for the broker block, which is filled
/// after the size check (a few integers).
const BROKER_BLOCK_ROOM: usize = 256;

fn reply_row(
    index: u64,
    step: &str,
    requested: &str,
    reply: &InvokeReply,
    latency_ms: u64,
    outcome: &str,
) -> CallRow {
    CallRow {
        index,
        step: step.to_string(),
        requested: requested.to_string(),
        model_id: reply.model_id.clone(),
        route: Some(reply.route.clone()),
        account: reply.account.clone(),
        input_tokens: reply.usage.map(|u| u.input),
        output_tokens: reply.usage.map(|u| u.output),
        cache_creation_tokens: reply.usage.map(|u| u.cache_creation),
        cache_read_tokens: reply.usage.map(|u| u.cache_read),
        latency_ms,
        outcome: outcome.to_string(),
        attempts: reply.attempts.max(1),
        cost_microusd: reply.cost_usd.and_then(usd_to_microusd),
    }
}

/// Admit, budget and run one call.
async fn run_call(
    shared: &Shared,
    rd: &mut BufReader<OwnedReadHalf>,
    index: u64,
    requested: &str,
    request: CompleteRequest,
) -> CallEnd {
    if request.prompt.len() > MAX_PROMPT_BYTES {
        return failed(
            ErrorSubtype::PromptTooLarge,
            format!("the prompt exceeds {MAX_PROMPT_BYTES} bytes"),
        );
    }
    let Some(model) = shared.declaration.declared(requested) else {
        let declared: Vec<&str> = shared
            .declaration
            .models
            .iter()
            .map(|m| m.name.as_str())
            .collect();
        return failed(
            ErrorSubtype::ModelNotDeclared,
            format!("model {requested:?} is not declared in [model] (declared: {declared:?})"),
        );
    };

    // ── A slot: at most `max_in_flight` calls run at once ──
    let _permit = tokio::select! {
        p = shared.permits.clone().acquire_owned() => match p {
            Ok(p) => p,
            Err(_) => return failed(ErrorSubtype::Cancelled, "the broker is stopping"),
        },
        _ = disconnected(rd) => return failed(ErrorSubtype::Cancelled, "the client disconnected"),
        _ = shared.shutdown.cancelled() => return failed(ErrorSubtype::Cancelled, "the dispatch ended"),
    };

    // ── The budget, checked once the call holds its slot ──
    let (wall_deadline, wall_deadline_at) = {
        let mut st = shared.state();
        let d = &shared.declaration;
        if let Some((t0, _)) = st.first_call {
            if t0.elapsed() >= shared.wall() {
                return failed(
                    ErrorSubtype::BudgetExhausted(Budget::Wall),
                    format!("the dispatch's {}s wall budget is spent", d.max_wall_secs),
                );
            }
        }
        if st.calls_started >= d.max_calls {
            return failed(
                ErrorSubtype::BudgetExhausted(Budget::Calls),
                format!("the dispatch's {} calls are spent", d.max_calls),
            );
        }
        if st.tokens_used >= d.max_tokens {
            return failed(
                ErrorSubtype::BudgetExhausted(Budget::Tokens),
                format!(
                    "the dispatch's {} tokens are spent ({} used)",
                    d.max_tokens, st.tokens_used
                ),
            );
        }
        st.calls_started += 1;
        let (t0, at0) = *st
            .first_call
            .get_or_insert_with(|| (Instant::now(), SystemTime::now()));
        (t0 + shared.wall(), at0 + shared.wall())
    };

    let invoke = InvokeRequest {
        call_index: index,
        requested: model.name.clone(),
        cli_model: model.cli_model.clone(),
        prompt: request.prompt,
    };
    let mut broker_attempts: u32 = 0;
    let mut holds: u32 = 0;
    loop {
        let remaining = wall_deadline.saturating_duration_since(Instant::now());
        if remaining.is_zero() {
            return failed(
                ErrorSubtype::BudgetExhausted(Budget::Wall),
                "the wall budget ran out",
            );
        }
        let wall_bound = remaining <= shared.call_timeout;
        let limit = remaining.min(shared.call_timeout);
        broker_attempts += 1;
        let call = shared.invoker.complete(invoke.clone());
        let result = tokio::select! {
            r = tokio::time::timeout(limit, call) => r,
            _ = disconnected(rd) => return failed(ErrorSubtype::Cancelled, "the client disconnected"),
            _ = shared.shutdown.cancelled() => return failed(ErrorSubtype::Cancelled, "the dispatch ended"),
        };
        match result {
            Err(_) if wall_bound => {
                return failed(
                    ErrorSubtype::BudgetExhausted(Budget::Wall),
                    "the wall budget ran out during the call",
                )
            }
            Err(_) => {
                return failed(
                    ErrorSubtype::Timeout,
                    format!("the call ran past {}s", shared.call_timeout.as_secs()),
                )
            }
            Ok(Ok(mut reply)) => {
                reply.attempts = reply.attempts.max(1) + broker_attempts - 1;
                if reply.result.len() > MAX_RESPONSE_BYTES {
                    return CallEnd::Failed {
                        subtype: ErrorSubtype::ResponseTooLarge,
                        message: format!("the answer exceeds {MAX_RESPONSE_BYTES} bytes"),
                        reset_at: None,
                        spent: Some(InvokeReply {
                            result: String::new(),
                            ..reply
                        }),
                    };
                }
                return CallEnd::Ok(reply);
            }
            Ok(Err(InvokeError::Timeout)) => {
                return failed(ErrorSubtype::Timeout, "the host's call deadline ran out")
            }
            Ok(Err(InvokeError::Provider { message, .. })) => {
                return failed(ErrorSubtype::ProviderError, message)
            }
            Ok(Err(InvokeError::AllAccountsExhausted { reset_at })) => {
                let reset_unix = unix_secs(reset_at);
                // Never hold for less than the minimum, even for a reset
                // already past: a host repeating a stale reset must not turn
                // the hold into a retry storm.
                let hold_until = reset_at.max(SystemTime::now() + shared.min_exhaustion_hold);
                holds += 1;
                if holds > MAX_EXHAUSTION_HOLDS {
                    return CallEnd::Failed {
                        subtype: ErrorSubtype::AllAccountsExhausted,
                        message: format!(
                            "every model account is still exhausted after {MAX_EXHAUSTION_HOLDS} \
                             holds (last reported reset: unix {reset_unix})"
                        ),
                        reset_at: Some(reset_unix),
                        spent: None,
                    };
                }
                if hold_until > wall_deadline_at {
                    return CallEnd::Failed {
                        subtype: ErrorSubtype::AllAccountsExhausted,
                        message: format!(
                            "every model account is exhausted; the earliest reset (unix \
                             {reset_unix}) falls outside the dispatch's wall budget"
                        ),
                        reset_at: Some(reset_unix),
                        spent: None,
                    };
                }
                // Hold until the reset, logging each interval (the log line
                // rides the progress POST that renews the dispatch lease).
                loop {
                    let wait = hold_until
                        .duration_since(SystemTime::now())
                        .unwrap_or(Duration::ZERO);
                    if wait.is_zero() {
                        break;
                    }
                    shared.sink.push(&format!(
                        "[ci-model] call {index} held: every model account is exhausted; \
                         retrying at the reset in {}s",
                        wait.as_secs()
                    ));
                    tokio::select! {
                        _ = tokio::time::sleep(wait.min(shared.hold_log_interval)) => {}
                        _ = disconnected(rd) => return failed(ErrorSubtype::Cancelled, "the client disconnected"),
                        _ = shared.shutdown.cancelled() => return failed(ErrorSubtype::Cancelled, "the dispatch ended"),
                    }
                }
            }
        }
    }
}

/// A plain-text HTTP exchange against `port`, for tests in this crate.
#[cfg(test)]
pub(crate) async fn raw_exchange(port: u16, request: &[u8]) -> (u16, String) {
    let mut s = TcpStream::connect(("127.0.0.1", port)).await.unwrap();
    s.write_all(request).await.unwrap();
    let mut buf = Vec::new();
    let _ = s.read_to_end(&mut buf).await;
    let text = String::from_utf8_lossy(&buf).to_string();
    let status = text
        .split(' ')
        .nth(1)
        .and_then(|c| c.parse().ok())
        .unwrap_or(0);
    let body = text
        .split_once("\r\n\r\n")
        .map(|(_, b)| b.to_string())
        .unwrap_or_default();
    (status, body)
}

#[cfg(test)]
mod tests;
