/**
 * Primitive IR shapes shared by states, transitions, and adapters.
 *
 * IR-only fields (provenance, metadata, effect) live here so that the runtime
 * adapter can identify and strip them in one place.
 *
 * See ADR-001 (qontinui-dev-notes/ui-bridge-redesign/section-1-foundations/)
 * for the rationale on each field.
 */
/**
 * Origin of an IR node. Set by the build plugin when extracting JSX wrappers,
 * by hand when authoring a JSON IR file directly, or by a generation pipeline.
 */
interface IRProvenance {
    /** How this declaration was authored. */
    source: "hand-authored" | "build-plugin" | "ai-generated" | "migrated";
    /** Source file (relative to the build root). */
    file?: string;
    /** Line number in the source file (1-based). */
    line?: number;
    /** Column in the source file (1-based). */
    column?: number;
    /** Build-plugin version that produced this node, if applicable. */
    pluginVersion?: string;
}
/**
 * Human-authored semantic context for an IR node. Aligns with the existing
 * ElementAnnotation shape (ui-bridge/packages/ui-bridge/src/annotations/types.ts)
 * so the runtime SDK can write straight into the global annotation store
 * without parallel infrastructure.
 */
interface IRMetadata {
    /** Short human-readable description of what this state/transition represents. */
    description?: string;
    /** What this state/transition is for, intent-wise. */
    purpose?: string;
    /** Tags for grouping, filtering, and search. */
    tags?: string[];
    /** IDs of related elements/states/transitions. */
    relatedElements?: string[];
    /** Free-form notes for nuance that doesn't fit description/purpose. */
    notes?: string;
}
/**
 * Whether a transition is read-only, mutating, or destructive.
 *
 * - "read"        — query/navigate; no persistent state change.
 * - "write"       — modifies persistent state but is reversible (or has an undo).
 * - "destructive" — irreversible state change (delete, send, charge, deploy).
 *
 * Drives counterfactual analysis (section 6) and gates auto-regression
 * generation (section 9) — destructive transitions are excluded from
 * automatic walks.
 */
type IREffect = "read" | "write" | "destructive";
/**
 * Pointer from one IR document to a state/transition declared elsewhere.
 * Used when a transition activates a state owned by another page's IR doc.
 */
interface IRCrossRef {
    /** Document ID containing the referenced node. */
    doc: string;
    /** ID of the referenced state or transition within that document. */
    ref: string;
}
/**
 * Pointer to a captured visual reference (screenshot region, design token).
 * Optional companion data — populated by section 8's visual-fusion work.
 */
interface IRVisualRef {
    /** Path or content-addressable ID of the captured asset. */
    asset: string;
    /** Capture region in viewport coordinates, if applicable. */
    region?: {
        x: number;
        y: number;
        width: number;
        height: number;
    };
    /** Capture timestamp (epoch ms). */
    capturedAt?: number;
}

/**
 * IR's canonical element-matching shape.
 *
 * Per decision #7 (SESSION_PROMPTS.md): IR adopts ui-bridge-auto's
 * `ElementCriteria` as the canonical authoring shape — criteria survive ID
 * drift and are descriptive rather than identity-based. The shape mirrors
 * ui-bridge-auto/src/types/match.ts:20 exactly so the IR -> WorkflowConfig
 * adapter is structurally identity (no rewriting at the criteria level).
 *
 * `elementIds: string[]` is an OPTIONAL companion the runtime SDK fills in at
 * registration time when criteria resolve to specific registered elements.
 * Authors write criteria; the runtime caches resolutions.
 */
/**
 * Minimal criteria to identify a DOM element.
 *
 * Mirrors `ElementCriteria` from `ui-bridge-auto/src/types/match.ts:20`.
 * Kept as a separate copy here (not re-exported) so the IR module has no
 * runtime dependency on ui-bridge-auto. The adapter is the single point of
 * type-level reconciliation.
 */
