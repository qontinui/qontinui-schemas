/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with `just generate-types` or
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */

/**
 * Undo/redo availability snapshot context.
 */
export interface UIBridgeUndoContext {
  canRedo: boolean;
  canUndo: boolean;
  redoDepth?: number | null;
  redoDescription?: string | null;
  summary: string;
  undoDepth?: number | null;
  undoDescription?: string | null;
}
