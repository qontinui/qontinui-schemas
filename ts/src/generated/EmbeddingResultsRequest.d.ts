/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with `just generate-types` or
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */

import type { EmbeddingResultItem } from './EmbeddingResultItem';

/**
 * Request containing embedding results from the runner.
 */
export interface EmbeddingResultsRequest {
  failed: number;
  project_id: string;
  results: EmbeddingResultItem[];
  successful: number;
  total_processed: number;
}
