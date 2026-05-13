/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with `just generate-types` or
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */

export interface IrProvenance {
  column?: number | null;
  file?: string | null;
  line?: number | null;
  pluginVersion?: string | null;
  /**
   * "hand-authored" | "build-plugin" | "ai-generated" | "migrated"
   */
  source: string;
  [k: string]: unknown;
}
