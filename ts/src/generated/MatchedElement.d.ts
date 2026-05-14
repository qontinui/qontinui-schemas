/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with `just generate-types` or
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */

/**
 * Identity of the bridge element that satisfied an assertion.
 */
export interface MatchedElement {
  /**
   * UI Bridge element identifier (if exposed).
   */
  elementId?: string | null;
  /**
   * CSS selector or DOM path identifying the element.
   */
  path: string;
  /**
   * ARIA role of the matched element.
   */
  role?: string | null;
  /**
   * Visible text of the matched element.
   */
  text?: string | null;
}
