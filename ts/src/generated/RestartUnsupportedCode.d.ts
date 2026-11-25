/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */

/**
 * Why a restart-mode loop cannot run against this target, decided BEFORE start.
 */
export type RestartUnsupportedCode =
  | "target_is_orchestrator"
  | "target_not_runner_managed"
  | "rebuild_needs_dev_supervisor";
