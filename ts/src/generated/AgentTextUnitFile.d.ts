/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with `just generate-types` or
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */

/**
 * One `relative path → text` entry of a `files` map, as a standalone record.
 *
 * The map is the storage and wire form (it is one JSONB column, and it makes
 * "no duplicate paths" structural rather than validated). This type is the
 * **entry** form: what a per-file API operation addresses, what a per-file
 * validation failure names, and what a provisioner iterates. Convert with
 * [`from_map`](Self::from_map) / [`into_map`](Self::into_map).
 */
export interface AgentTextUnitFile {
  /**
   * Path relative to the unit's own directory, `/`-separated, e.g.
   * `SKILL.md` or `coord-revive.sh`. Validated by
   * [`validate_agent_text_unit_file_path`].
   */
  path: string;
  /**
   * The file's full text. Written verbatim (modulo the provisioner's own
   * line-ending handling); never given an executable bit, because scripts
   * in this corpus are invoked as `bash <path>` and Windows has no exec bit
   * anyway. "Account-supplied text written to disk" must not become
   * "account-supplied program registered with the OS".
   */
  text: string;
  [k: string]: unknown;
}
