/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with `just generate-types` or
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */

import type { DomainKnowledge } from './DomainKnowledge';
import type { MouseButton } from './MouseButton';
import type { Point } from './Point';
import type { ScrollDirection } from './ScrollDirection';
import type { StandardActionType } from './StandardActionType';
import type { StateMachineState } from './StateMachineState';
import type { StateMachineTransition } from './StateMachineTransition';
import type { TransitionAction } from './TransitionAction';
import type { TransitionActionValue } from './TransitionActionValue';

/**
 * A config with all its states and transitions loaded.
 *
 * Used when the full state machine needs to be displayed or exported. This
 * mirrors the TypeScript `StateMachineConfigFull extends StateMachineConfig`
 * by flattening the base config's fields via `#[serde(flatten)]`.
 */
export interface StateMachineConfigFull {
  /**
   * ISO 8601 timestamp of creation.
   */
  created_at: string;
  /**
   * Optional description.
   */
  description?: string | null;
  /**
   * Number of unique elements discovered in this config.
   */
  element_count: number;
  /**
   * Unique identifier (UUID).
   */
  id: string;
  /**
   * Whether to include HTML IDs when generating selectors.
   */
  include_html_ids: boolean;
  /**
   * Display name.
   */
  name: string;
  /**
   * Number of DOM renders collected for this config.
   */
  render_count: number;
  /**
   * All states belonging to this config.
   */
  states: StateMachineState[];
  /**
   * All transitions belonging to this config.
   */
  transitions: StateMachineTransition[];
  /**
   * ISO 8601 timestamp of last modification.
   */
  updated_at: string;
  [k: string]: unknown;
}
