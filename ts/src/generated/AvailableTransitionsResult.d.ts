/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with `just generate-types` or
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */

import type { TransitionInfo } from './TransitionInfo';

/**
 * Result of querying available transitions from the current state.
 */
export interface AvailableTransitionsResult {
  /**
   * The state the transitions are taken from.
   */
  current_state?: string | null;
  /**
   * Error message when the query failed.
   */
  error?: string | null;
  /**
   * Human-readable message (e.g., "no transitions available").
   */
  message?: string | null;
  /**
   * Whether the query succeeded.
   */
  success: boolean;
  /**
   * Transitions available from the current state.
   */
  transitions: TransitionInfo[];
  [k: string]: unknown;
}
