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
  coveragePercentage: number;
  /**
   * Per-state visit counts, keyed by state ID.
   */
  stateVisitCounts?: {
    [k: string]: number;
  } | null;
  /**
   * Number of states visited at least once.
   */
  statesCovered: number;
  /**
   * Total number of states in the workflow.
   */
  totalStates: number;
  /**
   * Total number of transitions in the workflow.
   */
  totalTransitions: number;
  /**
   * Per-transition execution counts, keyed by transition ID.
   */
  transitionExecutionCounts?: {
    [k: string]: number;
  } | null;
  /**
   * Number of transitions executed at least once.
   */
  transitionsCovered: number;
  /**
   * IDs of states that were not visited.
   */
  uncoveredStates?: string[] | null;
  /**
   * IDs of transitions that were not executed.
   */
  uncoveredTransitions?: string[] | null;
}
