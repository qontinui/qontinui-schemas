/**
 * Auto-generated TypeScript types from qontinui-schemas
 * DO NOT EDIT - regenerate with: poetry run python scripts/generate_typescript.py
 */

export enum ExtractionStatus {
  PENDING = "pending",
  RUNNING = "running",
  COMPLETED = "completed",
  FAILED = "failed",
}

export enum StateType {
  PAGE = "page",
  NAVIGATION = "navigation",
  HEADER = "header",
  FOOTER = "footer",
  SIDEBAR = "sidebar",
  MAIN = "main",
  MODAL = "modal",
  DIALOG = "dialog",
  FORM = "form",
  CARD = "card",
  LIST = "list",
  TABLE = "table",
  MENU = "menu",
  TOOLBAR = "toolbar",
  TAB_PANEL = "tab_panel",
  ACCORDION = "accordion",
  CAROUSEL = "carousel",
  ALERT = "alert",
  TOAST = "toast",
  TOOLTIP = "tooltip",
  POPOVER = "popover",
  DROPDOWN = "dropdown",
  UNKNOWN = "unknown",
}

export enum TriggerType {
  CLICK = "click",
  HOVER = "hover",
  FOCUS = "focus",
  SCROLL = "scroll",
  NAVIGATION = "navigation",
  KEYBOARD = "keyboard",
  STATE_CHANGE = "state_change",
  UNKNOWN = "unknown",
}

export interface BoundingBox {
  /** X coordinate of top-left corner */
  x: number;
  /** Y coordinate of top-left corner */
  y: number;
  /** Width of the bounding box */
  width: number;
  /** Height of the bounding box */
  height: number;
}

export interface ExtractedElement {
  /** Unique identifier for the element */
  id: string;
  /** HTML tag name (e.g., 'button', 'a', 'input') */
  tagName: string;
  /** Semantic type (e.g., 'button', 'link', 'input') */
  elementType: string;
  /** Text content of the element */
  text?: string | null;
  /** Bounding box of the element */
  bbox: BoundingBox;
  /** CSS selector for the element */
  selector?: string | null;
  /** HTML attributes of the element */
  attributes?: Record<string, any>;
  /** Whether the element is interactive */
  isInteractive?: boolean;
  /** Confidence score for element detection */
  confidence?: number;
}

export interface ElementAnnotation {
  /** Unique identifier */
  id: string;
  /** Human-readable name for the element (OCR-based or derived) */
  name?: string | null;
  /** Type of element */
  elementType: string;
  /** Bounding box */
  bbox: BoundingBox;
  /** Text content */
  text?: string | null;
  /** CSS selector */
  selector?: string | null;
  confidence?: number;
}

export interface StateAnnotation {
  /** Unique identifier for the state */
  id: string;
  /** Human-readable name for the state */
  name: string;
  /** Bounding box of the state region */
  bbox: BoundingBox;
  /** Semantic type of the state */
  stateType?: StateType | string;
  /** IDs of elements contained in this state */
  elementIds?: string[];
  /** ID of the screenshot showing this state */
  screenshotId?: string | null;
  /** URL where this state was discovered */
  sourceUrl?: string | null;
  /** How the state was detected (semantic, heuristic, etc.) */
  detectionMethod?: string | null;
  /** Confidence score for state detection */
  confidence?: number;
  /** Additional metadata about the state */
  metadata?: Record<string, any>;
}

export interface InferredTransition {
  /** Unique identifier for the transition */
  id: string;
  /** ID of the source state */
  fromStateId: string;
  /** ID of the target state */
  toStateId: string;
  /** Type of trigger that causes the transition */
  triggerType?: TriggerType | string;
  /** CSS selector of the trigger element */
  triggerSelector?: string | null;
  /** Text of the trigger element (for non-image triggers) */
  triggerText?: string | null;
  /** Image URL of the trigger element (for image triggers) */
  triggerImage?: string | null;
  /** Whether the trigger is an image link */
  hasImage?: boolean;
  /** URL of the source page */
  sourceUrl?: string | null;
  /** URL of the target page */
  targetUrl?: string | null;
  /** Confidence score for transition detection */
  confidence?: number;
  /** Additional metadata about the transition */
  metadata?: Record<string, any>;
}

