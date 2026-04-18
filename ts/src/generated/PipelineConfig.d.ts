/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with `just generate-types` or
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */

import type { BuildPhaseConfig } from './BuildPhaseConfig';
import type { DiagnosePhaseConfig } from './DiagnosePhaseConfig';
import type { ImplementFixesConfig } from './ImplementFixesConfig';

/**
 * Pipeline-mode configuration for the build → execute → diagnose → reflect →
 * fix cycle.
 */
export interface PipelineConfig {
  /**
   * Generate the workflow from a description (optional — if absent, the
   * top-level `workflow_id` is used).
   */
  build?: BuildPhaseConfig | null;
  /**
   * Diagnostic-evaluation phase — runs after Execute, before Reflect.
   * Captures UI state via UI Bridge and classifies failure root causes.
   */
  diagnose?: DiagnosePhaseConfig | null;
  /**
   * Implement fixes via Claude CLI after reflection finds issues.
   */
  implementFixes?: ImplementFixesConfig | null;
}
