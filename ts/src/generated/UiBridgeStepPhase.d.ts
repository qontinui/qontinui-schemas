/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with `just generate-types` or
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */

/**
 * Phases in which a [`UiBridgeStep`] may appear.
 *
 * UI-bridge interactions only run in deterministic phases — never inside
 * the agentic loop (where the AI drives steps directly via prompts).
 */
export type UiBridgeStepPhase = "setup" | "verification" | "completion";
