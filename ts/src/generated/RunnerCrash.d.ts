/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with `just generate-types` or
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */

/**
 * Most recent crash dump metadata, if any.
 */
export interface RunnerCrash {
  /**
   * Path to the crash dump file on disk.
   */
  filePath: string;
  /**
   * Source location (file/line) where the panic originated.
   */
  panicLocation: string;
  /**
   * Panic message captured from the runner process.
   */
  panicMessage: string;
  /**
   * ISO 8601 timestamp when the crash was reported.
   */
  reportedAt: string;
  /**
   * Name of the thread that panicked.
   */
  thread: string;
}
