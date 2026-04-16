/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with `just generate-types` or
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */

import type { TransitionTriggerType } from './TransitionTriggerType';

/**
 * Trigger for a discovered state transition.
 *
 * Describes the action (click, type, …) that caused a transition. All
 * identifying fields are optional — different discovery sources populate
 * different subsets.
 */
export interface DiscoveryTransitionTrigger {
  /**
   * ID of the DOM element (for web extraction).
   */
  elementId?: string | null;
  /**
   * ID of the image that was clicked/interacted with.
   */
  imageId?: string | null;
  /**
   * CSS selector for the trigger element.
   */
  selector?: string | null;
  /**
   * Type of trigger action. Defaults to `click` when omitted on the wire.
   */
  type?: TransitionTriggerType & string;
  /**
   * Value for type actions (text input).
   */
  value?: string | null;
  [k: string]: unknown;
}
