/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with `just generate-types` or
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */

import type { ExecutionRunResponse } from './ExecutionRunResponse';
import type { Pagination } from './Pagination';
import type { RunStatus } from './RunStatus';
import type { RunType } from './RunType';

/**
 * Paginated list of execution runs.
 */
export interface ExecutionRunListResponse {
  pagination: Pagination;
  /**
   * Page of matching runs.
   */
  runs: ExecutionRunResponse[];
  [k: string]: unknown;
}
