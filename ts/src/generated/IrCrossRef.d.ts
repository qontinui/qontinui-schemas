/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with `just generate-types` or
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */

export interface IrCrossRef {
  doc: string;
  /**
   * `ref` is reserved in Rust — serialize as `ref` for the wire while
   * using `r#ref` as the Rust field name.
   */
  ref: string;
  [k: string]: unknown;
}
