/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with `just generate-types` or
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */

/**
 * A finding discovered during work.
 */
export interface StructuredFinding {
  /**
   * Finding category (e.g., "bug", "security", "performance").
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
   * Severity level.
   */
  severity: string;
  /**
   * Short title describing the finding.
   */
  title: string;
  [k: string]: unknown;
}
