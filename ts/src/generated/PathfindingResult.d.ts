/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with `just generate-types` or
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */

import type { PathfindingStep } from './PathfindingStep';

/**
 * Result of a pathfinding search.
 */
export interface PathfindingResult {
  /**
   * Error message when pathfinding failed.
   */
  error?: string | null;
  /**
   * Whether a valid path was found.
   */
  found: boolean;
  /**
   * Ordered steps on the computed path (empty when `found` is `false`).
   */
  steps: PathfindingStep[];
  /**
   * Sum of `path_cost` across steps.
   */
  total_cost: number;
  [k: string]: unknown;
}
