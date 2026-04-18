/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with `just generate-types` or
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */

import type { IssueSeverity } from './IssueSeverity';
import type { IssueStatus } from './IssueStatus';

/**
 * Request payload for updating an existing issue.
 */
export interface ExecutionIssueUpdate {
  /**
   * User ID to assign the issue to.
   */
  assignedToUserId?: string | null;
  /**
   * Resolution notes.
   */
  resolutionNotes?: string | null;
  /**
   * New severity.
   */
  severity?: IssueSeverity | null;
  /**
   * New lifecycle status.
   */
  status?: IssueStatus | null;
}
