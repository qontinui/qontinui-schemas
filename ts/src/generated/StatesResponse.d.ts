/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with `just generate-types` or
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */

import type { StateFilterItem } from './StateFilterItem';

/**
 * Response with list of states for filtering.
 */
export interface StatesResponse {
  /**
   * Total number of states.
   */
  count: number;
  states: StateFilterItem[];
}
