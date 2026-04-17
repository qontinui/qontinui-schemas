/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with `just generate-types` or
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */

import type { DiscoveredElement } from './DiscoveredElement';
import type { ElementRect } from './ElementRect';
import type { ElementState } from './ElementState';

/**
 * Response from a UI Bridge discovery scan.
 */
export interface DiscoveryResponse {
  /**
   * Time taken for the discovery scan in milliseconds.
   */
  durationMs: number;
  /**
   * Discovered elements matching the request filters.
   */
  elements?: DiscoveredElement[];
  /**
   * Unix-epoch millisecond timestamp of the scan.
   */
  timestamp: number;
  /**
   * Total number of elements found (before any limit).
   */
  total: number;
  [k: string]: unknown;
}
