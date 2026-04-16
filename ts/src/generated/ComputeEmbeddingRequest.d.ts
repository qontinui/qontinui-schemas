/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with `just generate-types` or
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */

/**
 * Request to compute embeddings for a single image.
 */
export interface ComputeEmbeddingRequest {
  /**
   * Also compute a text embedding from OCR.
   */
  compute_text_embedding: boolean;
  /**
   * Base64-encoded image data.
   */
  image_data: string;
  /**
   * Optional text description for the text embedding.
   */
  text_description?: string | null;
  [k: string]: unknown;
}
