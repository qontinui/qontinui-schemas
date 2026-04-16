/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with `just generate-types` or
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */

import type { DiscoveryBoundingBox } from './DiscoveryBoundingBox';

/**
 * Visual element within a discovered state.
 *
 * Represents an image crop from a screenshot with its bounding box and
 * optional pixel-level identification.
 */
export interface DiscoveredStateImage {
  bbox: DiscoveryBoundingBox;
  /**
   * Confidence score for this image (0.0–1.0). Defaults to `1.0`.
   */
  confidence: number;
  /**
   * Semantic type of the element (e.g. `button`, `input`).
   */
  elementType?: string | null;
  /**
   * Unique identifier for the image.
   */
  id: string;
  /**
   * Human-readable label for the image.
   */
  label?: string | null;
  /**
   * Additional free-form metadata.
   */
  metadata?: {
    [k: string]: unknown;
  } | null;
  /**
   * Hash of pixel data for deduplication.
   */
  pixelHash?: string | null;
  /**
   * ID of the source screenshot.
   */
  screenshotId?: string | null;
  /**
   * URL to the source screenshot.
   */
  screenshotUrl?: string | null;
  /**
   * ID of the state this image belongs to.
   */
  stateId?: string | null;
  [k: string]: unknown;
}
