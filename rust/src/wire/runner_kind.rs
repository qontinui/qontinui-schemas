//! Runner kind classifier — single source of truth for "what kind of runner
//! is this?".
//!
//! Used by both qontinui-runner (deriving from env for self-identification)
//! and qontinui-supervisor (storing on `ManagedRunner` config and classifying
//! by the runner id prefix).
//!
//! ## Prefix scheme
//!
//! The supervisor assigns runner IDs at spawn time:
//!
//! | Pattern               | Source                                      | Variant   |
//! |-----------------------|---------------------------------------------|-----------|
//! | `"primary"`           | `RunnerConfig::primary()`                   | `Primary` |
//! | `"test-{uuid}"`       | `routes::runners::spawn_test`               | `Temp`    |
//! | `"named-{port}-{uuid}"` | `routes::runners::spawn_named`           | `Named`   |
//! | anything else         | user-provided, supervisor only observes     | `External`|
//!
//! The user-friendly display name of a `Named` runner lives on
//! `RunnerConfig.name`, NOT in the id — the id always carries
//! `named-{port}-{uuid}`.
//!
//! ## Runner-side asymmetry
//!
//! From the runner's own perspective, the env var `QONTINUI_INSTANCE_NAME`
//! gets set for both `Temp` and `Named` runners, so the runner alone cannot
//! distinguish them. `crate::instance::runner_kind()` therefore returns
//! `Named { name }` for any secondary; the supervisor uses
//! `RunnerKind::from_id` to disambiguate.

use serde::{Deserialize, Serialize};

/// Classification of a runner.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
#[non_exhaustive]
pub enum RunnerKind {
    /// The user's primary runner. Supervisor observes only.
    Primary,
    /// A persistent named runner spawned via `POST /runners/spawn-named`.
    /// `name` is the user-supplied display name (mirrored from
    /// `RunnerConfig.name`); the id-derived path uses the literal id string.
    Named { name: String },
    /// An ephemeral test runner spawned via `POST /runners/spawn-test`.
    /// `id` is the `test-{uuid}` id assigned by the supervisor.
    Temp { id: String },
    /// A user-managed runner the supervisor only observes.
    External,
}

impl RunnerKind {
    /// Classify a runner from its supervisor-assigned id.
    ///
    /// * `"primary"` → [`RunnerKind::Primary`]
    /// * `"test-..."` → [`RunnerKind::Temp`] with the full id retained
    /// * `"named-..."` → [`RunnerKind::Named`] with `name` set to the id
    ///   (callers with access to `RunnerConfig.name` should override)
    /// * anything else → [`RunnerKind::External`]
    pub fn from_id(id: &str) -> Self {
        if id == "primary" {
            Self::Primary
        } else if id.starts_with("test-") {
            Self::Temp { id: id.to_string() }
        } else if id.starts_with("named-") {
            Self::Named {
                name: id.to_string(),
            }
        } else {
            Self::External
        }
    }

    pub fn is_primary(&self) -> bool {
        matches!(self, Self::Primary)
    }

    pub fn is_temp(&self) -> bool {
        matches!(self, Self::Temp { .. })
    }

    pub fn is_named(&self) -> bool {
        matches!(self, Self::Named { .. })
    }

    pub fn is_external(&self) -> bool {
        matches!(self, Self::External)
    }

    /// True for everything except [`RunnerKind::Primary`].
    pub fn is_secondary(&self) -> bool {
        !self.is_primary()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn from_id_primary() {
        assert_eq!(RunnerKind::from_id("primary"), RunnerKind::Primary);
    }

    #[test]
    fn from_id_temp() {
        assert_eq!(
            RunnerKind::from_id("test-abc"),
            RunnerKind::Temp {
                id: "test-abc".to_string()
            }
        );
    }

    #[test]
    fn from_id_named() {
        // Real shape from supervisor: `named-{port}-{uuid}`. The brief's
        // `named-foo` was a simplification — the id always carries the full
        // prefix-port-uuid string.
        assert_eq!(
            RunnerKind::from_id("named-9880-deadbeef"),
            RunnerKind::Named {
                name: "named-9880-deadbeef".to_string()
            }
        );
    }

    #[test]
    fn from_id_external() {
        assert_eq!(RunnerKind::from_id("randomthing"), RunnerKind::External);
    }

    #[test]
    fn predicates_classify_correctly() {
        assert!(RunnerKind::Primary.is_primary());
        assert!(!RunnerKind::Primary.is_secondary());

        let temp = RunnerKind::Temp {
            id: "test-x".to_string(),
        };
        assert!(temp.is_temp());
        assert!(temp.is_secondary());

        let named = RunnerKind::Named {
            name: "foo".to_string(),
        };
        assert!(named.is_named());
        assert!(named.is_secondary());

        assert!(RunnerKind::External.is_external());
        assert!(RunnerKind::External.is_secondary());
    }
}
