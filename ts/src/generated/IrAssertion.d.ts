/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with `just generate-types` or
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */

import type { IrAssertionTarget } from "./IrAssertionTarget";

/**
 * Mirror of TS `IRAssertion`. Field shape matches `LegacyAssertion` one-to-one
 * (modulo the `target.criteria` looseness).
 */
export interface IrAssertion {
  assertionType: string;
  category: string;
  description: string;
  enabled: boolean;
  id: string;
  precondition?: string | null;
  reviewed: boolean;
  severity: string;
  source: string;
  target: IrAssertionTarget;
  [k: string]: unknown;
}
