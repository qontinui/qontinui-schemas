/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with `just generate-types` or
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */

import type { ActionStatus } from './ActionStatus';
import type { ErrorType } from './ErrorType';
import type { ExecutionActionType } from './ExecutionActionType';
import type { ExecutionMatchLocation } from './ExecutionMatchLocation';
import type { LLMMetrics } from './LLMMetrics';

/**
 * Request payload for reporting a single action execution.
 *
 * Usually submitted as part of a batch; see [`ActionExecutionResponse`].
 */
export interface ActionExecutionCreate {
  /**
   * Human-readable action name.
   */
  action_name: string;
  action_type: ExecutionActionType;
  /**
   * IDs of states active when the action ran.
   */
  active_states?: string[] | null;
  /**
   * ISO 8601 timestamp when the action completed.
   */
  completed_at: string;
  /**
   * Confidence score of the match in the range `[0.0, 1.0]`.
   */
  confidence_score?: number | null;
  /**
   * Action duration in milliseconds.
   */
  duration_ms: number;
  /**
   * Error message if the action failed.
   */
  error_message?: string | null;
  /**
   * Captured stack trace, if any.
   */
  error_stack?: string | null;
  /**
   * Category of error, if any.
   */
  error_type?: ErrorType | null;
  /**
   * Source state ID for state-machine actions.
   */
  from_state?: string | null;
  /**
   * Opaque input data captured for the action.
   */
  input_data?: {
    [k: string]: unknown;
  } | null;
  /**
   * LLM token and cost metrics, if the action used an LLM.
   */
  llm_metrics?: LLMMetrics | null;
  /**
   * Pixel location of the match.
   */
  match_location?: ExecutionMatchLocation | null;
  /**
   * Opaque additional metadata.
   */
  metadata?: {
    [k: string]: unknown;
  } | null;
  /**
   * Opaque output data produced by the action.
   */
  output_data?: {
    [k: string]: unknown;
  } | null;
  /**
   * ID of a parent action, if this is a sub-action.
   */
  parent_action_id?: string | null;
  /**
   * Parent action ID for child actions within a sequence.
   */
  parent_id?: string | null;
  /**
   * ID of the pattern the action targeted.
   */
  pattern_id?: string | null;
  /**
   * Human-readable pattern name.
   */
  pattern_name?: string | null;
  /**
   * ID of a screenshot associated with the action.
   */
  screenshot_id?: string | null;
  /**
   * Zero-based sequence number of the action within the run.
   */
  sequence_number: number;
  /**
   * Span type for tracing (e.g., `"llm"`, `"tool"`, `"agent"`).
   */
  span_type?: string | null;
  /**
   * ISO 8601 timestamp when the action started.
   */
  started_at: string;
  status: ActionStatus;
  /**
   * Destination state ID for state-machine actions.
   */
  to_state?: string | null;
  /**
   * Trace ID correlating related actions.
   */
  trace_id?: string | null;
  [k: string]: unknown;
}
