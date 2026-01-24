/**
 * Shared Render Log Types
 *
 * These types define the render logging schema used across:
 * - qontinui-runner (Tauri desktop app) - Component-level logging
 * - qontinui-web (Next.js web app) - Full DOM snapshot logging
 *
 * The runner reads both its own logs and web's logs for verification
 * in AI feedback loops.
 *
 * NOTE: Field names use snake_case to match Rust/Python JSON serialization.
 * This ensures compatibility across all services.
 */

// =============================================================================
// Common Types
// =============================================================================

/**
 * Trigger types for render log capture.
 * What caused the render log to be captured.
 */
export type RenderLogTrigger =
  | "mount" // Component/page mounted
  | "navigation" // Page navigation occurred
  | "mutation" // DOM mutation detected (web only)
  | "data_update" // Data changed (runner components)
  | "prop_change" // Props changed (runner components)
  | "manual" // Explicitly triggered capture
  | "interval"; // Periodic capture

/**
 * DOM mutation types (for web's MutationObserver).
 */
export type DomMutationType = "childList" | "attributes" | "characterData";

// =============================================================================
// Base Render Log Entry
// =============================================================================

/**
 * Base render log entry with common fields.
 * Extended by component logs and DOM snapshot logs.
 */
export interface RenderLogEntryBase {
  /** Unique identifier for this log entry */
  id: string;

  /** Unix timestamp in milliseconds when the capture occurred */
  timestamp: number;

  /** What triggered this capture */
  trigger: RenderLogTrigger;

  /** Session ID for grouping related captures */
  session_id?: string;
}

// =============================================================================
// Runner Component Render Log (Lightweight)
// =============================================================================

/**
 * Component-level render log entry.
 * Used by qontinui-runner to log what data components render.
 *
 * Components explicitly call logRender() with their rendered data.
 * This is lightweight and focused on verifying data display.
 */
export interface ComponentRenderLogEntry extends RenderLogEntryBase {
  /** Type discriminator */
  type: "component";

  /** Component name (e.g., "RunRecapTab", "StatusBanner") */
  component: string;

  /** The data that was rendered (component-specific structure) */
  data: Record<string, unknown>;

  /** Sections that are visible/expanded in the component */
  visible_sections?: string[];

  /** Associated task run ID if applicable */
  task_run_id?: number;
}

// =============================================================================
// Web DOM Snapshot Render Log (Comprehensive)
// =============================================================================

/**
 * Bounding rectangle for an element.
 */
export interface ElementRect {
  x: number;
  y: number;
  width: number;
  height: number;
}

/**
 * Single DOM element in a snapshot.
 * Recursive structure representing the DOM tree.
 */
export interface DomElementSnapshot {
  /** HTML tag name (lowercase) */
  tag: string;

  /** Element ID if present */
  id?: string;

  /** CSS class list */
  class_list: string[];

  /** Direct text content (not from children) */
  text_content?: string;

  /** Length of innerHTML (for size estimation) */
  inner_html_length?: number;

  /** Bounding rectangle relative to viewport */
  rect?: ElementRect;

  /** Whether the element is visible */
  is_visible: boolean;

  /** Computed opacity */
  opacity?: number;

  /** Computed display value */
  display?: string;

  /** Key HTML attributes */
  attributes: Record<string, string>;

  /** Selected computed styles */
  computed_styles?: Record<string, string>;

  /** Child elements (recursive) */
  children: DomElementSnapshot[];
}

/**
 * Form data extracted from the page.
 */
export interface FormSnapshot {
  id?: string;
  action?: string;
  method?: string;
  inputs: Array<{
    type: string;
    name: string;
    value: string;
  }>;
}

/**
 * Link data extracted from the page.
 */
export interface LinkSnapshot {
  href: string;
  text: string;
}

/**
 * Image data extracted from the page.
 */
export interface ImageSnapshot {
  src: string;
  alt?: string;
  width?: number;
  height?: number;
}

/**
 * Full DOM snapshot data.
 * Contains the entire page state for debugging.
 *
 * NOTE: The snapshot is stored as JSONB in the database, so field names
 * in the nested structure (root, forms, links, images) may vary by
 * implementation. The web frontend uses camelCase internally but that's
 * stored as-is in the JSONB field.
 */
