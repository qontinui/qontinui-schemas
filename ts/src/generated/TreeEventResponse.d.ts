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

/**
 * Response for a stored tree event.
 */
export interface TreeEventResponse {
  /**
   * ISO 8601 timestamp.
   */
  created_at: string;
  error_message?: string | null;
  event_timestamp: number;
  event_type: TreeEventType;
  /**
   * UUID as string (wire-format — see crate-level docs).
   */
  id: string;
  metadata?: NodeMetadata | null;
  node_id: string;
  node_name: string;
  node_type: NodeType;
  parent_node_id?: string | null;
  path: PathElement[];
  /**
   * Run UUID as string.
   */
  run_id: string;
  sequence: number;
  status: NodeStatus;
  [k: string]: unknown;
}
