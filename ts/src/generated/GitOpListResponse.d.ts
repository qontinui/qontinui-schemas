/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with `just generate-types` or
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */

import type { GitOpRecord } from './GitOpRecord';

/**
 * `GET /coord/git-ops/list` + `GET /coord/git-ops/by-session/:id`
 * response envelope. Wrapped in a struct (rather than a bare
 * `Vec<GitOpRecord>`) so the generated TS / Python schemas get a named
 * exported interface rather than an anonymous array alias.
 */
export interface GitOpListResponse {
  items: GitOpRecord[];
  [k: string]: unknown;
}
