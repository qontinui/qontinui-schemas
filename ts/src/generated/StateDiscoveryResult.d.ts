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
 * Complete state-machine result from discovery.
 *
 * Unified output format regardless of the source (Playwright, UI Bridge,
 * Recording, Vision, Manual).
 */
export interface StateDiscoveryResult {
  /**
   * Overall confidence score (0.0–1.0).
   */
  confidence: number;
  /**
   * ISO 8601 timestamp of creation.
   */
  createdAt: string;
  /**
   * Description of this state machine.
   */
  description?: string | null;
  /**
   * Additional discovery metadata.
   */
  discoveryMetadata?: {
    [k: string]: unknown;
  };
  /**
   * Strategy used for discovery (`auto`, `fingerprint`, `legacy`, …).
   */
  discoveryStrategy?: string | null;
  /**
   * Mapping of element IDs to render IDs where they appear.
   */
  elementToRenders?: {
    [k: string]: string[];
  };
  /**
   * Unique identifier for the result.
   */
  id: string;
  /**
   * Number of images (statistic).
   */
  imageCount: number;
  /**
   * All discovered images.
   */
  images?: DiscoveredStateImage[];
  /**
   * Human-readable name.
   */
  name: string;
  /**
   * ID of the project this belongs to.
   */
  projectId: string;
  /**
   * Number of renders analyzed (statistic).
   */
  renderCount: number;
  /**
   * ID of the source session (extraction, recording, …).
   */
  sourceSessionId?: string | null;
  sourceType: DiscoverySourceType;
  /**
   * Number of states (statistic).
   */
  stateCount: number;
  /**
   * All discovered states.
   */
  states?: DiscoveredState[];
  /**
   * Number of transitions (statistic).
   */
  transitionCount: number;
  /**
   * All discovered transitions.
   */
  transitions?: DiscoveredTransition[];
  /**
   * Number of unique elements (statistic).
   */
  uniqueElementCount: number;
  /**
   * ISO 8601 timestamp of last update.
   */
  updatedAt: string;
  [k: string]: unknown;
}
