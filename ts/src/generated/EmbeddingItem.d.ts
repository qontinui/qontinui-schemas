/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with `just generate-types` or
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */

/**
 * Single embedding record for display.
 */
export interface EmbeddingItem {
  /**
   * ISO 8601 UTC timestamp (`Z` suffix).
   */
  created_at: string;
  embedding_model: string;
  embedding_version: string;
  /**
   * Whether a text embedding vector is available.
   */
  has_text_embedding: boolean;
  /**
   * Embedding UUID.
   */
  id: string;
  image_height: number;
  image_id: string;
  image_storage_path: string;
  /**
   * Presigned URL for displaying the image.
   */
  image_url?: string | null;
  image_width: number;
  pattern_id: string;
  /**
   * Additional pattern metadata.
   */
  pattern_metadata: {
    [k: string]: unknown;
  };
  pattern_name?: string | null;
  state_id: string;
  state_name: string;
  text_description?: string | null;
  /**
   * ISO 8601 UTC timestamp (`Z` suffix).
   */
  updated_at: string;
  [k: string]: unknown;
}
