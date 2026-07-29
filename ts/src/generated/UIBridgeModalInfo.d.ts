/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with `just generate-types` or
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */

/**
 * Modal/dialog entry in the active modal stack.
 */
export interface UIBridgeModalInfo {
  /**
   * Whether this modal blocks interaction with content behind it.
   */
  blocking: boolean;
  /**
   * Timestamp when the modal was detected (epoch ms).
   */
  detectedAt: number;
  /**
   * Whether the modal is dismissible (RN-specific). Optional so the
   * web shape (which doesn't carry this bit) round-trips cleanly.
   */
  dismissible?: boolean | null;
  /**
   * Web-only: whether a backdrop/overlay is present.
   */
  hasBackdrop?: boolean | null;
  id: string;
  title?: string | null;
  /**
   * Modal kind. Web: dialog/alertdialog/modal/drawer/popover/sheet.
   * Native (RN): modal/sheet/drawer/popover/alertdialog/dialog.
   */
  type?: string | null;
  /**
   * Web-only: computed z-index of the modal.
   */
  zIndex?: number | null;
}
