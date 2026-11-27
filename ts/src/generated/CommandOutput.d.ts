/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */

/**
 * What the runner does with the output of a [`ScheduledTaskType::Command`]
 * child.
 *
 * Internally tagged by `type`: `{"type":"Discard"}` or
 * `{"type":"Tail","bytes":2048}`.
 */
export type CommandOutput =
  | {
      type: "Discard";
      [k: string]: unknown;
    }
  | {
      /**
       * How many trailing bytes of stderr to keep: `1..=4096`.
       */
      bytes: number;
      type: "Tail";
      [k: string]: unknown;
    };
