/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */

/**
 * What caused an edge — structurally only.
 *
 * `deny_unknown_fields` and the deliberate absence of any text/value field
 * are the privacy control: a caller cannot smuggle a typed value into the
 * ledger, because no such key deserializes and no such field exists to set.
 */
export interface JourneyTrigger {
  /**
   * The action verb (`click`, `type`, `select`, a component action id, …).
   */
  actionType: string;
  /**
   * Which runner choke point captured it.
   */
  chokePoint:
    | "element_action"
    | "batch_action"
    | "component_action"
    | "sdk_element_action"
    | "execute_with_diff"
    | "navigation";
  /**
   * The affordance's declared effect; `None` = undeclared (NOT "safe").
   */
  declaredEffect?: ("read" | "write" | "destructive") | null;
  /**
   * How the transition was initiated.
   */
  navigationTrigger:
    "affordance" | "push" | "replace" | "pop" | "initial" | "hash";
  /**
   * Structural fingerprint of the target element; `None` = not resolved.
   */
  targetFingerprint?: string | null;
  /**
   * ARIA role of the target; `None` = not reported.
   */
  targetRole?: string | null;
}
