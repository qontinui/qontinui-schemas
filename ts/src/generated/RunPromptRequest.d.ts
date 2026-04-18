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
  displayPrompt?: string | null;
  /**
   * Attached image paths.
   */
  imagePaths?: string[] | null;
  /**
   * Optional cap on AI sessions.
   */
  maxSessions?: number | null;
  /**
   * Cap on trace screenshots to include.
   */
  maxTraceScreenshots?: number | null;
  /**
   * Cap on video frames to extract for the prompt.
   */
  maxVideoFrames?: number | null;
  /**
   * Display name for the task.
   */
  name: string;
  /**
   * Hard timeout in seconds.
   */
  timeoutSeconds?: number | null;
  /**
   * Optional path to a trace file.
   */
  tracePath?: string | null;
  /**
   * Attached video paths.
   */
  videoPaths?: string[] | null;
}
