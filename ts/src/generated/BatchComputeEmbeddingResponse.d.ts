/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with `just generate-types` or
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */

import type { BatchEmbeddingResult } from './BatchEmbeddingResult';

/**
 * Response with batch-computed embeddings.
 */
export interface BatchComputeEmbeddingResponse {
  failed: number;
  processing_time_ms: number;
  results: BatchEmbeddingResult[];
  /**
   * Whether the batch succeeded overall.
   */
  success: boolean;
  successful: number;
  total_processed: number;
}
