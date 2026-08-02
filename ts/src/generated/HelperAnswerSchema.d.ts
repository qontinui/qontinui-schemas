/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with `just generate-types` or
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */

import type { HelperVerdict } from "./HelperVerdict";

/**
 * The answers a helper is allowed to give, plus the preset reason codes the UI
 * offers them.
 *
 * Defines the choice set for a task: which [`HelperVerdict`]s are offered, the
 * preset reason codes shown when the helper rejects, and whether free text and
 * the 🤔 "not sure" escape hatch are available.
 */
export interface HelperAnswerSchema {
  /**
   * Whether the helper may add a free-text note alongside the verdict.
   */
  allowFreeText: boolean;
  /**
   * Whether the 🤔 [`HelperVerdict::NotSure`] option is offered. First-class
   * by design; defaults to `true`.
   */
  allowNotSure: boolean;
  /**
   * Preset reason codes the UI surfaces as tap-to-select chips on a reject —
   * e.g. `"text_cut_off"`, `"overlapping"`, `"wrong_color"`,
   * `"button_missing"`. Selected codes land in [`HelperAnswer::reasons`].
   */
  presetReasons: string[];
  /**
   * The verdicts offered for this task (e.g. approve/reject/not_sure for a
   * SpotCheck).
   */
  verdicts: HelperVerdict[];
  [k: string]: unknown;
}
