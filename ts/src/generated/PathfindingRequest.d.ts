/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with `just generate-types` or
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */

/**
 * Request to compute a path between state sets.
 *
 * The pathfinding types are for the graph editor's path-preview feature.
 * Runtime navigation uses the qontinui Python library (`multistate`)
 * directly.
 */
export interface PathfindingRequest {
  /**
   * States assumed to be active at the start of the search.
   */
  from_states: string[];
  /**
   * States that must all be active at the end.
   */
  target_states: string[];
}
