/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with `just generate-types` or
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */

/**
 * Response with computed embeddings for a single image.
 */
export interface ComputeEmbeddingResponse {
  error?: string | null;
  /**
   * CLIP image embedding vector (512 dimensions).
   */
  image_embedding?: number[] | null;
  /**
   * OCR confidence score (0-1).
   */
  ocr_confidence?: number | null;
  /**
   * Text extracted via OCR.
   */
  ocr_text?: string | null;
  /**
   * Processing time in milliseconds.
   */
  processing_time_ms: number;
  success: boolean;
  /**
   * AI-generated text description.
   */
  text_description?: string | null;
  /**
   * Text embedding vector (384 dimensions).
   */
  text_embedding?: number[] | null;
  [k: string]: unknown;
}
