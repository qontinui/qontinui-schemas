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
 * Result of embedding a GUI element.
 *
 * Contains the original element plus computed embeddings.
 * Mirrors `rag.models.EmbeddedElement`.
 */
export interface EmbeddedElement {
  element: GUIElementChunk;
  /**
   * Model used for embedding.
   */
  embedding_model: string;
  /**
   * ISO 8601 UTC timestamp (`Z` suffix).
   */
  embedding_timestamp?: string | null;
  /**
   * Image embedding vector.
   */
  image_embedding?: number[] | null;
  /**
   * Text embedding vector.
   */
  text_embedding?: number[] | null;
  [k: string]: unknown;
}
