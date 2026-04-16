/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with `just generate-types` or
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */

/**
 * Event payload emitted when a terminal process exits.
 *
 * Delivered over the Tauri `terminal-exit` channel and re-broadcast on the
 * WebSocket relay. After this event fires the corresponding
 * [`TerminalInfo::is_alive`] will be `false` and [`TerminalInfo::exit_code`]
 * will carry the same value surfaced here.
 */
export interface TerminalExitEvent {
  /**
   * Exit code reported by the OS, or `None` if the status could not be
   * captured (e.g., the wait call itself errored).
   */
  exit_code?: number | null;
  /**
   * ID of the terminal session that exited.
   */
  terminal_id: string;
  [k: string]: unknown;
}
