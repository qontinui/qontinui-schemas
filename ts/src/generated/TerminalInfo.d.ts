/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with `just generate-types` or
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */

/**
 * Information about a terminal session, returned to the frontend.
 *
 * Returned from the `terminal_create` and `terminal_list` Tauri commands
 * and emitted as the payload of the `terminal-created` event. Derived
 * fresh from the live `TerminalSession` each time — never persisted.
 */
export interface TerminalInfo {
  /**
   * Current terminal width in columns.
   */
  cols: number;
  /**
   * Unix timestamp in milliseconds when the session was created.
   *
   * Used as the sort key for `TerminalManager::list` so the UI shows
   * terminals in creation order.
   */
  createdAt: number;
  /**
   * Process exit code, once the shell has exited.
   */
  exitCode?: number | null;
  /**
   * Unique session identifier (UUID v4 minted by the runner).
   */
  id: string;
  /**
   * Whether the shell process is still running.
   */
  isAlive: boolean;
  /**
   * Which terminal page this session belongs to (for multi-page support).
   *
   * Older wire forms without this field hydrate to `"default"` via
   * [`default_page_id`].
   */
  pageId: string;
  /**
   * OS process ID of the spawned shell, if still known.
   */
  pid?: number | null;
  /**
   * Current terminal height in rows.
   */
  rows: number;
  /**
   * Human-readable title shown in the UI tab (e.g., "Terminal 1").
   */
  title: string;
  /**
   * Monotonic counter of all bytes ever produced by this PTY.
   *
   * Read by the frontend to detect missed output after a reconnect; the
   * scrollback buffer's `start_offset` is derived from this counter.
   */
  totalBytesProduced: number;
  /**
   * Absolute working directory the shell was started in.
   */
  workingDir: string;
  [k: string]: unknown;
}
