/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with `just generate-types` or
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */

import type { ExecutionIssueResponse } from './ExecutionIssueResponse';
import type { Pagination } from './Pagination';

/**
 * Paginated list of issues with a severity/status summary.
 */
export interface ExecutionIssueListResponse {
  /**
   * Page of matching issues.
   */
  issues: ExecutionIssueResponse[];
  pagination: Pagination;
  /**
   * Summary keyed by severity or status. Shape is intentionally opaque.
   */
  summary?: {
    [k: string]: unknown;
  };
}
