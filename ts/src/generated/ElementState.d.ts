/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with `just generate-types` or
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */

import type { ElementRect } from './ElementRect';

/**
 * Observable state of a UI Bridge element as returned from the React
 * registry.
 *
 * Every element returned by the bridge includes a snapshot of its current
 * visibility, interactivity, and form-control value.
 */
export interface ElementState {
  /**
   * Current checked state for checkbox/radio elements.
   */
  checked?: boolean | null;
  /**
   * Whether the element is enabled (not disabled).
   */
  enabled: boolean;
  /**
   * Whether the element currently has keyboard focus.
   */
  focused: boolean;
  rect: ElementRect;
  /**
   * Currently selected options for `<select>` elements.
   */
  selectedOptions?: string[] | null;
  /**
   * Text content of the element (innerText).
   */
  textContent?: string | null;
  /**
   * Current value for input/textarea elements.
   */
  value?: string | null;
  /**
   * Whether the element is currently visible in the viewport.
   */
  visible: boolean;
  [k: string]: unknown;
}
