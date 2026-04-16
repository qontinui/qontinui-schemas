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
 * A transition between states, consisting of one or more actions.
 *
 * Transitions define the edges of the state machine graph.
 */
export interface StateMachineTransition {
  /**
   * Ordered list of actions executed as part of this transition.
   */
  actions: TransitionAction[];
  /**
   * States activated when this transition fires.
   */
  activate_states: string[];
  /**
   * Parent config ID.
   */
  config_id: string;
  /**
   * ISO 8601 timestamp of creation.
   */
  created_at: string;
  /**
   * States exited when this transition fires.
   */
  exit_states: string[];
  /**
   * Free-form metadata bag.
   */
  extra_metadata: {
    [k: string]: unknown;
  };
  /**
   * States this transition may be taken from.
   */
  from_states: string[];
  /**
   * Unique database identifier (UUID).
   */
  id: string;
  /**
   * Display name.
   */
  name: string;
  /**
   * Cost used by pathfinding to prefer cheaper transitions.
   */
  path_cost: number;
  /**
   * Whether source states stay visible after activation.
   */
  stays_visible: boolean;
  /**
   * Stable logical transition ID (distinct from `id`).
   */
  transition_id: string;
  /**
   * ISO 8601 timestamp of last modification.
   */
  updated_at: string;
  [k: string]: unknown;
}
