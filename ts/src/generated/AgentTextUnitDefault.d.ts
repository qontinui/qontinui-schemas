/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with `just generate-types` or
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */

/**
 * One embedded default as the **runner binary** ships it, on its way to the
 * account so there is a baseline to diff an override against.
 *
 * ## Why this exists as a third layer
 *
 * [`AgentTextUnit`]'s resolution chain is
 * `account override → fleet default → embedded default (runner binary)`, and
 * the store holds rows for the first two only — the binary's copy has never
 * had a row anywhere. That is the gap this type closes. A user who overrode
 * `/implement-plan` can diff their versions against each other but not
 * against what actually ships, and `ResetToDefaultDialog` cannot preview the
 * text it is about to restore, because there is no baseline to put on the
 * left-hand side.
 *
 * Plan `2026-08-31-runner-publishes-embedded-command-defaults`; the deferral
 * it discharges is recorded in the code at qontinui-web's
 * `settings/agent-commands/_components/VersionDiff.tsx`.
 *
 * ## What it is NOT
 *
 * * **Not an [`AgentTextUnit`].** There is no `id`, no version chain, and no
 *   `is_shared` — a published default is content plus provenance, not a
 *   corpus row with an edit history. It is the *input* to a row.
 * * **Not fleet-scoped.** `organization_id` is deliberately absent from the
 *   wire: the runner publishes with the operator's own user bearer, so the
 *   server assigns the org from that credential. A client-supplied org — or
 *   the fleet layer's `organization_id IS NULL` — would let any signed-in
 *   user rewrite another tenant's baseline, or silently clobber an
 *   operator's deliberate fleet default.
 * * **Not authoritative for provisioning.** The runner keeps resolving
 *   `fresh fetch → disk cache → embedded default`; this is a *display*
 *   baseline. Publishing it must never put the network on the
 *   out-of-the-box path.
 *
 * ## The checksum is the files-map digest, not the single-body one
 *
 * [`checksum`](Self::checksum) is [`agent_text_unit_files_checksum`] — the
 * same digest [`AgentTextUnit::checksum`] and [`AgentTextUnitVersion`] carry,
 * so a published default and the override it is diffed against are
 * comparable. It is **not**
 * [`crate::agent_commands::agent_command_checksum`], which digests a single
 * body and is what the legacy `/agent-commands` wire still carries. Those two
 * deliberately disagree even for a one-entry map, so a default digested with
 * the wrong one would never compare equal to its override — an always-drifted
 * baseline, which is strictly worse than the honest "baseline unavailable"
 * state it would replace.
 */
export interface AgentTextUnitDefault {
  /**
   * Canonical digest over [`files`](Self::files), from
   * [`agent_text_unit_files_checksum`].
   *
   * Required, unlike [`AgentTextUnit::checksum`] — that one is optional only
   * to describe rows written before a checksum was computed, whereas a
   * publish is always freshly computed. The receiving store **recomputes it
   * and rejects a mismatch**: a client-asserted digest is not evidence.
   * [`checksum_matches`](Self::checksum_matches) is that check.
   */
  checksum: string;
  /**
   * The embedded content: **relative path → text**. A command carries one
   * entry; a skill carries `SKILL.md` plus siblings. Same arity-agnostic
   * shape as [`AgentTextUnit::files`].
   */
  files: {
    [k: string]: string;
  };
  /**
   * What sort of unit this default is. The corpus is `kind`-discriminated,
   * so the default layer must be too — otherwise a command and a skill of
   * the same name collapse into one baseline.
   */
  kind: string;
  /**
   * The unit slug, e.g. `vet-plan`. Must satisfy the same
   * [`validate_agent_text_unit_name`] rules as an override: a default that
   * could not be named as an override could never be paired with one for a
   * diff, which is the only thing it is for.
   */
  name: string;
  /**
   * ISO 8601 (RFC 3339) publish timestamp, matching this module's wire
   * convention for [`AgentTextUnit::updated_at`] and
   * [`AgentTextUnitVersion::created_at`].
   */
  published_at: string;
  /**
   * The runner version that published this body, e.g. `"0.4.12"`.
   *
   * Carried so the UI can label the baseline **"published by runner
   * vX.Y.Z"** rather than "the default" — an org whose devices run
   * different builds has no single default, and the label must not claim
   * otherwise. It is also the monotonic guard's input: the store rejects a
   * publish older than the version it already holds.
   *
   * That guard is a **mitigation, not a fix** — a genuine downgrade still
   * wins and equal versions tie-break last-writer — so neither this field
   * nor the UI built on it may describe the baseline as authoritative.
   */
  published_by_version: string;
  [k: string]: unknown;
}
