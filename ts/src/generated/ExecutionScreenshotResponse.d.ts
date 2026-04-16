/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with `just generate-types` or
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */

/**
 * Response envelope returned after uploading a screenshot.
 */
export interface ExecutionScreenshotResponse {
  /**
   * File size in bytes.
   */
  file_size_bytes: number;
  /**
   * URL to the full-size image.
   */
  image_url: string;
  /**
   * Associated run ID.
   */
  run_id: string;
  /**
   * Assigned screenshot identifier.
   */
  screenshot_id: string;
  /**
   * URL to a thumbnail image, if generated.
   */
  thumbnail_url?: string | null;
  /**
   * ISO 8601 timestamp when the image was uploaded.
   */
  uploaded_at: string;
  [k: string]: unknown;
}
