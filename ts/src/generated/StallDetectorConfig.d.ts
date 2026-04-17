/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with `just generate-types` or
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */

/**
 * Configuration for the stall-detection subsystem.
 *
 * Controls how the loop engine decides a run is stuck in a repeated-action /
 * oscillation / runaway-step pattern and forces an exit.
 */
export interface StallDetectorConfig {
  /**
   * Maximum times the same action may repeat before stall is declared.
   */
  maxRepeatedActions: number;
  /**
   * Absolute ceiling on total steps across all iterations.
   */
  maxTotalSteps: number;
  /**
   * Window (in actions) used to detect oscillation between two states.
   */
  oscillationWindow: number;
  /**
   * Wall-clock seconds without progress before stall is declared.
   */
  stallTimeoutSecs: number;
  [k: string]: unknown;
}
