/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with `just generate-types` or
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */

/**
 * Extract a named variable from an API response via JSONPath.
 */
export interface ApiVariableExtraction {
  /**
   * Default value if the path does not resolve.
   */
  defaultValue?: string | null;
  /**
   * JSONPath expression used to extract the value.
   */
  jsonPath: string;
  /**
   * Variable name to bind the extracted value to.
   */
  variableName: string;
}
