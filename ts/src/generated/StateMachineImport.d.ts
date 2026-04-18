/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with `just generate-types` or
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */

import type { DiscoveredState } from './DiscoveredState';
import type { DiscoveredStateImage } from './DiscoveredStateImage';
import type { DiscoveredTransition } from './DiscoveredTransition';
import type { DiscoveryBoundingBox } from './DiscoveryBoundingBox';
import type { DiscoveryTransitionTrigger } from './DiscoveryTransitionTrigger';
import type { StateMachineExport } from './StateMachineExport';
import type { TransitionTriggerType } from './TransitionTriggerType';

/**
 * Request payload to import a state machine.
 */
export interface StateMachineImport {
  /**
   * Override name (uses export name when omitted).
   */
  name?: string | null;
  stateMachine: StateMachineExport;
}
