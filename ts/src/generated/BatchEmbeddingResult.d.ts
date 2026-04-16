/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with `just generate-types` or
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */

/**
 * Result for a single image in batch processing.
 */
export interface BatchEmbeddingResult {
  error?: string | null;
  /**
   * Image identifier from the request.
   */
  id: string;
  image_embedding?: number[] | null;
  ocr_confidence?: number | null;
  ocr_text?: string | null;
  success: boolean;
  text_description?: string | null;
  text_embedding?: number[] | null;
  [k: string]: unknown;
}
