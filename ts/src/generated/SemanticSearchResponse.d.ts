/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with `just generate-types` or
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */

import type { EmbeddingItem } from './EmbeddingItem';
import type { SearchResultItem } from './SearchResultItem';

/**
 * Response from semantic search.
 */
export interface SemanticSearchResponse {
  /**
   * The original search query.
   */
  query: string;
  results: SearchResultItem[];
  total_found: number;
  [k: string]: unknown;
}
