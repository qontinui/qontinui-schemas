/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with `just generate-types` or
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */

/**
 * Configuration for condition-only schedules.
 */
export interface ConditionScheduleConfig {
  /**
   * Minutes to wait after an execution completes before re-evaluating
   * conditions for another run.
   */
  rearm_delay_minutes: number;
  [k: string]: unknown;
}
