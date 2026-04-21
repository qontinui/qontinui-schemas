/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with `just generate-types` or
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */

import type { ComponentActionInfo } from './ComponentActionInfo';
import type { ElementBbox } from './ElementBbox';
import type { ElementIdentifier } from './ElementIdentifier';
import type { ElementRect } from './ElementRect';
import type { ElementState } from './ElementState';
import type { UIBridgeComponent } from './UIBridgeComponent';
import type { UIBridgeElement } from './UIBridgeElement';
import type { WorkflowInfo } from './WorkflowInfo';

/**
 * Full snapshot of the UI Bridge state.
 *
 * Captures all registered elements, components, and active workflows
 * at a single point in time.
 */
export interface UIBridgeSnapshot {
  /**
   * All registered components.
   */
  components?: UIBridgeComponent[];
  /**
   * All registered elements.
   */
  elements?: UIBridgeElement[];
  /**
   * Unix-epoch millisecond timestamp of the snapshot.
   */
  timestamp: number;
  /**
   * Active workflows.
   */
  workflows?: WorkflowInfo[];
}
