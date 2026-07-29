/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with `just generate-types` or
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */

/**
 * Information about a single action exposed by a UI Bridge component.
 */
export interface ComponentActionInfo {
  /**
   * Longer description of what the action does.
   */
  description?: string | null;
  /**
   * Unique action identifier within the component.
   */
  id: string;
  /**
   * Human-readable label.
   */
  label?: string | null;
}
