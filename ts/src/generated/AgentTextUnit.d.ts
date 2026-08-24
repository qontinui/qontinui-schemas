/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with `just generate-types` or
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */

/**
 * One unit of the agent text corpus: a `(kind, name)` addressed bundle of
 * text files, owned either by an organization or by the fleet-default layer.
 *
 * The content is **untrusted remote content** from the runner's point of
 * view — it is markdown and shell text rather than compiled code, but it is
 * instructions to an agent and it becomes files on a fleet device. Consumers
 * fail soft (a malformed unit falls back to the next rung of the resolution
 * chain and warns) and never fetch cross-org.
 *
 * This mirrors qontinui-web's `AgentTextUnitResponse` field for field, plus
 * the two forward provenance fields Phase 5 fills in.
 */
export interface AgentTextUnit {
  /**
   * Canonical digest over the whole [`files`](Self::files) map, from
   * [`agent_text_unit_files_checksum`]. `None` on a row written before a
   * checksum was computed.
   *
   * This is **not** [`crate::agent_commands::agent_command_checksum`]: that
   * one digests a single body and is what the legacy `/agent-commands` wire
   * still carries. The two deliberately disagree even for a one-entry map.
   */
  checksum?: string | null;
  /**
   * ISO 8601 (RFC 3339) creation timestamp.
   */
  created_at: string;
  /**
   * User who last authored this unit. **Attribution only** — it does not
   * scope visibility; [`organization_id`](Self::organization_id) does.
   */
  created_by_user_id?: string | null;
  /**
   * Head of the version chain — the `version_number` of the
   * [`AgentTextUnitVersion`] whose content this unit currently serves.
   * Starts at 1 and only ever increases (a revert bumps it).
   */
  current_version: number;
  /**
   * The key of [`files`](Self::files) holding this unit's primary text,
   * derived server-side from `(kind, name)` — see
   * [`agent_text_unit_entrypoint`]. Read-only: it is recomputed on every
   * response and ignored on write.
   */
  entrypoint: string;
  /**
   * The unit's current content: **relative path → text**. A command carries
   * one entry; a skill carries `SKILL.md` plus siblings.
   *
   * Carried inline on the unit, matching the landed
   * `AgentTextUnitResponse`. Note the cost that buys the convenience: a
   * whole-corpus list is ~88 units / ~1.8 MB, and the runner fetches it on
   * the spawn critical path under a 4 s budget, so a caller that only needs
   * to know whether its cache is current should compare
   * [`checksum`](Self::checksum) rather than re-reading these bytes.
   */
  files: {
    [k: string]: string;
  };
  /**
   * Unit id (UUID v4 string).
   */
  id: string;
  /**
   * Whether the harness may offer this unit as an invocable slash command.
   *
   * `false` means "carried by the corpus, never invocable": the
   * underscore-prefixed units (`_gate-registration`, `_loop-control`) are
   * copy-source specs that other units paste from, and `.claude/commands/`
   * has no include mechanism, so they must be *present on disk* without
   * appearing as `/_gate-registration`. Absent means `true`.
   *
   * Storage enforces the pairing with a CHECK (`left(name,1) <> '_' OR
   * is_invocable = false`); [`validate_agent_text_unit_invocability`] is the
   * same rule at this boundary.
   * Always written on the wire, matching the landed `AgentTextUnitResponse`
   * (which emits it unconditionally). `#[serde(default)]` covers reading an
   * older payload that predates the field; the `#[schemars(default)]` twin
   * is what puts `true` — rather than `null` — into the generated Python and
   * TypeScript bindings, so a consumer that sees the field absent defaults
   * it the same way Rust does.
   */
  is_invocable: boolean;
  /**
   * Whether this unit is visible to the whole organization rather than only
   * its author. Absent means `false`.
   */
  is_shared: boolean;
  /**
   * What sort of unit this is. Open discriminator — see
   * [`AgentTextUnitKind`].
   */
  kind: string;
  /**
   * The unit slug, e.g. `vet-plan` or `coord-revive`. It is the filename
   * stem (single-file kinds) or the directory name (directory kinds) under
   * `.claude/<subdir>/`, so it is a write-boundary value — validate it with
   * [`validate_agent_text_unit_name`] before persisting.
   */
  name: string;
  /**
   * Owning organization (`auth.organizations.id`).
   *
   * **`None` is meaningful**: it is the *fleet-default* layer, resolved
   * after an account override and before the runner's embedded default. It
   * is also what an organization deletion leaves behind (`ondelete SET
   * NULL`), which is why storage keys the NULL bucket with its own partial
   * unique index — see the module docs.
   */
  organization_id?: string | null;
  /**
   * Which resolution layer this row was served from —
   * [`AGENT_TEXT_UNIT_SOURCE_USER`] or [`AGENT_TEXT_UNIT_SOURCE_FLEET`].
   * Server-derived, defaults to `user`.
   *
   * ⚠️ **Unrelated to [`source_path`](Self::source_path) /
   * [`source_commit`](Self::source_commit) despite the shared prefix.** This
   * field names the *layer the row came from*; those two name the *config
   * repo the text was imported from*. Adjacent names, different concepts.
   */
  source: string;
  /**
   * Import provenance: the commit of the source repo (full 40-char SHA).
   * `None` for a console-authored unit, or for an import from a dirty tree
   * where no commit honestly describes the bytes.
   */
  source_commit?: string | null;
  /**
   * Import provenance: the repo-relative path this unit's text came from,
   * e.g. `.claude/skills/coord-revive/`. `None` for a unit authored directly
   * in the console.
   *
   * Recorded per unit at import so the console can show an operator whether
   * a unit still matches its source. The path is **repo-relative on
   * purpose** — an absolute one would pin a build machine's layout into
   * account data.
   */
  source_path?: string | null;
  /**
   * ISO 8601 (RFC 3339) last-modification timestamp.
   */
  updated_at: string;
  [k: string]: unknown;
}
