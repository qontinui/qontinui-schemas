/**
 * Auto-generated TypeScript types from qontinui-schemas
 * DO NOT EDIT - regenerate with: poetry run python scripts/generate_typescript.py
 */

import type { Region } from './geometry';

export enum WorkflowVisibility {
  PUBLIC = "public",
  INTERNAL = "internal",
  SYSTEM = "system",
}

export enum LogLevel {
  DEBUG = "debug",
  INFO = "info",
  WARNING = "warning",
  ERROR = "error",
}

export enum PositionName {
  TOPLEFT = "TOPLEFT",
  TOPMIDDLE = "TOPMIDDLE",
  TOPRIGHT = "TOPRIGHT",
  MIDDLELEFT = "MIDDLELEFT",
  MIDDLEMIDDLE = "MIDDLEMIDDLE",
  MIDDLERIGHT = "MIDDLERIGHT",
  BOTTOMLEFT = "BOTTOMLEFT",
  BOTTOMMIDDLE = "BOTTOMMIDDLE",
  BOTTOMRIGHT = "BOTTOMRIGHT",
}

export enum TransitionType {
  OUTGOING = "OutgoingTransition",
  INCOMING = "IncomingTransition",
}

export enum SearchMode {
  DEFAULT = "default",
  RAG = "rag",
  TEMPLATE = "template",
}

export enum MultiPatternMode {
  ALL = "all",
  COMBINED = "combined",
}

export interface LoggingOptions {
  beforeActionMessage?: string | null;
  afterActionMessage?: string | null;
  successMessage?: string | null;
  failureMessage?: string | null;
  logBeforeAction?: boolean | null;
  logAfterAction?: boolean | null;
  logOnSuccess?: boolean | null;
  logOnFailure?: boolean | null;
  beforeActionLevel?: LogLevel | null;
  afterActionLevel?: LogLevel | null;
  successLevel?: LogLevel | null;
  failureLevel?: LogLevel | null;
  logType?: string | null;
}

export interface RepetitionOptions {
  count?: number | null;
  pauseBetween?: number | null;
  stopOnSuccess?: boolean | null;
  stopOnFailure?: boolean | null;
}

export interface BaseActionSettings {
  pauseBeforeBegin?: number | null;
  pauseAfterEnd?: number | null;
  illustrate?: "YES" | "NO" | "USE_GLOBAL" | null;
  loggingOptions?: LoggingOptions | null;
}

export interface ExecutionSettings {
  timeout?: number | null;
  retryCount?: number | null;
  repetition?: RepetitionOptions | null;
}

export interface Position {
  /** Horizontal position: 0.0 = left, 1.0 = right, 0.5 = center */
  percentW?: number;
  /** Vertical position: 0.0 = top, 1.0 = bottom, 0.5 = center */
  percentH?: number;
  /** Optional named position for convenience */
  positionName?: PositionName | null;
}

export interface SearchRegion {
  /** Unique identifier for the search region */
  id: string;
  /** Human-readable name */
  name: string;
  /** X coordinate of top-left corner */
  x: number;
  /** Y coordinate of top-left corner */
  y: number;
  /** Width of the region */
  width: number;
  /** Height of the region */
  height: number;
  /** ID of StateImage for relative positioning */
  referenceImageId?: string | null;
  /** Position within referenced image region */
  position?: Position | null;
  /** X offset in pixels from reference position */
  offsetX?: number;
  /** Y offset in pixels from reference position */
  offsetY?: number;
}

export interface Pattern {
  /** Unique identifier for the pattern */
  id: string;
  /** Optional name for the pattern */
  name?: string | null;
  /** ID of ImageAsset in library (library is source of truth) */
  imageId?: string | null;
  /** Pattern-level search regions (precedence level 2) */
  searchRegions?: SearchRegion[];
  /** If true, pattern position is fixed on screen */
  fixed?: boolean;
  /** Similarity threshold (0.0-1.0) */
  similarity?: number | null;
  /** Click position within pattern (default: center 0.5, 0.5) */
  targetPosition?: Position | null;
  /** Pixel offset for click position X */
  offsetX?: number | null;
  /** Pixel offset for click position Y */
  offsetY?: number | null;
}

