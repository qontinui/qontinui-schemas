/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with `just generate-types` or
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */

import type { InitialStateRef } from './InitialStateRef';
import type { InitialStatesSource } from './InitialStatesSource';

/**
 * Result envelope for the "resolve initial states" operation.
 *
 * Unlike [`ResolvedInitialStates`], this shape is non-optional: `states` and
 * `workflowId` are always present (possibly empty), and a `success` / `error`
 * pair is provided.
 */
export interface ResolvedInitialStatesResult {
  /**
   * Error message when resolution failed.
   */
  error?: string | null;
  source: InitialStatesSource;
  /**
   * Resolved initial state IDs.
   */
  stateIds: string[];
  /**
   * Display-ready references for each state.
   */
  states: InitialStateRef[];
  /**
   * Whether resolution succeeded.
   */
  success: boolean;
  /**
   * Workflow ID (may be empty).
   */
  workflowId: string;
  [k: string]: unknown;
}
