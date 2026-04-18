/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with `just generate-types` or
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */

import type { ElementRect } from './ElementRect';
import type { ElementState } from './ElementState';

/**
 * Response from executing an action on an element or component.
 */
export interface ActionResponse {
  /**
   * Time taken to execute the action in milliseconds.
   */
  durationMs: number;
  /**
   * Updated element state after the action (if applicable).
   */
  elementState?: ElementState | null;
  /**
   * Error message if the action failed.
   */
  error?: string | null;
  /**
   * Action-specific return value.
   */
  result?: {
    [k: string]: unknown;
  };
  /**
   * Stack trace if the action threw an exception.
   */
  stack?: string | null;
  /**
   * Whether the action completed successfully.
   */
  success: boolean;
  /**
   * Unix-epoch millisecond timestamp when the action completed.
   */
  timestamp: number;
}
