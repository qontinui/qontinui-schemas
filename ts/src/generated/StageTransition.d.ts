/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with `just generate-types` or
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */

/**
 * A record of a stage transition during task execution.
 *
 * Used for building the stage-based timeline on the recap page.
 */
export interface StageTransition {
  /**
   * Previous stage.
   */
  from: string;
  /**
   * Iteration number at the time of the transition.
   */
  iteration: number;
  /**
   * When the transition occurred (ISO 8601).
   */
  timestamp: string;
  /**
   * New stage.
   */
  to: string;
  [k: string]: unknown;
}
