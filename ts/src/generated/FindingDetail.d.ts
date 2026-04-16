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
import type { FindingStatus } from './FindingStatus';
import type { FindingUserInput } from './FindingUserInput';

/**
 * Detailed finding information (Backend → Frontend).
 *
 * Used when retrieving individual finding details. The `id` is a UUID v4
 * string (see crate-level wire-format note).
 */
export interface FindingDetail {
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
   * ISO 8601 timestamp (UTC) when the finding was detected.
   */
  detected_at: string;
  /**
   * Finding ID (UUID v4 string).
   */
  id: string;
  /**
   * Resolution description if resolved.
   */
  resolution?: string | null;
  /**
   * ISO 8601 timestamp (UTC) when the finding was resolved.
   */
  resolved_at?: string | null;
  /**
   * Session number where the finding was resolved.
   */
  resolved_in_session?: number | null;
  /**
   * Session number where the finding was detected.
   */
  session_num: number;
  severity: FindingSeverity;
  /**
   * Hash used to deduplicate findings across sessions.
   */
  signature_hash?: string | null;
  status: FindingStatus;
  /**
   * Parent task run ID.
   */
  task_run_id: string;
  /**
   * Brief title describing the finding.
   */
  title: string;
  /**
   * User-input request, if `action_type` requires a user decision.
   */
  user_input?: FindingUserInput | null;
  /**
   * User's response, if input was requested.
   */
  user_response?: string | null;
  [k: string]: unknown;
}
