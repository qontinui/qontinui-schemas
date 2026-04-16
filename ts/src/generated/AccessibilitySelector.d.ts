/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with `just generate-types` or
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */

import type { AccessibilityRole } from './AccessibilityRole';
import type { AccessibilityState } from './AccessibilityState';

/**
 * Selector for finding nodes in an accessibility tree.
 *
 * Provides flexible matching criteria for locating elements by role, name,
 * automation ID, or other attributes. Multiple criteria are combined with
 * AND logic.
 */
export interface AccessibilitySelector {
  /**
   * Required ancestor selector.
   */
  ancestor?: AccessibilitySelector | null;
  /**
   * Match by automation / test ID.
   */
  automation_id?: string | null;
  /**
   * Whether string matching is case-sensitive.
   */
  case_sensitive: boolean;
  /**
   * Match by CSS / control class name.
   */
  class_name?: string | null;
  /**
   * Match by HTML tag name.
   */
  html_tag?: string | null;
  /**
   * Filter by interactivity.
   */
  is_interactive?: boolean | null;
  /**
   * Maximum tree depth to search.
   */
  max_depth?: number | null;
  /**
   * Exact name match.
   */
  name?: string | null;
  /**
   * Partial name match (contains).
   */
  name_contains?: string | null;
  /**
   * Regex pattern for name matching.
   */
  name_pattern?: string | null;
  /**
   * Match by role (single or list).
   */
  role?: RoleCriterion | null;
  /**
   * Required state flags (partial match).
   */
  state?: AccessibilityState | null;
  /**
   * Exact value match.
   */
  value?: string | null;
  /**
   * Partial value match (contains).
   */
  value_contains?: string | null;
  [k: string]: unknown;
}
