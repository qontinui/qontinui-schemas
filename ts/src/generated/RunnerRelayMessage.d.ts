/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with `just generate-types` or
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */

/**
 * Discriminated union of every WS envelope `mcp/backend_relay.rs::handle_outbound`
 * ships to the backend after rewrapping a Tauri-side `AppEvent`.
 *
 * Internally tagged on `"type"` with snake_case variant names because that's
 * what the prior `serde_json::json!` literals emitted on the wire.
 *
 * Wire keys are snake_case (NOT camelCase) — verified against the literal
 * strings in `handle_outbound` lines ~605-638. Consumers that expect
 * camelCase (e.g. some web hooks) are wrong; the relay never emitted
 * camelCase here.
 */
export type RunnerRelayMessage =
  | {
      data: {
        [k: string]: unknown;
      };
      type: "phase_completed";
      [k: string]: unknown;
    }
  | {
      data: {
        [k: string]: unknown;
      };
      type: "ui_error";
      [k: string]: unknown;
    }
  | {
      data: {
        [k: string]: unknown;
      };
      type: "recent_crash";
      [k: string]: unknown;
    }
  | {
      data: {
        [k: string]: unknown;
      };
      type: "chat_response";
      [k: string]: unknown;
    }
  | {
      data: {
        [k: string]: unknown;
      };
      type: "chat_session_state";
      [k: string]: unknown;
    }
  | {
      data: {
        [k: string]: unknown;
      };
      terminal_id: {
        [k: string]: unknown;
      };
      type: "terminal_output";
      [k: string]: unknown;
    }
  | {
      exit_code: {
        [k: string]: unknown;
      };
      terminal_id: {
        [k: string]: unknown;
      };
      type: "terminal_exit";
      [k: string]: unknown;
    };
