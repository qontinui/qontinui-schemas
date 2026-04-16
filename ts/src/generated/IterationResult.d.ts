/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with `just generate-types` or
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */

import type { DiagnosticResult } from './DiagnosticResult';
import type { ExitCheckResult } from './ExitCheckResult';
import type { RootCauseCategory } from './RootCauseCategory';

/**
 * Result of a single iteration.
 */
export interface IterationResult {
  /**
   * ISO 8601 completion timestamp.
   */
  completed_at: string;
  /**
   * Whether the loop context was summarized during this iteration.
   */
  context_summarized?: boolean | null;
  /**
   * Diagnostic-phase result (if the diagnose phase is configured).
   */
  diagnostic_result?: DiagnosticResult | null;
  exit_check: ExitCheckResult;
  /**
   * Number of fixes proposed during reflection.
   */
  fix_count?: number | null;
  /**
   * Pipeline mode: whether fixes were implemented during this iteration.
   */
  fixes_implemented?: boolean | null;
  /**
   * Pipeline mode: ID of the workflow generated during the build phase.
   */
  generated_workflow_id?: string | null;
  /**
   * 1-based iteration index.
   */
  iteration: number;
  /**
   * Pipeline mode: whether a rebuild was triggered for the next iteration.
   */
  rebuild_triggered?: boolean | null;
  /**
   * Task-run ID produced by the reflection step (if any).
   */
  reflection_task_run_id?: string | null;
  /**
   * Stall-detection reason, if a stall was detected.
   */
  stall_detected?: string | null;
  /**
   * ISO 8601 start timestamp.
   */
  started_at: string;
  /**
   * Task-run ID produced by the workflow execution.
   */
  task_run_id: string;
  [k: string]: unknown;
}
