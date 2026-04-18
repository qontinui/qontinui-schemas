/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with `just generate-types` or
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */

/**
 * Configuration for the diagnostic-evaluation phase.
 *
 * Captures UI state after workflow execution and classifies failure root
 * causes.
 */
export interface DiagnosePhaseConfig {
  /**
   * Assertions to run against the UI after workflow execution.
   * Each assertion is a JSON object passed to
   * `POST /ui-bridge/control/assert`.
   */
  assertions?: unknown[];
  /**
   * Whether to capture a full DOM snapshot for AI triage context.
   */
  captureSnapshot: boolean;
  /**
   * Model override for the triage AI call. If `None`, uses default routing.
   */
  modelOverride?: string | null;
  /**
   * Maximum characters to include from the snapshot in the AI triage
   * prompt.
   */
  snapshotMaxChars: number;
}
