/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with `just generate-types` or
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */

/**
 * Resource limits for workflow execution.
 *
 * When a limit is approached (within the warning threshold), the tracker
 * emits context injection actions. When exceeded, it emits stronger actions.
 */
export interface ResourceLimits {
  /**
   * Maximum agentic phase durations summed (milliseconds).
   */
  maxAgenticTimeMs?: number | null;
  /**
   * Maximum number of unique files modified across all iterations.
   */
  maxFilesModified?: number | null;
  /**
   * Maximum wall-clock time for the entire workflow (seconds).
   */
  maxWallTimeSecs?: number | null;
  /**
   * Warning threshold as a fraction (0.0-1.0). When resource usage exceeds
   * this fraction of the limit, a warning is injected.
   * Default: 0.75 (warn at 75% of limit).
   */
  warningThreshold?: number | null;
}
