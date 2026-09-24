/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */

/**
 * How to evaluate whether the loop should exit.
 */
export type ExitStrategy =
  | {
      type: "reflection";
      [k: string]: unknown;
    }
  | {
      type: "workflow_verification";
      [k: string]: unknown;
    }
  | {
      type: "fixed_iterations";
      [k: string]: unknown;
    }
  | {
      type: "diagnostic_evaluation";
      [k: string]: unknown;
    };
