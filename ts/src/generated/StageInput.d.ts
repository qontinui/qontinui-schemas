/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with `just generate-types` or
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */

/**
 * An input required by a stage, referencing a prior stage's output.
 */
export interface StageInput {
  /**
   * Which stage provides this input (stage id). If omitted, searches all
   * prior stages.
   */
  from_stage?: string | null;
  /**
   * The key to bind this input to (matches a [`StageOutput::key`] from a
   * prior stage).
   */
  key: string;
  /**
   * Whether this input is required (default: `true`). Missing required
   * inputs are Critical findings.
   */
  required: boolean;
  [k: string]: unknown;
}
