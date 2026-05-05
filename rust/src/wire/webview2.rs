//! WebView2 user-data folder layout — single source of truth for both
//! the supervisor (forwards via `WEBVIEW2_USER_DATA_FOLDER` env var on the
//! spawn command) and the runner (falls back to this when launched
//! standalone without a supervisor setting the env var).
//!
//! Windows-only: every other platform returns `None`. WebView2 is a Windows
//! component and the runner only consults this folder on Windows; on other
//! platforms the runner uses the Tauri default and the supervisor never sets
//! the env var.
//!
//! ## Folder layout
//!
//! Under `%LOCALAPPDATA%\com.qontinui.runner\`:
//!
//! | Runner kind | Folder name |
//! |-------------|-------------|
//! | `Primary`   | `EBWebView` (Tauri default — preserves existing auth, terminal layouts, and other primary-runner local state) |
//! | everything else | `EBWebView-<sanitized-runner-id>` (per-runner profile so localStorage, IndexedDB, cookies are isolated) |
//!
//! ## Why we take both `kind` and `runner_id`
//!
//! The non-primary path historically used the supervisor-assigned **id** for
//! the folder suffix:
//!
//! * Temp runner id `test-abc123`     → `EBWebView-test-abc123`
//! * Named runner id `named-9880-uuid` → `EBWebView-named-9880-uuid`
//!
//! Switching the suffix to `RunnerKind::Named { name }`'s display name
//! would orphan every already-deployed named-runner profile on disk, so we
//! preserve the historical id-based naming. `kind` is consulted only to
//! pick the special-cased `Primary` branch; `runner_id` carries the literal
//! id string for everyone else.
//!
//! ## Sanitization
//!
//! Filesystem-unsafe characters in the id collapse to `_`. In practice the
//! supervisor only ever produces ids matching `[a-zA-Z0-9_-]+`, so this is
//! defensive — but keeping the supervisor's exact pre-`RunnerKind`
//! sanitization rule means existing on-disk profiles stay reachable.

use crate::wire::RunnerKind;
use std::path::PathBuf;

/// WebView2 user-data folder for the given runner.
///
/// * `Primary`              → `%LOCALAPPDATA%\com.qontinui.runner\EBWebView`
/// * everything else        → `%LOCALAPPDATA%\com.qontinui.runner\EBWebView-<sanitized runner_id>`
///
/// Returns `None` on non-Windows or when `LOCALAPPDATA` is unset / empty.
///
/// `runner_id` is ignored for `Primary`. For all other variants the literal
/// id string (sanitized) becomes the folder suffix; see module-level docs
/// for why the id is preserved instead of switching to
/// `RunnerKind::Named.name`.
#[cfg(target_os = "windows")]
pub fn webview2_data_dir(kind: &RunnerKind, runner_id: &str) -> Option<PathBuf> {
    let local_app_data = std::env::var("LOCALAPPDATA")
        .ok()
        .filter(|s| !s.is_empty())?;
    let base = PathBuf::from(local_app_data).join("com.qontinui.runner");
    match kind {
        RunnerKind::Primary => Some(base.join("EBWebView")),
        _ => Some(base.join(format!("EBWebView-{}", sanitize(runner_id)))),
    }
}

#[cfg(not(target_os = "windows"))]
pub fn webview2_data_dir(_kind: &RunnerKind, _runner_id: &str) -> Option<PathBuf> {
    None
}

/// Sanitize an id for use as a filesystem folder suffix: keep ASCII
/// alphanumerics, `-`, and `_`; everything else collapses to `_`.
///
/// Mirrors the pre-`RunnerKind` supervisor behavior at
/// `qontinui-supervisor/src/process/windows.rs::webview2_user_data_folder`
/// so existing on-disk profiles continue to be addressable.
fn sanitize(s: &str) -> String {
    s.chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() || c == '-' || c == '_' {
                c
            } else {
                '_'
            }
        })
        .collect()
}

#[cfg(all(test, target_os = "windows"))]
mod tests {
    use super::*;

    /// `LOCALAPPDATA` is always set on a real Windows desktop session, but
    /// CI / test harnesses may run with a sanitized env. Use a guard helper
    /// so the asserts only fire when the env actually has a value.
    fn with_local_app_data<R>(f: impl FnOnce(PathBuf) -> R) -> Option<R> {
        let lad = std::env::var("LOCALAPPDATA")
            .ok()
            .filter(|s| !s.is_empty())?;
        Some(f(PathBuf::from(lad).join("com.qontinui.runner")))
    }

    #[test]
    fn primary_uses_default_folder() {
        let _ = with_local_app_data(|base| {
            // `runner_id` is ignored for Primary — pass anything.
            let dir = webview2_data_dir(&RunnerKind::Primary, "primary").unwrap();
            assert_eq!(dir, base.join("EBWebView"));
            // Even an obviously different id must not affect the Primary path.
            let dir = webview2_data_dir(&RunnerKind::Primary, "test-foo").unwrap();
            assert_eq!(dir, base.join("EBWebView"));
        });
    }

    #[test]
    fn temp_uses_id_suffix() {
        let _ = with_local_app_data(|base| {
            let kind = RunnerKind::Temp {
                id: "test-abc123".into(),
            };
            let dir = webview2_data_dir(&kind, "test-abc123").unwrap();
            assert_eq!(dir, base.join("EBWebView-test-abc123"));
        });
    }

    #[test]
    fn named_uses_id_suffix_not_name() {
        let _ = with_local_app_data(|base| {
            // Real id shape from supervisor: named-{port}-{uuid}.
            let kind = RunnerKind::Named {
                name: "Friendly Name".into(),
            };
            let dir = webview2_data_dir(&kind, "named-9880-deadbeef").unwrap();
            // Folder is keyed by id, NOT by display name — preserves existing
            // profiles when display names get renamed.
            assert_eq!(dir, base.join("EBWebView-named-9880-deadbeef"));
            // And human-readable name characters that hit sanitize() do NOT
            // appear in the path because we use id, not name.
            assert!(!dir.to_string_lossy().contains("Friendly"));
        });
    }

    #[test]
    fn external_uses_id_suffix() {
        let _ = with_local_app_data(|base| {
            // External runners aren't supervisor-spawned but the supervisor
            // can still observe them; if anything ever asks for their folder
            // we should produce a deterministic answer (rather than None).
            let dir = webview2_data_dir(&RunnerKind::External, "user-runner-1").unwrap();
            assert_eq!(dir, base.join("EBWebView-user-runner-1"));
        });
    }

    #[test]
    fn sanitizes_unsafe_characters() {
        let _ = with_local_app_data(|base| {
            let kind = RunnerKind::Temp {
                id: "test-a/b\\c d.e".into(),
            };
            let dir = webview2_data_dir(&kind, "test-a/b\\c d.e").unwrap();
            // All four unsafe chars (slash, backslash, space, dot) collapse
            // to `_`, consistent with pre-RunnerKind supervisor behavior.
            assert_eq!(dir, base.join("EBWebView-test-a_b_c_d_e"));
        });
    }

    #[test]
    fn preserves_alphanumeric_dash_underscore() {
        assert_eq!(sanitize("abc-123_DEF"), "abc-123_DEF");
    }
}

#[cfg(all(test, not(target_os = "windows")))]
mod tests {
    use super::*;

    #[test]
    fn returns_none_on_non_windows() {
        assert!(webview2_data_dir(&RunnerKind::Primary, "primary").is_none());
        assert!(webview2_data_dir(
            &RunnerKind::Temp {
                id: "test-x".into()
            },
            "test-x"
        )
        .is_none());
    }
}
