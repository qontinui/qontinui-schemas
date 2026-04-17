/**
 * Unified State Machine types for the Qontinui ecosystem.
 *
 * These types are the single source of truth for state machine data structures
 * used by both qontinui-web and qontinui-runner. They use snake_case to match
 * the API/database layer.
 *
 * Architecture:
 * - Config: A named state machine configuration (collection of states + transitions)
 * - State: A UI state defined by which elements are present
 * - Transition: A named action sequence that moves between states
 * - Action: A single UI interaction (click, type, navigate, etc.)
 *
 * Runtime navigation (pathfinding through states) is handled by the qontinui
 * Python library using multistate. The TypeScript pathfinding types here are
 * for the graph editor's path preview only.
 *
 * Most wire types in this module are generated from Rust (source of truth:
 * qontinui-schemas/rust/src/state_machine.rs). Do not edit those by hand —
 * regenerate via `just generate-types` (or the runner's generate_types.sh).
 * A small number of UI-only sugar types remain hand-authored below.
 */

// =============================================================================
// Action Types
// =============================================================================

export type { StandardActionType } from "../generated/StandardActionType";
export type { Point } from "../generated/Point";
export type { ScrollDirection } from "../generated/ScrollDirection";
export type { MouseButton } from "../generated/MouseButton";
export type { TransitionActionValue } from "../generated/TransitionActionValue";
export type { TransitionAction } from "../generated/TransitionAction";

// =============================================================================
// Domain Knowledge
// =============================================================================

export type { DomainKnowledge } from "../generated/DomainKnowledge";

// =============================================================================
// State Machine Config
// =============================================================================

export type { StateMachineConfig } from "../generated/StateMachineConfig";
export type { StateMachineConfigCreate } from "../generated/StateMachineConfigCreate";
export type { StateMachineConfigUpdate } from "../generated/StateMachineConfigUpdate";
export type { StateMachineConfigFull } from "../generated/StateMachineConfigFull";

// =============================================================================
// State
// =============================================================================

export type { StateMachineState } from "../generated/StateMachineState";
export type { StateMachineStateCreate } from "../generated/StateMachineStateCreate";
export type { StateMachineStateUpdate } from "../generated/StateMachineStateUpdate";

// =============================================================================
// Transition
// =============================================================================

export type { StateMachineTransition } from "../generated/StateMachineTransition";
export type { StateMachineTransitionCreate } from "../generated/StateMachineTransitionCreate";
export type { StateMachineTransitionUpdate } from "../generated/StateMachineTransitionUpdate";

// =============================================================================
// Pathfinding (Graph Editor Visualization)
// =============================================================================

/**
 * These pathfinding types are for the graph editor's path preview feature.
 * Runtime navigation uses the qontinui Python library (multistate) directly.
 */

export type { PathfindingRequest } from "../generated/PathfindingRequest";
export type { PathfindingStep } from "../generated/PathfindingStep";
export type { PathfindingResult } from "../generated/PathfindingResult";

// =============================================================================
// Execution Results (Runtime)
// =============================================================================

export type { TransitionExecutionResult } from "../generated/TransitionExecutionResult";
export type { NavigationResult } from "../generated/NavigationResult";
export type { ActiveStatesResult } from "../generated/ActiveStatesResult";

export type { TransitionInfo } from "../generated/TransitionInfo";
export type { AvailableTransitionsResult } from "../generated/AvailableTransitionsResult";

// =============================================================================
// Initial States
// =============================================================================

export type { InitialStatesSource } from "../generated/InitialStatesSource";
export type { InitialStateRef } from "../generated/InitialStateRef";
export type { ResolvedInitialStates } from "../generated/ResolvedInitialStates";
export type { ResolvedInitialStatesResult } from "../generated/ResolvedInitialStatesResult";

// =============================================================================
// Discovery
// =============================================================================

export type { DiscoveryStrategy } from "../generated/DiscoveryStrategy";

// =============================================================================
// Graph Display Types (ReactFlow)
// =============================================================================

// NOTE: `StateNodeData` is kept hand-authored because the generated DTO
// intentionally omits the `onStartElementDrag` callback (a UI-layer concern
// with no Rust analogue). Downstream ReactFlow nodes need the callback, so we
// can't silently substitute the generated shape. The rest of the fields match
// the generated `StateNodeData` one-for-one.

/**
 * Data passed to a state node in the ReactFlow graph editor.
 *
 * UI-only sugar: mirrors the generated `StateNodeData` but adds the
 * `onStartElementDrag` callback that the graph editor wires up at render time.
 */
export interface StateNodeData {
  stateId: string;
  name: string;
  elementCount: number;
  confidence: number;
  elementIds: string[];
  description: string | null;
  isBlocking: boolean;
  isSelected: boolean;
  isInitial: boolean;
  outgoingCount?: number;
  incomingCount?: number;
  isDropTarget?: boolean;
  onStartElementDrag?: (stateId: string, elementId: string) => void;
  /** Optional map of element ID → base64 thumbnail image (data URL or raw base64 PNG). */
  elementThumbnails?: Record<string, string>;
}

export type { TransitionEdgeData } from "../generated/TransitionEdgeData";

// =============================================================================
// Export/Import
// =============================================================================

export type { StateMachineExportFormat } from "../generated/StateMachineExportFormat";
