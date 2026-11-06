/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */

/**
 * Body of `GET /apps/{app_id}/journey/health`, and the ledger block every
 * journey query response repeats.
 */
export interface JourneyLedgerHealth {
  /**
   * Human-readable cause, e.g. `"embedded DB predates the journey tables"`.
   */
  detail: string;
  /**
   * The ledger's state.
   */
  state: "writing" | "schema_absent" | "write_failing";
}
