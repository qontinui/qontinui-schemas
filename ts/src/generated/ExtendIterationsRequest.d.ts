/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with `just generate-types` or
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */

/**
 * Request to extend iterations after the max has been reached.
 */
export interface ExtendIterationsRequest {
  /**
   * Additional iterations to add.
   */
  additionalIterations: number;
  /**
   * Optional guidance for the worker.
   */
  guidance?: string | null;
  [k: string]: unknown;
}
