/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with `just generate-types` or
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */

import type { DiagnosticResult } from './DiagnosticResult';
import type { ExitCheckResult } from './ExitCheckResult';
import type { IterationResult } from './IterationResult';
import type { LoopInstanceStatus } from './LoopInstanceStatus';
import type { LoopPhase } from './LoopPhase';
import type { OrchestrationLoopStatus } from './OrchestrationLoopStatus';
import type { RootCauseCategory } from './RootCauseCategory';

/**
 * Aggregated status across all active loops.
 */
export interface MultiLoopStatus {
  /**
   * Whether every loop has reached a terminal phase.
   */
  allComplete: boolean;
  /**
   * Whether any loop is in the `Error` phase.
   */
  anyError: boolean;
  /**
   * Per-loop status snapshots.
   */
  loops: LoopInstanceStatus[];
  /**
   * Whether the multi-loop manager is configured to abort all loops on the
   * first error.
   */
  stopAllOnError: boolean;
}
