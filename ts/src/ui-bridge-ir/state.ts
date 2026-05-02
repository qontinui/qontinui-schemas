/**
 * IR-level state declaration.
 *
 * Authoring-time superset of ui-bridge-auto's runtime State shape. The
 * IR -> WorkflowConfig adapter strips IR-only fields (provenance, metadata,
 * elementIds, incomingTransitions, visualRefs) before handing to the runtime.
 */

import type { IRElementCriteria } from "./element-criteria";
import type { IRCrossRef, IRMetadata, IRProvenance, IRVisualRef } from "./primitives";

/**
 * Property-level condition on a matched element. Mirrors
 * `StateCondition` in ui-bridge-auto/src/types/state.ts:83.
 */
export interface IRStateCondition {
  element: IRElementCriteria;
  property:
    | "visible"
    | "enabled"
    | "checked"
    | "expanded"
    | "selected"
    | "text"
    | "value";
  expected: unknown;
  comparator?: "equals" | "contains" | "matches" | "greaterThan" | "lessThan";
}

/**
 * IR state declaration.
 *
 * Anchored to ui-bridge-auto's `State` shape (ui-bridge-auto/src/types/state.ts:23)
 * — extends it with IR-only fields. Runtime tracking fields (enteredAt,
 * exitedAt, observationCount) are deliberately absent: those are managed by
 * the state machine at execution time.
 */
export interface IRState {
  // Identity --------------------------------------------------------------
  id: string;
  name: string;
  description?: string;

  // Predicates (canonical) -----------------------------------------------
  /** Element criteria that must ALL be satisfied for this state to be active. */
  requiredElements: IRElementCriteria[];
  /** Element criteria where NONE may be satisfied (any match disqualifies). */
  excludedElements?: IRElementCriteria[];
  /** Additional property checks on matched elements. */
  conditions?: IRStateCondition[];

  // Lifecycle / pathfinding hints ----------------------------------------
  /** Whether this is a valid initial/starting state. */
  isInitial?: boolean;
  /** Whether this is a terminal state (no outgoing transitions). */
  isTerminal?: boolean;
  /** Whether this state is modal/blocking (e.g., a dialog). */
  blocking?: boolean;
  /** Logical group this state belongs to. */
  group?: string;
  /** Navigation cost weight for pathfinding (default 1.0). */
  pathCost?: number;
  /**
   * Free-text precondition required for this state to be entered, mirroring
   * legacy `assertion.precondition` strings (e.g., "A workflow is paused at a
   * breakpoint"). Authoring-time documentation; not enforced at runtime.
   */
  precondition?: string;

  // IR-only fields (stripped by adapter) ---------------------------------
  /**
   * Optional companion list of element IDs that resolved at registration
   * time. The runtime SDK fills this in when criteria resolve to specific
   * registered elements. Authors do not set this directly.
   */
  elementIds?: string[];
  /**
   * Derived/computed at IR-emission time — list of transitions that activate
   * this state. Authoring-time hint for visualization; not authored directly.
   */
  incomingTransitions?: string[];
  /** Semantic metadata routed through useUIAnnotation store at runtime. */
  metadata?: IRMetadata;
  /** Where this declaration came from. */
  provenance?: IRProvenance;
  /** Cross-document references. */
  crossRefs?: IRCrossRef[];
  /** Optional visual reference (section 8). */
  visualRefs?: IRVisualRef[];
}
