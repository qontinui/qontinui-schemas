/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with `just generate-types` or
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */

/**
 * Request for semantic search.
 */
export interface SemanticSearchRequest {
  /**
   * Max results to return (1–100).
   */
  limit: number;
  /**
   * Minimum similarity threshold (0–1). CLIP text-to-image similarities
   * typically fall in 0.15–0.35, so 0.2 is a reasonable default.
   */
  min_similarity: number;
  /**
   * Search query text (min length 1).
   */
  query: string;
  state_filter?: string | null;
}
