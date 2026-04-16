/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with `just generate-types` or
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */

import type { McpCallRecord } from './McpCallRecord';

/**
 * Result envelope for an `mcp_calls` query scoped to a single task run.
 */
export interface McpCallsResult {
  /**
   * Matched call records (paginated; see `total_count` for the full size).
   */
  calls: McpCallRecord[];
  /**
   * Number of calls returned in `calls`.
   */
  count: number;
  /**
   * Number of failed calls in the current page.
   */
  failed_count: number;
  /**
   * Whether there are more rows after the current page.
   */
  has_more: boolean;
  /**
   * Number of successful calls in the current page.
   */
  success_count: number;
  /**
   * Task run the query was scoped to.
   */
  task_run_id: string;
  /**
   * Total matching rows across all pages. Defaults to `0` on older rows
   * that predate the field; consumers should prefer `count` when this is
   * absent.
   */
  total_count: number;
  [k: string]: unknown;
}
