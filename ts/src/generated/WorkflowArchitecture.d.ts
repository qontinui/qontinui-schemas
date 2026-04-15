/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with `just generate-types` or
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */

/**
 * The workflow execution architecture to use.
 *
 * This is a first-class workflow architecture option, allowing direct
 * comparison between traditional deterministic verification and agentic
 * verification approaches. Mirrors the runner-side enum in
 * `crate::agentic_verification`.
 */
export type WorkflowArchitecture = "traditional" | "agentic_verification" | "multi_agent_pipeline";
