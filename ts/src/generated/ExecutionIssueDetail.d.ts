/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with `just generate-types` or
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */

import type { ExecutionScreenshotResponse } from './ExecutionScreenshotResponse';
import type { IssueSeverity } from './IssueSeverity';
import type { IssueSource } from './IssueSource';
import type { IssueStatus } from './IssueStatus';
import type { IssueType } from './IssueType';

/**
 * Detailed issue information (superset of [`ExecutionIssueResponse`]).
 */
export interface ExecutionIssueDetail {
  /**
   * Sequence number of the associated action, if any.
   */
  action_sequence_number?: number | null;
  /**
   * Assigned user record, if any. Shape is intentionally opaque here; the
   * Python source types this as `dict[str, Any]`.
   */
  assigned_to?: {
    [k: string]: unknown;
  } | null;
  /**
   * ISO 8601 timestamp when the issue was created.
   */
  created_at: string;
  /**
   * Full description.
   */
  description: string;
  /**
   * Error details such as stack traces.
   */
  error_details?: {
    [k: string]: unknown;
  };
  /**
   * Issue ID.
   */
  id: string;
  issue_type: IssueType;
  /**
   * Opaque additional metadata.
   */
  metadata?: {
    [k: string]: unknown;
  };
  /**
   * Steps to reproduce the issue.
   */
  reproduction_steps?: string[];
  /**
   * Resolution notes, if any.
   */
  resolution_notes?: string | null;
  /**
   * Associated run ID.
   */
  run_id: string;
  /**
   * Number of associated screenshots.
   */
  screenshot_count: number;
  /**
   * Full screenshot records associated with the issue.
   */
  screenshots?: ExecutionScreenshotResponse[];
  severity: IssueSeverity;
  source: IssueSource;
  /**
   * State ID where the issue was observed, if any.
   */
  state_name?: string | null;
  status: IssueStatus;
  /**
   * Short title.
   */
  title: string;
  /**
   * ISO 8601 timestamp when the issue was last updated.
   */
  updated_at: string;
  [k: string]: unknown;
}