export interface ExtractionStats {
  /** Number of pages crawled */
  pagesExtracted?: number;
  /** Number of elements discovered */
  elementsFound?: number;
  /** Number of states discovered */
  statesFound?: number;
  /** Number of transitions discovered */
  transitionsFound?: number;
  /** Extraction ID where screenshots are stored */
  screenshotExtractionId?: string | null;
}

export interface ExtractionAnnotation {
  /** Unique identifier */
  id: string;
  /** ID of the parent extraction session */
  sessionId: string;
  /** ID of the screenshot for this annotation */
  screenshotId: string;
  /** URL of the page */
  sourceUrl: string;
  /** Viewport width when screenshot was taken */
  viewportWidth: number;
  /** Viewport height when screenshot was taken */
  viewportHeight: number;
  /** Elements discovered on this page */
  elements?: ElementAnnotation[];
  /** States discovered on this page */
  states?: StateAnnotation[];
  /** When this annotation was created */
  createdAt?: string | null;
  /** When this annotation was last updated */
  updatedAt?: string | null;
}

export interface ExtractionSessionConfig {
  /** Viewport sizes to use for extraction */
  viewports?: any[];
  /** Whether to capture hover states */
  captureHoverStates?: boolean;
  /** Whether to capture focus states */
  captureFocusStates?: boolean;
  /** Maximum crawl depth */
  maxDepth?: number;
  /** Maximum number of pages to crawl */
  maxPages?: number;
  /** Authentication cookies for the target site */
  authCookies?: Record<string, any> | null;
}

export interface ExtractionSession {
  /** Unique identifier for the session */
  id: string;
  /** ID of the project this extraction belongs to */
  projectId: string;
  /** URLs to extract from */
  sourceUrls: string[];
  /** Extraction configuration */
  config: ExtractionSessionConfig | Record<string, any>;
  /** Current status of the extraction */
  status: ExtractionStatus | string;
  /** Extraction statistics */
  stats: ExtractionStats | Record<string, any>;
  /** Error message if extraction failed */
  errorMessage?: string | null;
  /** When the session was created */
  createdAt: string | string;
  /** When extraction started */
  startedAt?: string | string | null;
  /** When extraction completed */
  completedAt?: string | string | null;
  /** User who created the session */
  createdBy?: string | null;
}

export interface ExtractionSessionDetail {
  /** Unique identifier for the session */
  id: string;
  /** ID of the project this extraction belongs to */
  projectId: string;
  /** URLs to extract from */
  sourceUrls: string[];
  /** Extraction configuration */
  config: ExtractionSessionConfig | Record<string, any>;
  /** Current status of the extraction */
  status: ExtractionStatus | string;
  /** Extraction statistics */
  stats: ExtractionStats | Record<string, any>;
  /** Error message if extraction failed */
  errorMessage?: string | null;
  /** When the session was created */
  createdAt: string | string;
  /** When extraction started */
  startedAt?: string | string | null;
  /** When extraction completed */
  completedAt?: string | string | null;
  /** User who created the session */
  createdBy?: string | null;
  /** Page annotations with states and elements */
  annotations?: ExtractionAnnotation[];
  /** Transitions discovered during extraction */
  transitions?: InferredTransition[];
}

export interface StateImportRequest {
  /** Specific state IDs to import (None = all) */
  stateIds?: string[] | null;
  /** Workflow to add states to */
  targetWorkflowId?: string | null;
}

export interface ImportResult {
  /** Number of states imported */
  importedStates: number;
  /** Number of transitions imported */
  importedTransitions: number;
  /** ID of the workflow states were added to */
  workflowId?: string | null;
}
