/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with `just generate-types` or
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */

/**
 * Lighter-weight projection for `/coord/memory/list` — strips the
 * (potentially-large) `content` blob so a list-all-memories scan stays
 * cheap. Fetch full payload via `GET /coord/memory/:name`.
 */
export interface MemorySummary {
  description?: string | null;
  name: string;
  type?: string | null;
  version: number;
  /**
   * ISO 8601 timestamp (RFC 3339).
   */
  written_at: string;
  [k: string]: unknown;
}
