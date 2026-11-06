/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */

/**
 * Narrows the EDGE set a graph is built from. Staleness is a filter, not a
 * verdict: the caller decides what window to ask about, and every result
 * echoes the filter it was computed under (`filterEcho`).
 *
 * The frontier is never filtered: an open frontier entry is current state,
 * and hiding one behind an edge filter would turn "not looked at" into
 * "nothing there".
 *
 * `since` is a `chrono::DateTime<Utc>` (serialized RFC 3339), an exception
 * to the crate's string-timestamp convention for the same reason as
 * `page::KeysetPosition`: it is compared, not merely carried.
 */
export interface JourneyFilter {
  /**
   * Keep edges whose `appVersion` equals this; `None` = every version.
   * An edge that reported no version never matches a set filter.
   */
  appVersion: string | null;
  /**
   * Keep edges of this run kind; `None` = every kind.
   */
  runKind: ("agent_action" | "explorer" | "passive_session") | null;
  /**
   * Keep edges with `observedAt >= since`; `None` = no lower bound.
   */
  since: string | null;
}
