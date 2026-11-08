/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */

/**
 * Which runner instance answered a [`TerminalLayout`] read, as the runner
 * reports itself.
 *
 * Each field is `null` when the runner does not know or report it. See
 * [`TerminalLayout`] for the null and forward-compatibility rules.
 */
export interface TerminalLayoutRunner {
  /**
   * The runner build that answered (the same value as `buildId` on the
   * runner's `GET /health`).
   */
  buildId: string | null;
  /**
   * The device id the runner reports for itself.
   */
  deviceId: string | null;
  /**
   * The runner instance's name.
   */
  instanceName: string | null;
  /**
   * The port the runner instance's HTTP API listens on.
   */
  port: number | null;
  [k: string]: unknown;
}
