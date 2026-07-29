/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with `just generate-types` or
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */

/**
 * Source of initial states configuration.
 *
 * - `Defaults`: states with `is_initial=true` in the state machine definition
 * - `Workflow`: initial states defined on the workflow (`initialStateIds`)
 * - `Override`: session-only override from the runner UI
 *
 * Serialized as a bare lowercase string (`"defaults"` / `"workflow"` /
 * `"override"`) to match the TS literal union.
 */
export type InitialStatesSource = "defaults" | "workflow" | "override";
