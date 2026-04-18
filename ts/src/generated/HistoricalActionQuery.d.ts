/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with `just generate-types` or
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */

import type { ExecutionActionType } from './ExecutionActionType';

/**
 * Query filters for looking up historical action results.
 */
export interface HistoricalActionQuery {
  /**
   * Filter by action name.
   */
  actionName?: string | null;
  /**
   * Filter by action type.
   */
  actionType?: ExecutionActionType | null;
  /**
   * Maximum number of results to return (Python constrains to `1..=100`).
   */
  limit: number;
  /**
   * Filter by project ID.
   */
  projectId?: string | null;
  /**
   * Filter by state name.
   */
  stateName?: string | null;
  /**
   * If true, only successful actions are returned.
   */
  successOnly: boolean;
  /**
   * Filter by workflow ID.
   */
  workflowId?: string | null;
}
