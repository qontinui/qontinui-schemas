/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with `just generate-types` or
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */

/**
 * Stdio-transport configuration.
 *
 * **Secret surface**: `command`, `args`, and `env` must be treated as
 * execution-critical. See module-level docs.
 */
export interface StdioConfig {
  /**
   * Command arguments.
   */
  args?: string[];
  /**
   * Command to execute (e.g. `"npx"`, `"python"`, `"/usr/local/bin/server"`).
   */
  command: string;
  /**
   * Working directory (absolute path). `None` inherits the runner's cwd.
   */
  cwd?: string | null;
  /**
   * Extra environment variables for the subprocess. **Secret surface** —
   * frequently holds API tokens.
   */
  env?: {
    [k: string]: string;
  };
}
