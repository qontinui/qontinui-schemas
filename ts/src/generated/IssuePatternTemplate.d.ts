/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */

import type { IssuePatternTemplateParameter } from "./IssuePatternTemplateParameter";

/**
 * An issue pattern template for reusable detection strategies.
 */
export interface IssuePatternTemplate {
  ai_prompt_template: string | null;
  built_in: boolean;
  category: string;
  created_at: string;
  description: string;
  detection_type: string;
  id: string;
  name: string;
  parameters: IssuePatternTemplateParameter[];
  status: string;
  step_template: {
    [k: string]: unknown;
  } | null;
  updated_at: string;
}
