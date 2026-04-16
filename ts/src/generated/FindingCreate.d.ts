/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with `just generate-types` or
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */

import type { FindingActionType } from './FindingActionType';
import type { FindingCategory } from './FindingCategory';
import type { FindingCodeContext } from './FindingCodeContext';
import type { FindingSeverity } from './FindingSeverity';
import type { FindingUserInput } from './FindingUserInput';

/**
 * Schema for creating a finding (Runner → Backend).
 *
 * Sent by the runner when an AI analysis session detects an issue or
 * observation.
 */
export interface FindingCreate {
  action_type: FindingActionType;
  category: FindingCategory;
  /**
   * Code context, if the finding relates to specific code.
   */
  code_context?: FindingCodeContext | null;
  /**
   * Detailed description of the finding.
   */
  description: string;
  /**
   * Session number where the finding was detected.
   */
  session_num: number;
  severity: FindingSeverity;
  /**
   * Hash used to deduplicate findings across sessions.
   */
  signature_hash?: string | null;
  /**
   * Parent task run ID.
   */
  task_run_id: string;
  /**
   * Brief title describing the finding (max 500 chars on the Python side).
   */
  title: string;
  /**
   * User-input request, if `action_type` requires a user decision.
   */
  user_input?: FindingUserInput | null;
  [k: string]: unknown;
}
