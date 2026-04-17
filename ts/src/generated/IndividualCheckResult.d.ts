/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with `just generate-types` or
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */

import type { CheckIssueDetail } from './CheckIssueDetail';

/**
 * Result of a single named verification check (e.g., `"eslint"`, `"mypy"`).
 */
export interface IndividualCheckResult {
  /**
   * How long the check took, in milliseconds.
   */
  durationMs: number;
  /**
   * Error message, if the check itself failed to run.
   */
  errorMessage: string | null;
  /**
   * Number of files the check inspected.
   */
  filesChecked: number;
  /**
   * Specific issue details.
   */
  issues: CheckIssueDetail[];
  /**
   * Number of issues auto-fixed by this check.
   */
  issuesFixed: number;
  /**
   * Number of issues surfaced by this check.
   */
  issuesFound: number;
  /**
   * Name of the check.
   */
  name: string;
  /**
   * Raw check output, if captured.
   */
  output: string | null;
  /**
   * Free-form status string (e.g., `"passed"`, `"failed"`, `"skipped"`).
   */
  status: string;
  [k: string]: unknown;
}
