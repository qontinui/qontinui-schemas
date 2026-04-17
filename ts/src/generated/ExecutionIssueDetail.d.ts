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
  actionSequenceNumber?: number | null;
  /**
   * Assigned user record, if any. Shape is intentionally opaque here; the
   * Python source types this as `dict[str, Any]`.
   */
  assignedTo?: {
    [k: string]: unknown;
  } | null;
  /**
   * ISO 8601 timestamp when the issue was created.
   */
  createdAt: string;
  /**
   * Full description.
   */
  description: string;
  /**
   * Error details such as stack traces.
   */
  errorDetails?: {
    [k: string]: unknown;
  };
  /**
   * Issue ID.
   */
  id: string;
  issueType: IssueType;
  /**
   * Opaque additional metadata.
   */
  metadata?: {
    [k: string]: unknown;
  };
  /**
   * Steps to reproduce the issue.
   */
  reproductionSteps?: string[];
  /**
   * Resolution notes, if any.
   */
  resolutionNotes?: string | null;
  /**
   * Associated run ID.
   */
  runId: string;
  /**
   * Number of associated screenshots.
   */
  screenshotCount: number;
  /**
   * Full screenshot records associated with the issue.
   */
  screenshots?: ExecutionScreenshotResponse[];
  severity: IssueSeverity;
  source: IssueSource;
  /**
   * State ID where the issue was observed, if any.
   */
  stateName?: string | null;
  status: IssueStatus;
  /**
   * Short title.
   */
  title: string;
  /**
   * ISO 8601 timestamp when the issue was last updated.
   */
  updatedAt: string;
  [k: string]: unknown;
}
