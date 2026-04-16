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
 * Request payload for creating a finding.
 */
export interface TaskRunFindingCreate {
  /**
   * Action type (defaults server-side if omitted).
   */
  action_type?: TaskRunFindingActionType | null;
  category: TaskRunFindingCategory;
  /**
   * Snippet of code illustrating the issue.
   */
  code_snippet?: string | null;
  /**
   * Column number where the issue was found.
   */
  column_number?: number | null;
  /**
   * Full description.
   */
  description: string;
  /**
   * Session number in which the finding was detected.
   */
  detected_in_session: number;
  /**
   * File path where the issue was found.
   */
  file_path?: string | null;
  /**
   * Optional client-generated ID.
   */
  id?: string | null;
  /**
   * Suggested response options for the user.
   */
  input_options?: string[] | null;
  /**
   * Line number where the issue was found.
   */
  line_number?: number | null;
  /**
   * Whether this finding requires user input.
   */
  needs_input?: boolean | null;
  /**
   * Question to pose to the user, if input is needed.
   */
  question?: string | null;
  /**
   * Resolution text, if already resolved.
   */
  resolution?: string | null;
  severity: TaskRunFindingSeverity;
  /**
   * Deduplication hash.
   */
  signature_hash?: string | null;
  /**
   * Initial status (defaults server-side if omitted).
   */
  status?: TaskRunFindingStatus | null;
  /**
   * Short title.
   */
  title: string;
  [k: string]: unknown;
}
