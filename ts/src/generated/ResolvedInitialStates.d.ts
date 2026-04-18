/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with `just generate-types` or
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */

import type { InitialStateRef } from './InitialStateRef';
import type { InitialStatesSource } from './InitialStatesSource';

/**
 * The resolved set of initial states for a run, along with the source the
 * resolution came from.
 */
export interface ResolvedInitialStates {
  source: InitialStatesSource;
  /**
   * Resolved initial state IDs.
   */
  stateIds: string[];
  /**
   * Display-ready references for each state (optional).
   */
  states?: InitialStateRef[] | null;
  /**
   * Workflow ID when `source == Workflow`.
   */
  workflowId?: string | null;
}
