/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with `just generate-types` or
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */

/**
 * Code context for a finding (runner-local wire shape).
 *
 * Renamed in the schema registry to `RunnerFindingCodeContext` to
 * disambiguate from `qontinui_types::findings::FindingCodeContext`.
 */
export interface RunnerFindingCodeContext {
  column?: number | null;
  file?: string | null;
  line?: number | null;
  snippet?: string | null;
  [k: string]: unknown;
}
