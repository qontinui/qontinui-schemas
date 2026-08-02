/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with `just generate-types` or
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */

/**
 * Workflow category for organization and automation control.
 *
 * Categories organize workflows and control which are available for
 * automation in the runner. Only workflows in categories with
 * `automationEnabled = true` appear in the runner's workflow list.
 */
export interface Category {
  /**
   * Whether workflows in this category are available for automation.
   */
  automationEnabled: boolean;
  /**
   * Category name (e.g., `"Main"`, `"Testing"`).
   */
  name: string;
}
