/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */

/**
 * Target of a `component_action` step: which registered component, which of
 * its actions, and the action's parameters.
 *
 * Carried JSON-encoded in [`UiBridgeStep::target`] (the runner's Builder
 * writes `JSON.stringify({componentId, actionId, params})`); decode it with
 * [`UiBridgeStep::component_action_target`].
 */
export interface UiBridgeComponentActionTarget {
  /**
   * Action ID on that component.
   */
  actionId: string;
  /**
   * Registered component ID.
   */
  componentId: string;
  /**
   * Action parameters (the action's `paramSchema` shape).
   */
  params?: {
    [k: string]: unknown;
  };
  [k: string]: unknown;
}
