/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with `just generate-types` or
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */

/**
 * Abstract, provider-neutral ticket lifecycle state.
 *
 * Maps onto provider-specific states (`open` / `closed` on GitHub, workflow
 * state types on Linear, etc.) inside the runner's provider implementations.
 * Serialized as `snake_case`.
 */
export type TicketState = "open" | "in_progress" | "done" | "closed";
