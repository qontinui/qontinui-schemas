/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with `just generate-types` or
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */

import type { ExecutionTrendDataPoint } from './ExecutionTrendDataPoint';
import type { RunType } from './RunType';

/**
 * Response envelope for the execution-trend analytics endpoint.
 */
export interface ExecutionTrendResponse {
  /**
   * Trend data points.
   */
  dataPoints: ExecutionTrendDataPoint[];
  /**
   * End date of the reporting window.
   */
  endDate: string;
  /**
   * Granularity label (`"daily"`, `"weekly"`, `"monthly"`).
   */
  granularity: string;
  /**
   * Overall statistics computed over the full window. Shape is
   * intentionally opaque (`dict[str, Any]` in Python).
   */
  overallStats?: {
    [k: string]: unknown;
  };
  /**
   * Project the trend was computed for.
   */
  projectId: string;
  /**
   * Run type filter, if applied.
   */
  runType?: RunType | null;
  /**
   * Start date of the reporting window.
   */
  startDate: string;
  [k: string]: unknown;
}
