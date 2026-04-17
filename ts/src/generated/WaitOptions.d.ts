/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with `just generate-types` or
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */

/**
 * Wait-condition options attached to an element action request.
 *
 * Before executing the action the bridge can optionally wait until the
 * target element reaches a specified visibility/enabled/focused state.
 */
export interface WaitOptions {
  /**
   * Wait until the element is enabled (or disabled if `false`).
   */
  enabled?: boolean | null;
  /**
   * Wait until the element has focus (or loses focus if `false`).
   */
  focused?: boolean | null;
  /**
   * Polling interval in milliseconds for condition checks.
   */
  interval?: number | null;
  /**
   * Maximum time to wait in milliseconds before timing out.
   */
  timeout?: number | null;
  /**
   * Wait until the element is visible (or hidden if `false`).
   */
  visible?: boolean | null;
  [k: string]: unknown;
}
