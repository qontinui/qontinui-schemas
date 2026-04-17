/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with `just generate-types` or
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */

import type { DisplayNode } from './DisplayNode';
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
 * Full execution tree reconstructed from events.
 */
export interface ExecutionTreeResponse {
  durationMs?: number | null;
  /**
   * Initial active states when the workflow started.
   */
  initialStateIds: string[];
  rootNodes: DisplayNode[];
  /**
   * Run UUID as string.
   */
  runId: string;
  /**
   * Mapping of state IDs to display names.
   */
  stateNameMap: {
    [k: string]: string;
  };
  status: NodeStatus;
  totalEvents: number;
  workflowName?: string | null;
  [k: string]: unknown;
}
