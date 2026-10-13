//! The model broker's wire protocol (plan
//! `2026-10-13-ci-node-runs-model-dependent-eval-steps`, D4).
//!
//! `POST /v1/complete` with `{"model": <name>|null, "prompt": <text>}` and the
//! dispatch's capability in `Authorization: Bearer`. The answer is a
//! `claude -p --output-format json`-compatible [`Envelope`] — `type`,
//! `subtype`, `is_error`, `result`, `usage`, `modelUsage`, optional
//! `total_cost_usd` — plus a `broker` block carrying the call index and the
//! budget left. Errors are envelopes too, with a typed [`ErrorSubtype`] and a
//! `retryable` flag, so a harness that already parses `claude -p` output reads
//! a broker error the same way.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

/// The one route the broker serves.
pub const COMPLETE_PATH: &str = "/v1/complete";

/// The largest prompt the broker accepts, in bytes.
pub const MAX_PROMPT_BYTES: usize = 1024 * 1024;
/// The largest answer the broker returns, in bytes. A larger one fails the call
/// `response_too_large` (its tokens are still counted: they were spent).
pub const MAX_RESPONSE_BYTES: usize = 1024 * 1024;
/// The largest request body the broker reads. JSON escaping can inflate a
/// prompt (a control character becomes six bytes), so the body cap is wider
/// than the prompt cap; the prompt cap is applied to the decoded prompt.
pub const MAX_REQUEST_BODY_BYTES: usize = 6 * MAX_PROMPT_BYTES + 4096;
/// The largest request head (request line + headers) the broker reads.
pub const MAX_REQUEST_HEAD_BYTES: usize = 16 * 1024;

/// A step's request.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CompleteRequest {
    /// A declared model name; `None` takes the first declared model.
    #[serde(default)]
    pub model: Option<String>,
    pub prompt: String,
}

/// Which budget ran out.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Budget {
    Calls,
    Tokens,
    Wall,
}

impl Budget {
    pub fn as_str(self) -> &'static str {
        match self {
            Budget::Calls => "calls",
            Budget::Tokens => "tokens",
            Budget::Wall => "wall",
        }
    }
}

/// Why a call did not produce an answer. Closed.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ErrorSubtype {
    /// `budget_exhausted:{calls|tokens|wall}`.
    BudgetExhausted(Budget),
    /// The request named a model `[model]` does not declare.
    ModelNotDeclared,
    /// Every model account is exhausted and the earliest reset falls outside
    /// the dispatch's remaining wall budget. Carries `reset_at`.
    AllAccountsExhausted,
    ProviderError,
    Timeout,
    PromptTooLarge,
    ResponseTooLarge,
    /// No, or a wrong, capability; or a request shape a browser or a DNS
    /// rebinding would send (an `Origin`, a foreign `Host`, a query string).
    Unauthorized,
    /// The client disconnected or the dispatch ended mid-call.
    Cancelled,
    /// A request that is not `POST /v1/complete` with a JSON body.
    BadRequest,
}

impl ErrorSubtype {
    pub fn as_str(self) -> String {
        match self {
            ErrorSubtype::BudgetExhausted(b) => format!("budget_exhausted:{}", b.as_str()),
            ErrorSubtype::ModelNotDeclared => "model_not_declared".to_string(),
            ErrorSubtype::AllAccountsExhausted => "all_accounts_exhausted".to_string(),
            ErrorSubtype::ProviderError => "provider_error".to_string(),
            ErrorSubtype::Timeout => "timeout".to_string(),
            ErrorSubtype::PromptTooLarge => "prompt_too_large".to_string(),
            ErrorSubtype::ResponseTooLarge => "response_too_large".to_string(),
            ErrorSubtype::Unauthorized => "unauthorized".to_string(),
            ErrorSubtype::Cancelled => "cancelled".to_string(),
            ErrorSubtype::BadRequest => "bad_request".to_string(),
        }
    }

    /// Whether the same request may succeed later in this dispatch. A spent
    /// budget, an undeclared model or an oversize prompt never will.
    pub fn retryable(self) -> bool {
        matches!(
            self,
            ErrorSubtype::AllAccountsExhausted | ErrorSubtype::Timeout
        )
    }

    /// The HTTP status the broker answers with.
    pub fn http_status(self) -> (u16, &'static str) {
        match self {
            ErrorSubtype::BudgetExhausted(_) => (429, "Too Many Requests"),
            ErrorSubtype::ModelNotDeclared | ErrorSubtype::BadRequest => (400, "Bad Request"),
            ErrorSubtype::AllAccountsExhausted | ErrorSubtype::Cancelled => {
                (503, "Service Unavailable")
            }
            ErrorSubtype::ProviderError | ErrorSubtype::ResponseTooLarge => (502, "Bad Gateway"),
            ErrorSubtype::Timeout => (504, "Gateway Timeout"),
            ErrorSubtype::PromptTooLarge => (413, "Payload Too Large"),
            ErrorSubtype::Unauthorized => (401, "Unauthorized"),
        }
    }
}

/// `usage`, in the `claude -p` JSON shape.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Usage {
    pub input_tokens: u64,
    pub output_tokens: u64,
    pub cache_creation_input_tokens: u64,
    pub cache_read_input_tokens: u64,
}

/// One `modelUsage` entry, in the `claude -p` JSON shape.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ModelUsage {
    pub input_tokens: u64,
    pub output_tokens: u64,
    pub cache_read_input_tokens: u64,
    pub cache_creation_input_tokens: u64,
    #[serde(rename = "costUSD", default, skip_serializing_if = "Option::is_none")]
    pub cost_usd: Option<f64>,
}

