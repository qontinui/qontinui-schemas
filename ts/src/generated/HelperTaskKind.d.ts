/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with `just generate-types` or
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */

/**
 * Kind of human-judgment task a helper is asked to perform.
 *
 * Phase 1 ships [`HelperTaskKind::SpotCheck`] only; the rest are Phase 2/3.
 * Each kind dictates which [`HelperTaskPayload`] fields are populated and which
 * [`HelperVerdict`]s are offered in the task's [`HelperAnswerSchema`].
 */
export type HelperTaskKind =
  "spot_check" | "compare" | "walk_through" | "describe" | "sort";
