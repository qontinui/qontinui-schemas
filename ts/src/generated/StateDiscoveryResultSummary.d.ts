/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with `just generate-types` or
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */

import type { DiscoverySourceType } from './DiscoverySourceType';

/**
 * Summary of a state-discovery result (for listings).
 *
 * Lightweight projection of `StateDiscoveryResult` used by list endpoints.
 */
export interface StateDiscoveryResultSummary {
  /**
   * Confidence score (0.0–1.0).
   */
  confidence: number;
  /**
   * ISO 8601 timestamp of creation.
   */
  createdAt: string;
  /**
   * Description.
   */
  description?: string | null;
  /**
   * Strategy used.
   */
  discoveryStrategy?: string | null;
  /**
   * Unique identifier.
   */
  id: string;
  /**
   * Number of images.
   */
  imageCount: number;
  /**
   * Human-readable name.
   */
  name: string;
  /**
   * ID of the project.
   */
  projectId: string;
  sourceType: DiscoverySourceType;
  /**
   * Number of states.
   */
  stateCount: number;
  /**
   * Number of transitions.
   */
  transitionCount: number;
  [k: string]: unknown;
}
