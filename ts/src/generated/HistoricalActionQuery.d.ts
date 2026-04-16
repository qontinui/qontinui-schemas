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
  action_name?: string | null;
  /**
   * Filter by action type.
   */
  action_type?: ExecutionActionType | null;
  /**
   * Maximum number of results to return (Python constrains to `1..=100`).
   */
  limit: number;
  /**
   * Filter by project ID.
   */
  project_id?: string | null;
  /**
   * Filter by state name.
   */
  state_name?: string | null;
  /**
   * If true, only successful actions are returned.
   */
  success_only: boolean;
  /**
   * Filter by workflow ID.
   */
  workflow_id?: string | null;
  [k: string]: unknown;
}
