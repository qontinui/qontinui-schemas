/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with `just generate-types` or
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */

/**
 * Response after applying embedding results.
 */
export interface EmbeddingResultsResponse {
  applied: number;
  failed: number;
  message: string;
  not_found: number;
  success: boolean;
  [k: string]: unknown;
}
