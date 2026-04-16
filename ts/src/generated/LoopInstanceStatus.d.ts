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
import type { OrchestrationLoopStatus } from './OrchestrationLoopStatus';
import type { RootCauseCategory } from './RootCauseCategory';

/**
 * Status of a single loop instance within a multi-loop.
 */
export interface LoopInstanceStatus {
  /**
   * Human label.
   */
  label?: string | null;
  /**
   * Unique identifier for this loop.
   */
  loop_id: string;
  status: OrchestrationLoopStatus;
  [k: string]: unknown;
}
