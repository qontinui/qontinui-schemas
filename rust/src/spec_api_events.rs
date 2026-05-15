//! Spec API broadcast event taxonomy.
//!
//! The runner's `spec_api/events.rs` broadcasts these on a single
//! `tokio::sync::broadcast` channel; `GET /spec/subscribe` forwards them as
//! SSE events with the variant's kebab-case discriminator as the event name.
//!
//! Variants:
//! - `SpecChanged` — emitted by `POST /spec/author` after a successful IR +
//!   projection write. The original (pre-Plan 06) event; kept on the same
//!   channel so existing `event: spec-changed` SSE subscribers keep working.
//! - `SpecCheckInvoked` / `SpecCheckCompleted` — Plan 06 B-lifecycle events,
//!   emitted by `spec_api/spec_check.rs` HTTP handlers on entry / after the
//!   batch evaluator returns.
//! - `SpecCheckPolicyViolation` — emitted by the Plan 03 workflow-step
//!   handler (`step_executor/spec_check.rs`) per failed `ConjunctEvaluation`.
//!   *Emit site lands when Plan 03 ships; the variant is reserved here so the
//!   wire is ready.*
//! - `FlywheelProposalPromoted` / `FlywheelProposalDemoted` — emitted by the
//!   Plan 05 sweep handler (`POST /spec/proposals/sweep-pending`) after
//!   `storage::promote_pending` / `storage::demote_pending` succeed. *Emit
//!   sites land when Plan 05 Step 9 ships; the variants are reserved here.*
//!
//! All variants carry `snapshot_id` (where applicable) as the cross-stream
//! join key against tracing spans and
//! `workflow_verification_phase_results.result_json->>'snapshot_id'`.

use serde::{Deserialize, Serialize};

/// Existing `POST /spec/author` change-broadcast payload. Pre-dates the
/// Plan 06 `SpecApiEvent` sum-type but keeps the same shape so external
/// SSE subscribers (the runner's `use-discovered-specs.ts`) keep working.
#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct SpecChanged {
    pub page_id: String,
    /// "ir-and-projection" today; future kinds may include
    /// "projection-only" if a regen runs without an IR write.
    pub kind: String,
    /// Epoch milliseconds the event was emitted at.
    pub at_ms: u64,
}

/// Tagged sum-type for every event broadcast on the Spec API channel.
///
/// Serialization uses `serde(tag = "type", rename_all = "kebab-case")` so
/// the wire discriminator is exactly the SSE event name (e.g.
/// `"spec-changed"`, `"spec-check-invoked"`).
#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(tag = "type", rename_all = "kebab-case")]
pub enum SpecApiEvent {
    /// Existing — emitted by `POST /spec/author` after a successful IR +
    /// projection write.
    SpecChanged(SpecChanged),

    /// B — entry into `evaluate` / `evaluate_batch` / policy. Emitted by
    /// `spec_api/spec_check.rs` HTTP handlers after `wrap_snapshot`
    /// returns and before the evaluator runs.
    SpecCheckInvoked {
        snapshot_id: String,
        page_ids: Vec<String>,
        /// "http" | "mcp" | "workflow-step"
        invoked_via: String,
        at_ms: u64,
    },

    /// B — completion roll-up. Payload joins via `snapshot_id`; no fat
    /// result body in the broadcast.
    SpecCheckCompleted {
        snapshot_id: String,
        page_count: usize,
        overall_match_rate: f32,
        perfect_match_count: usize,
        partial_match_count: usize,
        no_match_count: usize,
        eval_error_count: usize,
        total_duration_ms: u64,
        at_ms: u64,
    },

    /// B — workflow-step policy conjunct failed. One emit per failed
    /// `ConjunctEvaluation`.
    SpecCheckPolicyViolation {
        snapshot_id: String,
        page_id: String,
        conjunct_name: String,
        /// serde-tagged variant name of `ConjunctRule`
        rule_kind: String,
        /// Small scalar/struct — NOT the full result.
        observed: serde_json::Value,
        at_ms: u64,
    },

    /// Flywheel — proposal moved `_pending/<id>/` → `pages/<id>/`.
    FlywheelProposalPromoted {
        proposal_id: String,
        page_id: String,
        /// Expected 2 per design §6.4.
        consecutive_greens: u32,
        /// The snapshot that produced the second green.
        snapshot_id: String,
        at_ms: u64,
    },

    /// Flywheel — proposal dropped back to the queue after a red B run.
    FlywheelProposalDemoted {
        proposal_id: String,
        page_id: String,
        failing_assertion_id: String,
        failing_state_id: String,
        snapshot_id: String,
        at_ms: u64,
    },
}
