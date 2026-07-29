/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with `just generate-types` or
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */

/**
 * Execution config captured for a verification step.
 *
 * The TypeScript type includes an index signature `[key: string]: unknown`,
 * so extra arbitrary fields are captured in `extra` via `serde(flatten)` and
 * passed through opaquely.
 */
export interface StepExecutionConfig {
  /**
   * Action type (e.g., click, type, wait).
   */
  actionType?: string | null;
  /**
   * Check type for verification steps.
   */
  checkType?: string | null;
  /**
   * Target image ID, if the action references an image.
   */
  targetImageId?: string | null;
  /**
   * Target image name, if the action references an image.
   */
  targetImageName?: string | null;
  /**
   * Timeout in seconds.
   */
  timeoutSeconds?: number | null;
  [k: string]: unknown;
}
