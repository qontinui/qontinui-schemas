/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with `just generate-types` or
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */

import type { Pagination } from './Pagination';
import type { TaskRunBackend } from './TaskRunBackend';
import type { TaskRunStatus } from './TaskRunStatus';

/**
 * Response for `GET /task-runs`.
 */
export interface TaskRunListResponse {
  pagination: Pagination;
  /**
   * Page of matching task runs.
   */
  tasks: TaskRunBackend[];
}
