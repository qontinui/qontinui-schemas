/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with `just generate-types` or
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */

/**
 * Pagination envelope attached to list responses.
 */
export interface Pagination {
  /**
   * Whether additional records are available after this page.
   */
  has_more: boolean;
  /**
   * Maximum number of records returned per page.
   */
  limit: number;
  /**
   * Offset into the full result set.
   */
  offset: number;
  /**
   * Total number of matching records.
   */
  total: number;
  [k: string]: unknown;
}
