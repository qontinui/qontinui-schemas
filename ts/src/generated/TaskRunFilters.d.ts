/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with `just generate-types` or
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */

import type { TaskRunStatus } from './TaskRunStatus';

/**
 * Filter parameters for listing task runs.
 */
export interface TaskRunFilters {
  /**
   * Include only runs before this ISO 8601 timestamp.
   */
  endDate?: string | null;
  /**
   * Pagination limit.
   */
  limit?: number | null;
  /**
   * Pagination offset.
   */
  offset?: number | null;
  /**
   * Restrict to a given project.
   */
  projectId?: string | null;
  /**
   * Include only runs after this ISO 8601 timestamp.
   */
  startDate?: string | null;
  /**
   * Restrict to a given status.
   */
  status?: TaskRunStatus | null;
  [k: string]: unknown;
}
