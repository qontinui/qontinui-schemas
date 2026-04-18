/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with `just generate-types` or
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */

import type { ExecutionActionType } from './ExecutionActionType';

/**
 * Reliability statistics aggregated for a single action type.
 */
export interface ActionReliabilityStats {
  /**
   * Human-readable action name.
   */
  actionName: string;
  actionType: ExecutionActionType;
  /**
   * Mean duration in milliseconds.
   */
  avgDurationMs: number;
  /**
   * Common error categories, as opaque records. Python types this as
   * `list[dict[str, Any]]`; typing it further requires product input.
   */
  commonErrors?: {
    [k: string]: unknown;
  }[];
  /**
   * Number of executions that failed.
   */
  failedExecutions: number;
  /**
   * Median duration in milliseconds.
   */
  p50DurationMs: number;
  /**
   * 95th-percentile duration in milliseconds.
   */
  p95DurationMs: number;
  /**
   * Success rate as a percentage.
   */
  successRate: number;
  /**
   * Number of executions that succeeded.
   */
  successfulExecutions: number;
  /**
   * Total number of executions observed.
   */
  totalExecutions: number;
}
