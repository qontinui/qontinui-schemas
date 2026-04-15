/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with `just generate-types` or
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */

/**
 * A finding reported by the AI.
 */
export interface FindingOutput {
  category: string;
  description: string;
  needs_input: boolean;
  resolved: boolean;
  severity: string;
  title: string;
  [k: string]: unknown;
}
