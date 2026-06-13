/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with `just generate-types` or
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */

/**
 * How the generation loop enforces quality — names existing claude-config skills
 * to run and which of them are hard blockers.
 */
export interface EnforcementProfile {
  /**
   * Which categories block the loop on failure (e.g. `["security"]`). A skill
   * in `run` but not `block_on` is advisory.
   */
  blockOn?: string[];
  /**
   * Skills to run on generated code (e.g. `["code-reviewer", "security-scan"]`).
   */
  run?: string[];
  [k: string]: unknown;
}