interface IRElementCriteria {
    /** ARIA role or inferred role. */
    role?: string;
    /** HTML tag name (e.g. "div", "button"). Heavily used in legacy specs. */
    tagName?: string;
    /** Exact text content (trimmed). */
    text?: string;
    /** Substring match on text content (case-insensitive). */
    textContains?: string;
    /**
     * ARIA label (case-insensitive substring match). Synonym of `accessibleName`
     * for the runtime SDK; the projection prefers `accessibleName` when both are
     * present.
     */
    ariaLabel?: string;
    /**
     * Computed accessible name (the same concept legacy specs serialize as
     * `accessibleName`). Added section 3 so the inverse projection can
     * round-trip without rewriting to `ariaLabel`.
     */
    accessibleName?: string;
    /** Element ID (exact string or pattern-source string). */
    id?: string;
    /** HTML attributes to check (exact string match). */
    attributes?: Record<string, string>;
}

/**
 * IR-level state declaration.
 *
 * Authoring-time superset of ui-bridge-auto's runtime State shape. The
 * IR -> WorkflowConfig adapter strips IR-only fields (provenance, metadata,
 * elementIds, incomingTransitions, visualRefs) before handing to the runtime.
 */

/**
 * Property-level condition on a matched element. Mirrors
 * `StateCondition` in ui-bridge-auto/src/types/state.ts:83.
 */
