/**
 * Tree Event Types
 *
 * Execution logging events emitted by qontinui during automation runs.
 * Source of truth: `qontinui-schemas/rust/src/tree_events.rs`.
 *
 * Used by:
 * - qontinui (Python library) — emits events during execution
 * - qontinui-runner (Tauri app) — receives and displays events
 * - qontinui-web (backend + frontend) — stores, forwards, and displays events
 *
 * Schema shape: workflows contain actions; actions can nest (`GO_TO_STATE`,
 * `RUN_WORKFLOW`); each event captures a node state change with rich
 * metadata.
 */

// Enums
export type { NodeType } from "../generated/NodeType";
export type { NodeStatus } from "../generated/NodeStatus";
export type { TreeEventType } from "../generated/TreeEventType";
export type { ActionType } from "../generated/ActionType";

// Nested metadata
export type { MatchLocation } from "../generated/MatchLocation";
export type { TopMatch } from "../generated/TopMatch";
export type { RuntimeData } from "../generated/RuntimeData";
export type { StateContext } from "../generated/StateContext";
export type { TimingInfo } from "../generated/TimingInfo";
export type { Outcome } from "../generated/Outcome";

// Main models
export type { NodeMetadata } from "../generated/NodeMetadata";
export type { TreeNode } from "../generated/TreeNode";
export type { PathElement } from "../generated/PathElement";
export type { TreeEvent } from "../generated/TreeEvent";

// Display + API response
export type { DisplayNode } from "../generated/DisplayNode";
export type { TreeEventCreate } from "../generated/TreeEventCreate";
export type { TreeEventResponse } from "../generated/TreeEventResponse";
export type { TreeEventListResponse } from "../generated/TreeEventListResponse";
export type { ExecutionTreeResponse } from "../generated/ExecutionTreeResponse";
