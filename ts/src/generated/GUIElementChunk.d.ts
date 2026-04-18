/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with `just generate-types` or
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */

import type { BoundingBox } from './BoundingBox';
import type { ElementType } from './ElementType';

/**
 * Complete representation of a GUI element chunk for RAG retrieval.
 *
 * Contains all information needed for storing, searching, and retrieving GUI
 * elements from a vector database. Mirrors `rag.models.GUIElementChunk`.
 */
export interface GUIElementChunk {
  area: number;
  aspect_ratio: number;
  average_brightness: number;
  bounding_box?: BoundingBox | null;
  color_histogram: number[];
  contrast_ratio: number;
  /**
   * ISO 8601 UTC timestamp (`Z` suffix).
   */
  created_at?: string | null;
  depth_in_hierarchy: number;
  /**
   * Dominant RGB colour tuples.
   */
  dominant_colors: number[][];
  edge_density: number;
  /**
   * More specific classification.
   */
  element_subtype: string;
  element_type?: ElementType & string;
  /**
   * How the element was extracted (e.g. `"manual"`).
   */
  extraction_method: string;
  has_text: boolean;
  height: number;
  /**
   * Unique identifier (UUID).
   */
  id: string;
  /**
   * Dense vector for visual features.
   */
  image_embedding?: number[] | null;
  /**
   * `"click"`, `"type"`, `"select"`, etc.
   */
  interaction_type: string;
  /**
   * Is this element required for state identification?
   */
  is_defining_element: boolean;
  is_enabled: boolean;
  /**
   * Does the element stay in the same position?
   */
  is_fixed_position: boolean;
  is_focused: boolean;
  is_interactive: boolean;
  /**
   * Is this element optional in the state?
   */
  is_optional_element: boolean;
  is_selected: boolean;
  /**
   * Is element shared across multiple states?
   */
  is_shared: boolean;
  ocr_confidence: number;
  ocr_text: string;
  /**
   * Parent container / region identifier.
   */
  parent_region?: string | null;
  /**
   * `"windows"`, `"macos"`, `"linux"`, `"web"`.
   */
  platform: string;
  /**
   * `"top-left"`, `"top-right"`, `"bottom-left"`, `"bottom-right"`, or `"center"`.
   */
  position_quadrant: string;
  /**
   * Probability of finding this element (0-1).
   */
  probability: number;
  /**
   * Search-region identifier.
   */
  search_region_id?: string | null;
  /**
   * Expected action when interacted with.
   */
  semantic_action: string;
  /**
   * `"save"`, `"cancel"`, `"submit"`, `"close"`, etc.
   */
  semantic_role: string;
  sibling_count: number;
  /**
   * Threshold for matching (0-1).
   */
  similarity_threshold: number;
  /**
   * Application name / identifier.
   */
  source_app: string;
  /**
   * Screenshot identifier.
   */
  source_screenshot_id?: string | null;
  /**
   * State-machine state ID.
   */
  source_state_id?: string | null;
  /**
   * Associated state ID.
   */
  state_id?: string | null;
  state_name: string;
  /**
   * UI toolkit / style (e.g. `"material"`, `"fluent"`, `"gtk"`).
   */
  style_family: string;
  /**
   * Human-readable description for embedding.
   */
  text_description: string;
  /**
   * Dense vector for text.
   */
  text_embedding?: number[] | null;
  text_length: number;
  /**
   * ISO 8601 UTC timestamp (`Z` suffix).
   */
  updated_at?: string | null;
  /**
   * `"normal"`, `"hover"`, `"pressed"`, `"disabled"`.
   */
  visual_state: string;
  width: number;
}
