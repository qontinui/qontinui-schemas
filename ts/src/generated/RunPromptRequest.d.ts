/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with `just generate-types` or
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */

/**
 * Request body for the runner's `run_prompt` endpoint.
 */
export interface RunPromptRequest {
  /**
   * Prompt content (the actual text sent to the AI).
   */
  content: string;
  /**
   * Optional free-form context string appended to the prompt.
   */
  context?: string | null;
  /**
   * Display-only version of the prompt (shown in the UI).
   */
  display_prompt?: string | null;
  /**
   * Attached image paths.
   */
  image_paths?: string[] | null;
  /**
   * Optional cap on AI sessions.
   */
  max_sessions?: number | null;
  /**
   * Cap on trace screenshots to include.
   */
  max_trace_screenshots?: number | null;
  /**
   * Cap on video frames to extract for the prompt.
   */
  max_video_frames?: number | null;
  /**
   * Display name for the task.
   */
  name: string;
  /**
   * Hard timeout in seconds.
   */
  timeout_seconds?: number | null;
  /**
   * Optional path to a trace file.
   */
  trace_path?: string | null;
  /**
   * Attached video paths.
   */
  video_paths?: string[] | null;
  [k: string]: unknown;
}