export interface StateImage {
  /** Unique identifier for the state image */
  id: string;
  /** Human-readable name */
  name: string;
  /** Multiple patterns for visual variations (e.g., normal, hover, clicked) */
  patterns?: Pattern[];
  /** If true, this image appears in multiple states */
  shared?: boolean;
  /** How the image was created (upload, pattern-optimization, image-extraction) */
  source?: string | null;
  /** Mock testing: probability image appears (0.0-1.0) */
  probability?: number | null;
  /** StateImage-level search regions (precedence level 3) */
  searchRegions?: SearchRegion[] | null;
  /** Monitor indices where this image should be searched */
  monitors?: number[] | null;
  /** How to search when StateImage has >1 pattern */
  ragMultiPatternMode?: MultiPatternMode | null;
  /** Search mode for this image (default, rag, template) */
  searchMode?: SearchMode | null;
  /** RAG image embedding vector */
  imageEmbedding?: number[] | null;
  /** RAG text embedding vector */
  textEmbedding?: number[] | null;
  /** OCR extracted text from image */
  ocrText?: string | null;
  /** Confidence score of OCR extraction */
  ocrConfidence?: number | null;
}

export interface StateRegion {
  /** Unique identifier for the region */
  id: string;
  /** Human-readable name */
  name: string;
  /** X coordinate of top-left corner */
  x: number;
  /** Y coordinate of top-left corner */
  y: number;
  /** Width of the region */
  width: number;
  /** Height of the region */
  height: number;
  /** Bounding box (alternative to x, y, width, height) */
  bounds?: Region | null;
  /** ID of StateImage for relative positioning */
  referenceImageId?: string | null;
  /** Position within referenced image region */
  position?: Position | null;
  /** X offset in pixels */
  offsetX?: number;
  /** Y offset in pixels */
  offsetY?: number;
  /** If true, used as a search region for StateImages */
  isSearchRegion?: boolean;
  /** Monitor indices where this region should be checked */
  monitors?: number[] | null;
}

export interface StateLocation {
  /** Unique identifier for the location */
  id: string;
  /** Human-readable name */
  name: string;
  /** X coordinate */
  x: number;
  /** Y coordinate */
  y: number;
  /** If true, location uses absolute coordinates */
  fixed?: boolean;
  /** If true, used as anchor point for relative positioning */
  anchor?: boolean;
  /** ID of StateImage for relative positioning */
  referenceImageId?: string | null;
  /** Position within referenced image region */
  position?: Position | null;
  /** X offset in pixels */
  offsetX?: number;
  /** Y offset in pixels */
  offsetY?: number;
  /** Width percentage (0.0-1.0) */
  percentW?: number | null;
  /** Height percentage (0.0-1.0) */
  percentH?: number | null;
  /** Position anchor type */
  anchorType?: string | null;
  /** Monitor indices where this location should be checked */
  monitors?: number[] | null;
  /** Additional metadata */
  metadata?: Record<string, any> | null;
}

export interface StateString {
  /** Unique identifier for the string */
  id: string;
  /** Human-readable name */
  name: string;
  /** The string value */
  value: string;
  /** If true, used for OCR verification */
  identifier?: boolean;
  /** If true, used as text to be typed */
  inputText?: boolean;
  /** If true, used for validation/expected text */
  expectedText?: boolean;
  /** If true, value is a regex pattern */
  regexPattern?: boolean;
  /** Monitor indices where this string should be checked */
  monitors?: number[] | null;
}

export interface StatePosition {
  /** X coordinate in graph */
  x: number;
  /** Y coordinate in graph */
  y: number;
}

export interface State {
  /** Unique identifier for the state */
  id: string;
  /** Human-readable name */
  name: string;
  /** Description of the state */
  description?: string;
  /** Images that identify this state */
  stateImages?: StateImage[];
  /** Regions associated with this state */
  regions?: StateRegion[];
  /** Locations associated with this state */
  locations?: StateLocation[];
  /** Strings associated with this state */
  strings?: StateString[];
  /** Position in the state machine graph */
  position: StatePosition;
  /** If true, this is an initial state */
  isInitial?: boolean;
  /** If true, this is a final state */
  isFinal?: boolean;
  /** Workflow IDs to run on state entry */
  entryActions?: string[] | null;
  /** Workflow IDs to run on state exit */
  exitActions?: string[] | null;
  /** Timeout for state detection in milliseconds */
  timeout?: number | null;
}

export interface TransitionCondition {
  /** Type of condition */
  type?: "always" | "image" | "time" | "custom";
  /** Image ID for image-based conditions */
  imageId?: string | null;
  /** Similarity threshold for image matching */
  threshold?: number | null;
  /** Time delay in milliseconds */
  timeDelay?: number | null;
  /** Custom condition script */
  customScript?: string | null;
}

