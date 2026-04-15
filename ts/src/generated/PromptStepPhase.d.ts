/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with `just generate-types` or
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */

/**
 * Phases in which a [`PromptStep`] may appear.
 *
 * Prompt steps are the only variant that may appear in the agentic phase.
 */
export type PromptStepPhase = "setup" | "verification" | "agentic" | "completion";
