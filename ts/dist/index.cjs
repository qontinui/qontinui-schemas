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

// src/index.ts
var src_exports = {};
__export(src_exports, {
  ActionStatus: () => ActionStatus,
  ActionType: () => ActionType,
  DEFAULT_SUMMARY_PROMPT: () => DEFAULT_SUMMARY_PROMPT,
  DETECTION_METHODS: () => DETECTION_METHODS,
  ErrorType: () => ErrorType,
  ISSUE_CATEGORIES: () => ISSUE_CATEGORIES,
  ISSUE_SEVERITIES: () => ISSUE_SEVERITIES,
  IssueSeverity: () => IssueSeverity,
  PHASE_INFO: () => PHASE_INFO,
  RunStatus: () => RunStatus,
  RunType: () => RunType,
  SOURCE_TYPE_COLORS: () => SOURCE_TYPE_COLORS,
  SOURCE_TYPE_LABELS: () => SOURCE_TYPE_LABELS,
  STEP_TYPES: () => STEP_TYPES,
  ScreenshotType: () => ScreenshotType,
  isComponentRenderLog: () => isComponentRenderLog,
  isDomSnapshotRenderLog: () => isDomSnapshotRenderLog,
  toDiscoveredState: () => toDiscoveredState,
  toDiscoveredStateImage: () => toDiscoveredStateImage,
  toDiscoveredTransition: () => toDiscoveredTransition,
  toStateDiscoveryResult: () => toStateDiscoveryResult,
  toStateDiscoveryResultSummary: () => toStateDiscoveryResultSummary
});
module.exports = __toCommonJS(src_exports);

// src/render-log.ts
function isComponentRenderLog(entry) {
  return entry.type === "component";
}
function isDomSnapshotRenderLog(entry) {
  return entry.type === "dom_snapshot";
}

// src/discovery.ts
var SOURCE_TYPE_LABELS = {
  playwright: "Web Extraction",
  ui_bridge: "UI Bridge",
  recording: "Recording",
  vision: "Vision",
  manual: "Manual"
};
var SOURCE_TYPE_COLORS = {
  playwright: "bg-blue-500/20 text-blue-400 border-blue-500/30",
  ui_bridge: "bg-purple-500/20 text-purple-400 border-purple-500/30",
  recording: "bg-green-500/20 text-green-400 border-green-500/30",
  vision: "bg-orange-500/20 text-orange-400 border-orange-500/30",
  manual: "bg-gray-500/20 text-gray-400 border-gray-500/30"
};
function toStateDiscoveryResult(data) {
  return {
    id: data.id,
    project_id: data.project_id,
    name: data.name,
    description: data.description,
    source_type: data.source_type,
    source_session_id: data.source_session_id,
    discovery_strategy: data.discovery_strategy,
    images: (data.images || []).map(toDiscoveredStateImage),
    states: (data.states || []).map(toDiscoveredState),
    transitions: (data.transitions || []).map(
      toDiscoveredTransition
    ),
    element_to_renders: data.element_to_renders || {},
    image_count: data.image_count,
    state_count: data.state_count,
    transition_count: data.transition_count,
    render_count: data.render_count,
    unique_element_count: data.unique_element_count,
    confidence: data.confidence,
    discovery_metadata: data.discovery_metadata || {},
    created_at: data.created_at,
    updated_at: data.updated_at
  };
}
function toDiscoveredStateImage(data) {
  const d = data;
  return {
    id: d.id,
    screenshot_id: d.screenshot_id,
    screenshot_url: d.screenshot_url,
    bbox: d.bbox,
    pixel_hash: d.pixel_hash,
    state_id: d.state_id,
    element_type: d.element_type,
    label: d.label,
    confidence: d.confidence ?? 1,
    metadata: d.metadata
  };
}
function toDiscoveredState(data) {
  const d = data;
  return {
    id: d.id,
    name: d.name,
    image_ids: d.image_ids || [],
    render_ids: d.render_ids || [],
    element_ids: d.element_ids || [],
    confidence: d.confidence ?? 1,
    description: d.description,
    metadata: d.metadata
  };
}
function toDiscoveredTransition(data) {
  const d = data;
  const trigger = d.trigger;
  return {
    id: d.id,
    from_state_id: d.from_state_id,
    to_state_id: d.to_state_id,
    trigger: trigger ? {
      type: trigger.type,
      image_id: trigger.image_id,
      element_id: trigger.element_id,
      selector: trigger.selector,
      value: trigger.value
    } : void 0,
    confidence: d.confidence ?? 1,
    metadata: d.metadata
  };
}
function toStateDiscoveryResultSummary(data) {
  return {
    id: data.id,
    project_id: data.project_id,
    name: data.name,
    description: data.description,
    source_type: data.source_type,
    discovery_strategy: data.discovery_strategy,
    image_count: data.image_count,
    state_count: data.state_count,
    transition_count: data.transition_count,
    confidence: data.confidence,
    created_at: data.created_at
  };
}

