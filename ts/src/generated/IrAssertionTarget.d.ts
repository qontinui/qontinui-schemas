/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with `just generate-types` or
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */

/**
 * Mirror of TS `IRAssertionTarget`. Always `type: "search"` for synthesis-
 * emitted assertions; the wider legacy schema also supports point/region
 * targets but the IR doesn't express those today.
 */
export interface IrAssertionTarget {
  /**
   * Free-form criteria object — kept as `Value` so synthesis can emit
   * partially-populated criteria without tripping the schema and so the
   * projection can pass the bytes through verbatim.
   */
  criteria: {
    [k: string]: unknown;
  };
  label: string;
  /**
   * Always `"search"` for synthesis-emitted assertions.
   */
  type: string;
  [k: string]: unknown;
}
