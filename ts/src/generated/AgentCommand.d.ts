/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with `just generate-types` or
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */

/**
 * An account-scoped **override** of a runner-embedded default agent command.
 *
 * Presence of this record means "this organization has customized the command
 * named `name`"; absence means the embedded default applies. See the module
 * docs for the full resolution order and the delete-is-reset semantics.
 *
 * `body` is the whole markdown procedure — the exact bytes written to
 * `.claude/commands/<name>.md`. It is **untrusted remote content** from the
 * runner's point of view: it is markdown rather than code, but it is
 * instructions to an agent, so consumers fail soft (a malformed override falls
 * back to the embedded default and warns) and never fetch cross-org.
 */
export interface AgentCommand {
  /**
   * The full markdown body of the command.
   */
  body: string;
  /**
   * Content hash of [`body`](Self::body), for cheap change detection
   * (cache invalidation, diff-against-default). `None` on rows written
   * before a checksum was computed.
   */
  checksum?: string | null;
  /**
   * ISO 8601 (RFC 3339) creation timestamp.
   */
  created_at: string;
  /**
   * User who last authored this override. **Attribution only** — it does not
   * scope visibility; [`organization_id`](Self::organization_id) does.
   */
  created_by_user_id?: string | null;
  /**
   * Head of the version chain — the `version_number` of the
   * [`AgentCommandVersion`] whose body this record currently carries.
   * Starts at 1 and only ever increases (a revert bumps it; see the module
   * docs).
   */
  current_version: number;
  /**
   * Override record id (UUID v4 string).
   */
  id: string;
  /**
   * Whether this override is visible to the whole organization rather than
   * only its author. Absent means `false`.
   */
  is_shared: boolean;
  /**
   * The command slug, e.g. `vet-plan` or `implement-plan`. This is the
   * override key: it must equal the name of the embedded default it
   * replaces, and it is the filename stem under `.claude/commands/`.
   */
  name: string;
  /**
   * Owning organization (`auth.organizations.id`). `None` where the row has
   * been orphaned by an organization deletion (`ondelete SET NULL`); such a
   * row is not resolvable by any account and behaves as no override.
   */
  organization_id?: string | null;
  /**
   * ISO 8601 (RFC 3339) last-modification timestamp.
   */
  updated_at: string;
  [k: string]: unknown;
}
