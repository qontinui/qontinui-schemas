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
  actionType: FindingActionType;
  category: FindingCategory;
  /**
   * Code context, if the finding relates to specific code.
   */
  codeContext?: FindingCodeContext | null;
  /**
   * Detailed description of the finding.
   */
  description: string;
  /**
   * ISO 8601 timestamp (UTC) when the finding was detected.
   */
  detectedAt: string;
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
  resolvedAt?: string | null;
  /**
   * Session number where the finding was resolved.
   */
  resolvedInSession?: number | null;
  /**
   * Session number where the finding was detected.
   */
  sessionNum: number;
  severity: FindingSeverity;
  /**
   * Hash used to deduplicate findings across sessions.
   */
  signatureHash?: string | null;
  status: FindingStatus;
  /**
   * Parent task run ID.
   */
  taskRunId: string;
  /**
   * Brief title describing the finding.
   */
  title: string;
  /**
   * User-input request, if `action_type` requires a user decision.
   */
  userInput?: FindingUserInput | null;
  /**
   * User's response, if input was requested.
   */
  userResponse?: string | null;
  [k: string]: unknown;
}