// src/workflow/_api.ts
var STEP_TYPES = {
  setup: [
    {
      type: "command",
      label: "Command",
      description: "Run shell commands, checks, or tests",
      icon: "Terminal",
      color: "gray",
      phase: "setup"
    },
    {
      type: "ui_bridge",
      label: "UI Bridge",
      description: "Interact with UI via UI Bridge SDK",
      icon: "Monitor",
      color: "emerald",
      phase: "setup"
    },
    {
      type: "prompt",
      label: "AI Task",
      description: "AI-driven task",
      icon: "Bot",
      color: "violet",
      phase: "setup"
    },
    {
      type: "workflow",
      label: "Workflow",
      description: "Run a saved workflow",
      icon: "Workflow",
      color: "blue",
      phase: "setup"
    }
  ],
  verification: [
    {
      type: "command",
      label: "Command",
      description: "Run commands, checks, or tests for verification",
      icon: "Terminal",
      color: "gray",
      phase: "verification"
    },
    {
      type: "ui_bridge",
      label: "UI Bridge",
      description: "Verify UI state via UI Bridge",
      icon: "Monitor",
      color: "emerald",
      phase: "verification"
    },
    {
      type: "prompt",
      label: "AI Verification",
      description: "AI-evaluated criteria",
      icon: "Bot",
      color: "violet",
      phase: "verification"
    },
    {
      type: "workflow",
      label: "Workflow",
      description: "Run a saved workflow for verification",
      icon: "Workflow",
      color: "blue",
      phase: "verification"
    },
    {
      type: "native_accessibility",
      label: "Native Accessibility",
      description: "Run a native accessibility audit using OS-level APIs",
      icon: "Accessibility",
      color: "teal",
      phase: "verification"
    }
  ],
  agentic: [
    {
      type: "prompt",
      label: "Prompt",
      description: "AI task instructions",
      icon: "MessageSquare",
      color: "amber",
      phase: "agentic"
    }
  ],
  completion: [
    {
      type: "command",
      label: "Command",
      description: "Run cleanup commands or final tests",
      icon: "Terminal",
      color: "gray",
      phase: "completion"
    },
    {
      type: "ui_bridge",
      label: "UI Bridge",
      description: "Final UI interactions",
      icon: "Monitor",
      color: "emerald",
      phase: "completion"
    },
    {
      type: "prompt",
      label: "AI Completion",
      description: "Final AI actions",
      icon: "Bot",
      color: "violet",
      phase: "completion"
    },
    {
      type: "workflow",
      label: "Workflow",
      description: "Run a saved workflow as a completion step",
      icon: "Workflow",
      color: "blue",
      phase: "completion"
    }
  ]
};
var PHASE_INFO = {
  setup: {
    label: "Setup",
    description: "Runs once at the beginning",
    color: "blue"
  },
  verification: {
    label: "Verification",
    description: "Checks success criteria, loops with agentic",
    color: "green"
  },
  agentic: {
    label: "Agentic",
    description: "AI work, iterates until verification passes",
    color: "amber"
  },
  completion: {
    label: "Completion",
    description: "Runs once after the loop exits",
    color: "purple"
  }
};
var DEFAULT_SUMMARY_PROMPT = `Write a one-paragraph summary of all the tasks completed in this workflow. Include what was accomplished, whether the stated goal was achieved, any issues encountered and how they were resolved, and remaining work if the goal was not fully achieved. Be concise but comprehensive.`;

