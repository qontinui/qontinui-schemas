/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with `just generate-types` or
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */

/**
 * A signal from the worker to the orchestrator.
 */
export interface StructuredSignal {
  /**
   * Optional message providing context for the signal.
   */
  message?: string | null;
  /**
   * Signal type (e.g., "complete", "blocked", "needs_input").
   */
  signal_type: string;
  [k: string]: unknown;
}
