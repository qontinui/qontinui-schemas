/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with `just generate-types` or
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */

/**
 * A single step on a computed path.
 */
export interface PathfindingStep {
  /**
   * States activated by this step.
   */
  activate_states: string[];
  /**
   * States exited by this step.
   */
  exit_states: string[];
  /**
   * States required to be active before this step.
   */
  from_states: string[];
  /**
   * Cost of this step.
   */
  path_cost: number;
  /**
   * Logical transition ID taken in this step.
   */
  transition_id: string;
  /**
   * Display name of the transition.
   */
  transition_name: string;
}
