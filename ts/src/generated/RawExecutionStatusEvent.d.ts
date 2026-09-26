/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */

import type { HookTrigger } from "./HookTrigger";
import type { RawCompressionResultPayload } from "./RawCompressionResultPayload";
import type { RawHookExecutionPayload } from "./RawHookExecutionPayload";
import type { RawRetryAttemptPayload } from "./RawRetryAttemptPayload";
import type { RawRetryStatePayload } from "./RawRetryStatePayload";
import type { RawRoutingDecisionPayload } from "./RawRoutingDecisionPayload";
import type { RawTokenCountPayload } from "./RawTokenCountPayload";
import type { TaskComplexity } from "./TaskComplexity";

/**
 * One event on the `execution-status` Tauri channel, tagged by `type`.
 */
export type RawExecutionStatusEvent =
  | (RoutingDecisionEvent & {
      type: "routing_decision";
    })
  | (RetryAttemptEvent & {
      type: "retry_attempt";
    })
  | (CompressionEvent & {
      type: "compression";
    })
  | (TokenCountUpdateEvent & {
      type: "token_count_update";
    })
  | (HookExecutionEvent & {
      type: "hook_execution";
    })
  | (HookStartedEvent & {
      type: "hook_started";
    })
  | (StatusChangeEvent & {
      type: "status_change";
    });

/**
 * `routing_decision` event body.
 */
export interface RoutingDecisionEvent {
  decision: RawRoutingDecisionPayload;
  task_run_id: string;
  /**
   * Unix timestamp in milliseconds
   */
  timestamp: number;
  [k: string]: unknown;
}
/**
 * `retry_attempt` event body.
 */
export interface RetryAttemptEvent {
  attempt: RawRetryAttemptPayload;
  exhausted: boolean;
  next_retry_delay_ms: number | null;
  state: RawRetryStatePayload;
  task_run_id: string;
  /**
   * Unix timestamp in milliseconds
   */
  timestamp: number;
  [k: string]: unknown;
}
/**
 * `compression` event body.
 */
export interface CompressionEvent {
  current_token_count: RawTokenCountPayload;
  result: RawCompressionResultPayload;
  task_run_id: string;
  /**
   * Unix timestamp in milliseconds
   */
  timestamp: number;
  [k: string]: unknown;
}
/**
 * `token_count_update` event body.
 */
export interface TokenCountUpdateEvent {
  compression_imminent: boolean;
  task_run_id: string;
  threshold_percentage: number;
  /**
   * Unix timestamp in milliseconds
   */
  timestamp: number;
  token_count: RawTokenCountPayload;
  [k: string]: unknown;
}
/**
 * `hook_execution` event body.
 */
export interface HookExecutionEvent {
  result: RawHookExecutionPayload;
  task_run_id: string;
  /**
   * Unix timestamp in milliseconds
   */
  timestamp: number;
  [k: string]: unknown;
}
/**
 * `hook_started` event body.
 */
export interface HookStartedEvent {
  hook_id: string;
  hook_name: string;
  task_run_id: string;
  /**
   * Unix timestamp in milliseconds
   */
  timestamp: number;
  trigger: HookTrigger;
  [k: string]: unknown;
}
/**
 * `status_change` event body.
 */
export interface StatusChangeEvent {
  iteration: number;
  status: string;
  task_name: string | null;
  task_run_id: string;
  /**
   * Unix timestamp in milliseconds
   */
  timestamp: number;
  [k: string]: unknown;
}
