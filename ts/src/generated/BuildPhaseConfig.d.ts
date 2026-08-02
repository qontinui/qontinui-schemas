/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with `just generate-types` or
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */

/**
 * Configuration for the build (workflow-generation) phase.
 */
export interface BuildPhaseConfig {
  /**
   * Additional free-form context to pass to the generator.
   */
  context?: string | null;
  /**
   * IDs of stored `Context` records to include.
   */
  contextIds?: string[] | null;
  /**
   * Human description of the desired workflow.
   */
  description: string;
}
