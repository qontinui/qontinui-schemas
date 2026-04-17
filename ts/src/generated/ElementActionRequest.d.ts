/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with `just generate-types` or
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */

import type { WaitOptions } from './WaitOptions';

/**
 * Request to execute an action on a UI Bridge element.
 */
export interface ElementActionRequest {
  /**
   * Action name (e.g. `"click"`, `"type"`, `"select"`).
   */
  action: string;
  /**
   * Optional action-specific parameters.
   */
  params?: {
    [k: string]: unknown;
  };
  /**
   * Optional wait conditions to satisfy before executing.
   */
  waitOptions?: WaitOptions | null;
  [k: string]: unknown;
}
