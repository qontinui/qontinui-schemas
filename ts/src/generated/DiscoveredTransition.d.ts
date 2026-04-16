/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with `just generate-types` or
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */

import type { DiscoveryTransitionTrigger } from './DiscoveryTransitionTrigger';
import type { TransitionTriggerType } from './TransitionTriggerType';

/**
 * A transition between discovered states.
 *
 * Transitions represent actions that change the active set of states on the
 * screen.
 */
export interface DiscoveredTransition {
  /**
   * Confidence score for transition detection (0.0–1.0). Defaults to `1.0`.
   */
  confidence: number;
  /**
   * ID of the source state.
   */
  fromStateId: string;
  /**
   * Unique identifier for the transition.
   */
  id: string;
  /**
   * Additional free-form metadata.
   */
  metadata?: {
    [k: string]: unknown;
  } | null;
  /**
   * ID of the target state.
   */
  toStateId: string;
  /**
   * What triggers this transition.
   */
  trigger?: DiscoveryTransitionTrigger | null;
  [k: string]: unknown;
}
