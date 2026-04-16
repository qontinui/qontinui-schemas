/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with `just generate-types` or
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */

import type { ActionExecutionResponse } from './ActionExecutionResponse';
import type { Pagination } from './Pagination';

/**
 * Paginated list of action executions.
 */
export interface ActionExecutionListResponse {
  /**
   * Page of matching actions.
   */
  actions: ActionExecutionResponse[];
  pagination: Pagination;
  [k: string]: unknown;
}
