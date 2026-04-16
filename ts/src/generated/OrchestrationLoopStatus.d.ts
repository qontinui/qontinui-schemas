/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with `just generate-types` or
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */

import type { DiagnosticResult } from './DiagnosticResult';
import type { ExitCheckResult } from './ExitCheckResult';
import type { IterationResult } from './IterationResult';
import type { LoopPhase } from './LoopPhase';
import type { RootCauseCategory } from './RootCauseCategory';

/**
 * Runtime state of an orchestration loop.
 */
export interface OrchestrationLoopStatus {
  /**
   * Iteration index (1-based) currently executing or just completed.
   */
  current_iteration: number;
  /**
   * Terminal error message (only set in the `Error` phase).
   */
  error?: string | null;
  /**
   * Whether this loop is in pipeline mode.
   */
  is_pipeline: boolean;
  /**
   * Per-iteration results accumulated so far.
   */
  iteration_results?: IterationResult[];
  /**
   * Iteration cap for this run. `None` renders as `"∞"`/unlimited in the UI.
   */
  max_iterations?: number | null;
  phase: LoopPhase;
  /**
   * Whether the loop is currently running.
   */
  running: boolean;
  /**
   * ISO 8601 start timestamp.
   */
  started_at?: string | null;
  /**
   * Target runner ID.
   */
  target_runner_id?: string | null;
  /**
   * Target runner port.
   */
  target_runner_port: number;
  /**
   * The workflow ID being executed.
   */
  workflow_id: string;
  [k: string]: unknown;
}
