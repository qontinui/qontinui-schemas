/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with `just generate-types` or
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */

import type { ElementRect } from "./ElementRect";

/**
 * Observable state of a UI Bridge element as returned from the React
 * registry.
 *
 * Every element returned by the bridge includes a snapshot of its current
 * visibility, interactivity, and form-control value.
 */
export interface ElementState {
  /**
   * Whether the element carries `aria-disabled="true"`.
   *
   * See `disabled` for why this is tracked separately, and for the
   * absent-reads-as-`false` caveat that applies identically here.
   */
  ariaDisabled: boolean;
  /**
   * Current checked state for checkbox/radio elements.
   */
  checked?: boolean | null;
  /**
   * Whether the element carries the native `disabled` attribute/property.
   *
   * Distinguished from `aria_disabled` because the two differ in effect:
   * a natively disabled control cannot receive events at all, whereas an
   * `aria-disabled` one is still focusable and clickable and merely
   * announces itself as disabled.
   *
   * `#[serde(default)]`: absent in snapshots from SDK builds predating
   * the split, where `enabled` is the only trustworthy signal. Absent
   * therefore reads as `false`, which is NOT the same as "observed
   * enabled": cross-check `enabled` before trusting it.
   */
  disabled: boolean;
  /**
   * Whether the element is enabled: the derived fold
   * `!disabled && !ariaDisabled`.
   *
   * Kept as the single-signal convenience view of the two fields below.
   * It is the ONLY interactivity signal an SDK build predating the
   * `disabled`/`ariaDisabled` split emits, so it stays required.
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
}
