/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with `just generate-types` or
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */

import type { AccessibilityBackend } from './AccessibilityBackend';
import type { AccessibilityBounds } from './AccessibilityBounds';
import type { AccessibilityNode } from './AccessibilityNode';
import type { AccessibilityRole } from './AccessibilityRole';
import type { AccessibilityState } from './AccessibilityState';

/**
 * Complete accessibility-tree snapshot.
 *
 * Full accessibility tree at a point in time, with metadata about the
 * capture source and summary statistics.
 */
export interface AccessibilitySnapshot {
  backend: AccessibilityBackend;
  /**
   * Number of interactive nodes.
   */
  interactive_nodes: number;
  root: AccessibilityNode;
  /**
   * Unix timestamp of capture.
   */
  timestamp: number;
  /**
   * Page / window title.
   */
  title?: string | null;
  /**
   * Total number of nodes in the tree.
   */
  total_nodes: number;
  /**
   * Page URL (for web targets).
   */
  url?: string | null;
  [k: string]: unknown;
}
