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
export type DiscoverySourceType = "playwright" | "ui_bridge" | "recording" | "vision" | "manual";
export type TransitionTriggerType = "click" | "type" | "scroll" | "hover" | "custom";
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
export declare const SOURCE_TYPE_LABELS: Record<DiscoverySourceType, string>;
export declare const SOURCE_TYPE_COLORS: Record<DiscoverySourceType, string>;
export declare function toStateDiscoveryResult(data: Record<string, unknown>): StateDiscoveryResult;
export declare function toDiscoveredStateImage(data: unknown): DiscoveredStateImage;
export declare function toDiscoveredState(data: unknown): DiscoveredState;
export declare function toDiscoveredTransition(data: unknown): DiscoveredTransition;
export declare function toStateDiscoveryResultSummary(data: Record<string, unknown>): StateDiscoveryResultSummary;
//# sourceMappingURL=discovery.d.ts.map