/**
 * IR -> WorkflowConfig adapter.
 *
 * Single point of translation between authoring-time IR (with provenance,
 * metadata, effect, crossRefs, visualRefs) and the runtime/storage shape
 * that ui-bridge-auto consumes. See ADR-001 (decision #6 / #7) for the
 * boundary rationale.
 *
 * The output is intentionally typed structurally — this module has no
 * runtime dependency on ui-bridge-auto. ui-bridge-auto's own tests verify
 * end-to-end compatibility with `engine.defineStates(...)` /
 * `engine.defineTransitions(...)` (see
 * `ui-bridge-auto/src/__tests__/spike-ir-contract.test.ts`).
 */

import type { IRElementCriteria } from "./element-criteria";
import type { IRDocument } from "./document";
import type { IRState, IRStateCondition } from "./state";
import type { IRTransition, IRTransitionAction, IRWaitSpec } from "./transition";

// ---------------------------------------------------------------------------
// Output shapes — structural mirrors of ui-bridge-auto's WorkflowConfig.
// Kept in sync via ui-bridge-auto's regression test, NOT compile-time
// imports (qontinui-schemas has no runtime deps by design).
// ---------------------------------------------------------------------------

/** Mirrors `StateConfig` from ui-bridge-auto/src/config/workflow.ts:47. */
export interface AdaptedState {
  id: string;
  name: string;
  description?: string;
  requiredElements: IRElementCriteria[];
  excludedElements?: IRElementCriteria[];
  conditions?: IRStateCondition[];
  isInitial?: boolean;
  isTerminal?: boolean;
  blocking?: boolean;
  group?: string;
  pathCost?: number;
}

/**
 * Mirrors the runtime `TransitionDefinition` shape from
 * ui-bridge-auto/src/state/state-machine.ts:40.
 *
 * Note: ui-bridge-auto has two related Transition shapes — the storage shape
 * (`TransitionConfig` in config/workflow.ts:54, with action field named
 * `type`) and the runtime engine shape (`TransitionDefinition` in
 * state-machine.ts:40, with action field named `action`). The runtime engine
 * is what consumers actually pass adapter output into via
 * `engine.defineTransitions(...)`, so the adapter targets the runtime shape.
 */
export interface AdaptedTransition {
  id: string;
  name: string;
  description?: string;
  fromStates: string[];
  activateStates: string[];
  /** Adapter normalizes IR's optional `exitStates` to a required array. */
  exitStates: string[];
  actions: AdaptedTransitionAction[];
  pathCost?: number;
  bidirectional?: boolean;
}

/**
 * Mirrors the runtime `TransitionAction` from
 * ui-bridge-auto/src/state/state-machine.ts:55.
 *
 * The IR uses `type` for the action verb (matching the storage-shape
 * convention); the adapter renames it to `action` for the runtime engine.
 *
 * `waitAfter` is narrower than `IRWaitSpec` — the runtime engine accepts
 * only a subset of wait types and a subset of fields. The adapter drops
 * IR-only wait variants (e.g., `state`, `condition`) and IR-only fields
 * (e.g., `property`, `quietPeriodMs`). Authors can still express richer
 * waits in the IR; they just don't survive the runtime contract.
 */
export interface AdaptedWaitAfter {
  type: "idle" | "element" | "time" | "vanish" | "change" | "stable";
  query?: IRElementCriteria;
  ms?: number;
  timeout?: number;
}

export interface AdaptedTransitionAction {
  /** Action verb — renamed from IR's `type` to match the runtime contract. */
  action: string;
  target: IRElementCriteria;
  params?: Record<string, unknown>;
  waitAfter?: AdaptedWaitAfter;
}

/** Mirrors `WorkflowConfig` from ui-bridge-auto/src/config/workflow.ts:67. */
export interface AdaptedWorkflowConfig {
  id: string;
  name: string;
  description?: string;
  version?: string;
  states: AdaptedState[];
  transitions: AdaptedTransition[];
  initialState?: string;
}

// ---------------------------------------------------------------------------
// Adapter implementation
// ---------------------------------------------------------------------------

const SUPPORTED_IR_VERSIONS = new Set<string>(["1.0"]);

/**
 * Convert an IR state declaration into the runtime/storage state shape.
 * Strips IR-only fields (metadata, provenance, elementIds,
 * incomingTransitions, crossRefs, visualRefs).
 */
