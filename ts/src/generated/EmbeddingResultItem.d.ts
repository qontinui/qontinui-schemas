/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with `just generate-types` or
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */

/**
 * Single embedding result from the runner.
 */
export interface EmbeddingResultItem {
  error?: string | null;
  /**
   * CLIP image embedding vector (512 dimensions).
   */
  image_embedding?: number[] | null;
  ocr_confidence?: number | null;
  ocr_text?: string | null;
  /**
   * ID of the state image that was processed.
   */
  state_image_id: string;
  success: boolean;
  text_description?: string | null;
  /**
   * Text embedding vector (384 dimensions for all-MiniLM-L6-v2).
   */
  text_embedding?: number[] | null;
  [k: string]: unknown;
}
