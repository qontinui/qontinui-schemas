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
  /**
   * Finding category.
   */
  category: string;
  /**
   * Detailed description.
   */
  description: string;
  /**
   * Whether this finding requires human input.
   */
  needs_input: boolean;
  /**
   * Whether this finding has been resolved.
   */
  resolved: boolean;
  /**
   * Severity level.
   */
  severity: string;
  /**
   * Short title describing the finding.
   */
  title: string;
  [k: string]: unknown;
}