/// What is left of the dispatch's budget after this call.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct BudgetLeft {
    pub calls: u64,
    pub tokens: u64,
    pub wall_secs: u64,
}

/// The broker's own block in every envelope.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct BrokerBlock {
    /// The call's 1-based index in the dispatch; 0 for a request refused
    /// before it was admitted as a call (unauthorized, malformed).
    pub call_index: u64,
    pub budget_left: BudgetLeft,
}

/// The `claude -p --output-format json`-compatible answer.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Envelope {
    #[serde(rename = "type")]
    pub kind: String,
    /// `success`, or the error subtype.
    pub subtype: String,
    pub is_error: bool,
    /// The answer, or the error's human-readable message.
    pub result: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub usage: Option<Usage>,
    #[serde(
        rename = "modelUsage",
        default,
        skip_serializing_if = "BTreeMap::is_empty"
    )]
    pub model_usage: BTreeMap<String, ModelUsage>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub total_cost_usd: Option<f64>,
    /// On an error: whether the same request may succeed later.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub retryable: Option<bool>,
    /// On `all_accounts_exhausted`: the earliest reset, unix seconds.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reset_at: Option<u64>,
    #[serde(default)]
    pub broker: BrokerBlock,
}

impl Envelope {
    /// A successful answer.
    pub fn success(
        result: String,
        usage: Option<Usage>,
        model_usage: BTreeMap<String, ModelUsage>,
        total_cost_usd: Option<f64>,
        broker: BrokerBlock,
    ) -> Self {
        Self {
            kind: "result".to_string(),
            subtype: "success".to_string(),
            is_error: false,
            result,
            usage,
            model_usage,
            total_cost_usd,
            retryable: None,
            reset_at: None,
            broker,
        }
    }

    /// An error answer.
    pub fn error(
        subtype: ErrorSubtype,
        message: String,
        reset_at: Option<u64>,
        broker: BrokerBlock,
    ) -> Self {
        Self {
            kind: "result".to_string(),
            subtype: subtype.as_str(),
            is_error: true,
            result: message,
            usage: None,
            model_usage: BTreeMap::new(),
            total_cost_usd: None,
            retryable: Some(subtype.retryable()),
            reset_at,
            broker,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn subtypes_speak_the_closed_vocabulary() {
        let all = [
            (
                ErrorSubtype::BudgetExhausted(Budget::Calls),
                "budget_exhausted:calls",
            ),
            (
                ErrorSubtype::BudgetExhausted(Budget::Tokens),
                "budget_exhausted:tokens",
            ),
            (
                ErrorSubtype::BudgetExhausted(Budget::Wall),
                "budget_exhausted:wall",
            ),
            (ErrorSubtype::ModelNotDeclared, "model_not_declared"),
            (ErrorSubtype::AllAccountsExhausted, "all_accounts_exhausted"),
            (ErrorSubtype::ProviderError, "provider_error"),
            (ErrorSubtype::Timeout, "timeout"),
            (ErrorSubtype::PromptTooLarge, "prompt_too_large"),
            (ErrorSubtype::ResponseTooLarge, "response_too_large"),
            (ErrorSubtype::Unauthorized, "unauthorized"),
            (ErrorSubtype::Cancelled, "cancelled"),
            (ErrorSubtype::BadRequest, "bad_request"),
        ];
        for (s, wire) in all {
            assert_eq!(s.as_str(), wire);
        }
        assert!(!ErrorSubtype::BudgetExhausted(Budget::Calls).retryable());
        assert!(ErrorSubtype::AllAccountsExhausted.retryable());
        assert!(!ErrorSubtype::PromptTooLarge.retryable());
    }

    /// The envelope keeps `claude -p`'s key spellings, so a harness parses a
    /// broker answer the way it parses the CLI's.
    #[test]
    fn the_envelope_uses_the_cli_key_spellings() {
        let mut mu = BTreeMap::new();
        mu.insert(
            "claude-sonnet-4-5-20250929".to_string(),
            ModelUsage {
                input_tokens: 3,
                output_tokens: 4,
                cache_read_input_tokens: 5,
                cache_creation_input_tokens: 6,
                cost_usd: Some(0.01),
            },
        );
        let env = Envelope::success(
            "hi".to_string(),
            Some(Usage {
                input_tokens: 3,
                output_tokens: 4,
                cache_creation_input_tokens: 6,
                cache_read_input_tokens: 5,
            }),
            mu,
            Some(0.01),
            BrokerBlock::default(),
        );
        let v = serde_json::to_value(&env).unwrap();
        assert_eq!(v["type"], "result");
        assert_eq!(v["subtype"], "success");
        assert_eq!(v["is_error"], false);
        assert_eq!(v["usage"]["cache_read_input_tokens"], 5);
        assert_eq!(
            v["modelUsage"]["claude-sonnet-4-5-20250929"]["costUSD"],
            0.01
        );
        assert_eq!(v["total_cost_usd"], 0.01);
        assert!(v.get("retryable").is_none());
        let back: Envelope = serde_json::from_value(v).unwrap();
        assert_eq!(back, env);

        let err = serde_json::to_value(Envelope::error(
            ErrorSubtype::AllAccountsExhausted,
            "wait".to_string(),
            Some(42),
            BrokerBlock::default(),
        ))
        .unwrap();
        assert_eq!(err["is_error"], true);
        assert_eq!(err["retryable"], true);
        assert_eq!(err["reset_at"], 42);
    }

    #[test]
    fn a_request_refuses_unknown_keys() {
        assert!(serde_json::from_str::<CompleteRequest>(r#"{"prompt":"p"}"#).is_ok());
        assert!(serde_json::from_str::<CompleteRequest>(r#"{"prompt":"p","token":"x"}"#).is_err());
    }
}
