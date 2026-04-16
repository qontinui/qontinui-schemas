/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with `just generate-types` or
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */

/**
 * Information about the runner environment that produced the run.
 */
export interface RunnerMetadata {
  /**
   * CPU description.
   */
  cpu_info?: string | null;
  /**
   * Arbitrary additional runner context.
   */
  extra?: {
    [k: string]: unknown;
  } | null;
  /**
   * Host machine name.
   */
  hostname: string;
  /**
   * Installed system memory in megabytes.
   */
  memory_mb?: number | null;
  /**
   * Operating system identifier (e.g., `"windows"`, `"macos"`, `"linux"`).
   */
  os: string;
  /**
   * Semantic version of the runner binary.
   */
  runner_version: string;
  /**
   * Screen resolution as a free-form string (e.g., `"1920x1080"`).
   */
  screen_resolution?: string | null;
  [k: string]: unknown;
}
