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
  completedAt: string;
  /**
   * Whether the loop context was summarized during this iteration.
   */
  contextSummarized?: boolean | null;
  /**
   * Diagnostic-phase result (if the diagnose phase is configured).
   */
  diagnosticResult?: DiagnosticResult | null;
  exitCheck: ExitCheckResult;
  /**
   * Number of fixes proposed during reflection.
   */
  fixCount?: number | null;
  /**
   * Pipeline mode: whether fixes were implemented during this iteration.
   */
  fixesImplemented?: boolean | null;
  /**
   * Pipeline mode: ID of the workflow generated during the build phase.
   */
  generatedWorkflowId?: string | null;
  /**
   * 1-based iteration index.
   */
  iteration: number;
  /**
   * Pipeline mode: whether a rebuild was triggered for the next iteration.
   */
  rebuildTriggered?: boolean | null;
  /**
   * Task-run ID produced by the reflection step (if any).
   */
  reflectionTaskRunId?: string | null;
  /**
   * Stall-detection reason, if a stall was detected.
   */
  stallDetected?: string | null;
  /**
   * ISO 8601 start timestamp.
   */
  startedAt: string;
  /**
   * Task-run ID produced by the workflow execution.
   */
  taskRunId: string;
  [k: string]: unknown;
}
