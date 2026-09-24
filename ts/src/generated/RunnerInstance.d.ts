/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */

import type { RunnerInstanceRole } from "./RunnerInstanceRole";

/**
 * One live runner instance under a [`Runner`] (machine) row.
 *
 * A machine can host several runner instances at once — the primary on
 * `:9876` and supervisor-spawned secondaries on `:9877-9899` — all presenting
 * the one machine `device_id`, so they share a single [`Runner`] row. One
 * entry per currently connected instance lets a consumer address `:9876` and
 * `:9877` on the same box separately rather than seeing only whichever
 * instance wrote the row's `port` last (plan
 * `2026-09-20-runner-selector-drives-a-transport-not-a-target`, reading A0.2
 * and Phase 6).
 */
export interface RunnerInstance {
  /**
   * ISO 8601 timestamp when this instance's current WebSocket connection
   * was established.
   */
  connectedAt: string;
  /**
   * Per-instance key, reported by the runner as `instanceKey`. Built from
   * namespaced sources so no source can spell another's key: `"primary"`
   * for the primary; `"runner:<id>"` for a supervisor-spawned secondary
   * (supervisor-assigned ids are unique, and a named runner's id is reused
   * across its restarts); `"name:<name>"` for a secondary carrying only an
   * instance name; `"port:<port>"` for a nameless one (stable only while it
   * keeps that port). NOT guaranteed unique: two processes that each believe
   * themselves the primary both report `"primary"`, so a consumer must treat
   * two live instances with the same key as a conflict, never as one
   * overwriting the other.
   */
  instanceKey: string;
  /**
   * Whether this instance is the machine's primary or a secondary.
   */
  instanceRole: RunnerInstanceRole;
  /**
   * Port this instance's HTTP API is listening on, if reported.
   */
  port?: number | null;
}
