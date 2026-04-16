/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with `just generate-types` or
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */

/**
 * A state machine configuration — a named collection of states and
 * transitions that describe the navigable structure of an application's UI.
 */
export interface StateMachineConfig {
  /**
   * ISO 8601 timestamp of creation.
   */
  created_at: string;
  /**
   * Optional description.
   */
  description?: string | null;
  /**
   * Number of unique elements discovered in this config.
   */
  element_count: number;
  /**
   * Unique identifier (UUID).
   */
  id: string;
  /**
   * Whether to include HTML IDs when generating selectors.
   */
  include_html_ids: boolean;
  /**
   * Display name.
   */
  name: string;
  /**
   * Number of DOM renders collected for this config.
   */
  render_count: number;
  /**
   * ISO 8601 timestamp of last modification.
   */
  updated_at: string;
  [k: string]: unknown;
}
