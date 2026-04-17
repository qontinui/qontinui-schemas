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

/**
 * Compact findings summary including the most recent findings.
 *
 * The TS type `TaskRunFindingResponse` is a type alias for `TaskRunFinding`;
 * in Rust we use `TaskRunFinding` directly.
 */
export interface FindingsSummary {
  /**
   * Count by category.
   */
  byCategory: {
    [k: string]: number;
  };
  /**
   * Count by severity.
   */
  bySeverity: {
    [k: string]: number;
  };
  /**
   * Count by status.
   */
  byStatus: {
    [k: string]: number;
  };
  /**
   * Most recent findings.
   */
  recent: TaskRunFinding[];
  /**
   * Total number of findings.
   */
  total: number;
  [k: string]: unknown;
}
