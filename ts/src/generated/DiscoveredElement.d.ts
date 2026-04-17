/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with `just generate-types` or
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */

import type { ElementRect } from './ElementRect';
import type { ElementState } from './ElementState';

/**
 * An element found during a discovery scan.
 */
export interface DiscoveredElement {
  /**
   * Computed accessible name.
   */
  accessibleName?: string | null;
  /**
   * Available actions for this element.
   */
  actions?: string[];
  /**
   * Unique element ID.
   */
  id: string;
  /**
   * Human-readable label.
   */
  label?: string | null;
  /**
   * Whether the element is already registered in the bridge registry.
   */
  registered: boolean;
  /**
   * ARIA role attribute.
   */
  role?: string | null;
  state: ElementState;
  /**
   * HTML tag name (e.g. `"BUTTON"`, `"INPUT"`).
   */
  tagName: string;
  /**
   * Element type (e.g. `"button"`, `"input"`).
   */
  type: string;
  [k: string]: unknown;
}
