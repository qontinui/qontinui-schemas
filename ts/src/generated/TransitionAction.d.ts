/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with `just generate-types` or
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */

import type { MouseButton } from './MouseButton';
import type { Point } from './Point';
import type { ScrollDirection } from './ScrollDirection';
import type { StandardActionType } from './StandardActionType';
import type { TransitionActionValue } from './TransitionActionValue';

/**
 * A single action within a transition.
 *
 * Each action targets an element and performs an interaction. Most fields are
 * only meaningful for specific action types — see the per-field docs.
 */
export interface TransitionAction {
  /**
   * Mouse button: `left`, `right`, `middle`
   * (`click` / `doubleClick` / `rightClick`).
   */
  button?: MouseButton | null;
  /**
   * Clear before typing (`type` action).
   */
  clear_first?: boolean | null;
  /**
   * Delay in milliseconds (`wait` action).
   */
  delay_ms?: number | null;
  /**
   * Hold delay before the first move in milliseconds (`drag` action).
   */
  drag_hold_delay?: number | null;
  /**
   * Dispatch HTML5 drag events alongside mouse events (`drag` action).
   */
  drag_html5?: boolean | null;
  /**
   * Number of intermediate mousemove steps (`drag` action).
   */
  drag_steps?: number | null;
  /**
   * Drag target element ID or selector (`drag` action).
   */
  drag_target?: string | null;
  /**
   * Drag target position (e.g., `{ x, y }` stringified, or a named
   * position) (`drag` action).
   */
  drag_target_position?: string | null;
  /**
   * Click position relative to the element
   * (`click` / `doubleClick` / `rightClick`).
   */
  position?: Point | null;
  /**
   * Scroll amount in pixels (`scroll` action).
   */
  scroll_amount?: number | null;
  /**
   * Scroll direction (`scroll` action).
   */
  scroll_direction?: ScrollDirection | null;
  /**
   * Select by label instead of value (`select` action).
   */
  select_by_label?: boolean | null;
  /**
   * Target element ID (used by most element actions).
   */
  target?: string | null;
  /**
   * Text to type (`type` action).
   */
  text?: string | null;
  type: StandardActionType;
  /**
   * Keystroke delay in milliseconds (`type` action).
   */
  type_delay?: number | null;
  /**
   * URL to navigate to (`navigate` action).
   */
  url?: string | null;
  /**
   * Value to select or set (`select` / `setValue` actions).
   */
  value?: TransitionActionValue | null;
  [k: string]: unknown;
}
