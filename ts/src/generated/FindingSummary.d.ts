/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with `just generate-types` or
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */

/**
 * Summary statistics for findings in a task run.
 *
 * Aggregated counts grouped along each axis (category, severity, status)
 * plus roll-up counts for UI dashboards.
 */
export interface FindingSummary {
  /**
   * Count of findings by category.
   */
  by_category?: {
    [k: string]: number;
  };
  /**
   * Count of findings by severity.
   */
  by_severity?: {
    [k: string]: number;
  };
  /**
   * Count of findings by status.
   */
  by_status?: {
    [k: string]: number;
  };
  /**
   * Number of findings awaiting user input.
   */
  needs_input_count: number;
  /**
   * Number of unresolved findings.
   */
  outstanding_count: number;
  /**
   * Number of resolved findings.
   */
  resolved_count: number;
  /**
   * Task run ID.
   */
  task_run_id: string;
  /**
   * Total number of findings.
   */
  total: number;
  [k: string]: unknown;
}
