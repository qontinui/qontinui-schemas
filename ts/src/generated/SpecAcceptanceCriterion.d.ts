/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */

import type { CriterionVerification } from "./CriterionVerification";

/**
 * One acceptance criterion of a requirement or story.
 */
export interface SpecAcceptanceCriterion {
  /**
   * Stable within its artifact, e.g. `AC-1`.
   */
  id: string;
  /**
   * The observable condition, ideally an EARS sentence.
   */
  text: string;
  /**
   * How the criterion is checked. Required: a spec states it rather than
   * inheriting a default.
   */
  verification: CriterionVerification;
  [k: string]: unknown;
}
