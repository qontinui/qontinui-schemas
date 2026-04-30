/**
 * IR -> Bundled-Page Legacy Spec projection.
 *
 * Single point of translation between authoring-time IR (`IRDocument`) and the
 * legacy `*.spec.uibridge.json` shape that existing tooling still consumes
 * (`/update-spec`, runner spec drift / verify, error monitor curator,
 * spec experimentation, AI session). See ADR-001 (decision #6 / #7) for the
 * boundary rationale; section 2 of the UI Bridge redesign for the projection's
 * role in keeping legacy consumers working through the migration.
 *
 * The projection is a PURE FUNCTION — same input always produces byte-identical
 * output. Keys within objects are sorted lexicographically, arrays preserve
 * input order, and no timestamps / random IDs are introduced. Downstream tests
 * may hash the JSON output for byte-stable assertions.
 *
 * ---------------------------------------------------------------------------
 * Mapping rules (worked example below)
 * ---------------------------------------------------------------------------
 *
 *   IRDocument                                   LegacySpec
 *   ----------                                   ----------
 *   doc.id                                  ->   metadata.component
 *   doc.metadata?.purpose ?? doc.id         ->   metadata.component (purpose wins if present)
 *   doc.description ?? doc.name             ->   description
 *   doc.metadata?.tags                      ->   metadata.tags
 *
 *   For each IRState:
 *     state                                 ->   one entry in groups[]
 *       state.id                            ->   group.id
 *       state.name                          ->   group.name
 *       state.description ??
 *         state.metadata?.description       ->   group.description
 *       "element-presence"                  ->   group.category (default)
 *       state.provenance?.source ??
 *         "ai-generated"                    ->   group.source
 *
 *     For each requiredElement of state:
 *       (state, criteria, index)            ->   one assertion in group.assertions[]
 *         "${state.id}-elem-${index}"       ->   assertion.id
 *         state.metadata?.description ??    ->   assertion.description
 *           "Required element ${index} ..."
 *         "element-presence"                ->   assertion.category
 *         "critical"                        ->   assertion.severity
 *         "exists"                          ->   assertion.assertionType
 *         {
 *           type: "search",
 *           criteria: convert(criteria),   ->   assertion.target
 *           label: "Required element ..."
 *         }
 *         "ai-generated"                    ->   assertion.source
 *         false                             ->   assertion.reviewed
 *         true                              ->   assertion.enabled
 *
 *     If state has zero requiredElements, emit a single placeholder assertion
 *     describing the state (legacy consumers expect at least one assertion per
 *     group). Same for criteria with empty-string text.
 *
 *   Each IRState                            ->   one entry in stateMachine.states[]
 *     state.id                              ->   sm-state.id
 *     state.name                            ->   sm-state.name
 *     state.description ?? ""               ->   sm-state.description
 *     state.requiredElements (converted)    ->   sm-state.elements
 *     state.isInitial ??
 *       (state.id === doc.initialState)     ->   sm-state.isInitial
 *     transitions originating from state    ->   sm-state.transitions
 *
 *   For each IRTransition:
 *     For each fromState in transition.fromStates:
 *       transition                          ->   one entry in
 *                                                stateMachine.states[fromState].transitions[]
 *         transition.id                     ->   sm-transition.id
 *         transition.name                   ->   sm-transition.name
 *         transition.activateStates         ->   sm-transition.activateStates
 *         transition.exitStates ?? []       ->   sm-transition.deactivateStates
 *         (exitStates.length === 0)         ->   sm-transition.staysVisible
 *
 *         For each action in transition.actions:
 *           action.type                     ->   process[].action  (rename type -> action)
 *           action.target                   ->   process[].target  (criteria converted)
 *           action.waitAfter                ->   process[].waitAfter (passed through)
 *
 *   Element-criteria conversion (IRElementCriteria -> legacy criteria):
 *       text          ->   textContent
 *       ariaLabel     ->   accessibleName
 *       attributes    ->   dataAttributes
 *       (others — role, textContains, id — pass through unchanged)
 *
 * ---------------------------------------------------------------------------
 * Worked example
 * ---------------------------------------------------------------------------
 *
 *   const ir: IRDocument = {
 *     version: "1.0",
 *     id: "active",
 *     name: "Active Dashboard",
 *     description: "Real-time monitoring hub",
 *     metadata: { tags: ["monitoring", "tier-1"] },
 *     states: [
 *       {
 *         id: "running",
 *         name: "Running",
 *         description: "Workflow executing",
 *         requiredElements: [{ role: "button", text: "Stop" }],
 *         isInitial: false,
 *       },
 *     ],
 *     transitions: [
 *       {
 *         id: "running-to-idle",
 *         name: "Stop",
 *         fromStates: ["running"],
 *         activateStates: ["idle"],
 *         exitStates: ["running"],
 *         actions: [
 *           {
 *             type: "click",
 *             target: { role: "button", text: "Stop" },
 *             waitAfter: { type: "idle", timeout: 3000 },
 *           },
 *         ],
 *       },
 *     ],
 *     initialState: "idle",
 *   };
 *
 *   projectIRToBundledPage(ir, "Notes from author")
 *   // -> {
 *   //   version: "1.0.0",
 *   //   description: "Real-time monitoring hub\n\nNotes from author",
 *   //   groups: [{
 *   //     id: "running",
 *   //     name: "Running",
 *   //     description: "Workflow executing",
 *   //     category: "element-presence",
 *   //     assertions: [{
 *   //       id: "running-elem-0",
 *   //       description: "Required element 0 for state Running",
 *   //       category: "element-presence",
 *   //       severity: "critical",
 *   //       assertionType: "exists",
 *   //       target: {
 *   //         type: "search",
 *   //         criteria: { role: "button", textContent: "Stop" },
 *   //         label: "Required element for Running",
 *   //       },
 *   //       source: "ai-generated",
 *   //       reviewed: false,
 *   //       enabled: true,
 *   //     }],
 *   //     source: "ai-generated",
 *   //   }],
 *   //   stateMachine: {
 *   //     states: [{
 *   //       id: "running",
 *   //       name: "Running",
 *   //       description: "Workflow executing",
 *   //       elements: [{ role: "button", textContent: "Stop" }],
 *   //       isInitial: false,
 *   //       transitions: [{
 *   //         id: "running-to-idle",
 *   //         name: "Stop",
 *   //         activateStates: ["idle"],
 *   //         deactivateStates: ["running"],
 *   //         staysVisible: false,
 *   //         process: [{
 *   //           action: "click",
 *   //           target: { role: "button", textContent: "Stop" },
 *   //           waitAfter: { type: "idle", timeout: 3000 },
 *   //         }],
 *   //       }],
 *   //     }],
 *   //   },
 *   //   metadata: { component: "active", tags: ["monitoring", "tier-1"] },
 *   // }
 */

