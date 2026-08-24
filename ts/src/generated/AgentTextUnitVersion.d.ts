/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with `just generate-types` or
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */

/**
 * One immutable entry in an [`AgentTextUnit`]'s append-only version chain.
 *
 * Rows are **only ever inserted**. An edit appends the next `version_number`;
 * a revert appends a new version whose `files` are copied from an older one
 * and whose [`restored_from`](Self::restored_from) names that older version.
 * Nothing in the chain is updated or deleted, so the chain is the complete
 * edit history and "revert" is itself a recorded edit.
 */
export interface AgentTextUnitVersion {
  /**
   * The [`AgentTextUnit`] this version belongs to. Deleting the unit
   * cascades to its versions.
   */
  agent_text_unit_id: string;
  /**
   * Free-text note describing what changed, supplied by the editor. A
   * revert writes a generated one (e.g. `"Restored from version 2"`).
   */
  change_description?: string | null;
  /**
   * Content hash of [`files`](Self::files), from
   * [`agent_text_unit_files_checksum`]. Equal checksums on two versions mean
   * the content is identical, which is what makes the Phase 5 re-import
   * idempotent — it is the field "did the text actually change?" is read
   * off. `None` on a row written before a checksum was computed.
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
   * The unit's whole content as of this version: **relative path → text**.
   *
   * Every key must satisfy [`validate_agent_text_unit_file_path`]; the map
   * must be non-empty and must contain the unit's
   * [`agent_text_unit_entrypoint`].
   */
  files: {
    [k: string]: string;
  };
  /**
   * Version-row id (UUID v4 string).
   */
  id: string;
  /**
   * Provenance for a revert: the `version_number` this version's content
   * was copied from. `None` for an ordinary edit.
   */
  restored_from?: number | null;
  /**
   * Monotonic version number within this unit, starting at 1. Unique per
   * `(agent_text_unit_id, version_number)`; the DB constraint — not
   * application code — is what rejects a duplicate under concurrent
   * appends.
   */
  version_number: number;
  [k: string]: unknown;
}
