/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with `just generate-types` or
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */

/**
 * HTTP-transport configuration.
 *
 * **Secret surface**: `headers` typically carries an `Authorization` token.
 * See module-level docs.
 */
export interface HttpConfig {
  /**
   * HTTP headers to include on every request. **Secret surface** —
   * typically includes `Authorization: Bearer …`.
   */
  headers?: {
    [k: string]: string;
  };
  /**
   * Server URL (e.g. `"http://localhost:8080/mcp"`). The runner appends
   * `/tools/list` and `/tools/call` to this base.
   */
  url: string;
}
