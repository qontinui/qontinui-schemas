/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with `just generate-types` or
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */

import type { EmbeddingItem } from './EmbeddingItem';

/**
 * Single search result.
 */
export interface SearchResultItem {
  embedding: EmbeddingItem;
  /**
   * Similarity score (0-1).
   */
  similarity_score: number;
}
