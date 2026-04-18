/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with `just generate-types` or
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */

/**
 * Code context for a finding.
 *
 * Provides location (file/line/column) and an optional snippet for findings
 * that relate to specific code.
 */
export interface FindingCodeContext {
  /**
   * Column number where the finding was detected.
   */
  column?: number | null;
  /**
   * File path where the finding was detected.
   */
  file?: string | null;
  /**
   * Line number where the finding was detected.
   */
  line?: number | null;
  /**
   * Code snippet related to the finding (max 1000 chars on the Python side).
   */
  snippet?: string | null;
}
