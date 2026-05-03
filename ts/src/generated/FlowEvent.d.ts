/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with `just generate-types` or
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */

/**
 * Events emitted during flow execution for UI updates.
 *
 * Internally tagged with `"type"` and snake_case variant names.
 */
export type FlowEvent =
  | {
      flow_id: string;
      flow_name: string;
      instance_id: string;
      type: "flow_started";
      [k: string]: unknown;
    }
  | {
      instance_id: string;
      step_id: string;
      step_name: string;
      step_type: string;
      type: "step_started";
      [k: string]: unknown;
    }
  | {
      duration_ms: number;
      error?: string | null;
      instance_id: string;
      outputs: {
        [k: string]: unknown;
      };
      step_id: string;
      success: boolean;
      type: "step_completed";
      [k: string]: unknown;
    }
  | {
      duration_ms: number;
      error?: string | null;
      flow_id: string;
      instance_id: string;
      success: boolean;
      total_steps: number;
      type: "flow_completed";
      [k: string]: unknown;
    }
  | {
      instance_id: string;
      options: string[];
      prompt: string;
      step_id: string;
      type: "waiting_for_input";
      [k: string]: unknown;
    }
  | {
      completed: number;
      instance_id: string;
      step_id: string;
      total: number;
      type: "parallel_progress";
      [k: string]: unknown;
    }
  | {
      flow_id: string;
      instance_id: string;
      /**
       * Step the flow was paused at (`null` if no current step).
       */
      step_id?: string | null;
      type: "flow_paused";
      [k: string]: unknown;
    }
  | {
      flow_id: string;
      instance_id: string;
      /**
       * Step execution will resume from (`null` if no current step).
       */
      step_id?: string | null;
      type: "flow_resumed";
      [k: string]: unknown;
    };
