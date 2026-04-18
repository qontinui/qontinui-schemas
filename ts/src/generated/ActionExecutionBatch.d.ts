/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with `just generate-types` or
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */

import type { ActionExecutionCreate } from './ActionExecutionCreate';
import type { ActionStatus } from './ActionStatus';
import type { ErrorType } from './ErrorType';
import type { ExecutionActionType } from './ExecutionActionType';
import type { ExecutionMatchLocation } from './ExecutionMatchLocation';
import type { LLMMetrics } from './LLMMetrics';

/**
 * Batch wrapper for reporting multiple action executions in one request.
 */
export interface ActionExecutionBatch {
  /**
   * Actions to record (Python enforces `1..=100`; enforced by the backend).
   */
  actions: ActionExecutionCreate[];
}
