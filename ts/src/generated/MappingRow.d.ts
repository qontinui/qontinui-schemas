/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */

/**
 * One row of an interface mapping.
 */
export interface MappingRow {
  /**
   * The business rule or requirement (`spec_ref`) the row implements.
   */
  rule_ref?: string | null;
  source_field: string;
  target_field: string;
  /**
   * The transformation applied, in prose or a short expression.
   */
  transform?: string | null;
  [k: string]: unknown;
}
