/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with `just generate-types` or
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */

export interface LegacyAssertionTarget {
  /**
   * Free-form criteria object — kept as `Value` so we don't lose any
   * inverse-projection round-trip information.
   */
  criteria: {
    [k: string]: unknown;
  };
  label: string;
  /**
   * Always `"search"` for the projection — point/region targets aren't
   * expressible in the IR.
   */
  type: string;
  [k: string]: unknown;
}
