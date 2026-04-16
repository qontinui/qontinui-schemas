/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with `just generate-types` or
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */

/**
 * Coverage data computed for a workflow run.
 */
export interface CoverageData {
  /**
   * Overall coverage as a percentage in the range `[0.0, 100.0]`.
   */
  coverage_percentage: number;
  /**
   * Per-state visit counts, keyed by state ID.
   */
  state_visit_counts?: {
    [k: string]: number;
  } | null;
  /**
   * Number of states visited at least once.
   */
  states_covered: number;
  /**
   * Total number of states in the workflow.
   */
  total_states: number;
  /**
   * Total number of transitions in the workflow.
   */
  total_transitions: number;
  /**
   * Per-transition execution counts, keyed by transition ID.
   */
  transition_execution_counts?: {
    [k: string]: number;
  } | null;
  /**
   * Number of transitions executed at least once.
   */
  transitions_covered: number;
  /**
   * IDs of states that were not visited.
   */
  uncovered_states?: string[] | null;
  /**
   * IDs of transitions that were not executed.
   */
  uncovered_transitions?: string[] | null;
  [k: string]: unknown;
}
