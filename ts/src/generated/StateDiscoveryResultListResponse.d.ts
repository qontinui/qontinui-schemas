/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with `just generate-types` or
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */

import type { DiscoverySourceType } from './DiscoverySourceType';
import type { StateDiscoveryResultSummary } from './StateDiscoveryResultSummary';

/**
 * API response for listing discovery results.
 */
export interface StateDiscoveryResultListResponse {
  /**
   * List of result summaries.
   */
  items: StateDiscoveryResultSummary[];
  /**
   * Total count of results.
   */
  total: number;
  [k: string]: unknown;
}
