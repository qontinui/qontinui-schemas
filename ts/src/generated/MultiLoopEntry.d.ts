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
import type { OrchestrationLoopConfig } from './OrchestrationLoopConfig';
import type { PipelineConfig } from './PipelineConfig';
import type { StallDetectorConfig } from './StallDetectorConfig';
import type { SummarizationConfig } from './SummarizationConfig';

/**
 * A single entry in a multi-loop configuration.
 */
export interface MultiLoopEntry {
  config: OrchestrationLoopConfig;
  /**
   * Human label (e.g., `"Pages 1–10"` or an app section name).
   */
  label?: string | null;
  /**
   * Unique identifier for this loop instance.
   */
  loop_id: string;
  [k: string]: unknown;
}
