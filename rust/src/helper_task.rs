//! Helper-task queue models — human-judgment micro-task brokering.
//!
//! Rust is the source of truth; TypeScript and Python bindings regenerate from
//! the JSON Schemas emitted here. See plan
//! `2026-06-29-helper-task-queue-non-programmer-dev.md`.
//!
//! A *helper task* brokers a small unit of human judgment that an automated
//! runner cannot make on its own: "does this screen look right?", "which of
//! these two renders is better?", "walk through this flow and tell me what
//! breaks". A runner emits a [`HelperTask`]; a **non-technical helper** (no code
//! access, no dev tooling) reviews it through a stripped-down UI and submits a
//! [`HelperAnswer`]. The broker collects up to [`HelperTask::required_votes`]
//! answers, then folds the verdict back into the originating
//! [`crate::findings`] / spec-check flow via [`HelperTaskSource`].
//!
//! ## Task kinds
//!
//! Five kinds are modeled by [`HelperTaskKind`]. **Phase 1 ships only
//! [`HelperTaskKind::SpotCheck`] as fully usable**; the remaining four
//! (`Compare`, `WalkThrough`, `Describe`, `Sort`) are defined here so the data
//! model is complete but are wired up in Phase 2/3. A SpotCheck shows the helper
//! one screenshot and asks for an approve / reject / not-sure verdict (with
//! optional preset reason codes on a reject).
//!
//! ## Wire-format notes
//!
//! - IDs are plain `String`s (UUID v4 on the wire; see crate-level docs).
//! - Timestamps are ISO 8601 `String`s — no `chrono` in the DTO layer.
//! - Vote counts are `u32`; match rates are `f64`.
//! - Enum string values are lowercase `snake_case`.
//! - Optional fields use `#[serde(default, skip_serializing_if = "Option::is_none")]`
//!   so absence and `null` round-trip faithfully and a SpotCheck payload need
//!   only populate the one field it uses.

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

// ============================================================================
// Enums
// ============================================================================

/// Kind of human-judgment task a helper is asked to perform.
///
/// Phase 1 ships [`HelperTaskKind::SpotCheck`] only; the rest are Phase 2/3.
/// Each kind dictates which [`HelperTaskPayload`] fields are populated and which
/// [`HelperVerdict`]s are offered in the task's [`HelperAnswerSchema`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum HelperTaskKind {
    /// "Does this screen look right?" — one screenshot, an approve/reject/
    /// not-sure verdict plus optional preset reason codes on a reject. The only
    /// kind fully usable in Phase 1.
    SpotCheck,
    /// "Which of these two looks better?" — two screenshots, an A/B/same
    /// verdict. (Phase 2.)
    Compare,
    /// "Follow these steps and tell me what breaks." — a guided list of steps
    /// the helper walks through live. (Phase 2/3.)
    WalkThrough,
    /// "Describe what you see on this live screen." — free-text description of a
    /// live target. (Phase 3.)
    Describe,
    /// "Group these feature cards." — a card-sort exercise over feature cards.
    /// (Phase 3.)
    Sort,
}

/// Lifecycle status of a helper task.
///
/// Lifecycle: `OPEN` → (`ANSWERED` | `EXPIRED` | `CANCELLED`). A task becomes
/// `ANSWERED` once [`HelperTask::required_votes`] answers are collected.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum HelperTaskStatus {
    /// Awaiting helper answers — still accepting votes.
    Open,
    /// Required votes collected; the verdict is final.
    Answered,
    /// Passed its `expires_at` deadline before enough votes arrived.
    Expired,
    /// Withdrawn by the emitting runner before completion.
    Cancelled,
}

/// A verdict a helper may submit for a task.
///
/// Verdict-to-kind mapping:
/// - [`HelperTaskKind::SpotCheck`] → [`HelperVerdict::Approve`],
///   [`HelperVerdict::Reject`], [`HelperVerdict::NotSure`].
/// - [`HelperTaskKind::Compare`] → [`HelperVerdict::ChoiceA`],
///   [`HelperVerdict::ChoiceB`], [`HelperVerdict::ChoiceSame`].
///
/// `NotSure` (🤔) is a first-class outcome — never an error — so a helper is
/// never forced into a false approve/reject.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum HelperVerdict {
    /// 👍 The screen looks right (SpotCheck).
    Approve,
    /// 👎 The screen looks wrong (SpotCheck) — usually paired with preset
    /// reason codes in [`HelperAnswer::reasons`].
    Reject,
    /// 🤔 The helper cannot tell (SpotCheck) — a first-class, expected outcome.
    NotSure,
    /// Option A is better (Compare).
    ChoiceA,
    /// Option B is better (Compare).
    ChoiceB,
    /// Both options look equivalent (Compare).
    ChoiceSame,
}

// ============================================================================
// Supporting structs
// ============================================================================

