/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with `just generate-types` or
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */

/**
 * Mirror of `IRElementCriteria`
 * (`qontinui-schemas/ts/src/ui-bridge-ir/element-criteria.ts`).
 */
export interface IrElementCriteria {
  accessibleName?: string | null;
  ariaLabel?: string | null;
  /**
   * HTML attributes to check (exact string match). Stored as a `BTreeMap`
   * so JSON serialization is deterministic (matters for byte-stable
   * projection output).
   */
  attributes?: {
    [k: string]: string;
  } | null;
  id?: string | null;
  role?: string | null;
  tagName?: string | null;
  text?: string | null;
  textContains?: string | null;
  [k: string]: unknown;
}
