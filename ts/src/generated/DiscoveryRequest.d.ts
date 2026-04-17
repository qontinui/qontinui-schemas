/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with `just generate-types` or
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */

/**
 * Options for a UI Bridge element-discovery scan.
 *
 * Discovery crawls the live DOM and returns elements that match the
 * provided filters, regardless of whether they are registered in the
 * bridge registry.
 */
export interface DiscoveryRequest {
  /**
   * If `true`, include hidden/off-screen elements.
   */
  includeHidden?: boolean | null;
  /**
   * If `true`, only return interactive elements (buttons, inputs, etc.).
   */
  interactiveOnly?: boolean | null;
  /**
   * Maximum number of elements to return.
   */
  limit?: number | null;
  /**
   * CSS selector for the root element to start scanning from.
   */
  root?: string | null;
  /**
   * CSS selector filter (only elements matching this selector).
   */
  selector?: string | null;
  /**
   * Filter by element types (e.g. `["button", "input"]`).
   */
  types?: string[] | null;
  [k: string]: unknown;
}
