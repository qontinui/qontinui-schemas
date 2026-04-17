/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with `just generate-types` or
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */

/**
 * Request payload for creating a new saved orchestration-loop config.
 */
export interface CreateOlConfigRequest {
  /**
   * The full loop config as a JSON blob (should match the
   * [`OrchestrationLoopConfig`] shape).
   */
  configJson: {
    [k: string]: unknown;
  };
  /**
   * Optional description.
   */
  description?: string | null;
  /**
   * Human-readable name for the new preset.
   */
  name: string;
  [k: string]: unknown;
}
