/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */

/**
 * Response with a computed text embedding.
 */
export interface ComputeTextEmbeddingResponse {
  /**
   * Text embedding vector.
   */
  embedding?: number[] | null;
  /**
   * Dimension of the embedding vector.
   */
  embedding_dim: number;
  /**
   * Error message if the request failed.
   */
  error?: string | null;
  /**
   * Processing time in milliseconds.
   */
  processing_time_ms: number;
  /**
   * Whether the embedding computation succeeded.
   */
  success: boolean;
}
