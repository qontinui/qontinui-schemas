/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */

import type { UnknownCode } from "./UnknownCode";

/**
 * One dimension the producer could NOT measure, how many items it affected,
 * and why.
 */
export interface UnmeasuredDimension {
  code: UnknownCode;
  /**
   * How many considered items lacked it.
   */
  count: number;
  /**
   * What went unmeasured, e.g. `bbox`, `stacking_order`, `text`.
   */
  dimension: string;
  [k: string]: unknown;
}
