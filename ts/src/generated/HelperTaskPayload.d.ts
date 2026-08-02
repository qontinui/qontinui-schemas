/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with `just generate-types` or
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */

/**
 * Kind-specific content shown to the helper.
 *
 * Every field is optional; a given [`HelperTaskKind`] populates only the
 * fields it needs. A [`HelperTaskKind::SpotCheck`] task populates only
 * [`screenshot_url`](HelperTaskPayload::screenshot_url).
 */
export interface HelperTaskPayload {
  /**
   * Sort: the feature cards the helper groups in the card-sort exercise.
   */
  cards?: string[] | null;
  /**
   * Compare: the two presigned screenshot URLs to put side by side (A, B).
   */
  compareUrls?: string[] | null;
  /**
   * Describe: a reference to the live screen / target the helper describes.
   */
  liveTarget?: string | null;
  /**
   * SpotCheck: a coord-served / presigned URL of the screenshot to review.
   */
  screenshotUrl?: string | null;
  /**
   * WalkThrough: the ordered guided steps the helper follows live.
   */
  steps?: string[] | null;
  [k: string]: unknown;
}
