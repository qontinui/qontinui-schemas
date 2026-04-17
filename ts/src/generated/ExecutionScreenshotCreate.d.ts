/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with `just generate-types` or
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */

import type { ScreenshotAnnotation } from './ScreenshotAnnotation';
import type { ScreenshotAnnotationShape } from './ScreenshotAnnotationShape';
import type { ScreenshotType } from './ScreenshotType';

/**
 * Request payload describing a screenshot being uploaded.
 */
export interface ExecutionScreenshotCreate {
  /**
   * Sequence number of the associated action, if any.
   */
  actionSequenceNumber?: number | null;
  /**
   * IDs of states active when the screenshot was taken.
   */
  activeStates?: string[] | null;
  /**
   * Overlaid annotations.
   */
  annotations?: ScreenshotAnnotation[] | null;
  /**
   * Image height in pixels.
   */
  height: number;
  /**
   * Opaque additional metadata.
   */
  metadata?: {
    [k: string]: unknown;
  } | null;
  /**
   * Client-generated screenshot identifier.
   */
  screenshotId: string;
  screenshotType: ScreenshotType;
  /**
   * Sequence number of the screenshot within the run.
   */
  sequenceNumber: number;
  /**
   * State ID active when the screenshot was taken.
   */
  state?: string | null;
  /**
   * ISO 8601 timestamp when the screenshot was taken.
   */
  timestamp: string;
  /**
   * Image width in pixels.
   */
  width: number;
  [k: string]: unknown;
}
