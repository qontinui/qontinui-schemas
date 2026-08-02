/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with `just generate-types` or
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */

import type { ElementRect } from "./ElementRect";
import type { ElementState } from "./ElementState";

/**
 * Response from executing an action on an element or component.
 */
export interface ActionResponse {
  /**
   * Time taken to execute the action in milliseconds.
   */
  durationMs: number;
  /**
   * D3 effect-calculus verification: the predicted-vs-observed outcome for
   * this action, present only when a handler effect signature resolved for
   * the `(action, element)` (opt-in; absent otherwise).
   *
   * Carried as an opaque JSON object on the wire, deliberately matching the
   * established lean-wire / rich-SDK split (the SDK's `EffectVerification`
   * has a richer shape than any consumer needs to type). The runner
   * deserializes the sub-shape it asserts on (`outcome` / `cause` /
   * `containment` / `durationMs`) with its own local struct in the
   * `effect_check` step handler, and relays the rest into `result_json`. See
   * `ui-bridge/.../control/effect-types.ts` for the SDK-side producer. Kept
   * opaque here so the nested effect types need no top-level codegen
   * registration (which would couple this crate's bindings to a runner
   * `schema_export.rs` change).
   */
  effectVerification?: {
    [k: string]: unknown;
  };
  /**
   * Updated element state after the action (if applicable).
   */
  elementState?: ElementState | null;
  /**
   * Error message if the action failed.
   */
  error?: string | null;
  /**
   * Action-specific return value.
   */
  result?: {
    [k: string]: unknown;
  };
  /**
   * Stack trace if the action threw an exception.
   */
  stack?: string | null;
  /**
   * Whether the action completed successfully.
   */
  success: boolean;
  /**
   * Unix-epoch millisecond timestamp when the action completed.
   */
  timestamp: number;
}