export interface DomSnapshot {
  /** Root element tree (document.body) */
  root: DomElementSnapshot | null;

  /** Total number of elements in the tree */
  total_elements: number;

  /** Concatenated visible text (truncated) */
  visible_text?: string;

  /** Forms on the page */
  forms: FormSnapshot[];

  /** Links on the page */
  links: LinkSnapshot[];

  /** Images on the page */
  images: ImageSnapshot[];

  /** JavaScript console errors captured */
  errors: string[];

  /** JavaScript console warnings captured */
  warnings: string[];
}

/**
 * Full DOM snapshot render log entry.
 * Used by qontinui-web to capture entire page state.
 *
 * Automatically captured via MutationObserver on DOM changes.
 * Provides comprehensive debugging data.
 */
export interface DomSnapshotRenderLogEntry extends RenderLogEntryBase {
  /** Type discriminator */
  type: "dom_snapshot";

  /** Current page URL */
  page_url: string;

  /** Page title */
  page_title?: string;

  /** Type of DOM mutation that triggered capture (if mutation trigger) */
  mutation_type?: DomMutationType;

  /** CSS selector of the element that was mutated */
  target_selector?: string;

  /** The full DOM snapshot */
  snapshot: DomSnapshot;

  /** Viewport dimensions */
  viewport_width?: number;
  viewport_height?: number;

  /** Scroll position */
  scroll_x?: number;
  scroll_y?: number;

  /** Time taken to capture the snapshot in milliseconds */
  capture_duration_ms?: number;

  /** Number of elements captured */
  element_count?: number;

  /** User ID if authenticated (web only) */
  user_id?: string;
}

// =============================================================================
// Union Type
// =============================================================================

/**
 * Any render log entry.
 * Use the `type` field to discriminate.
 */
export type RenderLogEntry = ComponentRenderLogEntry | DomSnapshotRenderLogEntry;

// =============================================================================
// API Types (for REST endpoints)
// =============================================================================

/**
 * Request to create a render log entry (web API).
 */
export interface CreateRenderLogRequest {
  session_id: string;
  page_url: string;
  page_title?: string;
  trigger: RenderLogTrigger;
  mutation_type?: DomMutationType;
  target_selector?: string;
  snapshot: DomSnapshot;
  viewport_width?: number;
  viewport_height?: number;
  scroll_x?: number;
  scroll_y?: number;
  capture_duration_ms?: number;
  element_count?: number;
}

/**
 * Render log response from web API.
 * Includes database-generated fields.
 */
export interface RenderLogResponse {
  id: number;
  session_id: string;
  timestamp: string; // ISO 8601 datetime
  page_url: string;
  page_title?: string;
  trigger: RenderLogTrigger;
  mutation_type?: DomMutationType;
  target_selector?: string;
  snapshot: DomSnapshot;
  viewport_width?: number;
  viewport_height?: number;
  scroll_x?: number;
  scroll_y?: number;
  capture_duration_ms?: number;
  element_count?: number;
  user_id?: string;
}

/**
 * Summary of a render log (without full snapshot).
 */
export interface RenderLogSummary {
  id: number;
  session_id: string;
  timestamp: string;
  page_url: string;
  page_title?: string;
  trigger: RenderLogTrigger;
  mutation_type?: DomMutationType;
  element_count?: number;
  capture_duration_ms?: number;
}

/**
 * Paginated list of render logs.
 */
export interface RenderLogList {
  items: RenderLogSummary[];
  total: number;
  page: number;
  page_size: number;
  has_more: boolean;
}

/**
 * Render logging statistics.
 */
export interface RenderLogStats {
  enabled: boolean;
  total_snapshots: number;
  total_sessions: number;
  oldest_snapshot?: string;
  newest_snapshot?: string;
  storage_used_bytes: number;
  image_count: number;
}

// =============================================================================
// Type Guards
// =============================================================================

/**
 * Check if a render log entry is a component log.
 */
export function isComponentRenderLog(
  entry: RenderLogEntry
): entry is ComponentRenderLogEntry {
  return entry.type === "component";
}

/**
 * Check if a render log entry is a DOM snapshot log.
 */
export function isDomSnapshotRenderLog(
  entry: RenderLogEntry
): entry is DomSnapshotRenderLogEntry {
  return entry.type === "dom_snapshot";
}
