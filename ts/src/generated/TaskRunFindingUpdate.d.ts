/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with `just generate-types` or
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */

import type { TaskRunFindingStatus } from './TaskRunFindingStatus';

/**
 * Request payload for updating a finding. All fields are optional.
 */
export interface TaskRunFindingUpdate {
  /**
   * Resolution text.
   */
  resolution?: string | null;
  /**
   * ISO 8601 timestamp of resolution.
   */
  resolvedAt?: string | null;
  /**
   * Session number in which the finding was resolved.
   */
  resolvedInSession?: number | null;
  /**
   * New lifecycle status.
   */
  status?: TaskRunFindingStatus | null;
  /**
   * User response, if the finding needed input.
   */
  userResponse?: string | null;
}
