/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with `just generate-types` or
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */

import type { AgenticStatus } from './AgenticStatus';
import type { FileChange } from './FileChange';
import type { FindingOutput } from './FindingOutput';
import type { ReflectionFixOutput } from './ReflectionFixOutput';

/**
 * Canonical structured output from the agentic phase.
 */
export interface AgenticPhaseOutput {
  /**
   * AI's confidence that the fixes will pass verification (0.0-1.0).
   */
  confidence: number | null;
  /**
   * Files modified during this agentic phase.
   */
  filesModified: FileChange[];
  /**
   * Findings reported by the AI.
   */
  findings: FindingOutput[];
  /**
   * Dynamically injected verification steps.
   */
  injectedSteps: unknown[];
  /**
   * Reflection fixes (when reflection_mode is enabled).
   */
  reflectionFixes: ReflectionFixOutput[];
  status: AgenticStatus;
  /**
   * Human-readable summary of what was done.
   */
  summary: string;
  /**
   * Whether the AI determined errors are unfixable.
   */
  unfixable: boolean;
  /**
   * Reason why errors are unfixable (if unfixable is true).
   */
  unfixableReason: string | null;
}
