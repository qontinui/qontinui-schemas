/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with `just generate-types` or
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */

import type { MatchLocation } from './MatchLocation';
import type { NodeMetadata } from './NodeMetadata';
import type { NodeStatus } from './NodeStatus';
import type { NodeType } from './NodeType';
import type { Outcome } from './Outcome';
import type { PathElement } from './PathElement';
import type { RuntimeData } from './RuntimeData';
import type { StateContext } from './StateContext';
import type { TimingInfo } from './TimingInfo';
import type { TopMatch } from './TopMatch';
import type { TreeEventType } from './TreeEventType';
import type { TreeNode } from './TreeNode';

/**
 * A tree event emitted during execution.
 *
 * Primary event type for execution logging. Carries the event type, the
 * affected node with full metadata, the path from root to this node, and
 * a sequence number for ordering.
 */
export interface TreeEvent {
  eventType: TreeEventType;
  node: TreeNode;
  /**
   * Path from root to this node (breadcrumb).
   */
  path: PathElement[];
  /**
   * Sequence number for ordering.
   */
  sequence: number;
  /**
   * When this event was emitted (Unix epoch seconds).
   */
  timestamp: number;
  /**
   * Event type identifier — always `"tree_event"` on the wire.
   */
  type: string;
  [k: string]: unknown;
}
