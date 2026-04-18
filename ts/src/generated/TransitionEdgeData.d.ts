/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with `just generate-types` or
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */

import type { StandardActionType } from './StandardActionType';

/**
 * Data passed to a transition edge in the ReactFlow graph editor.
 */
export interface TransitionEdgeData {
  /**
   * Number of actions in this transition.
   */
  actionCount: number;
  /**
   * Distinct action types used.
   */
  actionTypes: StandardActionType[];
  /**
   * Target of the first action, for label rendering.
   */
  firstActionTarget?: string | null;
  /**
   * Whether the edge is highlighted (e.g., part of a path preview).
   */
  isHighlighted: boolean;
  /**
   * Display name.
   */
  name: string;
  /**
   * Path cost.
   */
  pathCost: number;
  /**
   * Whether source states stay visible after activation.
   */
  staysVisible: boolean;
  /**
   * Transition ID this edge represents.
   */
  transitionId: string;
}
