/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */

/**
 * Three-valued capability fact.
 *
 * `Unknown` is the default for anything a profile or catalog does not state.
 * It is a distinct value from `Unsupported` on purpose: a boolean cannot
 * represent "we never asked", and coercing that to "no" silently disables
 * working capabilities.
 *
 * Moved here from qontinui-runner `src-tauri/src/model_catalog.rs` so the
 * model/API layer (`ModelFacts`) and the CLI-session layer ([`CliProfile`])
 * speak one tri-state; `model_catalog` re-exports it. Serialized as
 * `"supported"` / `"unsupported"` / `"unknown"`.
 *
 * [`CapabilityState::Unknown`] must never be a reason to strip content or
 * refuse a route. Callers that must act on an unknown fact should attempt the
 * capability and let the backend be the authority.
 */
export type CapabilityState = "supported" | "unsupported" | "unknown";
