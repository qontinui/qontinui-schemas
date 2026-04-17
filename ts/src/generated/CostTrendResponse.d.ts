/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with `just generate-types` or
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */

import type { CostTrendDataPoint } from './CostTrendDataPoint';

/**
 * Response envelope for the cost-trend analytics endpoint.
 */
export interface CostTrendResponse {
  /**
   * Cost trend data points.
   */
  dataPoints: CostTrendDataPoint[];
  /**
   * End date of the reporting window.
   */
  endDate: string;
  /**
   * Granularity label (`"daily"`, `"weekly"`, `"monthly"`).
   */
  granularity: string;
  /**
   * Overall cost statistics over the full window. Shape is intentionally
   * opaque (`dict[str, Any]` in Python).
   */
  overallStats?: {
    [k: string]: unknown;
  };
  /**
   * Project the trend was computed for.
   */
  projectId: string;
  /**
   * Start date of the reporting window.
   */
  startDate: string;
  [k: string]: unknown;
}
