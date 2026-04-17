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
  fileSizeBytes: number;
  /**
   * URL to the full-size image.
   */
  imageUrl: string;
  /**
   * Associated run ID.
   */
  runId: string;
  /**
   * Assigned screenshot identifier.
   */
  screenshotId: string;
  /**
   * URL to a thumbnail image, if generated.
   */
  thumbnailUrl?: string | null;
  /**
   * ISO 8601 timestamp when the image was uploaded.
   */
  uploadedAt: string;
  [k: string]: unknown;
}
