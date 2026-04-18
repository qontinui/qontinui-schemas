/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with `just generate-types` or
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */

import type { IssueSeverity } from './IssueSeverity';

/**
 * Request payload for reporting an issue observed during a run.
 */
export interface ExecutionIssueCreate {
  /**
   * Sequence number of the associated action, if any.
   */
  actionSequenceNumber?: number | null;
  /**
   * Actual observed behavior.
   */
  actualBehavior?: string | null;
  /**
   * Full issue description.
   */
  description: string;
  /**
   * Expected behavior.
   */
  expectedBehavior?: string | null;
  /**
   * Free-form issue type label (e.g., `"visual_regression"`, `"flaky"`).
   */
  issueType: string;
  /**
   * Opaque additional metadata.
   */
  metadata?: {
    [k: string]: unknown;
  } | null;
  /**
   * Steps to reproduce the issue.
   */
  reproductionSteps?: string[] | null;
  /**
   * IDs of screenshots illustrating the issue.
   */
  screenshotIds?: string[] | null;
  severity: IssueSeverity;
  /**
   * State ID active when the issue was observed.
   */
  state?: string | null;
  /**
   * Short human-readable title.
   */
  title: string;
}
