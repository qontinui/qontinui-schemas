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
    projectId: data.projectId ?? data.project_id,
    name: data.name,
    description: data.description,
    sourceType: data.sourceType ?? data.source_type,
    sourceSessionId: data.sourceSessionId ?? data.source_session_id,
    discoveryStrategy: data.discoveryStrategy ?? data.discovery_strategy,
    images: (data.images || []).map(toDiscoveredStateImage),
    states: (data.states || []).map(toDiscoveredState),
    transitions: (data.transitions || []).map(
      toDiscoveredTransition
    ),
    elementToRenders: (data.elementToRenders ?? data.element_to_renders) || {},
    imageCount: data.imageCount ?? data.image_count,
    stateCount: data.stateCount ?? data.state_count,
    transitionCount: data.transitionCount ?? data.transition_count,
    renderCount: data.renderCount ?? data.render_count,
    uniqueElementCount: data.uniqueElementCount ?? data.unique_element_count,
    confidence: data.confidence,
    discoveryMetadata: (data.discoveryMetadata ?? data.discovery_metadata) || {},
    createdAt: data.createdAt ?? data.created_at,
    updatedAt: data.updatedAt ?? data.updated_at
  };
}
function toDiscoveredStateImage(data) {
  const d = data;
  return {
    id: d.id,
    screenshotId: d.screenshotId ?? d.screenshot_id,
    screenshotUrl: d.screenshotUrl ?? d.screenshot_url,
    bbox: d.bbox,
    pixelHash: d.pixelHash ?? d.pixel_hash,
    stateId: d.stateId ?? d.state_id,
    elementType: d.elementType ?? d.element_type,
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
    imageIds: (d.imageIds ?? d.image_ids) || [],
    renderIds: (d.renderIds ?? d.render_ids) || [],
    elementIds: (d.elementIds ?? d.element_ids) || [],
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
    fromStateId: d.fromStateId ?? d.from_state_id,
    toStateId: d.toStateId ?? d.to_state_id,
    trigger: trigger ? {
      type: trigger.type ?? "click",
      imageId: trigger.imageId ?? trigger.image_id,
      elementId: trigger.elementId ?? trigger.element_id,
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
    projectId: data.projectId ?? data.project_id,
    name: data.name,
    description: data.description,
    sourceType: data.sourceType ?? data.source_type,
    discoveryStrategy: data.discoveryStrategy ?? data.discovery_strategy,
    imageCount: data.imageCount ?? data.image_count,
    stateCount: data.stateCount ?? data.state_count,
    transitionCount: data.transitionCount ?? data.transition_count,
    confidence: data.confidence,
    createdAt: data.createdAt ?? data.created_at
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
var RunType = {
  QA_TEST: "qa_test",
  INTEGRATION_TEST: "integration_test",
  LIVE_AUTOMATION: "live_automation",
  RECORDING: "recording",
  DEBUG: "debug"
};
var RunStatus = {
  PENDING: "pending",
  RUNNING: "running",
  COMPLETED: "completed",
  FAILED: "failed",
  TIMEOUT: "timeout",
  CANCELLED: "cancelled",
  PAUSED: "paused"
};
var ActionStatus = {
  SUCCESS: "success",
  FAILED: "failed",
  TIMEOUT: "timeout",
  SKIPPED: "skipped",
  ERROR: "error",
  PENDING: "pending"
};
var ActionType = {
  // Vision actions
  FIND: "find",
  FIND_ALL: "find_all",
  WAIT_FOR: "wait_for",
  WAIT_UNTIL_GONE: "wait_until_gone",
  // Input actions
  CLICK: "click",
  DOUBLE_CLICK: "double_click",
  RIGHT_CLICK: "right_click",
  TYPE: "type",
  PRESS_KEY: "press_key",
  HOTKEY: "hotkey",
  SCROLL: "scroll",
  DRAG: "drag",
  // State machine actions
  GO_TO_STATE: "go_to_state",
  TRANSITION: "transition",
  VERIFY_STATE: "verify_state",
  // Control flow
  CONDITIONAL: "conditional",
  LOOP: "loop",
  PARALLEL: "parallel",
  SEQUENCE: "sequence",
  // Utility
  WAIT: "wait",
  SCREENSHOT: "screenshot",
  LOG: "log",
  ASSERT: "assert",
  // AI actions
  AI_PROMPT: "ai_prompt",
  RUN_PROMPT_SEQUENCE: "run_prompt_sequence",
  // Custom/plugin
  CUSTOM: "custom"
};
var ErrorType = {
  ELEMENT_NOT_FOUND: "element_not_found",
  TIMEOUT: "timeout",
  ASSERTION_FAILED: "assertion_failed",
  CRASH: "crash",
  NETWORK_ERROR: "network_error",
  VALIDATION_ERROR: "validation_error",
  OTHER: "other"
};
var IssueSeverity = {
  CRITICAL: "critical",
  HIGH: "high",
  MEDIUM: "medium",
  LOW: "low",
  INFORMATIONAL: "informational"
};
var ScreenshotType = {
  ERROR: "error",
  SUCCESS: "success",
  MANUAL: "manual",
  PERIODIC: "periodic",
  ACTION_RESULT: "action_result",
  STATE_VERIFICATION: "state_verification"
};

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