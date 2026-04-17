/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with `just generate-types` or
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */

import type { BetweenIterations } from './BetweenIterations';
import type { BuildPhaseConfig } from './BuildPhaseConfig';
import type { DecomposerConfig } from './DecomposerConfig';
import type { DiagnosePhaseConfig } from './DiagnosePhaseConfig';
import type { ExitStrategy } from './ExitStrategy';
import type { ImplementFixesConfig } from './ImplementFixesConfig';
import type { PipelineConfig } from './PipelineConfig';
import type { StallDetectorConfig } from './StallDetectorConfig';
import type { SummarizationConfig } from './SummarizationConfig';

/**
 * Full configuration for an orchestration loop run.
 *
 * Covers the simple mode (fixed `workflow_id`, `max_iterations`, exit
 * strategy) as well as pipeline mode (build / diagnose / reflect / fix
 * phases) when `pipeline` is populated.
 */
export interface OrchestrationLoopConfig {
  /**
   * What to do between iterations.
   */
  betweenIterations?: BetweenIterations & {};
  /**
   * Task-decomposition sub-config (omit to disable).
   */
  decomposition?: DecomposerConfig | null;
  /**
   * How to decide when to stop.
   */
  exitStrategy?: ExitStrategy & {};
  /**
   * Maximum number of iterations.
   * `None` (omitted) means no cap — loop until success or explicit stop.
   */
  maxIterations?: number | null;
  /**
   * Pipeline-mode configuration. When present, enables build / reflect /
   * fix phases.
   */
  pipeline?: PipelineConfig | null;
  /**
   * When `true`, a failed workflow iteration doesn't count as a terminal
   * failure. Instead, the loop waits for the fixer workflow to complete,
   * then re-runs. The loop only exits on success or `max_iterations`.
   */
  retryOnFailure: boolean;
  /**
   * Stall-detection sub-config (omit to disable).
   */
  stallDetection?: StallDetectorConfig | null;
  /**
   * Context-summarization sub-config (omit to disable).
   */
  summarization?: SummarizationConfig | null;
  /**
   * Supervisor port for restart/build operations.
   */
  supervisorPort: number;
  /**
   * Target runner ID (for supervisor restart calls). If `None`, uses
   * `"primary"`.
   */
  targetRunnerId?: string | null;
  /**
   * Target runner port to execute workflows on.
   * If `None`, targets self (this runner's own port).
   */
  targetRunnerPort?: number | null;
  /**
   * Whether to wait for the fixer workflow before starting the next
   * iteration. Only applies when `retry_on_failure` is `true`.
   */
  waitForFixer: boolean;
  /**
   * Workflow ID to execute each iteration.
   * Required for simple mode; optional in pipeline mode if the build phase
   * is configured.
   */
  workflowId: string;
  [k: string]: unknown;
}
