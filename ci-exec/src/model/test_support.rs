//! A fake [`ModelInvoker`] and helpers for this crate's model tests.

use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex};

use crate::host::{BoxFuture, InvokeError, InvokeReply, InvokeRequest, ModelInvoker, TokenUsage};
use crate::model::declaration::{resolve_model, ModelDeclaration};
use crate::report::LogSink;

/// What a fake call does.
pub(crate) type Behavior = Arc<
    dyn Fn(InvokeRequest) -> BoxFuture<'static, Result<InvokeReply, InvokeError>> + Send + Sync,
>;

/// A configurable invoker: every call runs `behavior`.
pub(crate) struct FakeInvoker {
    pub behavior: Behavior,
    pub ready: Result<(), String>,
    pub shim: Option<Vec<String>>,
    pub admit_peers: bool,
    pub calls: AtomicU64,
}

impl FakeInvoker {
    pub fn new(behavior: Behavior) -> Self {
        Self {
            behavior,
            ready: Ok(()),
            shim: None,
            admit_peers: true,
            calls: AtomicU64::new(0),
        }
    }

    /// Answers `answer` with `tokens` input tokens, immediately.
    pub fn answering(answer: &'static str, tokens: u64) -> Self {
        Self::new(Arc::new(move |_req| {
            Box::pin(async move { Ok(ok_reply(answer, tokens)) })
        }))
    }
}

impl ModelInvoker for FakeInvoker {
    fn complete(
        &self,
        request: InvokeRequest,
    ) -> BoxFuture<'static, Result<InvokeReply, InvokeError>> {
        self.calls.fetch_add(1, Ordering::SeqCst);
        (self.behavior)(request)
    }
    fn ready(&self) -> Result<(), String> {
        self.ready.clone()
    }
    fn shim_argv(&self) -> Option<Vec<String>> {
        self.shim.clone()
    }
    fn accept_peer(&self, _peer: &tokio::net::TcpStream) -> bool {
        self.admit_peers
    }
}

/// A successful reply spending `tokens` input tokens.
pub(crate) fn ok_reply(answer: &str, tokens: u64) -> InvokeReply {
    InvokeReply {
        result: answer.to_string(),
        model_id: Some("claude-sonnet-4-5-20250929".to_string()),
        usage: Some(TokenUsage {
            input: tokens,
            output: 0,
            cache_creation: 0,
            cache_read: 0,
        }),
        cost_usd: Some(0.001),
        route: "subscription".to_string(),
        account: Some("acct-a".to_string()),
        attempts: 1,
    }
}

/// A declaration of `claude-sonnet` (and `claude-haiku`) with these budgets.
pub(crate) fn declaration(max_calls: u64, max_tokens: u64, max_wall_secs: u64) -> ModelDeclaration {
    ModelDeclaration {
        models: vec![
            resolve_model("claude-sonnet").unwrap(),
            resolve_model("claude-haiku").unwrap(),
        ],
        max_calls,
        max_tokens,
        max_wall_secs,
    }
}

/// A log sink that keeps every line.
#[derive(Clone, Default)]
pub(crate) struct Lines(pub Arc<Mutex<Vec<String>>>);

impl Lines {
    pub fn all(&self) -> Vec<String> {
        self.0.lock().unwrap().clone()
    }
}

impl LogSink for Lines {
    fn push(&self, line: &str) {
        self.0.lock().unwrap().push(line.to_string());
    }
}
