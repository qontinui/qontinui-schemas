//! Multi-tenant app registry types.
//!
//! Each `App` registers a directory on disk (`repo_root`) as a Qontinui
//! application — the runner serves that app's specs from
//! `<repo_root>/specs/pages/`. The Spec API endpoints are nested under
//! `/apps/<app_id>/spec/*`.
//!
//! `app_id` is a slug-style string: lowercase ASCII letters, digits, and
//! hyphens, length 1–64, must start with `[a-z0-9]`.

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct App {
    pub app_id: String,
    pub repo_root: String,
    pub ui_bridge_url: String,
    pub display_name: String,
    pub created_at_ms: i64,
    pub last_seen_at_ms: i64,
    /// Whether this app requires authentication before spec checks.
    /// If true, workflows will auto-inject an auth setup step.
    #[serde(default)]
    pub auth_required: bool,
    /// Red threshold for spec match rates (0.0–1.0). Match rates below this are Red (fail).
    /// Must be less than `yellow_threshold`. Defaults to 0.5.
    #[serde(default = "default_red_threshold")]
    pub red_threshold: f64,
    /// Yellow threshold for spec match rates (0.0–1.0). Match rates >= this are Green (pass),
    /// below are Yellow (warn). Must be greater than `red_threshold`. Defaults to 0.8.
    #[serde(default = "default_yellow_threshold")]
    pub yellow_threshold: f64,
    /// Auto-fresh update strategy: "pull_only" (pull code, no restart) or
    /// "pull_build" (pull, run build_command, run start_command). Used by the
    /// runner's P3 auto-fresh engine to decide what actions to take when
    /// pulling updated source code. Defaults to "pull_only".
    #[serde(default = "default_update_strategy")]
    pub update_strategy: String,
    /// Build command to run after pulling updated source (P3 auto-fresh).
    /// Ignored if `update_strategy` is "pull_only". Optional; if None,
    /// the auto-fresh engine skips build and goes straight to start_command.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub build_command: Option<String>,
    /// Start command to run after a successful build (P3 auto-fresh).
    /// Restarts the deployed instance and updates app_deploy_state.
    /// Ignored if `update_strategy` is "pull_only". Optional.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub start_command: Option<String>,
}

fn default_red_threshold() -> f64 {
    0.5
}

fn default_yellow_threshold() -> f64 {
    0.8
}

fn default_update_strategy() -> String {
    "pull_only".to_string()
}

#[derive(Debug, Clone, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct RegisterAppRequest {
    pub app_id: String,
    pub repo_root: String,
    pub ui_bridge_url: String,
    pub display_name: String,
    /// Whether this app requires authentication before spec checks.
    #[serde(default)]
    pub auth_required: bool,
    /// Red threshold for spec match rates. Defaults to 0.5.
    #[serde(default = "default_red_threshold")]
    pub red_threshold: f64,
    /// Yellow threshold for spec match rates. Defaults to 0.8.
    #[serde(default = "default_yellow_threshold")]
    pub yellow_threshold: f64,
    /// Auto-fresh update strategy (P3 fleet-fresh engine). Defaults to "pull_only".
    #[serde(default = "default_update_strategy")]
    pub update_strategy: String,
    /// Build command for "pull_build" strategy. Optional.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub build_command: Option<String>,
    /// Start command to restart after build. Optional.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub start_command: Option<String>,
}

impl RegisterAppRequest {
    /// Construct a request from the four required identity fields; every
    /// config field takes its registration default (mirrors the `#[serde(default)]`
    /// values used on the wire). Call sites that need a non-default config
    /// mutate the returned value — fields are public.
    pub fn new(
        app_id: impl Into<String>,
        repo_root: impl Into<String>,
        ui_bridge_url: impl Into<String>,
        display_name: impl Into<String>,
    ) -> Self {
        Self {
            app_id: app_id.into(),
            repo_root: repo_root.into(),
            ui_bridge_url: ui_bridge_url.into(),
            display_name: display_name.into(),
            auth_required: false,
            red_threshold: default_red_threshold(),
            yellow_threshold: default_yellow_threshold(),
            update_strategy: default_update_strategy(),
            build_command: None,
            start_command: None,
        }
    }
}

