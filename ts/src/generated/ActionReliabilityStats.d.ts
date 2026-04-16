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
  action_name: string;
  action_type: ExecutionActionType;
  /**
   * Mean duration in milliseconds.
   */
  avg_duration_ms: number;
  /**
   * Common error categories, as opaque records. Python types this as
   * `list[dict[str, Any]]`; typing it further requires product input.
   */
  common_errors?: {
    [k: string]: unknown;
  }[];
  /**
   * Number of executions that failed.
   */
  failed_executions: number;
  /**
   * Median duration in milliseconds.
   */
  p50_duration_ms: number;
  /**
   * 95th-percentile duration in milliseconds.
   */
  p95_duration_ms: number;
  /**
   * Success rate as a percentage.
   */
  success_rate: number;
  /**
   * Number of executions that succeeded.
   */
  successful_executions: number;
  /**
   * Total number of executions observed.
   */
  total_executions: number;
  [k: string]: unknown;
}
