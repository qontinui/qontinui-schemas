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
import type { MultiLoopEntry } from './MultiLoopEntry';
import type { OrchestrationLoopConfig } from './OrchestrationLoopConfig';
import type { PipelineConfig } from './PipelineConfig';
import type { StallDetectorConfig } from './StallDetectorConfig';
import type { SummarizationConfig } from './SummarizationConfig';

/**
 * Configuration for launching multiple loops at once.
 */
export interface MultiLoopConfig {
  /**
   * Individual loop configurations, each targeting a different runner.
   */
  loops: MultiLoopEntry[];
  /**
   * Stop all loops if any single loop errors out.
   */
  stopAllOnError: boolean;
  [k: string]: unknown;
}
