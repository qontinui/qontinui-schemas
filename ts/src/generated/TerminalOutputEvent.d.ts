/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with `just generate-types` or
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */

/**
 * Event payload emitted when a terminal produces output.
 *
 * Delivered over the Tauri `terminal-output` channel and re-broadcast on
 * the WebSocket relay for remote/mobile consumers. The `data` payload is
 * base64-encoded raw bytes because xterm.js needs raw bytes (PTY output
 * can contain partial UTF-8 sequences that would corrupt a `String`
 * field).
 */
export interface TerminalOutputEvent {
  /**
   * Base64-encoded bytes produced by the PTY.
   *
   * Raw bytes are required (not UTF-8 text) because PTY output can
   * contain partial UTF-8 sequences that span read boundaries.
   */
  data: string;
  /**
   * ID of the terminal session producing this output.
   */
  terminal_id: string;
  [k: string]: unknown;
}
