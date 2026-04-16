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
  action_sequence_number?: number | null;
  /**
   * Actual observed behavior.
   */
  actual_behavior?: string | null;
  /**
   * Full issue description.
   */
  description: string;
  /**
   * Expected behavior.
   */
  expected_behavior?: string | null;
  /**
   * Free-form issue type label (e.g., `"visual_regression"`, `"flaky"`).
   */
  issue_type: string;
  /**
   * Opaque additional metadata.
   */
  metadata?: {
    [k: string]: unknown;
  } | null;
  /**
   * Steps to reproduce the issue.
   */
  reproduction_steps?: string[] | null;
  /**
   * IDs of screenshots illustrating the issue.
   */
  screenshot_ids?: string[] | null;
  severity: IssueSeverity;
  /**
   * State ID active when the issue was observed.
   */
  state?: string | null;
  /**
   * Short human-readable title.
   */
  title: string;
  [k: string]: unknown;
}
