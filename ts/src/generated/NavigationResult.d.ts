/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with `just generate-types` or
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */

/**
 * Result of navigating to one or more target states.
 *
 * Navigation uses multistate pathfinding to determine the optimal path.
 * Nested `results` allow a single top-level navigation to fan out to several
 * targets and report per-target outcomes.
 */
export interface NavigationResult {
  /**
   * States active after the navigation completed.
   */
  active_states: string[];
  /**
   * Error message when navigation failed.
   */
  error?: string | null;
  /**
   * Ordered list of state IDs visited.
   */
  path: string[];
  /**
   * Per-target sub-results for fan-out navigation.
   */
  results?: NavigationResult[] | null;
  /**
   * Whether the overall navigation succeeded.
   */
  success: boolean;
  /**
   * Target state for this navigation branch (if applicable).
   */
  target_state?: string | null;
}
