/**
 * Unified State Discovery Result types.
 *
 * These types represent the output of state discovery from any source:
 * - Playwright (web extraction)
 * - UI Bridge (render log analysis)
 * - Recording (user session recording)
 * - Vision (screenshot analysis)
 * - Manual (user-defined)
 *
 * All types use camelCase field names to match the generated types from Rust
 * schemas. Converter functions are provided to transform snake_case API
 * responses into these typed objects.
 */

// =============================================================================
// Re-exports from generated types
// =============================================================================

export type { DiscoverySourceType } from "./generated/DiscoverySourceType";
export type { TransitionTriggerType } from "./generated/TransitionTriggerType";
export type { DiscoveryBoundingBox } from "./generated/DiscoveryBoundingBox";
export type { DiscoveryTransitionTrigger } from "./generated/DiscoveryTransitionTrigger";
export type { DiscoveredStateImage } from "./generated/DiscoveredStateImage";
export type { DiscoveredState } from "./generated/DiscoveredState";
export type { DiscoveredTransition } from "./generated/DiscoveredTransition";
export type { StateDiscoveryResult } from "./generated/StateDiscoveryResult";
export type { StateDiscoveryResultSummary } from "./generated/StateDiscoveryResultSummary";
export type { StateDiscoveryResultListResponse } from "./generated/StateDiscoveryResultListResponse";
export type { StateDiscoveryResultCreate } from "./generated/StateDiscoveryResultCreate";
export type { StateDiscoveryResultUpdate } from "./generated/StateDiscoveryResultUpdate";

// =============================================================================
// Export/Import
// =============================================================================

export type { StateMachineExport } from "./generated/StateMachineExport";
export type { StateMachineImport } from "./generated/StateMachineImport";

import type { DiscoverySourceType } from "./generated/DiscoverySourceType";
import type { DiscoveredStateImage } from "./generated/DiscoveredStateImage";
import type { DiscoveredState } from "./generated/DiscoveredState";
import type { DiscoveredTransition } from "./generated/DiscoveredTransition";

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
// Converter Functions (snake_case API responses -> camelCase typed objects)
// =============================================================================

import type { StateDiscoveryResult } from "./generated/StateDiscoveryResult";
import type { StateDiscoveryResultSummary } from "./generated/StateDiscoveryResultSummary";
import type { DiscoveryBoundingBox } from "./generated/DiscoveryBoundingBox";
import type { DiscoveryTransitionTrigger } from "./generated/DiscoveryTransitionTrigger";
import type { TransitionTriggerType } from "./generated/TransitionTriggerType";

export function toStateDiscoveryResult(
  data: Record<string, unknown>,
): StateDiscoveryResult {
  return {
    id: data.id as string,
    projectId: (data.projectId ?? data.project_id) as string,
    name: data.name as string,
    description: data.description as string | undefined,
    sourceType: (data.sourceType ?? data.source_type) as DiscoverySourceType,
    sourceSessionId: (data.sourceSessionId ?? data.source_session_id) as
      | string
      | undefined,
    discoveryStrategy: (data.discoveryStrategy ?? data.discovery_strategy) as
      | string
      | undefined,
    images: ((data.images as unknown[]) || []).map(toDiscoveredStateImage),
    states: ((data.states as unknown[]) || []).map(toDiscoveredState),
    transitions: ((data.transitions as unknown[]) || []).map(
      toDiscoveredTransition,
    ),
    elementToRenders:
      ((data.elementToRenders ?? data.element_to_renders) as Record<
        string,
        string[]
      >) || {},
    imageCount: (data.imageCount ?? data.image_count) as number,
    stateCount: (data.stateCount ?? data.state_count) as number,
    transitionCount: (data.transitionCount ?? data.transition_count) as number,
    renderCount: (data.renderCount ?? data.render_count) as number,
    uniqueElementCount: (data.uniqueElementCount ??
      data.unique_element_count) as number,
    confidence: data.confidence as number,
    discoveryMetadata:
      ((data.discoveryMetadata ?? data.discovery_metadata) as Record<
        string,
        unknown
      >) || {},
    createdAt: (data.createdAt ?? data.created_at) as string,
    updatedAt: (data.updatedAt ?? data.updated_at) as string,
  };
}

export function toDiscoveredStateImage(data: unknown): DiscoveredStateImage {
  const d = data as Record<string, unknown>;
  return {
    id: d.id as string,
    screenshotId: (d.screenshotId ?? d.screenshot_id) as string | undefined,
    screenshotUrl: (d.screenshotUrl ?? d.screenshot_url) as string | undefined,
    bbox: d.bbox as DiscoveryBoundingBox,
    pixelHash: (d.pixelHash ?? d.pixel_hash) as string | undefined,
    stateId: (d.stateId ?? d.state_id) as string | undefined,
    elementType: (d.elementType ?? d.element_type) as string | undefined,
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
    imageIds: ((d.imageIds ?? d.image_ids) as string[]) || [],
    renderIds: ((d.renderIds ?? d.render_ids) as string[]) || [],
    elementIds: ((d.elementIds ?? d.element_ids) as string[]) || [],
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
    fromStateId: (d.fromStateId ?? d.from_state_id) as string,
    toStateId: (d.toStateId ?? d.to_state_id) as string,
    trigger: trigger
      ? {
          type: (trigger.type as TransitionTriggerType) ?? "click",
          imageId: (trigger.imageId ?? trigger.image_id) as string | undefined,
          elementId: (trigger.elementId ?? trigger.element_id) as
            | string
            | undefined,
          selector: trigger.selector as string | undefined,
          value: trigger.value as string | undefined,
        }
      : undefined,
    confidence: (d.confidence as number) ?? 1.0,
    metadata: d.metadata as Record<string, unknown> | undefined,
  };
}

export function toStateDiscoveryResultSummary(
  data: Record<string, unknown>,
): StateDiscoveryResultSummary {
  return {
    id: data.id as string,
    projectId: (data.projectId ?? data.project_id) as string,
    name: data.name as string,
    description: data.description as string | undefined,
    sourceType: (data.sourceType ?? data.source_type) as DiscoverySourceType,
    discoveryStrategy: (data.discoveryStrategy ?? data.discovery_strategy) as
      | string
      | undefined,
    imageCount: (data.imageCount ?? data.image_count) as number,
    stateCount: (data.stateCount ?? data.state_count) as number,
    transitionCount: (data.transitionCount ?? data.transition_count) as number,
    confidence: data.confidence as number,
    createdAt: (data.createdAt ?? data.created_at) as string,
  };
}
