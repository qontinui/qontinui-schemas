/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with `just generate-types` or
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */

/**
 * One immutable entry in an [`AgentCommand`]'s append-only version chain.
 *
 * Rows are **only ever inserted**. An edit appends the next `version_number`;
 * a revert appends a new version whose `body` is copied from an older one and
 * whose [`restored_from`](Self::restored_from) names that older version.
 * Nothing in the chain is updated or deleted, so the chain is the complete
 * edit history and "revert" is itself a recorded edit.
 */
export interface AgentCommandVersion {
  /**
   * The [`AgentCommand`] this version belongs to. Deleting the override
   * cascades to its versions — resetting to the default discards the
   * account's edit history along with the override.
   */
  agent_command_id: string;
  /**
   * The full markdown body as of this version.
   */
  body: string;
  /**
   * Free-text note describing what changed, supplied by the editor. Revert
   * writes a generated one (e.g. `"Restored from version 2"`).
   */
  change_description?: string | null;
  /**
   * Content hash of [`body`](Self::body) at this version.
   */
  checksum?: string | null;
  /**
   * ISO 8601 (RFC 3339) creation timestamp. Versions are immutable, so
   * there is deliberately no `updated_at`.
   */
  created_at: string;
  /**
   * User who authored this version. Attribution only.
   */
  created_by_user_id?: string | null;
  /**
   * Version-row id (UUID v4 string).
   */
  id: string;
  /**
   * Provenance for a revert: the `version_number` this version's body was
   * copied from. `None` for an ordinary edit. Present only on versions
   * produced by a restore, which is how the UI distinguishes "edited" from
   * "reverted to v2" in the history list.
   */
  restored_from?: number | null;
  /**
   * Monotonic version number within this command, starting at 1. Unique per
   * `(agent_command_id, version_number)`; the DB constraint — not
   * application code — is what rejects a duplicate under concurrent appends.
   */
  version_number: number;
  [k: string]: unknown;
}
