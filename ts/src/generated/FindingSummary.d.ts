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
  byCategory?: {
    [k: string]: number;
  };
  /**
   * Count of findings by severity.
   */
  bySeverity?: {
    [k: string]: number;
  };
  /**
   * Count of findings by status.
   */
  byStatus?: {
    [k: string]: number;
  };
  /**
   * Number of findings awaiting user input.
   */
  needsInputCount: number;
  /**
   * Number of unresolved findings.
   */
  outstandingCount: number;
  /**
   * Number of resolved findings.
   */
  resolvedCount: number;
  /**
   * Task run ID.
   */
  taskRunId: string;
  /**
   * Total number of findings.
   */
  total: number;
}
