/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with `just generate-types` or
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */

import type { TaskRunFinding } from './TaskRunFinding';
import type { TaskRunFindingActionType } from './TaskRunFindingActionType';
import type { TaskRunFindingCategory } from './TaskRunFindingCategory';
import type { TaskRunFindingSeverity } from './TaskRunFindingSeverity';
import type { TaskRunFindingStatus } from './TaskRunFindingStatus';
import type { TaskRunFindingSummary } from './TaskRunFindingSummary';

/**
 * Response for `GET /task-runs/{id}/findings`.
 */
export interface TaskRunFindingsListResponse {
  /**
   * Findings for the task run.
   */
  findings: TaskRunFinding[];
  summary: TaskRunFindingSummary;
  [k: string]: unknown;
}
