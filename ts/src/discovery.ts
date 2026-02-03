/**
 * Unified State Discovery Result types.
 *
 * These types represent the output of state discovery from any source:
 * - Playwright (web extraction)
 * - UI Bridge (render log analysis)
 * - Recording (user session recording)
 * - Vision (screenshot analysis)
 * - Manual (user-defined)
 */

// =============================================================================
// Enums
// =============================================================================

export type DiscoverySourceType =
  | "playwright"
  | "ui_bridge"
  | "recording"
  | "vision"
  | "manual";

export type TransitionTriggerType =
  | "click"
  | "type"
  | "scroll"
  | "hover"
  | "custom";

// =============================================================================
// Core Components
// =============================================================================

export interface DiscoveryBoundingBox {
  x: number;
  y: number;
  width: number;
  height: number;
}

export interface DiscoveryTransitionTrigger {
  type: TransitionTriggerType;
  image_id?: string;
  element_id?: string;
  selector?: string;
  value?: string;
}

// =============================================================================
// State Machine Components
// =============================================================================

export interface DiscoveredStateImage {
  id: string;
  screenshot_id?: string;
  screenshot_url?: string;
  bbox: DiscoveryBoundingBox;
  pixel_hash?: string;
  state_id?: string;
  element_type?: string;
  label?: string;
  confidence: number;
  metadata?: Record<string, unknown>;
}

export interface DiscoveredState {
  id: string;
  name: string;
  image_ids: string[];
  render_ids: string[];
  element_ids: string[];
  confidence: number;
  description?: string;
  metadata?: Record<string, unknown>;
}

export interface DiscoveredTransition {
  id: string;
  from_state_id: string;
  to_state_id: string;
  trigger?: DiscoveryTransitionTrigger;
  confidence: number;
  metadata?: Record<string, unknown>;
}

// =============================================================================
// Complete Result
// =============================================================================

export interface StateDiscoveryResult {
  id: string;
  project_id: string;
  name: string;
  description?: string;
  source_type: DiscoverySourceType;
  source_session_id?: string;
  discovery_strategy?: string;
  images: DiscoveredStateImage[];
  states: DiscoveredState[];
  transitions: DiscoveredTransition[];
  element_to_renders: Record<string, string[]>;
  image_count: number;
  state_count: number;
  transition_count: number;
  render_count: number;
  unique_element_count: number;
  confidence: number;
  discovery_metadata: Record<string, unknown>;
  created_at: string;
  updated_at: string;
}

export interface StateDiscoveryResultSummary {
  id: string;
  project_id: string;
  name: string;
  description?: string;
  source_type: DiscoverySourceType;
  discovery_strategy?: string;
  image_count: number;
  state_count: number;
  transition_count: number;
  confidence: number;
  created_at: string;
}

export interface StateDiscoveryResultListResponse {
  items: StateDiscoveryResultSummary[];
  total: number;
}

// =============================================================================
// API Schemas
// =============================================================================

export interface StateDiscoveryResultCreate {
  name: string;
  description?: string;
  source_type: DiscoverySourceType;
  source_session_id?: string;
  discovery_strategy?: string;
  images: DiscoveredStateImage[];
  states: DiscoveredState[];
  transitions: DiscoveredTransition[];
  element_to_renders: Record<string, string[]>;
  confidence: number;
  discovery_metadata: Record<string, unknown>;
}

export interface StateDiscoveryResultUpdate {
  name?: string;
  description?: string;
  images?: DiscoveredStateImage[];
  states?: DiscoveredState[];
  transitions?: DiscoveredTransition[];
  discovery_metadata?: Record<string, unknown>;
}

// =============================================================================
// Export/Import
// =============================================================================

export interface StateMachineExport {
  version: string;
  name: string;
  description?: string;
  source_type: DiscoverySourceType | string;
  images: DiscoveredStateImage[];
  states: DiscoveredState[];
  transitions: DiscoveredTransition[];
  element_to_renders: Record<string, string[]>;
  metadata: Record<string, unknown>;
}

export interface StateMachineImport {
  state_machine: StateMachineExport;
  name?: string;
}

// =============================================================================
// Display Helpers
// =============================================================================

export const SOURCE_TYPE_LABELS: Record<DiscoverySourceType, string> = {
  playwright: "Web Extraction",
  ui_bridge: "UI Bridge",
  recording: "Recording",
  vision: "Vision",
  manual: "Manual",
};

