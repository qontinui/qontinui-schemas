/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with `just generate-types` or
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */

/**
 * Request payload asking for playback frames for a sequence of actions.
 */
export interface PlaybackFrameRequest {
  /**
   * Action execution IDs, in playback order (Python constrains to
   * `1..=100`).
   */
  actionIds: string[];
  /**
   * Whether to include screenshot URLs.
   */
  includeScreenshots: boolean;
  [k: string]: unknown;
}
