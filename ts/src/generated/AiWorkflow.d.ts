/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with `just generate-types` or
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */

import type { ExecutionStep } from './ExecutionStep';

/**
 * A saved AI Builder workflow template.
 *
 * Users create these in the Automation Builder tab. Each workflow carries an
 * ordered list of `ExecutionStep`s plus metadata (goal, category, tags,
 * context references). The runner persists them in `ai_workflows.json` and
 * surfaces them through Tauri commands and the HTTP API.
 */
export interface AiWorkflow {
  /**
   * Whether to auto-include contexts based on task mentions (default: true).
   */
  auto_include_contexts: boolean;
  /**
   * Whether to capture input for coordinate validation.
   */
  capture_input_validation?: boolean;
  /**
   * Category for organization (e.g., `"Testing"`, `"Development"`).
   */
  category?: string;
  /**
   * Manually added context IDs.
   */
  context_ids?: string[];
  /**
   * ISO 8601 timestamp of creation.
   */
  created_at: string;
  /**
   * Optional description of what this workflow does.
   */
  description?: string;
  /**
   * Disabled context IDs (excluded from auto-include).
   */
  disabled_context_ids?: string[];
  /**
   * The goal/objective for this workflow.
   */
  goal?: string;
  /**
   * Unique identifier (UUID v4).
   */
  id: string;
  /**
   * Maximum iterations for the AI loop.
   * `None` (omitted) means no cap — loop until success or explicit stop.
   */
  max_iterations?: number | null;
  /**
   * ISO 8601 timestamp of last modification.
   */
  modified_at: string;
  /**
   * Display name for the workflow.
   */
  name: string;
  /**
   * The ordered list of execution steps.
   */
  steps?: ExecutionStep[];
  /**
   * Tags for filtering/searching.
   */
  tags?: string[];
  [k: string]: unknown;
}
