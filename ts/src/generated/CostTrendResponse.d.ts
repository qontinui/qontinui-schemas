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
  data_points: CostTrendDataPoint[];
  /**
   * End date of the reporting window.
   */
  end_date: string;
  /**
   * Granularity label (`"daily"`, `"weekly"`, `"monthly"`).
   */
  granularity: string;
  /**
   * Overall cost statistics over the full window. Shape is intentionally
   * opaque (`dict[str, Any]` in Python).
   */
  overall_stats?: {
    [k: string]: unknown;
  };
  /**
   * Project the trend was computed for.
   */
  project_id: string;
  /**
   * Start date of the reporting window.
   */
  start_date: string;
  [k: string]: unknown;
}
