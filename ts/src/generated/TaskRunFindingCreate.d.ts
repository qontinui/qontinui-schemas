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
  actionType?: TaskRunFindingActionType | null;
  category: TaskRunFindingCategory;
  /**
   * Snippet of code illustrating the issue.
   */
  codeSnippet?: string | null;
  /**
   * Column number where the issue was found.
   */
  columnNumber?: number | null;
  /**
   * Full description.
   */
  description: string;
  /**
   * Session number in which the finding was detected.
   */
  detectedInSession: number;
  /**
   * File path where the issue was found.
   */
  filePath?: string | null;
  /**
   * Optional client-generated ID.
   */
  id?: string | null;
  /**
   * Suggested response options for the user.
   */
  inputOptions?: string[] | null;
  /**
   * Line number where the issue was found.
   */
  lineNumber?: number | null;
  /**
   * Whether this finding requires user input.
   */
  needsInput?: boolean | null;
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
  signatureHash?: string | null;
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
