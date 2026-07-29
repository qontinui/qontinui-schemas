/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with `just generate-types` or
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */

import type { IrElementCriteria } from "./IrElementCriteria";

export interface IrStateCondition {
  comparator?: string | null;
  element: IrElementCriteria;
  expected: unknown;
  /**
   * "visible" | "enabled" | "checked" | "expanded" | "selected" | "text" | "value"
   */
  property: string;
  [k: string]: unknown;
}
