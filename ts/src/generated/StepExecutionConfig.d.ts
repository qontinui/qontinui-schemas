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
  action_type?: string | null;
  /**
   * Check type for verification steps.
   */
  check_type?: string | null;
  /**
   * Target image ID, if the action references an image.
   */
  target_image_id?: string | null;
  /**
   * Target image name, if the action references an image.
   */
  target_image_name?: string | null;
  /**
   * Timeout in seconds.
   */
  timeout_seconds?: number | null;
  [k: string]: unknown;
}
