/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with `just generate-types` or
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */

/**
 * Request to compute embeddings for multiple images.
 */
export interface BatchComputeEmbeddingRequest {
  /**
   * Compute text embeddings for all images.
   */
  compute_text_embeddings: boolean;
  /**
   * Extract OCR text from images.
   */
  extract_ocr: boolean;
  /**
   * Images, each a map with `id`, `image_data` (base64), and optional
   * `text_description`.
   */
  images: {
    [k: string]: unknown;
  }[];
}
