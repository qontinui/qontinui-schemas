/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with `just generate-types` or
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */

/**
 * `GET /coord/memory/list` query string.
 */
export interface MemoryListQuery {
  limit?: number | null;
  name_prefix?: string | null;
  type?: string | null;
  [k: string]: unknown;
}
