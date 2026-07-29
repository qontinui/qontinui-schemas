/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with `just generate-types` or
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */

/**
 * Action to take between iterations.
 */
export type BetweenIterations =
  | {
      /**
       * Whether to also rebuild the target before restarting.
       */
      rebuild: boolean;
      type: "restart_runner";
      [k: string]: unknown;
    }
  | {
      /**
       * Whether to also rebuild when a signal triggers the restart.
       */
      rebuild: boolean;
      type: "restart_on_signal";
      [k: string]: unknown;
    }
  | {
      type: "wait_healthy";
      [k: string]: unknown;
    }
  | {
      type: "none";
      [k: string]: unknown;
    };
