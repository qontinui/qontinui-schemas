/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with `just generate-types` or
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */

/**
 * A specific issue detail from an individual verification check.
 */
export interface CheckIssueDetail {
  /**
   * Error code or lint rule, if applicable.
   */
  code: string | null;
  /**
   * Column number, if applicable.
   */
  column: number | null;
  /**
   * File path where the issue was detected.
   */
  file: string;
  /**
   * Whether the check can auto-fix this issue.
   */
  fixable: boolean;
  /**
   * Line number, if applicable.
   */
  line: number | null;
  /**
   * Human-readable message.
   */
  message: string;
  /**
   * Severity label (free-form string from the upstream check).
   */
  severity: string;
}
