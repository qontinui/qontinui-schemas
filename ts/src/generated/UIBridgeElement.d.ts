/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with `just generate-types` or
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */

import type { ElementIdentifier } from './ElementIdentifier';
import type { ElementRect } from './ElementRect';
import type { ElementState } from './ElementState';

/**
 * A registered element in the UI Bridge registry.
 *
 * This is the serializable subset of the React `RegisteredElement`; it
 * includes identity, available actions, current state, and lifecycle info.
 */
export interface UIBridgeElement {
  /**
   * Standard actions available on this element.
   */
  actions?: string[];
  /**
   * Custom (application-defined) actions.
   */
  customActions?: string[] | null;
  /**
   * Unique element ID within the registry.
   */
  id: string;
  identifier: ElementIdentifier;
  /**
   * Human-readable label for the element.
   */
  label?: string | null;
  /**
   * Whether the element's React component is currently mounted.
   */
  mounted: boolean;
  /**
   * Unix-epoch millisecond timestamp when the element was registered.
   */
  registeredAt: number;
  state: ElementState;
  /**
   * Element type (e.g. `"button"`, `"input"`, `"select"`).
   */
  type: string;
}
