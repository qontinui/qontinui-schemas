import "./chunk-ECJPAJRK.js";
import {
  DEFAULT_SUMMARY_PROMPT,
  PHASE_INFO,
  STEP_TYPES
} from "./chunk-SS5G4TR7.js";
import "./chunk-KNU3BP5F.js";
import {
  ActionStatus,
  ActionType,
  ErrorType,
  IssueSeverity,
  RunStatus,
  RunType,
  ScreenshotType
} from "./chunk-CX47AO4E.js";
import "./chunk-JF3RFB7K.js";
import "./chunk-DCVXHLBX.js";
import "./chunk-4ZNI2G73.js";
import "./chunk-S7M5FPLQ.js";

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