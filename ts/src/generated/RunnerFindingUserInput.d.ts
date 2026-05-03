/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with `just generate-types` or
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */

/**
 * User input request for a finding (runner-local wire shape).
 *
 * Renamed in the schema registry to `RunnerFindingUserInput` to
 * disambiguate from `qontinui_types::findings::FindingUserInput`.
 */
export interface RunnerFindingUserInput {
  input_type: string;
  options?: string[] | null;
  question: string;
  [k: string]: unknown;
}
