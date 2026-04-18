/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with `just generate-types` or
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */

import type { NextTaskInfo } from './NextTaskInfo';

/**
 * Summary of the scheduler's current runtime state, returned from the status
 * API.
 */
export interface SchedulerStatus {
  /**
   * Whether the scheduler is enabled.
   */
  enabled: boolean;
  /**
   * The next task scheduled to run, if any.
   */
  nextTask?: NextTaskInfo | null;
  /**
   * Number of tasks scheduled but not yet running.
   */
  pendingTasks: number;
  /**
   * Number of tasks currently running.
   */
  runningTasks: number;
}
