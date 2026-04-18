/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with `just generate-types` or
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */

/**
 * Configuration for the task-decomposition subsystem.
 *
 * When enabled, a single high-level task may be split into a plan of
 * sub-tasks executed in sequence within the loop.
 */
export interface DecomposerConfig {
  /**
   * Whether decomposition is active.
   */
  enabled: boolean;
  /**
   * Upper bound on plan length.
   */
  maxSubtasks: number;
  /**
   * Lower bound on plan length.
   */
  minSubtasks: number;
  /**
   * Override the default AI model used for planning.
   */
  modelOverride?: string | null;
}
