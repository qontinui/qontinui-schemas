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
 * Portable export format for state machines.
 *
 * Used when exporting a discovery result to a shareable artifact.
 * `source_type` is kept as a free-form `String` to match Python's
 * `DiscoverySourceType | str` union (enabling imports that predate the enum).
 */
export interface StateMachineExport {
  /**
   * Description.
   */
  description?: string | null;
  /**
   * Element to renders mapping.
   */
  elementToRenders?: {
    [k: string]: string[];
  };
  /**
   * State images.
   */
  images?: DiscoveredStateImage[];
  /**
   * Export metadata (original ID, export timestamp, …).
   */
  metadata?: {
    [k: string]: unknown;
  };
  /**
   * State machine name.
   */
  name: string;
  /**
   * Original discovery source (string for forward compatibility — Python
   * accepts `DiscoverySourceType | str`).
   */
  sourceType: string;
  /**
   * States.
   */
  states?: DiscoveredState[];
  /**
   * Transitions.
   */
  transitions?: DiscoveredTransition[];
  /**
   * Export format version. Defaults to `"1.0.0"`.
   */
  version: string;
}
