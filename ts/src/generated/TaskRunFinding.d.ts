/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with `just generate-types` or
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */

import type { TaskRunFindingActionType } from './TaskRunFindingActionType';
import type { TaskRunFindingCategory } from './TaskRunFindingCategory';
import type { TaskRunFindingSeverity } from './TaskRunFindingSeverity';
import type { TaskRunFindingStatus } from './TaskRunFindingStatus';

/**
 * A finding surfaced during a task run (bug, enhancement, TODO, etc.).
 *
 * All nullable fields here are required-nullable on the wire (always present,
 * possibly `null`), so they use `serde(default)` without `skip_serializing_if`.
 */
export interface TaskRunFinding {
  actionType: TaskRunFindingActionType;
  category: TaskRunFindingCategory;
  /**
   * Snippet of code illustrating the issue.
   */
  codeSnippet: string | null;
  /**
   * Column number where the issue was found.
   */
  columnNumber: number | null;
  /**
   * Full description.
   */
  description: string;
  /**
   * ISO 8601 timestamp when the finding was detected.
   */
  detectedAt: string;
  /**
   * Session number in which the finding was detected.
   */
  detectedInSession: number;
  /**
   * File path where the issue was found.
   */
  filePath: string | null;
  /**
   * Unique identifier (UUID v4 string).
   */
  id: string;
  /**
   * Suggested response options for the user, if input is needed.
   */
  inputOptions: string[] | null;
  /**
   * Line number where the issue was found.
   */
  lineNumber: number | null;
  /**
   * Whether this finding requires user input.
   */
  needsInput: boolean;
  /**
   * Question posed to the user, if input is needed.
   */
  question: string | null;
  /**
   * How the finding was resolved, if applicable.
   */
  resolution: string | null;
  /**
   * ISO 8601 timestamp when the finding was resolved.
   */
  resolvedAt: string | null;
  /**
   * Session number in which the finding was resolved.
   */
  resolvedInSession: number | null;
  severity: TaskRunFindingSeverity;
  /**
   * Hash used to deduplicate findings across runs.
   */
  signatureHash: string | null;
  status: TaskRunFindingStatus;
  /**
   * Parent task run ID.
   */
  taskRunId: string;
  /**
   * Short human-readable title.
   */
  title: string;
  /**
   * ISO 8601 timestamp when the record was last updated.
   */
  updatedAt: string;
  /**
   * The user's response, if any.
   */
  userResponse: string | null;
  [k: string]: unknown;
}
