/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with `just generate-types` or
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */

/**
 * Data passed to a state node in the ReactFlow graph editor.
 *
 * Mirrors the TypeScript shape with camelCase field names. The
 * `onStartElementDrag` callback from the TS type is a UI-layer concern and is
 * intentionally omitted from the DTO.
 */
export interface StateNodeData {
  /**
   * Discovery confidence.
   */
  confidence: number;
  /**
   * Optional description (may be `null`).
   */
  description?: string | null;
  /**
   * Number of elements defining this state.
   */
  elementCount: number;
  /**
   * Element IDs defining this state.
   */
  elementIds: string[];
  /**
   * Optional map of element ID → base64 thumbnail image (data URL or raw
   * base64 PNG).
   */
  elementThumbnails?: {
    [k: string]: string;
  } | null;
  /**
   * Incoming transition count.
   */
  incomingCount?: number | null;
  /**
   * Whether the state blocks navigation.
   */
  isBlocking: boolean;
  /**
   * Whether this node is the current drop target (drag-and-drop).
   */
  isDropTarget?: boolean | null;
  /**
   * Whether this is an initial state.
   */
  isInitial: boolean;
  /**
   * Whether the node is currently selected in the editor.
   */
  isSelected: boolean;
  /**
   * Display name.
   */
  name: string;
  /**
   * Outgoing transition count.
   */
  outgoingCount?: number | null;
  /**
   * State ID this node represents.
   */
  stateId: string;
}
