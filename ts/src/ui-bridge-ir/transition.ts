/**
 * IR-level transition declaration.
 *
 * Anchored to ui-bridge-auto's `Transition` shape
 * (ui-bridge-auto/src/types/transition.ts:103) and extended with IR-only
 * fields (effect, provenance, crossRefs).
 *
 * `exitStates` is a HINT at the IR level — when omitted, the adapter populates
 * `[]` so the runtime contract (which requires the field) is satisfied.
 */

import type { IRElementCriteria } from "./element-criteria";
import type { IRCrossRef, IREffect, IRMetadata, IRProvenance } from "./primitives";

/**
 * Wait specification for the runtime to settle after an action.
 * Mirrors `WaitSpec` in ui-bridge-auto/src/types/transition.ts:56.
 */
export interface IRWaitSpec {
  type: "idle" | "element" | "state" | "time" | "condition" | "vanish" | "change" | "stable";
  query?: IRElementCriteria;
  stateId?: string;
  ms?: number;
  timeout?: number;
  property?: string;
  quietPeriodMs?: number;
}

/**
 * A single action step within a transition. Mirrors `TransitionAction` in
 * ui-bridge-auto/src/types/transition.ts:81 with the difference that `type`
 * is a free-form string here — the IR doesn't bind authors to a specific
 * action vocabulary; that constraint applies at the runtime adapter layer.
 */
export interface IRTransitionAction {
  /** The action to perform (e.g., "click", "type", "submit"). */
  type: string;
  /** Criteria to find the target element. */
  target: IRElementCriteria;
  /** Action-specific parameters. */
  params?: Record<string, unknown>;
  /** Optional wait specification applied after this action completes. */
  waitAfter?: IRWaitSpec;
}

/**
 * IR transition declaration.
 *
 * Authoring-time superset of ui-bridge-auto's `Transition`. Reliability
 * tracking fields (successCount, failureCount, averageDurationMs,
 * lastExecutedAt) are deliberately absent — the runtime owns them.
 */
export interface IRTransition {
  // Identity --------------------------------------------------------------
  id: string;
  name: string;
  description?: string;

  // Pre/post conditions ---------------------------------------------------
  /** Precondition: all of these states must be active. */
  fromStates: string[];
  /** States to enter after the transition completes. */
  activateStates: string[];
  /**
   * States to leave after the transition completes.
   *
   * IR-level HINT: omit it and the adapter defaults to `[]`. The runtime
   * type (`Transition.exitStates: string[]` in
   * ui-bridge-auto/src/types/transition.ts:116) requires the field; the
   * adapter normalizes.
   */
  exitStates?: string[];

  // Action sequence -------------------------------------------------------
  actions: IRTransitionAction[];

  // Pathfinding hints -----------------------------------------------------
  /** Navigation cost for pathfinding (default 1.0). */
  pathCost?: number;
  /** Whether this transition can be reversed. */
  bidirectional?: boolean;

  // IR-only fields (stripped by adapter) ----------------------------------
  /** Side-effect annotation. Drives counterfactual + auto-regression gating. */
  effect?: IREffect;
  /** Semantic metadata routed through useUIAnnotation store at runtime. */
  metadata?: IRMetadata;
  /** Where this declaration came from. */
  provenance?: IRProvenance;
  /** Cross-document references. */
  crossRefs?: IRCrossRef[];
}