interface IRStateCondition {
    element: IRElementCriteria;
    property: "visible" | "enabled" | "checked" | "expanded" | "selected" | "text" | "value";
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
interface IRState {
    id: string;
    name: string;
    description?: string;
    /** Element criteria that must ALL be satisfied for this state to be active. */
    requiredElements: IRElementCriteria[];
    /** Element criteria where NONE may be satisfied (any match disqualifies). */
    excludedElements?: IRElementCriteria[];
    /** Additional property checks on matched elements. */
    conditions?: IRStateCondition[];
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

/**
 * Wait specification for the runtime to settle after an action.
 * Mirrors `WaitSpec` in ui-bridge-auto/src/types/transition.ts:56.
 */
interface IRWaitSpec {
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
interface IRTransitionAction {
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
interface IRTransition {
    id: string;
    name: string;
    description?: string;
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
    actions: IRTransitionAction[];
    /** Navigation cost for pathfinding (default 1.0). */
    pathCost?: number;
    /** Whether this transition can be reversed. */
    bidirectional?: boolean;
    /** Side-effect annotation. Drives counterfactual + auto-regression gating. */
    effect?: IREffect;
    /** Semantic metadata routed through useUIAnnotation store at runtime. */
    metadata?: IRMetadata;
    /** Where this declaration came from. */
    provenance?: IRProvenance;
    /** Cross-document references. */
    crossRefs?: IRCrossRef[];
}

/**
 * Top-level IR document.
 *
 * Versioned (`'1.0'`) container for the states + transitions that describe
 * a page or scope of an application. Build plugins emit this shape; the
 * adapter folds it into ui-bridge-auto's `WorkflowConfig`.
 */

/**
 * IR schema version. Bump when the shape changes in a backwards-incompatible
 * way. The adapter rejects documents whose version it does not understand.
 */
type IRVersion = "1.0";
/**
 * Top-level IR document.
 */
interface IRDocument {
    /** Schema version. */
    version: IRVersion;
    /** Stable document identifier (typically the page or scope name). */
    id: string;
    /** Human-readable document name. */
    name: string;
    /** Optional description of what this document covers. */
    description?: string;
    /** Document-level semantic metadata. */
    metadata?: IRMetadata;
    /** Where this document came from. */
    provenance?: IRProvenance;
    /** State declarations within this document. */
    states: IRState[];
    /** Transition declarations within this document. */
    transitions: IRTransition[];
    /** ID of the initial/starting state, if applicable. */
    initialState?: string;
}

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

/** Mirrors `StateConfig` from ui-bridge-auto/src/config/workflow.ts:47. */
interface AdaptedState {
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
interface AdaptedTransition {
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
interface AdaptedWaitAfter {
    type: "idle" | "element" | "time" | "vanish" | "change" | "stable";
    query?: IRElementCriteria;
    ms?: number;
    timeout?: number;
}
interface AdaptedTransitionAction {
    /** Action verb — renamed from IR's `type` to match the runtime contract. */
    action: string;
    target: IRElementCriteria;
    params?: Record<string, unknown>;
    waitAfter?: AdaptedWaitAfter;
}
/** Mirrors `WorkflowConfig` from ui-bridge-auto/src/config/workflow.ts:67. */
interface AdaptedWorkflowConfig {
    id: string;
    name: string;
    description?: string;
    version?: string;
    states: AdaptedState[];
    transitions: AdaptedTransition[];
    initialState?: string;
}
/**
 * Convert an IR state declaration into the runtime/storage state shape.
 * Strips IR-only fields (metadata, provenance, elementIds,
 * incomingTransitions, crossRefs, visualRefs).
 */
declare function adaptIRState(state: IRState): AdaptedState;
/**
 * Convert an IR transition action into the runtime action shape.
 * Renames IR's `type` to runtime's `action`; narrows `waitAfter` to the
 * runtime-accepted subset.
 */
declare function adaptIRTransitionAction(action: IRTransitionAction): AdaptedTransitionAction;
/**
 * Convert an IR transition declaration into the runtime/storage transition
 * shape. Strips IR-only fields (effect, metadata, provenance, crossRefs)
 * and defaults `exitStates` to `[]` when omitted.
 */
declare function adaptIRTransition(transition: IRTransition): AdaptedTransition;
/**
 * Convert a top-level IR document into the runtime `WorkflowConfig` shape.
 *
 * Throws on unknown IR versions to fail loudly rather than silently emit
 * stale shapes.
 */
declare function adaptIRDocumentToWorkflowConfig(doc: IRDocument): AdaptedWorkflowConfig;

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

/**
 * Projection version. Bump on shape changes that consumers must adapt to.
 * Independent from `IRVersion` because the legacy spec shape and the IR shape
 * evolve on different cadences.
 */
declare const projectionVersion = "1.0";
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
interface LegacyCriteria {
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
interface LegacyAssertionTarget {
    type: "search";
    criteria: LegacyCriteria;
    label: string;
}
/**
 * Legacy assertion. Mirrors the shape used by `groups[].assertions[]` in
 * `*.spec.uibridge.json`.
 */
interface LegacyAssertion {
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
interface LegacyGroup {
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
interface LegacyProcessStep {
    action: string;
    target: LegacyCriteria;
    waitAfter?: IRWaitSpec;
}
/**
 * Legacy state-machine transition.
 */
interface LegacyTransition {
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
interface LegacyStateMachineState {
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
interface LegacyStateMachine {
    states: LegacyStateMachineState[];
}
/**
 * Legacy spec metadata block. The projection writes a deterministic subset;
 * downstream tooling may layer additional keys on hand-authored specs.
 */
interface LegacyMetadata {
    component: string;
    tags?: string[];
    [key: string]: unknown;
}
/**
 * Top-level legacy spec shape.
 */
interface LegacySpec {
    version: string;
    description: string;
    groups: LegacyGroup[];
    stateMachine: LegacyStateMachine;
    metadata: LegacyMetadata;
}
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
declare function projectIRToBundledPage(doc: IRDocument, notes?: string): LegacySpec;
/**
 * Options for the inverse projection.
 *
 * - `docId`        — explicit IRDocument.id override. Falls back to
 *                    `legacy.metadata.component` (or `fallbackName` if absent).
 * - `fallbackName` — used when `legacy.metadata.component` is missing AND no
 *                    `docId` was supplied.
 */
interface ProjectLegacyToIROptions {
    docId?: string;
    fallbackName?: string;
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
declare function projectLegacyToIR(legacy: LegacySpec, opts?: ProjectLegacyToIROptions): IRDocument;

export { type AdaptedState, type AdaptedTransition, type AdaptedTransitionAction, type AdaptedWaitAfter, type AdaptedWorkflowConfig, type IRCrossRef, type IRDocument, type IREffect, type IRElementCriteria, type IRMetadata, type IRProvenance, type IRState, type IRStateCondition, type IRTransition, type IRTransitionAction, type IRVersion, type IRVisualRef, type IRWaitSpec, type LegacyAssertion, type LegacyAssertionTarget, type LegacyCriteria, type LegacyGroup, type LegacyMetadata, type LegacyProcessStep, type LegacySpec, type LegacyStateMachine, type LegacyStateMachineState, type LegacyTransition, type ProjectLegacyToIROptions, adaptIRDocumentToWorkflowConfig, adaptIRState, adaptIRTransition, adaptIRTransitionAction, projectIRToBundledPage, projectLegacyToIR, projectionVersion };
