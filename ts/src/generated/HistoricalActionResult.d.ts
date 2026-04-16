/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with `just generate-types` or
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */

import type { ActionStatus } from './ActionStatus';
import type { ExecutionActionType } from './ExecutionActionType';

/**
 * A historical action execution, shaped for playback.
 */
export interface HistoricalActionResult {
  /**
   * Human-readable action name.
   */
  action_name: string;
  action_type: ExecutionActionType;
  /**
   * Duration in milliseconds.
   */
  duration_ms: number;
  /**
   * Source state.
   */
  from_state?: string | null;
  /**
   * Whether a screenshot exists.
   */
  has_screenshot: boolean;
  /**
   * Action execution ID.
   */
  id: string;
  /**
   * Input parameters captured for the action.
   */
  input_data?: {
    [k: string]: unknown;
  };
  /**
   * Output produced by the action.
   */
  output_data?: {
    [k: string]: unknown;
  };
  /**
   * URL to an associated screenshot, if any.
   */
  screenshot_url?: string | null;
  status: ActionStatus;
  /**
   * Target state.
   */
  to_state?: string | null;
  [k: string]: unknown;
}