export function adaptIRState(state: IRState): AdaptedState {
  const out: AdaptedState = {
    id: state.id,
    name: state.name,
    requiredElements: state.requiredElements,
  };
  if (state.description !== undefined) out.description = state.description;
  if (state.excludedElements !== undefined) out.excludedElements = state.excludedElements;
  if (state.conditions !== undefined) out.conditions = state.conditions;
  if (state.isInitial !== undefined) out.isInitial = state.isInitial;
  if (state.isTerminal !== undefined) out.isTerminal = state.isTerminal;
  if (state.blocking !== undefined) out.blocking = state.blocking;
  if (state.group !== undefined) out.group = state.group;
  if (state.pathCost !== undefined) out.pathCost = state.pathCost;
  return out;
}

/**
 * Wait types the runtime engine accepts. IR-authored waits with other types
 * (e.g., `state`, `condition`) are dropped from the adapter output.
 */
const RUNTIME_WAIT_TYPES = new Set<AdaptedWaitAfter["type"]>([
  "idle",
  "element",
  "time",
  "vanish",
  "change",
  "stable",
]);

function adaptIRWaitAfter(wait: IRWaitSpec): AdaptedWaitAfter | undefined {
  if (!RUNTIME_WAIT_TYPES.has(wait.type as AdaptedWaitAfter["type"])) {
    // IR-authored wait is richer than runtime supports — drop it.
    // Runtime defaults (default idle behaviour) take over.
    return undefined;
  }
  const out: AdaptedWaitAfter = { type: wait.type as AdaptedWaitAfter["type"] };
  if (wait.query !== undefined) out.query = wait.query;
  if (wait.ms !== undefined) out.ms = wait.ms;
  if (wait.timeout !== undefined) out.timeout = wait.timeout;
  return out;
}

/**
 * Convert an IR transition action into the runtime action shape.
 * Renames IR's `type` to runtime's `action`; narrows `waitAfter` to the
 * runtime-accepted subset.
 */
export function adaptIRTransitionAction(action: IRTransitionAction): AdaptedTransitionAction {
  const out: AdaptedTransitionAction = {
    action: action.type,
    target: action.target,
  };
  if (action.params !== undefined) out.params = action.params;
  if (action.waitAfter !== undefined) {
    const adaptedWait = adaptIRWaitAfter(action.waitAfter);
    if (adaptedWait !== undefined) out.waitAfter = adaptedWait;
  }
  return out;
}

/**
 * Convert an IR transition declaration into the runtime/storage transition
 * shape. Strips IR-only fields (effect, metadata, provenance, crossRefs)
 * and defaults `exitStates` to `[]` when omitted.
 */
export function adaptIRTransition(transition: IRTransition): AdaptedTransition {
  const out: AdaptedTransition = {
    id: transition.id,
    name: transition.name,
    fromStates: transition.fromStates,
    activateStates: transition.activateStates,
    exitStates: transition.exitStates ?? [],
    actions: transition.actions.map(adaptIRTransitionAction),
  };
  if (transition.description !== undefined) out.description = transition.description;
  if (transition.pathCost !== undefined) out.pathCost = transition.pathCost;
  if (transition.bidirectional !== undefined) out.bidirectional = transition.bidirectional;
  return out;
}

/**
 * Convert a top-level IR document into the runtime `WorkflowConfig` shape.
 *
 * Throws on unknown IR versions to fail loudly rather than silently emit
 * stale shapes.
 */
export function adaptIRDocumentToWorkflowConfig(doc: IRDocument): AdaptedWorkflowConfig {
  if (!SUPPORTED_IR_VERSIONS.has(doc.version)) {
    throw new Error(
      `adaptIRDocumentToWorkflowConfig: unsupported IR version "${doc.version}". ` +
        `Supported: ${Array.from(SUPPORTED_IR_VERSIONS).join(", ")}`,
    );
  }

  const out: AdaptedWorkflowConfig = {
    id: doc.id,
    name: doc.name,
    states: doc.states.map(adaptIRState),
    transitions: doc.transitions.map(adaptIRTransition),
  };
  if (doc.description !== undefined) out.description = doc.description;
  if (doc.initialState !== undefined) out.initialState = doc.initialState;
  return out;
}
