/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with `just generate-types` or
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */

export interface UpdateAppRequest {
  /**
   * Whether this app requires authentication before spec checks.
   */
  authRequired?: boolean | null;
  /**
   * Build command for "pull_build" strategy.
   */
  buildCommand?: string | null;
  displayName?: string | null;
  /**
   * Red threshold for spec match rates (0.0–1.0). Must be < yellow_threshold.
   */
  redThreshold?: number | null;
  /**
   * Start command to restart after build.
   */
  startCommand?: string | null;
  uiBridgeUrl?: string | null;
  /**
   * Auto-fresh update strategy: "pull_only" or "pull_build".
   */
  updateStrategy?: string | null;
  /**
   * Yellow threshold for spec match rates (0.0–1.0). Must be > red_threshold.
   */
  yellowThreshold?: number | null;
  [k: string]: unknown;
}
