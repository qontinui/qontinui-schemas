/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with `just generate-types` or
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */

/**
 * Request payload for partially updating an existing saved preset.
 *
 * All fields are optional — only those set are applied.
 */
export interface UpdateOlConfigRequest {
  /**
   * Replace the stored config JSON blob.
   */
  configJson?: {
    [k: string]: unknown;
  };
  /**
   * Replace the description.
   */
  description?: string | null;
  /**
   * Toggle favorite status.
   */
  isFavorite?: boolean | null;
  /**
   * Rename the preset.
   */
  name?: string | null;
  [k: string]: unknown;
}
