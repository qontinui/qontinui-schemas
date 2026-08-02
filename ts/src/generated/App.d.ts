/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with `just generate-types` or
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */

export interface App {
  appId: string;
  /**
   * Whether this app requires authentication before spec checks.
   * If true, workflows will auto-inject an auth setup step.
   */
  authRequired: boolean;
  /**
   * Build command to run after pulling updated source (P3 auto-fresh).
   * Ignored if `update_strategy` is "pull_only". Optional; if None,
   * the auto-fresh engine skips build and goes straight to start_command.
   */
  buildCommand?: string | null;
  createdAtMs: number;
  displayName: string;
  lastSeenAtMs: number;
  /**
   * Red threshold for spec match rates (0.0–1.0). Match rates below this are Red (fail).
   * Must be less than `yellow_threshold`. Defaults to 0.5.
   */
  redThreshold: number;
  repoRoot: string;
  /**
   * Start command to run after a successful build (P3 auto-fresh).
   * Restarts the deployed instance and updates app_deploy_state.
   * Ignored if `update_strategy` is "pull_only". Optional.
   */
  startCommand?: string | null;
  uiBridgeUrl: string;
  /**
   * Auto-fresh update strategy: "pull_only" (pull code, no restart) or
   * "pull_build" (pull, run build_command, run start_command). Used by the
   * runner's P3 auto-fresh engine to decide what actions to take when
   * pulling updated source code. Defaults to "pull_only".
   */
  updateStrategy: string;
  /**
   * Yellow threshold for spec match rates (0.0–1.0). Match rates >= this are Green (pass),
   * below are Yellow (warn). Must be greater than `red_threshold`. Defaults to 0.8.
   */
  yellowThreshold: number;
  [k: string]: unknown;
}
