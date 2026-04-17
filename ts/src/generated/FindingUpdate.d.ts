/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with `just generate-types` or
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */

import type { FindingStatus } from './FindingStatus';

/**
 * Schema for updating a finding.
 *
 * Used to update status, record a resolution, or capture a user response.
 * All fields are optional; only those supplied are applied.
 */
export interface FindingUpdate {
  /**
   * Resolution description.
   */
  resolution?: string | null;
  /**
   * Session number where the finding was resolved.
   */
  resolvedInSession?: number | null;
  /**
   * New status for the finding.
   */
  status?: FindingStatus | null;
  /**
   * User's response to a finding requiring input.
   */
  userResponse?: string | null;
  [k: string]: unknown;
}
