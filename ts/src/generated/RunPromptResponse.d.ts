/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with `just generate-types` or
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */

import type { RunPromptResponseData } from './RunPromptResponseData';

/**
 * Response from the runner's `run_prompt` endpoint.
 */
export interface RunPromptResponse {
  /**
   * Structured data payload from a synchronous prompt run.
   */
  data?: RunPromptResponseData | null;
  /**
   * Error message if the call failed.
   */
  error?: string | null;
  /**
   * Path to the log file for the session.
   */
  log_file?: string | null;
  /**
   * Immediate output if the call ran synchronously.
   */
  output?: string | null;
  /**
   * OS process ID of the spawned AI session, if any.
   */
  pid?: number | null;
  /**
   * ID of the created AI session, if any.
   */
  session_id?: string | null;
  /**
   * Path to the state file tracking the session.
   */
  state_file?: string | null;
  /**
   * Whether the prompt was accepted and started successfully.
   */
  success: boolean;
  /**
   * ID of the created task run, if any.
   */
  task_run_id?: string | null;
  [k: string]: unknown;
}
