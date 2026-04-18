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
 * Payload for creating a new transition.
 */
export interface StateMachineTransitionCreate {
  /**
   * Ordered list of actions executed as part of this transition.
   */
  actions: TransitionAction[];
  /**
   * States activated when this transition fires.
   */
  activate_states: string[];
  /**
   * States exited when this transition fires.
   */
  exit_states: string[];
  /**
   * Free-form metadata bag.
   */
  extra_metadata?: {
    [k: string]: unknown;
  } | null;
  /**
   * States this transition may be taken from.
   */
  from_states: string[];
  /**
   * Display name.
   */
  name: string;
  /**
   * Cost used by pathfinding.
   */
  path_cost?: number | null;
  /**
   * Whether source states stay visible after activation.
   */
  stays_visible?: boolean | null;
}
