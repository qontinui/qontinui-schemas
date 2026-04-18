/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with `just generate-types` or
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */

import type { BoundingBox } from './BoundingBox';
import type { ElementType } from './ElementType';
import type { GUIElementChunk } from './GUIElementChunk';

/**
 * Result from a vector database search query.
 *
 * Contains the matched element and relevance scores.
 * Mirrors `rag.models.SearchResult`.
 */
export interface VectorSearchResult {
  /**
   * Distance metric from query.
   */
  distance: number;
  element: GUIElementChunk;
  /**
   * Which embedding was matched: `"text"`, `"image"`, or `"hybrid"`.
   */
  matched_on: string;
  /**
   * The original query text.
   */
  query_text: string;
  /**
   * ISO 8601 UTC timestamp (`Z` suffix).
   */
  query_timestamp?: string | null;
  /**
   * Position in result list.
   */
  rank: number;
  /**
   * Similarity score (0-1).
   */
  score: number;
  /**
   * Type of search performed.
   */
  search_type: string;
}
