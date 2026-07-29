/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with `just generate-types` or
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */

/**
 * A single action inside a [`VgaAutomateStep`] action sequence.
 *
 * Wire shape is internally tagged with `"kind"`:
 * - `{ "kind": "click", "elementId": "<uuid>", "timeoutMs": 10000 }`
 * - `{ "kind": "type", "text": "hello", "elementId": "<uuid>", "timeoutMs": 10000 }`
 *   (`elementId` omitted/null = type into the currently focused element)
 * - `{ "kind": "wait_for", "elementId": "<uuid>", "timeoutMs": 30000 }`
 */
export type VgaAction =
  | {
      /**
       * UUID of the target element in the VGA state machine's element set.
       */
      elementId: string;
      kind: "click";
      /**
       * Per-action timeout in milliseconds. Defaults to `10000` on the
       * consumer side.
       */
      timeoutMs?: number | null;
      [k: string]: unknown;
    }
  | {
      /**
       * UUID of the target element. Absent / `null` means "type into
       * whichever element currently has focus".
       */
      elementId?: string | null;
      kind: "type";
      /**
       * Literal text to type.
       */
      text: string;
      /**
       * Per-action timeout in milliseconds. Defaults to `10000` on the
       * consumer side.
       */
      timeoutMs?: number | null;
      [k: string]: unknown;
    }
  | {
      /**
       * UUID of the target element.
       */
      elementId: string;
      kind: "wait_for";
      /**
       * Per-action timeout in milliseconds. Defaults to `30000` on the
       * consumer side.
       */
      timeoutMs?: number | null;
      [k: string]: unknown;
    };