/// Kind-specific content shown to the helper.
///
/// Every field is optional; a given [`HelperTaskKind`] populates only the
/// fields it needs. A [`HelperTaskKind::SpotCheck`] task populates only
/// [`screenshot_url`](HelperTaskPayload::screenshot_url).
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct HelperTaskPayload {
    /// SpotCheck: a coord-served / presigned URL of the screenshot to review.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub screenshot_url: Option<String>,
    /// Compare: the two presigned screenshot URLs to put side by side (A, B).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub compare_urls: Option<Vec<String>>,
    /// WalkThrough: the ordered guided steps the helper follows live.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub steps: Option<Vec<String>>,
    /// Sort: the feature cards the helper groups in the card-sort exercise.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cards: Option<Vec<String>>,
    /// Describe: a reference to the live screen / target the helper describes.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub live_target: Option<String>,
}

/// The answers a helper is allowed to give, plus the preset reason codes the UI
/// offers them.
///
/// Defines the choice set for a task: which [`HelperVerdict`]s are offered, the
/// preset reason codes shown when the helper rejects, and whether free text and
/// the 🤔 "not sure" escape hatch are available.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct HelperAnswerSchema {
    /// The verdicts offered for this task (e.g. approve/reject/not_sure for a
    /// SpotCheck).
    pub verdicts: Vec<HelperVerdict>,
    /// Preset reason codes the UI surfaces as tap-to-select chips on a reject —
    /// e.g. `"text_cut_off"`, `"overlapping"`, `"wrong_color"`,
    /// `"button_missing"`. Selected codes land in [`HelperAnswer::reasons`].
    pub preset_reasons: Vec<String>,
    /// Whether the helper may add a free-text note alongside the verdict.
    pub allow_free_text: bool,
    /// Whether the 🤔 [`HelperVerdict::NotSure`] option is offered. First-class
    /// by design; defaults to `true`.
    #[serde(default = "default_allow_not_sure")]
    pub allow_not_sure: bool,
}

fn default_allow_not_sure() -> bool {
    true
}

/// Round-trip provenance linking a task back to what produced it.
///
/// Carries the originating [`crate::findings`] / page / spec-check context so a
/// collected verdict can be folded back into the source flow. Spec-check exposes
/// `match_rate` / classification (not a "confidence" scalar), so the rate field
/// is named [`match_rate`](HelperTaskSource::match_rate) accordingly.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct HelperTaskSource {
    /// The finding this task was raised from, if any.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub finding_id: Option<String>,
    /// The page / spec the screenshot was captured from, if any.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub page_id: Option<String>,
    /// The spec-check match rate that triggered the task, if any — the
    /// `[0.0, 1.0]` structural match score, **not** a confidence scalar.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub match_rate: Option<f64>,
}

// ============================================================================
// Task + Answer
// ============================================================================

/// A human-judgment micro-task emitted by a runner for a helper to answer.
///
/// The broker holds the task `Open` until [`required_votes`](HelperTask::required_votes)
/// [`HelperAnswer`]s are collected (or it `expires_at`), then resolves the
/// verdict and routes it back through [`source`](HelperTask::source).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct HelperTask {
    /// Task ID (UUID v4 string).
    pub id: String,
    /// Tenant that owns the task (coord tenant id).
    pub tenant_id: String,
    /// Application the task pertains to.
    pub app_id: String,
    /// Which kind of judgment is being asked for.
    pub kind: HelperTaskKind,
    /// Human-readable question shown to the helper (e.g. "Does this login
    /// screen look right?").
    pub prompt: String,
    /// Kind-specific content (screenshot URL, compare URLs, steps, …).
    pub payload: HelperTaskPayload,
    /// The set of allowed answers and preset reason codes offered to the helper.
    pub answer_schema: HelperAnswerSchema,
    /// Number of helper answers required before the task is `Answered`.
    pub required_votes: u32,
    /// Current lifecycle status.
    pub status: HelperTaskStatus,
    /// Provenance linking the task back to its originating finding / page /
    /// spec-check.
    pub source: HelperTaskSource,
    /// ISO 8601 timestamp (UTC) when the task was created.
    pub created_at: String,
    /// ISO 8601 timestamp (UTC) after which the task can no longer be answered.
    /// Absent means the task does not expire.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub expires_at: Option<String>,
}

/// One helper's answer to a [`HelperTask`].
///
/// The broker collects up to [`HelperTask::required_votes`] of these per task.
/// On a 👎 [`HelperVerdict::Reject`], [`reasons`](HelperAnswer::reasons) carries
/// the preset reason codes the helper selected from
/// [`HelperAnswerSchema::preset_reasons`].
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct HelperAnswer {
    /// Answer ID (UUID v4 string).
    pub id: String,
    /// The [`HelperTask`] this answers.
    pub task_id: String,
    /// The helper (user) who submitted the answer.
    pub helper_user_id: String,
    /// The verdict the helper gave.
    pub verdict: HelperVerdict,
    /// Preset reason codes selected on a reject — drawn from
    /// [`HelperAnswerSchema::preset_reasons`]. Empty for non-reject verdicts.
    #[serde(default)]
    pub reasons: Vec<String>,
    /// Optional free-text note, when [`HelperAnswerSchema::allow_free_text`] is
    /// set.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub free_text: Option<String>,
    /// ISO 8601 timestamp (UTC) when the answer was submitted.
    pub created_at: String,
}