// src/execution/_api.ts
var RunType = /* @__PURE__ */ ((RunType2) => {
  RunType2["QA_TEST"] = "qa_test";
  RunType2["INTEGRATION_TEST"] = "integration_test";
  RunType2["LIVE_AUTOMATION"] = "live_automation";
  RunType2["RECORDING"] = "recording";
  RunType2["DEBUG"] = "debug";
  return RunType2;
})(RunType || {});
var RunStatus = /* @__PURE__ */ ((RunStatus2) => {
  RunStatus2["PENDING"] = "pending";
  RunStatus2["RUNNING"] = "running";
  RunStatus2["COMPLETED"] = "completed";
  RunStatus2["FAILED"] = "failed";
  RunStatus2["TIMEOUT"] = "timeout";
  RunStatus2["CANCELLED"] = "cancelled";
  RunStatus2["PAUSED"] = "paused";
  return RunStatus2;
})(RunStatus || {});
var ActionStatus = /* @__PURE__ */ ((ActionStatus2) => {
  ActionStatus2["SUCCESS"] = "success";
  ActionStatus2["FAILED"] = "failed";
  ActionStatus2["TIMEOUT"] = "timeout";
  ActionStatus2["SKIPPED"] = "skipped";
  ActionStatus2["ERROR"] = "error";
  ActionStatus2["PENDING"] = "pending";
  return ActionStatus2;
})(ActionStatus || {});
var ActionType = /* @__PURE__ */ ((ActionType2) => {
  ActionType2["FIND"] = "find";
  ActionType2["FIND_ALL"] = "find_all";
  ActionType2["WAIT_FOR"] = "wait_for";
  ActionType2["WAIT_UNTIL_GONE"] = "wait_until_gone";
  ActionType2["CLICK"] = "click";
  ActionType2["DOUBLE_CLICK"] = "double_click";
  ActionType2["RIGHT_CLICK"] = "right_click";
  ActionType2["TYPE"] = "type";
  ActionType2["PRESS_KEY"] = "press_key";
  ActionType2["HOTKEY"] = "hotkey";
  ActionType2["SCROLL"] = "scroll";
  ActionType2["DRAG"] = "drag";
  ActionType2["GO_TO_STATE"] = "go_to_state";
  ActionType2["TRANSITION"] = "transition";
  ActionType2["VERIFY_STATE"] = "verify_state";
  ActionType2["CONDITIONAL"] = "conditional";
  ActionType2["LOOP"] = "loop";
  ActionType2["PARALLEL"] = "parallel";
  ActionType2["SEQUENCE"] = "sequence";
  ActionType2["WAIT"] = "wait";
  ActionType2["SCREENSHOT"] = "screenshot";
  ActionType2["LOG"] = "log";
  ActionType2["ASSERT"] = "assert";
  ActionType2["AI_PROMPT"] = "ai_prompt";
  ActionType2["RUN_PROMPT_SEQUENCE"] = "run_prompt_sequence";
  ActionType2["CUSTOM"] = "custom";
  return ActionType2;
})(ActionType || {});
var ErrorType = /* @__PURE__ */ ((ErrorType2) => {
  ErrorType2["ELEMENT_NOT_FOUND"] = "element_not_found";
  ErrorType2["TIMEOUT"] = "timeout";
  ErrorType2["ASSERTION_FAILED"] = "assertion_failed";
  ErrorType2["CRASH"] = "crash";
  ErrorType2["NETWORK_ERROR"] = "network_error";
  ErrorType2["VALIDATION_ERROR"] = "validation_error";
  ErrorType2["OTHER"] = "other";
  return ErrorType2;
})(ErrorType || {});
var IssueSeverity = /* @__PURE__ */ ((IssueSeverity2) => {
  IssueSeverity2["CRITICAL"] = "critical";
  IssueSeverity2["HIGH"] = "high";
  IssueSeverity2["MEDIUM"] = "medium";
  IssueSeverity2["LOW"] = "low";
  IssueSeverity2["INFORMATIONAL"] = "informational";
  return IssueSeverity2;
})(IssueSeverity || {});
var ScreenshotType = /* @__PURE__ */ ((ScreenshotType2) => {
  ScreenshotType2["ERROR"] = "error";
  ScreenshotType2["SUCCESS"] = "success";
  ScreenshotType2["MANUAL"] = "manual";
  ScreenshotType2["PERIODIC"] = "periodic";
  ScreenshotType2["ACTION_RESULT"] = "action_result";
  ScreenshotType2["STATE_VERIFICATION"] = "state_verification";
  return ScreenshotType2;
})(ScreenshotType || {});

// src/known-issues/index.ts
var ISSUE_CATEGORIES = [
  { value: "duplication", label: "Duplication" },
  { value: "rendering", label: "Rendering" },
  { value: "data_integrity", label: "Data Integrity" },
  { value: "timing", label: "Timing" },
  { value: "layout", label: "Layout" },
  { value: "state", label: "State" },
  { value: "performance", label: "Performance" },
  { value: "encoding", label: "Encoding" },
  { value: "navigation", label: "Navigation" },
  { value: "authentication", label: "Authentication" },
  { value: "other", label: "Other" }
];
var ISSUE_SEVERITIES = [
  { value: "critical", label: "Critical" },
  { value: "high", label: "High" },
  { value: "medium", label: "Medium" },
  { value: "low", label: "Low" }
];
var DETECTION_METHODS = [
  { value: "algorithmic", label: "Algorithmic (automatic)" },
  { value: "ai_judgment", label: "AI Judgment" },
  { value: "visual", label: "Visual (screenshot)" },
  { value: "command", label: "Shell Command" },
  { value: "ui_bridge", label: "UI Bridge" }
];
// Annotate the CommonJS export names for ESM import in node:
0 && (module.exports = {
  ActionStatus,
  ActionType,
  DEFAULT_SUMMARY_PROMPT,
  DETECTION_METHODS,
  ErrorType,
  ISSUE_CATEGORIES,
  ISSUE_SEVERITIES,
  IssueSeverity,
  PHASE_INFO,
  RunStatus,
  RunType,
  SOURCE_TYPE_COLORS,
  SOURCE_TYPE_LABELS,
  STEP_TYPES,
  ScreenshotType,
  isComponentRenderLog,
  isDomSnapshotRenderLog,
  toDiscoveredState,
  toDiscoveredStateImage,
  toDiscoveredTransition,
  toStateDiscoveryResult,
  toStateDiscoveryResultSummary
});
//# sourceMappingURL=index.cjs.map