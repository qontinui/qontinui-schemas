//! Typed `/spawn-placement/{preview,temp}` client.
//!
//! Internalizes the URL building, HTTP call, and envelope-or-bare response
//! parsing the supervisor used to do inline at
//! `qontinui-supervisor/src/process/manager.rs::forward_window_position_env`.
//! Callers get a typed [`SpawnPlacementResponse`] back or a single
//! [`SpawnPlacementClientError`] enum covering every failure mode.

use qontinui_types::wire::placement::SpawnPlacementResponse;
use reqwest::Client;
use url::Url;

/// Overflow behavior when the requested slot/index is past the end of the
/// configured placement list.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Overflow {
    /// Default runner behavior: the runner returns `404` when the index is
    /// out of range. The client surfaces that as
    /// [`SpawnPlacementClientError::Status`] with the runner's body.
    Default,
    /// `overflow=wrap` — the runner round-robins via `index % len`.
    Wrap,
}

impl Overflow {
    fn as_query(&self) -> Option<&'static str> {
        match self {
            Self::Default => None,
            Self::Wrap => Some("wrap"),
        }
    }
}

/// All failure modes of [`SpawnPlacementClient`].
///
/// Variants are non-exhaustive (`#[non_exhaustive]`) so future variants don't
/// break callers that exhaustively match.
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum SpawnPlacementClientError {
    /// Network error, timeout, or local request building error from
    /// `reqwest`.
    #[error("HTTP error: {0}")]
    Http(#[from] reqwest::Error),
    /// Failed to construct the request URL (e.g. invalid base, query
    /// encoding bug).
    #[error("URL build error: {0}")]
    Url(#[from] url::ParseError),
    /// The endpoint returned a non-2xx status. The body is captured verbatim
    /// (truncation is the caller's responsibility) so the supervisor's
    /// existing log message can format the runner's `{"error": "..."}` JSON.
    #[error("Endpoint returned {status}: {body}")]
    Status {
        /// HTTP status code.
        status: reqwest::StatusCode,
        /// Response body (truncated by the caller if needed).
        body: String,
    },
    /// The body was 2xx but failed to parse as either the
    /// `ApiResponse<SpawnPlacementResponse>` envelope or the bare payload.
    #[error("Response parsing error: {0}")]
    Parse(String),
    /// The envelope had `success: false` (or no `data`). The runner sets
    /// this when it fails internally but still wants to return a structured
    /// error.
    #[error("Endpoint returned envelope without success/data: {error:?}")]
    EnvelopeError {
        /// The `error` field from the envelope, if any.
        error: Option<String>,
    },
}

/// Typed client for the runner's `/spawn-placement/{preview,temp}` endpoints.
///
/// Holds a base URL (e.g. `http://localhost:9876`) and a `reqwest::Client`.
/// The supervisor reuses its existing shared `http_client` so connection
/// pooling and timeouts are uniform across calls.
///
/// Per-call timeouts can be set on the inner `reqwest::Client` (recommended)
/// or by wrapping a call in `tokio::time::timeout`. The client doesn't
/// impose a default — the supervisor's existing 3s budget is set on the
/// shared client.
#[derive(Debug, Clone)]
pub struct SpawnPlacementClient {
    base: Url,
    http: Client,
}

impl SpawnPlacementClient {
    /// Construct a client with the given base URL and HTTP client.
    ///
    /// `base` should be the runner's API root (e.g. `http://localhost:9876`).
    /// The endpoint paths (`/spawn-placement/preview`, `/spawn-placement/temp`)
    /// are appended internally via [`Url::join`].
    pub fn new(base: Url, http: Client) -> Self {
        Self { base, http }
    }

    /// `GET /spawn-placement/preview?slot=N[&overflow=wrap]`
    ///
    /// Returns the placement for the configured runner-instance slot. Slot
    /// 0 is the primary runner; slots 1.. are configured `runner_instances`
    /// in saved order.
    pub async fn preview(
        &self,
        slot: usize,
        overflow: Overflow,
    ) -> Result<SpawnPlacementResponse, SpawnPlacementClientError> {
        let mut url = self.base.join("/spawn-placement/preview")?;
        {
            let mut q = url.query_pairs_mut();
            q.append_pair("slot", &slot.to_string());
            if let Some(o) = overflow.as_query() {
                q.append_pair("overflow", o);
            }
        }
        self.fetch(url).await
    }

    /// `GET /spawn-placement/temp?index=N[&overflow=wrap]`
    ///
    /// Returns the placement for the `index`-th temp-runner placement.
    /// `Overflow::Wrap` round-robins via `index % len`; `Overflow::Default`
    /// returns `404` when `index >= len`.
    pub async fn temp(
        &self,
        index: usize,
        overflow: Overflow,
    ) -> Result<SpawnPlacementResponse, SpawnPlacementClientError> {
        let mut url = self.base.join("/spawn-placement/temp")?;
        {
            let mut q = url.query_pairs_mut();
            q.append_pair("index", &index.to_string());
            if let Some(o) = overflow.as_query() {
                q.append_pair("overflow", o);
            }
        }
        self.fetch(url).await
    }

    /// Shared GET / parse path for both endpoints. Mirrors the
    /// envelope-vs-bare unwrap the supervisor used to do inline.
    async fn fetch(&self, url: Url) -> Result<SpawnPlacementResponse, SpawnPlacementClientError> {
        let resp = self.http.get(url).send().await?;
        let status = resp.status();
        if !status.is_success() {
            let body = resp.text().await.unwrap_or_default();
            return Err(SpawnPlacementClientError::Status { status, body });
        }

        // Same envelope-vs-bare unwrap the supervisor used to do inline:
        // the runner wraps responses in `{success, data, error?}`. We accept
        // either the envelope or a bare payload so this client works against
        // any runner version (and against test fixtures that emit either
        // shape).
        let value: serde_json::Value = resp.json().await?;

        // Envelope shape: `{success, data, error}`. We treat the presence
        // of `data` (or `success`/`error`) as the discriminator. A response
        // that happens to have a top-level `data` key on a bare placement
        // payload would be ambiguous, but `SpawnPlacementResponse` has no
        // `data` field so this is unambiguous in practice.
        if value.get("data").is_some()
            || value.get("success").is_some()
            || value.get("error").is_some()
        {
            // Wrapped — pull `data` out, or surface the envelope error.
            let success = value
                .get("success")
                .and_then(|v| v.as_bool())
                .unwrap_or(false);
            if let Some(data) = value.get("data") {
                if success {
                    return serde_json::from_value(data.clone())
                        .map_err(|e| SpawnPlacementClientError::Parse(e.to_string()));
                }
            }
            // Either `success: false` or `data` missing — both flow through
            // EnvelopeError so the supervisor can log the runner's reason.
            let error = value
                .get("error")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string());
            return Err(SpawnPlacementClientError::EnvelopeError { error });
        }

        // Bare payload.
        serde_json::from_value(value).map_err(|e| SpawnPlacementClientError::Parse(e.to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use qontinui_types::wire::placement::SpawnPlacementResponse;
    use wiremock::matchers::{method, path, query_param};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    fn sample_response() -> SpawnPlacementResponse {
        SpawnPlacementResponse::new(
            100,
            200,
            1280,
            720,
            "primary".to_string(),
            0,
            "temp[0]".to_string(),
            "temp".to_string(),
        )
        .with_decorations(Some(false))
    }

    fn client_for(server: &MockServer) -> SpawnPlacementClient {
        let base = Url::parse(&server.uri()).unwrap();
        SpawnPlacementClient::new(base, Client::new())
    }

    #[tokio::test]
    async fn temp_parses_wrapped_envelope() {
        let server = MockServer::start().await;
        let body = serde_json::json!({
            "success": true,
            "data": sample_response(),
            "error": serde_json::Value::Null,
        });
        Mock::given(method("GET"))
            .and(path("/spawn-placement/temp"))
            .and(query_param("index", "0"))
            .and(query_param("overflow", "wrap"))
            .respond_with(ResponseTemplate::new(200).set_body_json(body))
            .mount(&server)
            .await;

        let placement = client_for(&server)
            .temp(0, Overflow::Wrap)
            .await
            .expect("temp() should succeed for wrapped envelope");
        assert_eq!(placement.global_x, 100);
        assert_eq!(placement.global_y, 200);
        assert_eq!(placement.width, 1280);
        assert_eq!(placement.height, 720);
        assert_eq!(placement.slot_label, "temp[0]");
        assert_eq!(placement.source, "temp");
        assert_eq!(placement.decorations, Some(false));
    }

    #[tokio::test]
    async fn temp_parses_bare_payload() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/spawn-placement/temp"))
            .respond_with(ResponseTemplate::new(200).set_body_json(sample_response()))
            .mount(&server)
            .await;

        let placement = client_for(&server)
            .temp(0, Overflow::Wrap)
            .await
            .expect("temp() should succeed for bare payload");
        assert_eq!(placement.global_x, 100);
        assert_eq!(placement.slot_label, "temp[0]");
    }

    #[tokio::test]
    async fn preview_parses_wrapped_envelope() {
        let server = MockServer::start().await;
        let mut placement = sample_response();
        placement.slot_label = "primary".into();
        placement.source = "configured".into();
        let body = serde_json::json!({
            "success": true,
            "data": placement.clone(),
        });
        Mock::given(method("GET"))
            .and(path("/spawn-placement/preview"))
            .and(query_param("slot", "0"))
            .respond_with(ResponseTemplate::new(200).set_body_json(body))
            .mount(&server)
            .await;

        let got = client_for(&server)
            .preview(0, Overflow::Default)
            .await
            .expect("preview() should succeed");
        assert_eq!(got.slot_label, "primary");
        assert_eq!(got.source, "configured");
    }

    #[tokio::test]
    async fn wrapped_and_bare_parse_identically() {
        let server_w = MockServer::start().await;
        let server_b = MockServer::start().await;
        let placement = sample_response();
        Mock::given(method("GET"))
            .and(path("/spawn-placement/temp"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "success": true,
                "data": placement.clone(),
            })))
            .mount(&server_w)
            .await;
        Mock::given(method("GET"))
            .and(path("/spawn-placement/temp"))
            .respond_with(ResponseTemplate::new(200).set_body_json(placement.clone()))
            .mount(&server_b)
            .await;

        let wrapped = client_for(&server_w)
            .temp(0, Overflow::Wrap)
            .await
            .expect("wrapped");
        let bare = client_for(&server_b)
            .temp(0, Overflow::Wrap)
            .await
            .expect("bare");

        assert_eq!(wrapped.global_x, bare.global_x);
        assert_eq!(wrapped.global_y, bare.global_y);
        assert_eq!(wrapped.width, bare.width);
        assert_eq!(wrapped.height, bare.height);
        assert_eq!(wrapped.monitor_label, bare.monitor_label);
        assert_eq!(wrapped.slot_index, bare.slot_index);
        assert_eq!(wrapped.slot_label, bare.slot_label);
        assert_eq!(wrapped.source, bare.source);
        assert_eq!(wrapped.decorations, bare.decorations);
    }

    #[tokio::test]
    async fn non_2xx_returns_status_error_with_body() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/spawn-placement/temp"))
            .respond_with(
                ResponseTemplate::new(404)
                    .set_body_json(serde_json::json!({"error": "no temp placements configured"})),
            )
            .mount(&server)
            .await;

        let err = client_for(&server)
            .temp(0, Overflow::Wrap)
            .await
            .expect_err("temp() should fail on 404");
        match err {
            SpawnPlacementClientError::Status { status, body } => {
                assert_eq!(status.as_u16(), 404);
                assert!(
                    body.contains("no temp placements configured"),
                    "body: {body}"
                );
            }
            other => panic!("expected Status, got {other:?}"),
        }
    }

    #[tokio::test]
    async fn envelope_with_success_false_yields_envelope_error() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/spawn-placement/temp"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "success": false,
                "error": "internal monitor enumeration failed",
            })))
            .mount(&server)
            .await;

        let err = client_for(&server)
            .temp(0, Overflow::Wrap)
            .await
            .expect_err("temp() should surface envelope error");
        match err {
            SpawnPlacementClientError::EnvelopeError { error } => {
                assert_eq!(
                    error.as_deref(),
                    Some("internal monitor enumeration failed")
                );
            }
            other => panic!("expected EnvelopeError, got {other:?}"),
        }
    }

    #[tokio::test]
    async fn overflow_default_omits_query_param() {
        let server = MockServer::start().await;
        // Match only the no-overflow case. Wiremock fails the test if the
        // request doesn't match, so this implicitly asserts that
        // Overflow::Default does NOT add `overflow=...`.
        Mock::given(method("GET"))
            .and(path("/spawn-placement/preview"))
            .and(query_param("slot", "1"))
            .respond_with(ResponseTemplate::new(200).set_body_json(sample_response()))
            .mount(&server)
            .await;

        let _ = client_for(&server)
            .preview(1, Overflow::Default)
            .await
            .expect("preview() should match no-overflow request");
    }
}
