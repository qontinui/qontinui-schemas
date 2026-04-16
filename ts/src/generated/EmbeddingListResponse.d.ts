/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with `just generate-types` or
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */

import type { EmbeddingItem } from './EmbeddingItem';

/**
 * Paginated list of embeddings.
 */
export interface EmbeddingListResponse {
  has_more: boolean;
  items: EmbeddingItem[];
  limit: number;
  /**
   * 1-indexed page number.
   */
  page: number;
  total: number;
  [k: string]: unknown;
}
