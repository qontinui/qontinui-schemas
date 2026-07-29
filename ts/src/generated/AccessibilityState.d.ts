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
  isChecked?: boolean | null;
  /**
   * Element is disabled / non-interactive.
   */
  isDisabled: boolean;
  /**
   * Element content can be edited.
   */
  isEditable: boolean;
  /**
   * Expandable element's expansion state.
   */
  isExpanded?: boolean | null;
  /**
   * Element can receive focus.
   */
  isFocusable: boolean;
  /**
   * Element has keyboard focus.
   */
  isFocused: boolean;
  /**
   * Element is hidden from the accessibility tree.
   */
  isHidden: boolean;
  /**
   * Element is a modal dialog.
   */
  isModal: boolean;
  /**
   * Element allows multiple selections.
   */
  isMultiselectable: boolean;
  /**
   * Pressable element's pressed state.
   */
  isPressed?: boolean | null;
  /**
   * Element is read-only.
   */
  isReadonly: boolean;
  /**
   * Element value is required.
   */
  isRequired: boolean;
  /**
   * Selectable element's selection state.
   */
  isSelected?: boolean | null;
}
