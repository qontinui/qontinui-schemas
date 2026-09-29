/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */

/**
 * Which producer answered, and at what version.
 */
export interface Producer {
  /**
   * e.g. `vision-core/layout`, `runner/page-health`, `sdk/page-health`.
   */
  id: string;
  /**
   * The producing crate's or package's version.
   */
  version: string;
  [k: string]: unknown;
}
