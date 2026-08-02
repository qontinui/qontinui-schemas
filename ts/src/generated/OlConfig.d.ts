/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with `just generate-types` or
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */

/**
 * A saved orchestration-loop configuration preset.
 *
 * Persisted in the runner's `orchestration_loop_configs` PostgreSQL table and
 * surfaced to the frontend through the `ol_list_configs` / `ol_get_config`
 * Tauri commands. The inner `config_json` is an `OrchestrationLoopConfig`
 * serialized as a JSON blob — stored as `Value` here to preserve forward
 * compatibility with older presets if the config schema grows new fields.
 */
export interface OlConfig {
  /**
   * The full [`OrchestrationLoopConfig`] as a JSON blob.
   */
  configJson: {
    [k: string]: unknown;
  };
  /**
   * ISO 8601 creation timestamp.
   */
  createdAt: string;
  /**
   * Optional free-form description.
   */
  description?: string | null;
  /**
   * UUID v4 identifier.
   */
  id: string;
  /**
   * Whether this preset is pinned as a favorite in the UI.
   */
  isFavorite: boolean;
  /**
   * Human-readable name (e.g., "Nightly regression sweep").
   */
  name: string;
  /**
   * ISO 8601 last-modified timestamp.
   */
  updatedAt: string;
}
