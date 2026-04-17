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
  cpuInfo?: string | null;
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
  memoryMb?: number | null;
  /**
   * Operating system identifier (e.g., `"windows"`, `"macos"`, `"linux"`).
   */
  os: string;
  /**
   * Semantic version of the runner binary.
   */
  runnerVersion: string;
  /**
   * Screen resolution as a free-form string (e.g., `"1920x1080"`).
   */
  screenResolution?: string | null;
  [k: string]: unknown;
}
