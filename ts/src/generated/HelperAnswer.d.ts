/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with `just generate-types` or
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */

import type { HelperVerdict } from "./HelperVerdict";

/**
 * One helper's answer to a [`HelperTask`].
 *
 * The broker collects up to [`HelperTask::required_votes`] of these per task.
 * On a 👎 [`HelperVerdict::Reject`], [`reasons`](HelperAnswer::reasons) carries
 * the preset reason codes the helper selected from
 * [`HelperAnswerSchema::preset_reasons`].
 */
export interface HelperAnswer {
  /**
   * ISO 8601 timestamp (UTC) when the answer was submitted.
   */
  createdAt: string;
  /**
   * Optional free-text note, when [`HelperAnswerSchema::allow_free_text`] is
   * set.
   */
  freeText?: string | null;
  /**
   * The helper (user) who submitted the answer.
   */
  helperUserId: string;
  /**
   * Answer ID (UUID v4 string).
   */
  id: string;
  /**
   * Preset reason codes selected on a reject — drawn from
   * [`HelperAnswerSchema::preset_reasons`]. Empty for non-reject verdicts.
   */
  reasons: string[];
  /**
   * The [`HelperTask`] this answers.
   */
  taskId: string;
  /**
   * The verdict the helper gave.
   */
  verdict: HelperVerdict;
  [k: string]: unknown;
}
