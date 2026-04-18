/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with `just generate-types` or
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */

import type { ComponentActionInfo } from './ComponentActionInfo';

/**
 * A registered component in the UI Bridge registry.
 *
 * Components group related elements and expose higher-level actions
 * (e.g. "submit form", "reset filters").
 */
export interface UIBridgeComponent {
  /**
   * Actions exposed by this component.
   */
  actions?: ComponentActionInfo[];
  /**
   * Human-readable description.
   */
  description?: string | null;
  /**
   * IDs of elements that belong to this component.
   */
  elementIds?: string[] | null;
  /**
   * Unique component ID within the registry.
   */
  id: string;
  /**
   * Whether the component's React component is currently mounted.
   */
  mounted: boolean;
  /**
   * Component name.
   */
  name: string;
  /**
   * Unix-epoch millisecond timestamp when the component was registered.
   */
  registeredAt: number;
}