export const SOURCE_TYPE_COLORS: Record<DiscoverySourceType, string> = {
  playwright: "bg-blue-500/20 text-blue-400 border-blue-500/30",
  ui_bridge: "bg-purple-500/20 text-purple-400 border-purple-500/30",
  recording: "bg-green-500/20 text-green-400 border-green-500/30",
  vision: "bg-orange-500/20 text-orange-400 border-orange-500/30",
  manual: "bg-gray-500/20 text-gray-400 border-gray-500/30",
};

// =============================================================================
// Converter Functions (snake_case to camelCase)
// =============================================================================

export function toStateDiscoveryResult(
  data: Record<string, unknown>
): StateDiscoveryResult {
  return {
    id: data.id as string,
    project_id: data.project_id as string,
    name: data.name as string,
    description: data.description as string | undefined,
    source_type: data.source_type as DiscoverySourceType,
    source_session_id: data.source_session_id as string | undefined,
    discovery_strategy: data.discovery_strategy as string | undefined,
    images: ((data.images as unknown[]) || []).map(toDiscoveredStateImage),
    states: ((data.states as unknown[]) || []).map(toDiscoveredState),
    transitions: ((data.transitions as unknown[]) || []).map(
      toDiscoveredTransition
    ),
    element_to_renders:
      (data.element_to_renders as Record<string, string[]>) || {},
    image_count: data.image_count as number,
    state_count: data.state_count as number,
    transition_count: data.transition_count as number,
    render_count: data.render_count as number,
    unique_element_count: data.unique_element_count as number,
    confidence: data.confidence as number,
    discovery_metadata:
      (data.discovery_metadata as Record<string, unknown>) || {},
    created_at: data.created_at as string,
    updated_at: data.updated_at as string,
  };
}

export function toDiscoveredStateImage(data: unknown): DiscoveredStateImage {
  const d = data as Record<string, unknown>;
  return {
    id: d.id as string,
    screenshot_id: d.screenshot_id as string | undefined,
    screenshot_url: d.screenshot_url as string | undefined,
    bbox: d.bbox as DiscoveryBoundingBox,
    pixel_hash: d.pixel_hash as string | undefined,
    state_id: d.state_id as string | undefined,
    element_type: d.element_type as string | undefined,
    label: d.label as string | undefined,
    confidence: (d.confidence as number) ?? 1.0,
    metadata: d.metadata as Record<string, unknown> | undefined,
  };
}

export function toDiscoveredState(data: unknown): DiscoveredState {
  const d = data as Record<string, unknown>;
  return {
    id: d.id as string,
    name: d.name as string,
    image_ids: (d.image_ids as string[]) || [],
    render_ids: (d.render_ids as string[]) || [],
    element_ids: (d.element_ids as string[]) || [],
    confidence: (d.confidence as number) ?? 1.0,
    description: d.description as string | undefined,
    metadata: d.metadata as Record<string, unknown> | undefined,
  };
}

export function toDiscoveredTransition(data: unknown): DiscoveredTransition {
  const d = data as Record<string, unknown>;
  const trigger = d.trigger as Record<string, unknown> | undefined;
  return {
    id: d.id as string,
    from_state_id: d.from_state_id as string,
    to_state_id: d.to_state_id as string,
    trigger: trigger
      ? {
          type: trigger.type as TransitionTriggerType,
          image_id: trigger.image_id as string | undefined,
          element_id: trigger.element_id as string | undefined,
          selector: trigger.selector as string | undefined,
          value: trigger.value as string | undefined,
        }
      : undefined,
    confidence: (d.confidence as number) ?? 1.0,
    metadata: d.metadata as Record<string, unknown> | undefined,
  };
}

export function toStateDiscoveryResultSummary(
  data: Record<string, unknown>
): StateDiscoveryResultSummary {
  return {
    id: data.id as string,
    project_id: data.project_id as string,
    name: data.name as string,
    description: data.description as string | undefined,
    source_type: data.source_type as DiscoverySourceType,
    discovery_strategy: data.discovery_strategy as string | undefined,
    image_count: data.image_count as number,
    state_count: data.state_count as number,
    transition_count: data.transition_count as number,
    confidence: data.confidence as number,
    created_at: data.created_at as string,
  };
}
