/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with `just generate-types` or
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */

/**
 * Format for exporting a state machine config to JSON.
 *
 * Compatible with `UIBridgeRuntime.from_dict()` in the qontinui library. The
 * nested maps hold opaque per-state / per-transition / per-config dictionaries
 * because the exporter serializes implementation-specific fields not captured
 * by the DTO types.
 */
export interface StateMachineExportFormat {
  /**
   * Config-level payload.
   */
  config: {
    [k: string]: unknown;
  };
  /**
   * State ID → state payload.
   */
  states: {
    [k: string]: {
      [k: string]: unknown;
    };
  };
  /**
   * Transition ID → transition payload.
   */
  transitions: {
    [k: string]: {
      [k: string]: unknown;
    };
  };
}
