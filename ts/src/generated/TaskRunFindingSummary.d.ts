/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with `just generate-types` or
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */

/**
 * Aggregated finding counts grouped along each axis.
 */
export interface TaskRunFindingSummary {
  /**
   * Count of findings by category.
   */
  byCategory: {
    [k: string]: number;
  };
  /**
   * Count of findings by severity.
   */
  bySeverity: {
    [k: string]: number;
  };
  /**
   * Count of findings by status.
   */
  byStatus: {
    [k: string]: number;
  };
  /**
   * Total number of findings.
   */
  total: number;
  [k: string]: unknown;
}
