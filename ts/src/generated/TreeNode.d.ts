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
import type { RuntimeData } from './RuntimeData';
import type { StateContext } from './StateContext';
import type { TimingInfo } from './TimingInfo';
import type { TopMatch } from './TopMatch';

/**
 * A node in the execution tree — a single workflow, action, or transition
 * in the execution hierarchy.
 */
export interface TreeNode {
  /**
   * Duration in seconds.
   */
  duration?: number | null;
  /**
   * When this node completed (Unix epoch seconds).
   */
  endTimestamp?: number | null;
  /**
   * Error message if `status == Failed`.
   */
  error?: string | null;
  /**
   * Unique identifier for this node.
   */
  id: string;
  metadata?: NodeMetadata & {};
  /**
   * Display name for this node.
   */
  name: string;
  nodeType: NodeType;
  /**
   * ID of parent node, `None` for root.
   */
  parentId?: string | null;
  status: NodeStatus;
  /**
   * When this node was created (Unix epoch seconds).
   */
  timestamp: number;
  [k: string]: unknown;
}
