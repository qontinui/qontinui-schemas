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
  | {
      decision: RawRoutingDecisionPayload;
      task_run_id: string;
      /**
       * Unix timestamp in milliseconds
       */
      timestamp: number;
      type: "routing_decision";
    }
  | {
      attempt: RawRetryAttemptPayload;
      exhausted: boolean;
      next_retry_delay_ms: number | null;
      state: RawRetryStatePayload;
      task_run_id: string;
      /**
       * Unix timestamp in milliseconds
       */
      timestamp: number;
      type: "retry_attempt";
    }
  | {
      current_token_count: RawTokenCountPayload;
      result: RawCompressionResultPayload;
      task_run_id: string;
      /**
       * Unix timestamp in milliseconds
       */
      timestamp: number;
      type: "compression";
    }
  | {
      compression_imminent: boolean;
      task_run_id: string;
      threshold_percentage: number;
      /**
       * Unix timestamp in milliseconds
       */
      timestamp: number;
      token_count: RawTokenCountPayload;
      type: "token_count_update";
    }
  | {
      result: RawHookExecutionPayload;
      task_run_id: string;
      /**
       * Unix timestamp in milliseconds
       */
      timestamp: number;
      type: "hook_execution";
    }
  | {
      hook_id: string;
      hook_name: string;
      task_run_id: string;
      /**
       * Unix timestamp in milliseconds
       */
      timestamp: number;
      trigger: HookTrigger;
      type: "hook_started";
    }
  | {
      iteration: number;
      status: string;
      task_name: string | null;
      task_run_id: string;
      /**
       * Unix timestamp in milliseconds
       */
      timestamp: number;
      type: "status_change";
    };
