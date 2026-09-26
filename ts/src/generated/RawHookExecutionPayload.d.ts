/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */

import type { HookTrigger } from "./HookTrigger";

/**
 * Result of one lifecycle-hook execution.
 */
export interface RawHookExecutionPayload {
  duration_ms: number;
  error: string | null;
  hook_id: string;
  hook_name: string;
  output: string | null;
  success: boolean;
  timestamp: string;
  trigger: HookTrigger;
  [k: string]: unknown;
}
