/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with `just generate-types` or
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */

import type { MouseButton } from './MouseButton';
import type { Point } from './Point';
import type { ScrollDirection } from './ScrollDirection';
import type { StandardActionType } from './StandardActionType';
import type { TransitionAction } from './TransitionAction';
import type { TransitionActionValue } from './TransitionActionValue';

/**
 * Payload for updating an existing transition.
 */
export interface StateMachineTransitionUpdate {
  /**
   * New action list.
   */
  actions?: TransitionAction[] | null;
  /**
   * New "activate" states.
   */
  activate_states?: string[] | null;
  /**
   * New "exit" states.
   */
  exit_states?: string[] | null;
  /**
   * New metadata bag.
   */
  extra_metadata?: {
    [k: string]: unknown;
  } | null;
  /**
   * New "from" states.
   */
  from_states?: string[] | null;
  /**
   * New display name.
   */
  name?: string | null;
  /**
   * New path cost.
   */
  path_cost?: number | null;
  /**
   * New stays-visible flag.
   */
  stays_visible?: boolean | null;
  [k: string]: unknown;
}
