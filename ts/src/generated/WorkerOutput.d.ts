/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with `just generate-types` or
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */

import type { ConfidenceLevel } from './ConfidenceLevel';
import type { StructuredFinding } from './StructuredFinding';
import type { StructuredOverride } from './StructuredOverride';
import type { StructuredSignal } from './StructuredSignal';

/**
 * Structured output from an AI worker agent.
 */
export interface WorkerOutput {
  /**
   * Confidence level in the work quality.
   */
  confidence?: ConfidenceLevel & string;
  /**
   * Criterion overrides with justifications.
   */
  criterionOverrides: StructuredOverride[];
  /**
   * Files that were modified in this iteration.
   */
  filesModified: string[];
  /**
   * Findings discovered during work.
   */
  findings: StructuredFinding[];
  /**
   * Optional suggestion for next action if work continues.
   */
  nextActionSuggestion?: string | null;
  /**
   * Optional notes for debugging or context.
   */
  notes?: string | null;
  /**
   * Optional progress estimate (0.0 to 1.0).
   */
  progressEstimate?: number | null;
  /**
   * Signals for orchestrator control flow.
   */
  signals: StructuredSignal[];
  /**
   * Summary of work performed in this iteration.
   */
  workSummary: string;
}