export interface BaseTransition {
  /** Unique identifier for the transition */
  id: string;
  /** Type of transition */
  type: TransitionType;
  /** Workflow IDs to execute during transition */
  workflows?: string[];
  /** Transition timeout in milliseconds */
  timeout?: number;
  /** Number of retry attempts */
  retryCount?: number;
  /** Optional name */
  name?: string | null;
  /** Optional description */
  description?: string | null;
  /** Priority for handling multiple valid transitions */
  priority?: number | null;
}

export interface OutgoingTransition {
  /** Unique identifier for the transition */
  id: string;
  /** Always 'OutgoingTransition' */
  type?: "OutgoingTransition";
  /** Workflow IDs to execute during transition */
  workflows?: string[];
  /** Transition timeout in milliseconds */
  timeout?: number;
  /** Number of retry attempts */
  retryCount?: number;
  /** Optional name */
  name?: string | null;
  /** Optional description */
  description?: string | null;
  /** Priority for handling multiple valid transitions */
  priority?: number | null;
  /** Source state ID */
  fromState: string;
  /** Target state ID */
  toState?: string | null;
  /** If true, source state remains visible after transition */
  staysVisible?: boolean;
  /** State IDs to activate */
  activateStates?: string[];
  /** State IDs to deactivate */
  deactivateStates?: string[];
  /** Condition for this transition */
  condition?: TransitionCondition | null;
}

export interface IncomingTransition {
  /** Unique identifier for the transition */
  id: string;
  /** Always 'IncomingTransition' */
  type?: "IncomingTransition";
  /** Workflow IDs to execute during transition */
  workflows?: string[];
  /** Transition timeout in milliseconds */
  timeout?: number;
  /** Number of retry attempts */
  retryCount?: number;
  /** Optional name */
  name?: string | null;
  /** Optional description */
  description?: string | null;
  /** Priority for handling multiple valid transitions */
  priority?: number | null;
  /** Target state ID */
  toState: string;
  /** OutgoingTransition IDs that trigger this */
  executeAfter?: string[] | null;
}

export interface Connection {
  /** Target action ID */
  action: string;
  /** Connection type (main, error, success) */
  type: string;
  /** Input index on target action */
  index: number;
}

export interface WorkflowMetadata {
  created?: string | null;
  updated?: string | null;
  author?: string | null;
  description?: string | null;
  version?: string | null;
  /** Preferred visualization mode for the workflow editor */
  viewMode?: "sequential" | "graph" | null;
}

export interface Variables {
  local?: Record<string, any> | null;
  process?: Record<string, any> | null;
  global?: Record<string, any> | null;
}

export interface WorkflowSettings {
  timeout?: number | null;
  retryCount?: number | null;
  parallelExecution?: boolean | null;
  maxParallelActions?: number | null;
}

export interface Workflow {
  /** Unique workflow identifier */
  id: string;
  /** Human-readable workflow name */
  name: string;
  /** Human-readable description of what this workflow does */
  description?: string | null;
  /** Category for organizing workflows (e.g., 'Main', 'Testing') */
  category?: string | null;
  /** Workflow version (e.g., '1.0.0') */
  version: string;
  /** Workflow format (always 'graph') */
  format?: "graph";
  /** List of actions in workflow */
  actions: Action[];
  /** Action connections (REQUIRED) */
  connections: Connections;
  /** Workflow visibility level (public, internal, or system) */
  visibility?: WorkflowVisibility;
  /** Workflow variables (local, process, global) */
  variables?: Variables | null;
  /** Workflow-level execution settings */
  settings?: WorkflowSettings | null;
  /** Workflow metadata (author, description, etc.) */
  metadata?: WorkflowMetadata | null;
  /** Tags for categorizing workflows */
  tags?: string[] | null;
  /** Initial active states when workflow starts. Required for Main category workflows for model-based GUI automation. */
  initialStateIds?: string[] | null;
}

export interface Action {
  id: string;
  type: string;
  name?: string | null;
  config: Record<string, any>;
  base?: BaseActionSettings | null;
  execution?: ExecutionSettings | null;
  position?: any | null;
}

/** Union type for any transition */
export type Transition = OutgoingTransition | IncomingTransition;

/** Connections between actions in graph format */
export type Connections = Record<string, Record<string, Connection[][]>>;

/** Action outputs for graph connections */
export interface ActionOutputs {
  main?: Connection[][];
  success?: Connection[][];
  error?: Connection[][];
  true?: Connection[][];
  false?: Connection[][];
  [key: string]: Connection[][] | undefined;
}

/** Workflow connections mapping */
export type WorkflowConnections = Record<string, ActionOutputs>;
