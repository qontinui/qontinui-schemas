/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with `just generate-types` or
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */

/**
 * Root-cause classification for a diagnostic failure.
 */
export type RootCauseCategory =
  | "bad_ui_rendering"
  | "bad_ui_bridge_evaluation"
  | "bad_verification_steps"
  | "bad_generation_prompt"
  | "bad_state_machine_logic"
  | "infrastructure_issue"
  | "unknown";
