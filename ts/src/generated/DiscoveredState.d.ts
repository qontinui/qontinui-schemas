/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with `just generate-types` or
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */

/**
 * A discovered UI state (collection of co-occurring elements).
 *
 * States represent distinct UI screens or views identified by the set of
 * images that consistently appear together.
 */
export interface DiscoveredState {
  /**
   * Confidence score for state detection (0.0–1.0). Defaults to `1.0`.
   */
  confidence: number;
  /**
   * Description of what this state represents.
   */
  description?: string | null;
  /**
   * IDs of DOM elements (for web extraction).
   */
  elementIds?: string[];
  /**
   * Unique identifier for the state.
   */
  id: string;
  /**
   * IDs of images in this state.
   */
  imageIds?: string[];
  /**
   * Additional free-form metadata.
   */
  metadata?: {
    [k: string]: unknown;
  } | null;
  /**
   * Human-readable name.
   */
  name: string;
  /**
   * IDs of renders where this state appears.
   */
  renderIds?: string[];
}
