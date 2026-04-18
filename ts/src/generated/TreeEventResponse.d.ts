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
  createdAt: string;
  errorMessage?: string | null;
  eventTimestamp: number;
  eventType: TreeEventType;
  /**
   * UUID as string (wire-format — see crate-level docs).
   */
  id: string;
  metadata?: NodeMetadata | null;
  nodeId: string;
  nodeName: string;
  nodeType: NodeType;
  parentNodeId?: string | null;
  path: PathElement[];
  /**
   * Run UUID as string.
   */
  runId: string;
  sequence: number;
  status: NodeStatus;
}
