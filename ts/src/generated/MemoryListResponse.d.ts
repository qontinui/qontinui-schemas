/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with `just generate-types` or
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */

import type { MemorySummary } from "./MemorySummary";

/**
 * `GET /coord/memory/list` response envelope. Wrapped in a struct
 * (rather than a bare `Vec<MemorySummary>`) so the generated TS / Python
 * schemas get a named exported interface rather than an anonymous array
 * alias.
 */
export interface MemoryListResponse {
  items: MemorySummary[];
  [k: string]: unknown;
}
