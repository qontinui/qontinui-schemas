/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with `just generate-types` or
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */

/**
 * Current phase of the orchestration loop.
 */
export type LoopPhase =
  | "idle"
  | "building_workflow"
  | "running_workflow"
  | "diagnosing"
  | "reflecting"
  | "implementing_fixes"
  | "evaluating_exit"
  | "waiting_for_fixer"
  | "between_iterations"
  | "waiting_for_runner"
  | "stall_detecting"
  | "planning"
  | "complete"
  | "stopped"
  | "error";
