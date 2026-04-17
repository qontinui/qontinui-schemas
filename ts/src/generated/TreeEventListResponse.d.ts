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
import type { TreeEventResponse } from './TreeEventResponse';
import type { TreeEventType } from './TreeEventType';

/**
 * Paginated list of tree events.
 */
export interface TreeEventListResponse {
  events: TreeEventResponse[];
  hasMore: boolean;
  limit: number;
  offset: number;
  total: number;
  [k: string]: unknown;
}
