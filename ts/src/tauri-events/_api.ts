/**
 * Tauri Event Payload Types
 *
 * Wire-format types for payloads emitted on Tauri event channels by the
 * runner. Use these to type the generic parameter of `listen<T>(...)` so
 * future Rust serde renames (e.g. snake_case → camelCase) and shape
 * changes break the TypeScript build instead of silently dropping events
 * at the listener guard.
 *
 * ## Adjacently-tagged envelope (`AppEvent`)
 *
 * Most runner events are variants of the `AppEvent` enum, which is
 * serialized via `#[serde(tag = "event_type", content = "data")]`. The
 * wire payload is therefore `{ event_type: "<VariantName>", data: { ... } }`
 * and consumers MUST read `event.payload.data.<field>` (NOT
 * `event.payload.<field>`). The `AppEvent` discriminated union below
 * encodes that envelope; reaching `data.<field>` after narrowing by
 * `event_type` is type-safe.
 *
 * ## Direct-payload events
 *
 * A handful of events bypass `AppEvent` and emit a payload struct
 * directly:
 *  - `terminal-output` → `TerminalOutputEvent`
 *  - `terminal-exit` → `TerminalExitEvent`
 *  - `terminal-created` → `TerminalInfo`
 *  - `finding_detected` / `finding_resolved` → wrap a `RunnerFinding`
 *    inside the `AppEvent` envelope's `data.finding` field as untyped
 *    JSON; consumers should cast `data.finding as RunnerFinding`.
 *
 * Source of truth:
 *  - `AppEvent`: `qontinui-runner/src-tauri/src/event_system/types.rs`
 *  - Terminal events: `qontinui-schemas/rust/src/terminal.rs`
 *  - `RunnerFinding`: `qontinui-runner/src-tauri/src/tauri_event_payloads.rs`
 *
 * Regenerate via `cd qontinui-runner && npm run gen-events`.
 */

export type { AppEvent } from "../generated/AppEvent";

// ── Terminal events (direct payloads, not wrapped in AppEvent) ──
export type { TerminalInfo } from "../generated/TerminalInfo";
export type { TerminalOutputEvent } from "../generated/TerminalOutputEvent";
export type { TerminalExitEvent } from "../generated/TerminalExitEvent";

// ── Runner-local finding payload (carried inside AppEvent::FindingDetected /
//    AppEvent::FindingResolved as `data.finding`). Distinct from
//    `qontinui_types::verification::Finding`. ──
export type { RunnerFinding } from "../generated/RunnerFinding";
export type { RunnerFindingCodeContext } from "../generated/RunnerFindingCodeContext";
export type { RunnerFindingUserInput } from "../generated/RunnerFindingUserInput";
