/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with `just generate-types` or
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */

/**
 * Identifier bundle for locating a UI Bridge element.
 *
 * Elements can be addressed by any combination of UI-Bridge ID, test ID,
 * AWAS ID, HTML ID, XPath, or CSS selector. The `xpath` and `selector`
 * fields are always present; the named IDs are optional.
 */
export interface ElementIdentifier {
  /**
   * AWAS-assigned action identifier.
   */
  awasId?: string | null;
  /**
   * Native HTML `id` attribute.
   */
  htmlId?: string | null;
  /**
   * CSS selector that uniquely identifies the element.
   */
  selector: string;
  /**
   * `data-testid` attribute value.
   */
  testId?: string | null;
  /**
   * Application-assigned UI Bridge ID.
   */
  uiId?: string | null;
  /**
   * Full XPath to the element.
   */
  xpath: string;
}
