/**
 * Runner Types
 *
 * Canonical wire-format types for a registered runner. One row per registered
 * runner; the same shape is served to every consumer (mobile, web, runner UI).
 *
 * Types generated from Rust (source of truth:
 * `qontinui-schemas/rust/src/runner.rs`). Do not edit by hand — regenerate via
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */
export type { Runner } from "../generated/Runner";
export type { RunnerStatus } from "../generated/RunnerStatus";
export type { RunnerUiError } from "../generated/RunnerUiError";
export type { RunnerCrash } from "../generated/RunnerCrash";
