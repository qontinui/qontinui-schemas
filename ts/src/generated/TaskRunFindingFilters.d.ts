/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with `just generate-types` or
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */

import type { TaskRunFindingCategory } from './TaskRunFindingCategory';
import type { TaskRunFindingSeverity } from './TaskRunFindingSeverity';
import type { TaskRunFindingStatus } from './TaskRunFindingStatus';

/**
 * Filter parameters for listing findings.
 */
export interface TaskRunFindingFilters {
  /**
   * Restrict to a given category.
   */
  category?: TaskRunFindingCategory | null;
  /**
   * Restrict to a given severity.
   */
  severity?: TaskRunFindingSeverity | null;
  /**
   * Restrict to a given status.
   */
  status?: TaskRunFindingStatus | null;
  [k: string]: unknown;
}
