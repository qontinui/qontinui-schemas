/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */

import type { JourneyNode } from "./JourneyNode";

/**
 * An interactive affordance seen at a node and not yet the trigger of an
 * edge from it — a row of `project.journey_frontier`, keyed
 * `(app_id, node_key, affordance_fingerprint)`.
 */
export interface FrontierEntry {
  /**
   * Structural fingerprint of the affordance.
   */
  affordanceFingerprint: string;
  /**
   * ARIA role of the affordance; `None` = not reported.
   */
  affordanceRole?: string | null;
  /**
   * The registered app id.
   */
  appId: string;
  /**
   * The affordance's declared effect; `None` = undeclared.
   */
  declaredEffect?: ("read" | "write" | "destructive") | null;
  /**
   * First time it was seen at this node (ISO 8601).
   */
  firstSeenAt: string;
  /**
   * Most recent time it was seen at this node (ISO 8601).
   */
  lastSeenAt: string;
  /**
   * The run that most recently saw it; `None` = not reported.
   */
  lastSeenRunId?: string | null;
  /**
   * The node the affordance was seen at.
   */
  node: JourneyNode;
  /**
   * `node.key()` — stored so the primary key is a plain text column.
   */
  nodeKey: string;
  /**
   * Why it is on the frontier.
   */
  reason:
    | "not_yet_activated"
    | "effect_undeclared"
    | "effect_write"
    | "effect_destructive"
    | "budget_exhausted"
    | "activation_failed";
}
