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
import type { DiscoverySourceType } from './DiscoverySourceType';
import type { DiscoveryTransitionTrigger } from './DiscoveryTransitionTrigger';
import type { TransitionTriggerType } from './TransitionTriggerType';

/**
 * Request payload to create a state-discovery result.
 */
export interface StateDiscoveryResultCreate {
  /**
   * Confidence score (0.0–1.0).
   */
  confidence: number;
  /**
   * Description.
   */
  description?: string | null;
  /**
   * Additional metadata.
   */
  discoveryMetadata?: {
    [k: string]: unknown;
  };
  /**
   * Strategy used.
   */
  discoveryStrategy?: string | null;
  /**
   * Element to renders mapping.
   */
  elementToRenders?: {
    [k: string]: string[];
  };
  /**
   * Discovered images.
   */
  images?: DiscoveredStateImage[];
  /**
   * Human-readable name.
   */
  name: string;
  /**
   * ID of the source session.
   */
  sourceSessionId?: string | null;
  sourceType: DiscoverySourceType;
  /**
   * Discovered states.
   */
  states?: DiscoveredState[];
  /**
   * Discovered transitions.
   */
  transitions?: DiscoveredTransition[];
}
