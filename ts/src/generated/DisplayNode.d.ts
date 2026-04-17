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
 * Display node structure used by the frontend — extended version of
 * `TreeNode` with tree-rendering properties. NOT persisted; constructed
 * from `TreeNode` data for UI display.
 */
export interface DisplayNode {
  /**
   * Child nodes in the tree.
   */
  children: DisplayNode[];
  duration?: number | null;
  endTimestamp?: number | null;
  error?: string | null;
  id: string;
  /**
   * Whether this node should be expanded in the UI (default: true).
   */
  isExpanded: boolean;
  /**
   * Nesting level in the tree (0 for root, 1 for first-level children).
   */
  level: number;
  metadata?: NodeMetadata & {};
  name: string;
  nodeType: NodeType;
  status: NodeStatus;
  timestamp: number;
  [k: string]: unknown;
}
