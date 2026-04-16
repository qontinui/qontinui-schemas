/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with `just generate-types` or
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */

import type { ExecutionIssueCreate } from './ExecutionIssueCreate';
import type { IssueSeverity } from './IssueSeverity';

/**
 * Batch wrapper for reporting multiple issues in one request.
 */
export interface ExecutionIssueBatch {
  /**
   * Issues to record (Python enforces `1..=50`; enforced by the backend).
   */
  issues: ExecutionIssueCreate[];
  [k: string]: unknown;
}
