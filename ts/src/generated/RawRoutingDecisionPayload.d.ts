/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */

import type { TaskComplexity } from "./TaskComplexity";

/**
 * Routing decision payload.
 */
export interface RawRoutingDecisionPayload {
  /**
   * The assessed complexity level
   */
  complexity: TaskComplexity;
  /**
   * Confidence in the assessment (0-1)
   */
  confidence: number;
  /**
   * Criteria count if analyzed
   */
  criteria_count: number | null;
  /**
   * Factors that contributed to this assessment
   */
  factors: string[];
  /**
   * File count if analyzed
   */
  file_count: number | null;
  /**
   * Task prompt preview (truncated)
   */
  prompt_preview: string | null;
  /**
   * The model selected
   */
  selected_model: string;
}