import type { IRElementCriteria } from "./element-criteria";
import type { IRDocument } from "./document";
import type { IRState } from "./state";
import type { IRTransition, IRTransitionAction, IRWaitSpec } from "./transition";

/**
 * Projection version. Bump on shape changes that consumers must adapt to.
 * Independent from `IRVersion` because the legacy spec shape and the IR shape
 * evolve on different cadences.
 */
export const projectionVersion = "1.0";

// ---------------------------------------------------------------------------
// Legacy spec shapes — derived from active.spec.uibridge.json (the reference
// the projection's output must structurally match).
// ---------------------------------------------------------------------------

/**
 * Legacy element-criteria shape (after conversion from `IRElementCriteria`).
 *
 * The differences from `IRElementCriteria`:
 *   - `text`        -> `textContent`
 *   - `ariaLabel`   -> `accessibleName`
 *   - `attributes`  -> `dataAttributes`
 *
 * Other fields (`role`, `textContains`, `id`) pass through unchanged. Legacy
 * specs in the wild also carry occasional `tagName` / extra fields, so the
 * shape is intentionally open via index signature.
 */
export interface LegacyCriteria {
  role?: string;
  textContent?: string;
  textContains?: string;
  accessibleName?: string;
  id?: string;
  dataAttributes?: Record<string, string>;
  [key: string]: unknown;
}

/**
 * Legacy assertion target. Always `type: "search"` for the projection — the
 * legacy schema also supports point/region targets, but the IR doesn't
 * express those.
 */
export interface LegacyAssertionTarget {
  type: "search";
  criteria: LegacyCriteria;
  label: string;
}

