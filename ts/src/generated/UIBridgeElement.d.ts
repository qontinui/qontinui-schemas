/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with `just generate-types` or
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */

import type { ElementBbox } from './ElementBbox';
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
   * W3C "accessible name" per the accessible-name algorithm. Distinct from
   * aria_label because the algorithm may consult aria-labelledby, associated
   * label elements, title, or visible content. Source of truth for
   * IrElementCriteria.accessible_name.
   */
  accessibleName?: string | null;
  /**
   * Standard actions available on this element.
   */
  actions?: string[];
  /**
   * Computed aria-label (explicit attribute, falling back to aria-labelledby
   * reference resolution). Source of truth for IrElementCriteria.aria_label.
   */
  ariaLabel?: string | null;
  /**
   * Viewport-relative bounding box in CSS pixels, when the SDK has a
   * live DOM ref. Absent for elements registered without a ref or when
   * the snapshot is served from the DOM-fallback scanner.
   */
  bbox?: ElementBbox | null;
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
  /**
   * ARIA role of the element (explicit `role=` or implicit per W3C ARIA-in-HTML).
   * Populated by the SDK's element walker. Source of truth for IrElementCriteria.role.
   */
  role?: string | null;
  state: ElementState;
  /**
   * HTML tag name in lowercase. Source of truth for IrElementCriteria.tag_name.
   */
  tagName?: string | null;
  /**
   * Visible text content with whitespace collapsed (DOM innerText-equivalent
   * on web; accessibilityLabel/text equivalent on native). Source of truth for
   * IrElementCriteria.text and text_contains. Distinct from state.text_content
   * which is a snapshot of the form-control value.
   */
  text?: string | null;
  /**
   * Element type (e.g. `"button"`, `"input"`, `"select"`).
   */
  type: string;
  /**
   * Cheap viewport-visibility signal derived by the SDK as
   * `bbox.width > 0 && bbox.height > 0`. Use `state.visible` for the
   * richer occlusion check.
   */
  visible?: boolean | null;
}
