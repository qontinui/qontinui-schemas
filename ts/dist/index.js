import {
  DEFAULT_SUMMARY_PROMPT,
  PHASE_INFO,
  STEP_TYPES
} from "./chunk-CTBDIP7K.js";
import {
  ActionStatus,
  ActionType,
  ErrorType,
  IssueSeverity,
  RunStatus,
  RunType,
  ScreenshotType
} from "./chunk-ZUZ3OLOH.js";

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
export {
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
};
//# sourceMappingURL=index.js.map