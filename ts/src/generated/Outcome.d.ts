/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with `just generate-types` or
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */

/**
 * Execution outcome of an action.
 */
export interface Outcome {
  error?: string | null;
  retryCount: number;
  success: boolean;
}
