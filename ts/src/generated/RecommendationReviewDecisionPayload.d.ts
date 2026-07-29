/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with `just generate-types` or
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */

/**
 * Payload shape for the `review-approved` and `review-rejected` Tauri events
 * emitted by `commands::productivity::approve_recommendation` /
 * `reject_recommendation` after a user resolves a medium-confidence
 * recommendation card. Field names are explicit camelCase to match the
 * `serde_json::json!()` literal previously used at the emit site.
 *
 * Single struct shared by both channels: only `user_decision` differs
 * (`"approved"` vs `"rejected"`), so a tagged enum would inflate the wire
 * format without payoff.
 */
export interface RecommendationReviewDecisionPayload {
  reviewId: string;
  taskId: string;
  userDecision: string;
  [k: string]: unknown;
}
