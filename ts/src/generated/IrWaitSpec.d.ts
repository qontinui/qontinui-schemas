/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with `just generate-types` or
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */

import type { IrElementCriteria } from "./IrElementCriteria";

export interface IrWaitSpec {
  ms?: number | null;
  property?: string | null;
  query?: IrElementCriteria | null;
  quietPeriodMs?: number | null;
  stateId?: string | null;
  timeout?: number | null;
  /**
   * "idle" | "element" | "state" | "time" | "condition" | "vanish" | "change" | "stable"
   */
  type: string;
  [k: string]: unknown;
}
