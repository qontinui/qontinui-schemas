/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */

import type { PlannedActionResult } from "./PlannedActionResult";

/**
 * Aggregated result of executing a full action plan.
 */
export interface ActionPlanResult {
  /**
   * Whether this plan was stored in the cache for future reuse
   */
  cached: boolean;
  executedCount: number;
  failedCount: number;
  goal?: string;
  results: PlannedActionResult[];
  skippedCount: number;
  success: boolean;
  totalDurationMs: number;
}
