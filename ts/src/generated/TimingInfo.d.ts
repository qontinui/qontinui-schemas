/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with `just generate-types` or
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */

/**
 * Precise timing information for an event. Times are ISO 8601 strings (see
 * crate-level docs for the rationale — the types crate is wire-only and
 * doesn't depend on a chrono version).
 */
export interface TimingInfo {
  durationMs?: number | null;
  /**
   * ISO 8601 timestamp. `None` while the event is still in flight.
   */
  endTime?: string | null;
  /**
   * ISO 8601 timestamp.
   */
  startTime: string;
  [k: string]: unknown;
}
