/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with `just generate-types` or
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */

/**
 * A single step in an AI Builder execution sequence.
 *
 * This is the **legacy step shape** persisted in `ai_workflows.json`. It
 * carries fields for every step flavour (`workflow`, `state`, `playwright`,
 * `prompt`, `action`, `screenshot`) as optional branches. Steps that don't
 * apply to the given `step_type` have their branch fields set to `None`.
 */
export interface ExecutionStep {
  /**
   * For action steps: the action type (`"click"`, `"double_click"`,
   * `"right_click"`).
   */
  actionType?: string | null;
  /**
   * Unique identifier for this step.
   */
  id: string;
  /**
   * Display name for the step.
   */
  name: string;
  /**
   * For playwright steps: the script content.
   */
  playwrightScriptContent?: string | null;
  /**
   * For playwright steps: the script ID.
   */
  playwrightScriptId?: string | null;
  /**
   * For playwright steps: the target URL.
   */
  playwrightTargetUrl?: string | null;
  /**
   * For prompt steps: the actual prompt content.
   */
  promptContent?: string | null;
  /**
   * For prompt steps: the prompt ID from the library.
   */
  promptId?: string | null;
  /**
   * Delay in seconds before taking screenshot (default 0).
   */
  screenshotDelay?: number | null;
  /**
   * Monitor for screenshot capture (number for specific monitor, `"all"`
   * for all monitors).
   */
  screenshotMonitor?: {
    [k: string]: unknown;
  };
  /**
   * Whether to capture a screenshot after this step.
   */
  takeScreenshot?: boolean;
  /**
   * For action steps: the target image ID.
   */
  targetImageId?: string | null;
  /**
   * For action steps: the target image name (for display).
   */
  targetImageName?: string | null;
  /**
   * Type of step: `"workflow"`, `"state"`, `"playwright"`, `"prompt"`,
   * `"action"`, or `"screenshot"`.
   */
  type: string;
}
