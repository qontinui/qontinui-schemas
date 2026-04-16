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
  action_type: TaskRunFindingActionType;
  category: TaskRunFindingCategory;
  /**
   * Snippet of code illustrating the issue.
   */
  code_snippet: string | null;
  /**
   * Column number where the issue was found.
   */
  column_number: number | null;
  /**
   * Full description.
   */
  description: string;
  /**
   * ISO 8601 timestamp when the finding was detected.
   */
  detected_at: string;
  /**
   * Session number in which the finding was detected.
   */
  detected_in_session: number;
  /**
   * File path where the issue was found.
   */
  file_path: string | null;
  /**
   * Unique identifier (UUID v4 string).
   */
  id: string;
  /**
   * Suggested response options for the user, if input is needed.
   */
  input_options: string[] | null;
  /**
   * Line number where the issue was found.
   */
  line_number: number | null;
  /**
   * Whether this finding requires user input.
   */
  needs_input: boolean;
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
  resolved_at: string | null;
  /**
   * Session number in which the finding was resolved.
   */
  resolved_in_session: number | null;
  severity: TaskRunFindingSeverity;
  /**
   * Hash used to deduplicate findings across runs.
   */
  signature_hash: string | null;
  status: TaskRunFindingStatus;
  /**
   * Parent task run ID.
   */
  task_run_id: string;
  /**
   * Short human-readable title.
   */
  title: string;
  /**
   * ISO 8601 timestamp when the record was last updated.
   */
  updated_at: string;
  /**
   * The user's response, if any.
   */
  user_response: string | null;
  [k: string]: unknown;
}