/**
 * Legacy assertion. Mirrors the shape used by `groups[].assertions[]` in
 * `*.spec.uibridge.json`.
 */
export interface LegacyAssertion {
  id: string;
  description: string;
  category: string;
  severity: "critical" | "warning" | "info";
  assertionType: string;
  target: LegacyAssertionTarget;
  source: string;
  reviewed: boolean;
  enabled: boolean;
  precondition?: string;
}

/**
 * Legacy group. One per IR state.
 */
export interface LegacyGroup {
  id: string;
  name: string;
  description: string;
  category: string;
  assertions: LegacyAssertion[];
  source: string;
}

/**
 * Process step within a transition. Note `action` (not `type`) — matches the
 * runtime engine's `TransitionAction` shape.
 */
export interface LegacyProcessStep {
  action: string;
  target: LegacyCriteria;
  waitAfter?: IRWaitSpec;
}

/**
 * Legacy state-machine transition.
 */
export interface LegacyTransition {
  id: string;
  name: string;
  activateStates: string[];
  deactivateStates: string[];
  staysVisible: boolean;
  process: LegacyProcessStep[];
}

/**
 * Legacy state-machine state.
 */
export interface LegacyStateMachineState {
  id: string;
  name: string;
  description: string;
  elements: LegacyCriteria[];
  isInitial: boolean;
  transitions: LegacyTransition[];
}

/**
 * Legacy state-machine block.
 */
export interface LegacyStateMachine {
  states: LegacyStateMachineState[];
}

/**
 * Legacy spec metadata block. The projection writes a deterministic subset;
 * downstream tooling may layer additional keys on hand-authored specs.
 */
export interface LegacyMetadata {
  component: string;
  tags?: string[];
  [key: string]: unknown;
}

/**
 * Top-level legacy spec shape.
 */
export interface LegacySpec {
  version: string;
  description: string;
  groups: LegacyGroup[];
  stateMachine: LegacyStateMachine;
  metadata: LegacyMetadata;
}

// ---------------------------------------------------------------------------
// Internal helpers
// ---------------------------------------------------------------------------

/**
 * Recursively sort object keys lexicographically. Arrays preserve input order
 * (ordering is meaningful for groups / assertions / transitions).
 *
 * Used as the final stage of every projected output to guarantee byte-stable
 * JSON serialization downstream.
 */
function sortKeys<T>(value: T): T {
  if (Array.isArray(value)) {
    return value.map(sortKeys) as unknown as T;
  }
  if (value !== null && typeof value === "object") {
    const sorted: Record<string, unknown> = {};
    const keys = Object.keys(value as Record<string, unknown>).sort();
    for (const key of keys) {
      sorted[key] = sortKeys((value as Record<string, unknown>)[key]);
    }
    return sorted as unknown as T;
  }
  return value;
}

/**
 * Convert IR element criteria to legacy criteria. Renames `text` ->
 * `textContent`, `ariaLabel` -> `accessibleName`, `attributes` ->
 * `dataAttributes`. Other fields pass through unchanged.
 */
function convertCriteria(criteria: IRElementCriteria): LegacyCriteria {
  const out: LegacyCriteria = {};
  if (criteria.role !== undefined) out.role = criteria.role;
  if (criteria.text !== undefined) out.textContent = criteria.text;
  if (criteria.textContains !== undefined) out.textContains = criteria.textContains;
  if (criteria.ariaLabel !== undefined) out.accessibleName = criteria.ariaLabel;
  if (criteria.id !== undefined) out.id = criteria.id;
  if (criteria.attributes !== undefined) out.dataAttributes = criteria.attributes;
  return out;
}

/**
 * Build a single legacy assertion from a state + index + (optional) criteria.
 *
 * Used both for normal `requiredElement` projection and for the placeholder
 * fallback when a state has no requiredElements (legacy consumers expect at
 * least one assertion per group).
 */
function buildAssertion(
  state: IRState,
  index: number,
  criteria: IRElementCriteria | undefined,
): LegacyAssertion {
  const description =
    state.metadata?.description ?? `Required element ${index} for state ${state.name}`;
  const targetCriteria: LegacyCriteria =
    criteria === undefined ? {} : convertCriteria(criteria);
  return {
    id: `${state.id}-elem-${index}`,
    description,
    category: "element-presence",
    severity: "critical",
    assertionType: "exists",
    target: {
      type: "search",
      criteria: targetCriteria,
      label: `Required element for ${state.name}`,
    },
    source: "ai-generated",
    reviewed: false,
    enabled: true,
  };
}

