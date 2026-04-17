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
  actionName: string;
  actionType: ExecutionActionType;
  /**
   * IDs of states active when the action ran.
   */
  activeStates?: string[] | null;
  /**
   * ISO 8601 timestamp when the action completed.
   */
  completedAt: string;
  /**
   * Confidence score of the match in the range `[0.0, 1.0]`.
   */
  confidenceScore?: number | null;
  /**
   * Action duration in milliseconds.
   */
  durationMs: number;
  /**
   * Error message if the action failed.
   */
  errorMessage?: string | null;
  /**
   * Captured stack trace, if any.
   */
  errorStack?: string | null;
  /**
   * Category of error, if any.
   */
  errorType?: ErrorType | null;
  /**
   * Source state ID for state-machine actions.
   */
  fromState?: string | null;
  /**
   * Opaque input data captured for the action.
   */
  inputData?: {
    [k: string]: unknown;
  } | null;
  /**
   * LLM token and cost metrics, if the action used an LLM.
   */
  llmMetrics?: LLMMetrics | null;
  /**
   * Pixel location of the match.
   */
  matchLocation?: ExecutionMatchLocation | null;
  /**
   * Opaque additional metadata.
   */
  metadata?: {
    [k: string]: unknown;
  } | null;
  /**
   * Opaque output data produced by the action.
   */
  outputData?: {
    [k: string]: unknown;
  } | null;
  /**
   * ID of a parent action, if this is a sub-action.
   */
  parentActionId?: string | null;
  /**
   * Parent action ID for child actions within a sequence.
   */
  parentId?: string | null;
  /**
   * ID of the pattern the action targeted.
   */
  patternId?: string | null;
  /**
   * Human-readable pattern name.
   */
  patternName?: string | null;
  /**
   * ID of a screenshot associated with the action.
   */
  screenshotId?: string | null;
  /**
   * Zero-based sequence number of the action within the run.
   */
  sequenceNumber: number;
  /**
   * Span type for tracing (e.g., `"llm"`, `"tool"`, `"agent"`).
   */
  spanType?: string | null;
  /**
   * ISO 8601 timestamp when the action started.
   */
  startedAt: string;
  status: ActionStatus;
  /**
   * Destination state ID for state-machine actions.
   */
  toState?: string | null;
  /**
   * Trace ID correlating related actions.
   */
  traceId?: string | null;
  [k: string]: unknown;
}
