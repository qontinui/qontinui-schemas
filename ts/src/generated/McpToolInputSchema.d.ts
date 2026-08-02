/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with `just generate-types` or
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */

/**
 * Input-parameter schema for an MCP tool.
 *
 * Subset of JSON Schema — enough to render a form and validate arguments
 * before dispatching a `tools/call`. The `properties` and `required` fields
 * are passed through verbatim from the server.
 */
export interface McpToolInputSchema {
  /**
   * Optional human-readable description.
   */
  description?: string | null;
  /**
   * JSON-Schema-shaped property descriptors, kept as opaque JSON.
   */
  properties?: {
    [k: string]: unknown;
  };
  /**
   * Names of required properties.
   */
  required?: string[] | null;
  /**
   * JSON Schema `type` (typically `"object"`).
   */
  type: string;
}
