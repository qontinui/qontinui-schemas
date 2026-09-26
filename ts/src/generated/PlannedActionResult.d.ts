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
  action: string;
  durationMs: number;
  elementState?: unknown;
  error?: string;
  index: number;
  resolvedElementId?: string;
  skippedLowConfidence: boolean;
  success: boolean;
  [k: string]: unknown;
}
