/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */

/**
 * Request to create a new pattern template.
 */
export interface CreatePatternTemplateRequest {
  ai_prompt_template?: string | null;
  category: string;
  description: string;
  detection_type: string;
  name: string;
  parameters?: string | null;
  [k: string]: unknown;
}
