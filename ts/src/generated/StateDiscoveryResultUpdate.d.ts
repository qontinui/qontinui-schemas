/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with `just generate-types` or
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */

import type { DiscoveredState } from './DiscoveredState';
import type { DiscoveredStateImage } from './DiscoveredStateImage';
import type { DiscoveredTransition } from './DiscoveredTransition';
import type { DiscoveryBoundingBox } from './DiscoveryBoundingBox';
import type { DiscoveryTransitionTrigger } from './DiscoveryTransitionTrigger';
import type { TransitionTriggerType } from './TransitionTriggerType';

/**
 * Request payload to update a state-discovery result.
 *
 * All fields optional; only supplied fields are applied.
 */
export interface StateDiscoveryResultUpdate {
  /**
   * Description.
   */
  description?: string | null;
  /**
   * Updated metadata.
   */
  discoveryMetadata?: {
    [k: string]: unknown;
  } | null;
  /**
   * Updated images.
   */
  images?: DiscoveredStateImage[] | null;
  /**
   * Human-readable name.
   */
  name?: string | null;
  /**
   * Updated states.
   */
  states?: DiscoveredState[] | null;
  /**
   * Updated transitions.
   */
  transitions?: DiscoveredTransition[] | null;
}