#[derive(Debug, Clone, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct UpdateAppRequest {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ui_bridge_url: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub display_name: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    /// Whether this app requires authentication before spec checks.
    pub auth_required: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    /// Red threshold for spec match rates (0.0–1.0). Must be < yellow_threshold.
    pub red_threshold: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    /// Yellow threshold for spec match rates (0.0–1.0). Must be > red_threshold.
    pub yellow_threshold: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    /// Auto-fresh update strategy: "pull_only" or "pull_build".
    pub update_strategy: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    /// Build command for "pull_build" strategy.
    pub build_command: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    /// Start command to restart after build.
    pub start_command: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct AppListResponse {
    pub ok: bool,
    pub apps: Vec<App>,
}

/// Failure modes for app-registry operations. Mirrors the
/// `responses::SpecError` tagged-enum pattern in
/// `qontinui-runner/src-tauri/src/spec_api/responses.rs`.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, thiserror::Error)]
#[serde(
    tag = "reason",
    rename_all = "kebab-case",
    rename_all_fields = "camelCase"
)]
pub enum AppError {
    #[error("app id '{app_id}' is not registered")]
    NotRegistered { app_id: String },

    #[error("app id '{app_id}' is not a valid slug")]
    InvalidAppId { app_id: String },

    #[error("repo root '{repo_root}' does not exist or is not a directory")]
    InvalidRepoRoot { repo_root: String },

    #[error("app id '{app_id}' is already registered")]
    AlreadyRegistered { app_id: String },
}

/// Validate an `app_id` slug. Returns `Ok(())` for valid ids, or
/// `Err(AppError::InvalidAppId)` for anything else.
///
/// Rules: 1–64 chars, lowercase ASCII letters / digits / hyphens, first
/// char must be `[a-z0-9]` (no leading hyphen).
pub fn validate_app_id(s: &str) -> Result<(), AppError> {
    let len_ok = (1..=64).contains(&s.len());
    let first_ok = s
        .chars()
        .next()
        .is_some_and(|c| c.is_ascii_lowercase() || c.is_ascii_digit());
    let rest_ok = s
        .chars()
        .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-');
    if len_ok && first_ok && rest_ok {
        Ok(())
    } else {
        Err(AppError::InvalidAppId { app_id: s.into() })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn validate_accepts_canonical_slugs() {
        assert!(validate_app_id("qontinui-runner").is_ok());
        assert!(validate_app_id("qontinui-web").is_ok());
        assert!(validate_app_id("my-saas").is_ok());
        assert!(validate_app_id("customer-portal-v2").is_ok());
        assert!(validate_app_id("a").is_ok()); // 1 char
        assert!(validate_app_id("0").is_ok()); // leading digit OK
        assert!(validate_app_id(&"a".repeat(64)).is_ok()); // max length
    }

    #[test]
    fn validate_rejects_invalid_slugs() {
        assert!(matches!(
            validate_app_id(""),
            Err(AppError::InvalidAppId { .. })
        ));
        assert!(matches!(
            validate_app_id("-leading-hyphen"),
            Err(AppError::InvalidAppId { .. })
        ));
        assert!(matches!(
            validate_app_id("Has_Underscore"),
            Err(AppError::InvalidAppId { .. })
        ));
        assert!(matches!(
            validate_app_id("Upper"),
            Err(AppError::InvalidAppId { .. })
        ));
        assert!(matches!(
            validate_app_id(&"a".repeat(65)),
            Err(AppError::InvalidAppId { .. })
        ));
    }

    #[test]
    fn app_error_serializes_with_tagged_reason() {
        let err = AppError::NotRegistered {
            app_id: "ghost".into(),
        };
        let s = serde_json::to_string(&err).unwrap();
        assert!(s.contains(r#""reason":"not-registered""#));
        assert!(s.contains(r#""appId":"ghost""#));
    }

    #[test]
    fn app_defaults_have_correct_thresholds() {
        let req = RegisterAppRequest::new(
            "test-app",
            "/path",
            "http://localhost:3000",
            "Test",
        );
        assert_eq!(req.red_threshold, 0.5);
        assert_eq!(req.yellow_threshold, 0.8);
        assert!(!req.auth_required);
        assert_eq!(req.update_strategy, "pull_only");
        assert!(req.build_command.is_none());
        assert!(req.start_command.is_none());
    }

    #[test]
    fn app_register_request_deserializes_with_defaults() {
        let json = r#"{"appId":"test","repoRoot":"/path","uiBridgeUrl":"http://localhost","displayName":"Test"}"#;
        let req: RegisterAppRequest = serde_json::from_str(json).unwrap();
        assert_eq!(req.red_threshold, 0.5);
        assert_eq!(req.yellow_threshold, 0.8);
        assert!(!req.auth_required);
    }
}
