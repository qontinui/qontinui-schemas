/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with `just generate-types` or
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */

export interface RegisterAppRequest {
  appId: string;
  /**
   * Whether this app requires authentication before spec checks.
   */
  authRequired: boolean;
  /**
   * Build command for "pull_build" strategy. Optional.
   */
  buildCommand?: string | null;
  displayName: string;
  /**
   * Red threshold for spec match rates. Defaults to 0.5.
   */
  redThreshold: number;
  repoRoot: string;
  /**
   * Start command to restart after build. Optional.
   */
  startCommand?: string | null;
  uiBridgeUrl: string;
  /**
   * Auto-fresh update strategy (P3 fleet-fresh engine). Defaults to "pull_only".
   */
  updateStrategy: string;
  /**
   * Yellow threshold for spec match rates. Defaults to 0.8.
   */
  yellowThreshold: number;
  [k: string]: unknown;
}
