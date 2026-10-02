/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */

/**
 * Result of a single planned action execution.
 */
export interface PlannedActionResult {
  /**
   * The request's `action` echoed back. The request accepts any string and
   * the runner forwards an unrecognised one to the UI Bridge rather than
   * rejecting it, so this is open — not the request-side
   * `PlannedActionType` union.
   */
  action: string;
  durationMs: number;
  /**
   * The UI Bridge's post-action `elementState` object. The handler keeps it
   * only when it IS an object, so the schema's `Record<string, unknown>`
   * is a guarantee, not a hope.
   */
  elementState?: {
    [k: string]: unknown;
  };
  error?: string;
  index: number;
  resolvedElementId?: string;
  skippedLowConfidence: boolean;
  success: boolean;
}
