/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with `just generate-types` or
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */

/**
 * Which ticket provider a config / ticket refers to.
 *
 * Serialized as the lowercase provider tag (`"github"`, `"linear"`, `"jira"`)
 * to match the on-the-wire tags used by the watcher config, the MCP API
 * layer, and the runner's DB storage keys. Variants are explicitly renamed
 * because `rename_all = "snake_case"` would emit `"git_hub"` for the
 * `GitHub` variant, which would drift from the existing persisted rows.
 */
export type TicketSource = "github" | "linear" | "jira";
