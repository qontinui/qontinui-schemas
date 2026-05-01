"use strict";
var __defProp = Object.defineProperty;
var __getOwnPropDesc = Object.getOwnPropertyDescriptor;
var __getOwnPropNames = Object.getOwnPropertyNames;
var __hasOwnProp = Object.prototype.hasOwnProperty;
var __export = (target, all) => {
  for (var name in all)
    __defProp(target, name, { get: all[name], enumerable: true });
};
var __copyProps = (to, from, except, desc) => {
  if (from && typeof from === "object" || typeof from === "function") {
    for (let key of __getOwnPropNames(from))
      if (!__hasOwnProp.call(to, key) && key !== except)
        __defProp(to, key, { get: () => from[key], enumerable: !(desc = __getOwnPropDesc(from, key)) || desc.enumerable });
  }
  return to;
};
var __toCommonJS = (mod) => __copyProps(__defProp({}, "__esModule", { value: true }), mod);

// src/ui-bridge-ir/index.ts
var ui_bridge_ir_exports = {};
__export(ui_bridge_ir_exports, {
  adaptIRDocumentToWorkflowConfig: () => adaptIRDocumentToWorkflowConfig,
  adaptIRState: () => adaptIRState,
  adaptIRTransition: () => adaptIRTransition,
  adaptIRTransitionAction: () => adaptIRTransitionAction,
  projectIRToBundledPage: () => projectIRToBundledPage,
  projectLegacyToIR: () => projectLegacyToIR,
  projectionVersion: () => projectionVersion
});
module.exports = __toCommonJS(ui_bridge_ir_exports);

// src/ui-bridge-ir/adapter.ts
var SUPPORTED_IR_VERSIONS = /* @__PURE__ */ new Set(["1.0"]);
function adaptIRState(state) {
  const out = {
    id: state.id,
    name: state.name,
    requiredElements: state.requiredElements
  };
  if (state.description !== void 0) out.description = state.description;
  if (state.excludedElements !== void 0) out.excludedElements = state.excludedElements;
  if (state.conditions !== void 0) out.conditions = state.conditions;
  if (state.isInitial !== void 0) out.isInitial = state.isInitial;
  if (state.isTerminal !== void 0) out.isTerminal = state.isTerminal;
  if (state.blocking !== void 0) out.blocking = state.blocking;
  if (state.group !== void 0) out.group = state.group;
  if (state.pathCost !== void 0) out.pathCost = state.pathCost;
  return out;
}
var RUNTIME_WAIT_TYPES = /* @__PURE__ */ new Set([
  "idle",
  "element",
  "time",
  "vanish",
  "change",
  "stable"
]);
function adaptIRWaitAfter(wait) {
  if (!RUNTIME_WAIT_TYPES.has(wait.type)) {
    return void 0;
  }
  const out = { type: wait.type };
  if (wait.query !== void 0) out.query = wait.query;
  if (wait.ms !== void 0) out.ms = wait.ms;
  if (wait.timeout !== void 0) out.timeout = wait.timeout;
  return out;
}
function adaptIRTransitionAction(action) {
  const out = {
    action: action.type,
    target: action.target
  };
  if (action.params !== void 0) out.params = action.params;
  if (action.waitAfter !== void 0) {
    const adaptedWait = adaptIRWaitAfter(action.waitAfter);
    if (adaptedWait !== void 0) out.waitAfter = adaptedWait;
  }
  return out;
}
function adaptIRTransition(transition) {
  const out = {
    id: transition.id,
    name: transition.name,
    fromStates: transition.fromStates,
    activateStates: transition.activateStates,
    exitStates: transition.exitStates ?? [],
    actions: transition.actions.map(adaptIRTransitionAction)
  };
  if (transition.description !== void 0) out.description = transition.description;
  if (transition.pathCost !== void 0) out.pathCost = transition.pathCost;
  if (transition.bidirectional !== void 0) out.bidirectional = transition.bidirectional;
  return out;
}
function adaptIRDocumentToWorkflowConfig(doc) {
  if (!SUPPORTED_IR_VERSIONS.has(doc.version)) {
    throw new Error(
      `adaptIRDocumentToWorkflowConfig: unsupported IR version "${doc.version}". Supported: ${Array.from(SUPPORTED_IR_VERSIONS).join(", ")}`
    );
  }
  const out = {
    id: doc.id,
    name: doc.name,
    states: doc.states.map(adaptIRState),
    transitions: doc.transitions.map(adaptIRTransition)
  };
  if (doc.description !== void 0) out.description = doc.description;
  if (doc.initialState !== void 0) out.initialState = doc.initialState;
  return out;
}

