/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */

/**
 * Trigger condition for a playbook.
 *
 * Determines when a playbook should be automatically included in AI prompts
 * based on the current automation context (app name, URL pattern, etc.).
 */
export interface SkillPlaybookTrigger {
  /**
   * Type of trigger: "app_name", "url_pattern", "tag".
   */
  trigger_type: string;
  /**
   * Value to match against (exact match for app_name, glob for url_pattern).
   */
  value: string;
  [k: string]: unknown;
}
