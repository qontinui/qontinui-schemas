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
  actionName: string;
  actionType: ExecutionActionType;
  /**
   * Duration in milliseconds.
   */
  durationMs: number;
  /**
   * Source state.
   */
  fromState?: string | null;
  /**
   * Whether a screenshot exists.
   */
  hasScreenshot: boolean;
  /**
   * Action execution ID.
   */
  id: string;
  /**
   * Input parameters captured for the action.
   */
  inputData?: {
    [k: string]: unknown;
  };
  /**
   * Output produced by the action.
   */
  outputData?: {
    [k: string]: unknown;
  };
  /**
   * URL to an associated screenshot, if any.
   */
  screenshotUrl?: string | null;
  status: ActionStatus;
  /**
   * Target state.
   */
  toState?: string | null;
  [k: string]: unknown;
}