// src/ui-bridge-ir/projection.ts
var projectionVersion = "1.0";
function sortKeys(value) {
  if (Array.isArray(value)) {
    return value.map(sortKeys);
  }
  if (value !== null && typeof value === "object") {
    const sorted = {};
    const keys = Object.keys(value).sort();
    for (const key of keys) {
      sorted[key] = sortKeys(value[key]);
    }
    return sorted;
  }
  return value;
}
function convertCriteria(criteria) {
  const out = {};
  if (criteria.role !== void 0) out.role = criteria.role;
  if (criteria.tagName !== void 0) out.tagName = criteria.tagName;
  if (criteria.text !== void 0) out.textContent = criteria.text;
  if (criteria.textContains !== void 0) out.textContains = criteria.textContains;
  if (criteria.accessibleName !== void 0) {
    out.accessibleName = criteria.accessibleName;
  } else if (criteria.ariaLabel !== void 0) {
    out.accessibleName = criteria.ariaLabel;
  }
  if (criteria.id !== void 0) out.id = criteria.id;
  if (criteria.attributes !== void 0) out.dataAttributes = criteria.attributes;
  return out;
}
function buildAssertion(state, index, criteria) {
  const description = state.metadata?.description ?? `Required element ${index} for state ${state.name}`;
  const targetCriteria = criteria === void 0 ? {} : convertCriteria(criteria);
  const assertion = {
    id: `${state.id}-elem-${index}`,
    description,
    category: "element-presence",
    severity: "critical",
    assertionType: "exists",
    target: {
      type: "search",
      criteria: targetCriteria,
      label: `Required element for ${state.name}`
    },
    source: "ai-generated",
    reviewed: false,
    enabled: true
  };
  if (state.precondition !== void 0) {
    assertion.precondition = state.precondition;
  }
  return assertion;
}
function buildGroup(state) {
  const elems = state.requiredElements ?? [];
  const assertions = elems.length === 0 ? [buildAssertion(state, 0, void 0)] : elems.map((c, i) => buildAssertion(state, i, c));
  return {
    id: state.id,
    name: state.name,
    description: state.description ?? state.metadata?.description ?? "",
    category: "element-presence",
    assertions,
    source: state.provenance?.source ?? "ai-generated"
  };
}
function buildProcessStep(action) {
  const out = {
    action: action.type,
    target: convertCriteria(action.target)
  };
  if (action.waitAfter !== void 0) out.waitAfter = action.waitAfter;
  return out;
}
function buildTransition(transition) {
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
    process: transition.actions.map(buildProcessStep)
  };
}
function buildStateMachineState(state, transitions, doc) {
  const outgoing = transitions.filter((t) => t.fromStates.includes(state.id)).map(buildTransition);
  const isInitial = state.isInitial !== void 0 ? state.isInitial : state.id === doc.initialState;
  return {
    id: state.id,
    name: state.name,
    description: state.description ?? "",
    elements: (state.requiredElements ?? []).map(convertCriteria),
    isInitial,
    transitions: outgoing
  };
}
function projectIRToBundledPage(doc, notes) {
  const baseDescription = doc.description ?? doc.name;
  const description = notes !== void 0 && notes.length > 0 ? `${baseDescription}

${notes}` : baseDescription;
  const component = doc.metadata?.purpose ?? doc.id;
  const metadata = { component };
  if (doc.metadata?.tags !== void 0) metadata.tags = doc.metadata.tags;
  const groups = doc.states.map(buildGroup);
  const smStates = doc.states.map(
    (s) => buildStateMachineState(s, doc.transitions, doc)
  );
  const spec = {
    version: "1.0.0",
    description,
    groups,
    stateMachine: { states: smStates },
    metadata
  };
  return sortKeys(spec);
}
function invertCriteria(criteria) {
  if (criteria === void 0 || criteria === null) return {};
  const out = {};
  if (criteria.role !== void 0) out.role = criteria.role;
  if (criteria.tagName !== void 0) out.tagName = criteria.tagName;
  if (criteria.textContent !== void 0) out.text = criteria.textContent;
  if (criteria.textContains !== void 0) out.textContains = criteria.textContains;
  if (criteria.accessibleName !== void 0) out.accessibleName = criteria.accessibleName;
  if (criteria.id !== void 0) out.id = criteria.id;
  if (criteria.dataAttributes !== void 0) out.attributes = criteria.dataAttributes;
  return out;
}
function firstPrecondition(group) {
  for (const a of group.assertions) {
    if (a.precondition !== void 0 && a.precondition.length > 0) {
      return a.precondition;
    }
  }
  return void 0;
}
function buildIRState(group, smState) {
  const requiredElements = group.assertions.map(
    (a) => invertCriteria(a.target?.criteria)
  );
  const description = group.description !== void 0 && group.description.length > 0 ? group.description : smState?.description;
  const precondition = firstPrecondition(group);
  const state = {
    id: group.id,
    name: smState?.name ?? group.name,
    requiredElements
  };
  if (description !== void 0 && description.length > 0) {
    state.description = description;
  }
  if (smState?.isInitial === true) {
    state.isInitial = true;
  } else if (smState?.isInitial === false) {
    state.isInitial = false;
  }
  if (precondition !== void 0) {
    state.precondition = precondition;
  }
  state.provenance = { source: "migrated" };
  return state;
}
function invertProcessStep(step) {
  const out = {
    type: step.action,
    target: invertCriteria(step.target)
  };
  if (step.waitAfter !== void 0) out.waitAfter = step.waitAfter;
  return out;
}
function buildIRTransitions(stateMachine) {
  if (stateMachine === void 0 || !Array.isArray(stateMachine.states)) return [];
  const byId = /* @__PURE__ */ new Map();
  const order = [];
  for (const smState of stateMachine.states) {
    const transitions = Array.isArray(smState.transitions) ? smState.transitions : [];
    for (const t of transitions) {
      const existing = byId.get(t.id);
      if (existing !== void 0) {
        if (!existing.fromStates.includes(smState.id)) {
          existing.fromStates.push(smState.id);
        }
        continue;
      }
      const ir = {
        id: t.id,
        name: t.name,
        fromStates: [smState.id],
        activateStates: Array.isArray(t.activateStates) ? [...t.activateStates] : [],
        actions: Array.isArray(t.process) ? t.process.map(invertProcessStep) : []
      };
      if (Array.isArray(t.deactivateStates)) {
        ir.exitStates = [...t.deactivateStates];
      }
      ir.provenance = { source: "migrated" };
      byId.set(t.id, ir);
      order.push(t.id);
    }
  }
  return order.map((id) => byId.get(id));
}
function projectLegacyToIR(legacy, opts) {
  const component = opts?.docId ?? legacy.metadata?.component ?? opts?.fallbackName ?? "legacy-spec";
  const name = legacy.metadata?.component ?? opts?.fallbackName ?? component;
  const smByGroupId = /* @__PURE__ */ new Map();
  if (legacy.stateMachine !== void 0 && Array.isArray(legacy.stateMachine.states)) {
    for (const s of legacy.stateMachine.states) {
      smByGroupId.set(s.id, s);
    }
  }
  const groups = Array.isArray(legacy.groups) ? legacy.groups : [];
  const states = groups.map(
    (group) => buildIRState(group, smByGroupId.get(group.id))
  );
  const groupIds = new Set(groups.map((g) => g.id));
  let filteredStateMachine = legacy.stateMachine;
  if (legacy.stateMachine !== void 0 && Array.isArray(legacy.stateMachine.states) && legacy.stateMachine.states.some((s) => !groupIds.has(s.id))) {
    filteredStateMachine = {
      states: legacy.stateMachine.states.filter((s) => groupIds.has(s.id))
    };
  }
  const transitions = buildIRTransitions(filteredStateMachine);
  let initialState;
  if (legacy.stateMachine !== void 0 && Array.isArray(legacy.stateMachine.states)) {
    const initial = legacy.stateMachine.states.find(
      (s) => s.isInitial === true && groupIds.has(s.id)
    );
    if (initial !== void 0) initialState = initial.id;
  }
  const doc = {
    version: "1.0",
    id: component,
    name,
    states,
    transitions,
    provenance: { source: "migrated" }
  };
  if (legacy.description !== void 0 && legacy.description.length > 0) {
    doc.description = legacy.description;
  }
  if (legacy.metadata !== void 0 && legacy.metadata.tags !== void 0) {
    doc.metadata = { tags: legacy.metadata.tags };
  }
  if (initialState !== void 0) {
    doc.initialState = initialState;
  }
  return doc;
}
// Annotate the CommonJS export names for ESM import in node:
0 && (module.exports = {
  adaptIRDocumentToWorkflowConfig,
  adaptIRState,
  adaptIRTransition,
  adaptIRTransitionAction,
  projectIRToBundledPage,
  projectLegacyToIR,
  projectionVersion
});
//# sourceMappingURL=ui-bridge-ir.cjs.map