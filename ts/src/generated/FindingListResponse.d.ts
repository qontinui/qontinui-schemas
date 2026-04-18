/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with `just generate-types` or
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */

import type { FindingActionType } from './FindingActionType';
import type { FindingCategory } from './FindingCategory';
import type { FindingCodeContext } from './FindingCodeContext';
import type { FindingDetail } from './FindingDetail';
import type { FindingSeverity } from './FindingSeverity';
import type { FindingStatus } from './FindingStatus';
import type { FindingUserInput } from './FindingUserInput';

/**
 * Response schema for a paginated finding list.
 */
export interface FindingListResponse {
  /**
   * Findings on this page.
   */
  findings: FindingDetail[];
  /**
   * Whether more items exist beyond this page.
   */
  hasMore: boolean;
  /**
   * Maximum items per page.
   */
  limit: number;
  /**
   * Number of items skipped.
   */
  offset: number;
  /**
   * Total count of findings matching the query.
   */
  total: number;
}
