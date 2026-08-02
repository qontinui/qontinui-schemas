/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with `just generate-types` or
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */

/**
 * Request to compute a text embedding for semantic search.
 */
export interface ComputeTextEmbeddingRequest {
  /**
   * Embedding model: `"clip"` (512-dim) or `"minilm"` (384-dim).
   */
  model: string;
  /**
   * Text to encode into embedding space.
   */
  text: string;
}
