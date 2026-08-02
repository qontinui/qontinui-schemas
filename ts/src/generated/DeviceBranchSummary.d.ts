/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with `just generate-types` or
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */

/**
 * One entry in the `GET /coord/git-ops/branches` response — the latest
 * branch a given device is on for a given repo, derived from the most
 * recent `checkout`/`branch_create` op per `(device_id, repo)`.
 */
export interface DeviceBranchSummary {
  branch?: string | null;
  device_id: string;
  /**
   * ISO 8601 timestamp (RFC 3339) of the op this summary was derived from.
   */
  recorded_at: string;
  repo: string;
  sha?: string | null;
  [k: string]: unknown;
}
