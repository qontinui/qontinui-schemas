/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with `just generate-types` or
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */

/**
 * Accessibility state flags for a node.
 *
 * These flags represent the current interactive state of an element.
 * Tri-state booleans use `Option<bool>` — `None` means "not applicable."
 */
export interface AccessibilityState {
  /**
   * Checkable element's checked state.
   */
  is_checked?: boolean | null;
  /**
   * Element is disabled / non-interactive.
   */
  is_disabled: boolean;
  /**
   * Element content can be edited.
   */
  is_editable: boolean;
  /**
   * Expandable element's expansion state.
   */
  is_expanded?: boolean | null;
  /**
   * Element can receive focus.
   */
  is_focusable: boolean;
  /**
   * Element has keyboard focus.
   */
  is_focused: boolean;
  /**
   * Element is hidden from the accessibility tree.
   */
  is_hidden: boolean;
  /**
   * Element is a modal dialog.
   */
  is_modal: boolean;
  /**
   * Element allows multiple selections.
   */
  is_multiselectable: boolean;
  /**
   * Pressable element's pressed state.
   */
  is_pressed?: boolean | null;
  /**
   * Element is read-only.
   */
  is_readonly: boolean;
  /**
   * Element value is required.
   */
  is_required: boolean;
  /**
   * Selectable element's selection state.
   */
  is_selected?: boolean | null;
  [k: string]: unknown;
}
