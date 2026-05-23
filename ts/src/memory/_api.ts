/**
 * Memory Types
 *
 * Canonical wire-format types for coord-mediated memory federation.
 * Promoted from inline definitions in `qontinui-coord/src/memories.rs`
 * by plan `2026-05-22-memories-on-coord-cross-machine.md` Phase 6 so
 * coord (HTTP routes), runner-side memory bridge, and qontinui-web
 * (proxy + dashboard browser) all speak one shape.
 *
 * Types generated from Rust (source of truth:
 * `qontinui-schemas/rust/src/memory.rs`). Do not edit by hand —
 * regenerate via `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */
export type { MemoryUpsertRequest } from "../generated/MemoryUpsertRequest";
export type { MemoryUpsertResponse } from "../generated/MemoryUpsertResponse";
export type { MemoryRestoreRequest } from "../generated/MemoryRestoreRequest";
export type { MemoryListQuery } from "../generated/MemoryListQuery";
export type { MemoryRow } from "../generated/MemoryRow";
export type { MemorySummary } from "../generated/MemorySummary";
export type { MemoryWithHistory } from "../generated/MemoryWithHistory";
export type { MemoryListResponse } from "../generated/MemoryListResponse";