/**
 * Build the legacy `groups[]` entry for one IR state.
 */
function buildGroup(state: IRState): LegacyGroup {
  const elems = state.requiredElements ?? [];
  const assertions: LegacyAssertion[] =
    elems.length === 0
      ? [buildAssertion(state, 0, undefined)]
      : elems.map((c, i) => buildAssertion(state, i, c));
  return {
    id: state.id,
    name: state.name,
    description: state.description ?? state.metadata?.description ?? "",
    category: "element-presence",
    assertions,
    source: state.provenance?.source ?? "ai-generated",
  };
}

/**
 * Convert an IR transition action into the legacy `process[]` step shape.
 * Renames IR's `type` to legacy `action` (matches the runtime engine
 * convention — same as the IR -> WorkflowConfig adapter does for runtime).
 */
function buildProcessStep(action: IRTransitionAction): LegacyProcessStep {
  const out: LegacyProcessStep = {
    action: action.type,
    target: convertCriteria(action.target),
  };
  if (action.waitAfter !== undefined) out.waitAfter = action.waitAfter;
  return out;
}

/**
 * Convert one IR transition into the legacy state-machine transition shape.
 * Each fromState gets its own copy in the projected output (legacy specs put
 * the transition under each originating state's `transitions[]`).
 */
function buildTransition(transition: IRTransition): LegacyTransition {
  const exitStates = transition.exitStates ?? [];
  return {
    id: transition.id,
    name: transition.name,
    activateStates: transition.activateStates,
    deactivateStates: exitStates,
    // Modal-style transitions (no exit states) stay visible behind the
    // activated state. Otherwise the originating state(s) deactivate, so the
    // page no longer shows them.
    staysVisible: exitStates.length === 0,
    process: transition.actions.map(buildProcessStep),
  };
}

/**
 * Build the legacy `stateMachine.states[]` entry for one IR state. Includes
 * any transitions that originate from this state (i.e. where state.id is in
 * transition.fromStates).
 */
function buildStateMachineState(
  state: IRState,
  transitions: IRTransition[],
  doc: IRDocument,
): LegacyStateMachineState {
  const outgoing = transitions
    .filter((t) => t.fromStates.includes(state.id))
    .map(buildTransition);
  const isInitial =
    state.isInitial !== undefined ? state.isInitial : state.id === doc.initialState;
  return {
    id: state.id,
    name: state.name,
    description: state.description ?? "",
    elements: (state.requiredElements ?? []).map(convertCriteria),
    isInitial,
    transitions: outgoing,
  };
}

// ---------------------------------------------------------------------------
// Public entry point
// ---------------------------------------------------------------------------

/**
 * Project an IR document into the legacy bundled-page spec shape.
 *
 * Pure / deterministic: same input always produces structurally identical
 * output (and byte-identical when JSON-serialized via the standard
 * `JSON.stringify` ordering). Arrays preserve input order; object keys are
 * sorted lexicographically.
 *
 * @param doc   The IR document to project.
 * @param notes Optional human-authored notes (carried in the page's
 *              `notes.md` companion file). Appended to `description` as a
 *              separate paragraph (separator: two newlines).
 */
export function projectIRToBundledPage(doc: IRDocument, notes?: string): LegacySpec {
  const baseDescription = doc.description ?? doc.name;
  const description =
    notes !== undefined && notes.length > 0
      ? `${baseDescription}\n\n${notes}`
      : baseDescription;

  const component = doc.metadata?.purpose ?? doc.id;
  const metadata: LegacyMetadata = { component };
  if (doc.metadata?.tags !== undefined) metadata.tags = doc.metadata.tags;

  const groups: LegacyGroup[] = doc.states.map(buildGroup);
  const smStates: LegacyStateMachineState[] = doc.states.map((s) =>
    buildStateMachineState(s, doc.transitions, doc),
  );

  const spec: LegacySpec = {
    version: "1.0.0",
    description,
    groups,
    stateMachine: { states: smStates },
    metadata,
  };

  return sortKeys(spec);
}
