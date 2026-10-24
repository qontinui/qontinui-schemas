/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */

/**
 * An incoming request — the raw ask a requirement is derived from.
 */
export interface RequestFrontmatter {
  id: string;
  /**
   * ISO 8601.
   */
  received_at?: string | null;
  /**
   * The role of the person asking — a role, never a name.
   */
  requester_role?: string | null;
  /**
   * Its reference there (a message id, a ticket key).
   */
  source_ref?: string | null;
  /**
   * Where the request came from, e.g. `email`, `jira`, `workshop`.
   */
  source_system: string;
  [k: string]: unknown;
}
