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
 *   - `ariaLabel`   -> `accessibleName` (when IR has no explicit `accessibleName`)
 *   - `attributes`  -> `dataAttributes`
 *
 * Other fields (`role`, `tagName`, `textContains`, `accessibleName`, `id`)
 * pass through unchanged. Index signature stays open for legacy fields the
 * IR doesn't yet model.
 */
export interface LegacyCriteria {
  role?: string;
  tagName?: string;
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
 * `textContent`, `attributes` -> `dataAttributes`. `accessibleName` (or
 * `ariaLabel` as a fallback) maps to legacy `accessibleName`. `tagName`,
 * `role`, `textContains`, `id` pass through unchanged.
 */
function convertCriteria(criteria: IRElementCriteria): LegacyCriteria {
  const out: LegacyCriteria = {};
  if (criteria.role !== undefined) out.role = criteria.role;
  if (criteria.tagName !== undefined) out.tagName = criteria.tagName;
  if (criteria.text !== undefined) out.textContent = criteria.text;
  if (criteria.textContains !== undefined) out.textContains = criteria.textContains;
  // Prefer explicit `accessibleName` when present; fall back to `ariaLabel`
  // for IR documents authored against the section-1 shape.
  if (criteria.accessibleName !== undefined) {
    out.accessibleName = criteria.accessibleName;
  } else if (criteria.ariaLabel !== undefined) {
    out.accessibleName = criteria.ariaLabel;
  }
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
  const assertion: LegacyAssertion = {
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
  if (state.precondition !== undefined) {
    assertion.precondition = state.precondition;
  }
  return assertion;
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

// ---------------------------------------------------------------------------
// Inverse projection — Legacy spec -> IR document
// ---------------------------------------------------------------------------

/**
 * Options for the inverse projection.
 *
 * - `docId`        — explicit IRDocument.id override. Falls back to
 *                    `legacy.metadata.component` (or `fallbackName` if absent).
 * - `fallbackName` — used when `legacy.metadata.component` is missing AND no
 *                    `docId` was supplied.
 */
export interface ProjectLegacyToIROptions {
  docId?: string;
  fallbackName?: string;
}

/**
 * Best-effort field-by-field invert of `LegacyCriteria` back into
 * `IRElementCriteria`.
 *
 * Rename map (the structural inverse of `convertCriteria`):
 *   textContent     -> text
 *   accessibleName  -> accessibleName  (passes through; the forward direction
 *                                       collapses ariaLabel into accessibleName,
 *                                       so we cannot reliably distinguish them
 *                                       on the inverse — pick accessibleName).
 *   dataAttributes  -> attributes
 *   role / tagName / textContains / id pass through unchanged.
 *
 * Unknown legacy keys are dropped (lossy, but documented). Document-level
 * unknown criteria are rare; if they show up we'll surface them in Phase A4.
 */
function invertCriteria(criteria: LegacyCriteria | undefined): IRElementCriteria {
  if (criteria === undefined || criteria === null) return {};
  const out: IRElementCriteria = {};
  if (criteria.role !== undefined) out.role = criteria.role;
  if (criteria.tagName !== undefined) out.tagName = criteria.tagName;
  if (criteria.textContent !== undefined) out.text = criteria.textContent;
  if (criteria.textContains !== undefined) out.textContains = criteria.textContains;
  if (criteria.accessibleName !== undefined) out.accessibleName = criteria.accessibleName;
  if (criteria.id !== undefined) out.id = criteria.id;
  if (criteria.dataAttributes !== undefined) out.attributes = criteria.dataAttributes;
  return out;
}

/**
 * Pick the first non-empty `precondition` across a group's assertions. Legacy
 * groups carry preconditions on individual assertions; IR carries it on the
 * state. If multiple distinct preconditions exist we keep the first
 * (deterministic, mirrors input ordering) — Phase A4 review will surface
 * conflicts for hand-fix.
 */
function firstPrecondition(group: LegacyGroup): string | undefined {
  for (const a of group.assertions) {
    if (a.precondition !== undefined && a.precondition.length > 0) {
      return a.precondition;
    }
  }
  return undefined;
}

/**
 * Build an `IRState` from one legacy group + its corresponding state-machine
 * state (when present).
 *
 * IMPORTANT: `requiredElements` is sourced from the GROUP's assertion criteria
 * (one criteria per assertion, in input order), NOT from `smState.elements`.
 * This preserves the round-trip identity for the assertion-count check —
 * legacy specs sometimes carry an SM block whose `elements[]` is a coarse
 * page-state fingerprint (1-2 entries) while the group has many assertions.
 * Using `smState.elements` would silently collapse those into one
 * requiredElement per state, then the forward direction would emit only one
 * assertion per group.
 *
 * `smState` (when present and matched by id) contributes:
 *   - `name` override (legacy `smState.name` is human-readable; group.name is
 *     also human-readable but may differ — prefer smState's name since the
 *     forward direction's `buildStateMachineState` echoes `state.name` to
 *     `smState.name`).
 *   - `description` fallback (when group.description is empty).
 *   - `isInitial` (the only place this lives in legacy specs).
 */
function buildIRState(
  group: LegacyGroup,
  smState: LegacyStateMachineState | undefined,
): IRState {
  // Always one requiredElement per assertion so the forward projection emits
  // the same number of assertions on the round-trip.
  const requiredElements: IRElementCriteria[] = group.assertions.map((a) =>
    invertCriteria(a.target?.criteria),
  );

  const description =
    group.description !== undefined && group.description.length > 0
      ? group.description
      : smState?.description;
  const precondition = firstPrecondition(group);

  const state: IRState = {
    id: group.id,
    name: smState?.name ?? group.name,
    requiredElements,
  };
  if (description !== undefined && description.length > 0) {
    state.description = description;
  }
  if (smState?.isInitial === true) {
    state.isInitial = true;
  } else if (smState?.isInitial === false) {
    state.isInitial = false;
  }
  if (precondition !== undefined) {
    state.precondition = precondition;
  }
  state.provenance = { source: "migrated" };
  return state;
}

/**
 * Convert one legacy `process[]` step back into `IRTransitionAction`. Renames
 * `action` -> `type`. Legacy specs occasionally omit a target on no-op steps;
 * we coerce to an empty criteria object in that case.
 */
function invertProcessStep(step: LegacyProcessStep): IRTransitionAction {
  const out: IRTransitionAction = {
    type: step.action,
    target: invertCriteria(step.target),
  };
  if (step.waitAfter !== undefined) out.waitAfter = step.waitAfter;
  return out;
}

/**
 * Collapse duplicate transitions across multiple originating states. The
 * forward projection emits the same transition under EACH `fromState`'s
 * `transitions[]` entry; the inverse must merge them back into ONE
 * `IRTransition` whose `fromStates` lists every originator.
 *
 * Same-id transitions are assumed to be byte-identical except for the
 * (implicit) originating state. If they differ in `process[]` /
 * `activateStates` / `deactivateStates`, we keep the first occurrence and
 * append the additional fromState. (Genuine divergence here is a defect to
 * surface in Phase A4 review.)
 */
function buildIRTransitions(stateMachine: LegacyStateMachine | undefined): IRTransition[] {
  if (stateMachine === undefined || !Array.isArray(stateMachine.states)) return [];
  const byId = new Map<string, IRTransition>();
  // Preserve insertion order — first state's transitions come first.
  const order: string[] = [];

  for (const smState of stateMachine.states) {
    const transitions = Array.isArray(smState.transitions) ? smState.transitions : [];
    for (const t of transitions) {
      const existing = byId.get(t.id);
      if (existing !== undefined) {
        if (!existing.fromStates.includes(smState.id)) {
          existing.fromStates.push(smState.id);
        }
        continue;
      }
      const ir: IRTransition = {
        id: t.id,
        name: t.name,
        fromStates: [smState.id],
        activateStates: Array.isArray(t.activateStates) ? [...t.activateStates] : [],
        actions: Array.isArray(t.process) ? t.process.map(invertProcessStep) : [],
      };
      // The forward direction maps `exitStates` -> `deactivateStates`; preserve
      // the field whenever it's present in legacy form (even if empty array)
      // so modal-style transitions (staysVisible: true, deactivateStates: [])
      // round-trip predictably.
      if (Array.isArray(t.deactivateStates)) {
        ir.exitStates = [...t.deactivateStates];
      }
      ir.provenance = { source: "migrated" };
      byId.set(t.id, ir);
      order.push(t.id);
    }
  }

  return order.map((id) => byId.get(id)!);
}

/**
 * Project a legacy bundled-page spec back into IR.
 *
 * Pure / deterministic: same input always produces structurally identical
 * output. Collapses transition duplicates (forward emits one copy per
 * `fromState`), preserves group/assertion ordering, and tags every node with
 * `provenance.source = "migrated"` so downstream tooling can distinguish
 * migrated content from hand-authored.
 *
 * The mapping is the structural inverse of `projectIRToBundledPage`:
 *
 *   LegacySpec                                IRDocument
 *   ----------                                ----------
 *   metadata.component                  ->   id (or opts.docId override)
 *   metadata.tags                       ->   metadata.tags
 *   description                         ->   description (kept verbatim;
 *                                            authored notes appended via
 *                                            forward `notes` arg are not
 *                                            stripped — that round-trip is
 *                                            best-effort and Phase A4 will
 *                                            split them back out)
 *
 *   groups[]                            ->   states[] (one per group)
 *     group.id / name / description     ->   state.id / name / description
 *     assertion.target.criteria         ->   state.requiredElements[i] (or,
 *                                            preferred, smState.elements when
 *                                            the stateMachine block is present)
 *     first-found assertion.precondition ->  state.precondition
 *
 *   stateMachine.states[]               ->   contributes isInitial + elements
 *     smState.transitions[]             ->   transitions[] (deduped by id)
 *       transition.activateStates       ->   transition.activateStates
 *       transition.deactivateStates     ->   transition.exitStates
 *       transition.process[]            ->   transition.actions[]
 *         step.action                   ->   action.type
 *         step.target                   ->   action.target (criteria inverted)
 *         step.waitAfter                ->   action.waitAfter
 *
 * Documented losses (acceptable for Phase A2):
 *   - Legacy assertion ids (`{stateId}-elem-{i}`) are NOT preserved across the
 *     round-trip — the forward direction regenerates them from index, so any
 *     hand-edited ids are lost. See Phase A4.
 *   - Legacy `assertionType` / `severity` / `category` / `source` per
 *     assertion are reduced to defaults on round-trip.
 *   - Legacy `expected` / `assertionType: "count"` and `notExists` semantics
 *     are not modeled in IR — the assertion's criteria still survive (as a
 *     state requiredElement) but the assertion-flavor metadata is dropped.
 *   - Legacy `testing` block + extra metadata keys outside `component` /
 *     `tags` are ignored. Authors keep them via the legacy spec file directly
 *     during Phase A2 — they will move into IR companion files in Phase A4.
 *   - The forward `notes` argument's appended paragraph is currently kept
 *     in `description`; Phase A4 will split it back out into a dedicated
 *     companion file.
 *
 * @param legacy The legacy spec to invert.
 * @param opts   Optional doc-level overrides.
 */
export function projectLegacyToIR(
  legacy: LegacySpec,
  opts?: ProjectLegacyToIROptions,
): IRDocument {
  const component =
    opts?.docId ??
    legacy.metadata?.component ??
    opts?.fallbackName ??
    "legacy-spec";
  const name = legacy.metadata?.component ?? opts?.fallbackName ?? component;

  // Index state-machine states by id for quick lookup when building IR states
  // out of groups. Older specs may not have a stateMachine block.
  const smByGroupId = new Map<string, LegacyStateMachineState>();
  if (legacy.stateMachine !== undefined && Array.isArray(legacy.stateMachine.states)) {
    for (const s of legacy.stateMachine.states) {
      smByGroupId.set(s.id, s);
    }
  }

  const groups = Array.isArray(legacy.groups) ? legacy.groups : [];
  const states: IRState[] = groups.map((group) =>
    buildIRState(group, smByGroupId.get(group.id)),
  );

  // NOTE: Some legacy specs declare `stateMachine.states[]` whose ids do NOT
  // correspond to any `groups[]` entry (e.g., page-level states like
  // `wrappers-installed-tab` vs assertion-category groups like
  // `page-structure`). The IR's group-state mapping is 1:1 with groups by
  // construction (the forward projection emits one group per IR state), so
  // these orphan SM states cannot survive the round-trip cleanly. They are
  // dropped here as a documented loss — Phase A4 will revisit by either
  // (a) introducing a separate IR concept for "page state" vs
  // "assertion group", or (b) hand-merging the legacy specs so groups and
  // SM states share ids.
  // Transitions originating from orphan SM states are likewise omitted from
  // IR (their fromStates would dangle). We filter them below.
  const groupIds = new Set(groups.map((g) => g.id));

  // Build transitions only from SM states whose ids appear in groups[]. Any
  // transition with a fromState that isn't a known group is dropped — the
  // forward direction would have nowhere to emit it.
  let filteredStateMachine: LegacyStateMachine | undefined = legacy.stateMachine;
  if (
    legacy.stateMachine !== undefined &&
    Array.isArray(legacy.stateMachine.states) &&
    legacy.stateMachine.states.some((s) => !groupIds.has(s.id))
  ) {
    filteredStateMachine = {
      states: legacy.stateMachine.states.filter((s) => groupIds.has(s.id)),
    };
  }
  const transitions = buildIRTransitions(filteredStateMachine);

  // Initial state: first state with isInitial:true wins (mirrors how the
  // forward direction picks the document's `initialState`). Only consider
  // SM states that correspond to a group (so initialState references a real
  // IR state).
  let initialState: string | undefined;
  if (legacy.stateMachine !== undefined && Array.isArray(legacy.stateMachine.states)) {
    const initial = legacy.stateMachine.states.find(
      (s) => s.isInitial === true && groupIds.has(s.id),
    );
    if (initial !== undefined) initialState = initial.id;
  }

  const doc: IRDocument = {
    version: "1.0",
    id: component,
    name,
    states,
    transitions,
    provenance: { source: "migrated" },
  };
  if (legacy.description !== undefined && legacy.description.length > 0) {
    doc.description = legacy.description;
  }
  if (legacy.metadata !== undefined && legacy.metadata.tags !== undefined) {
    doc.metadata = { tags: legacy.metadata.tags };
  }
  if (initialState !== undefined) {
    doc.initialState = initialState;
  }
  return doc;
}
