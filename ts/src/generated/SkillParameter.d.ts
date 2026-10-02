/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */

import type { SkillParameterDependency } from "./SkillParameterDependency";
import type { SkillParameterOption } from "./SkillParameterOption";
import type { SkillParameterType } from "./SkillParameterType";

/**
 * A single skill / prompt-template parameter.
 *
 * `label`, `description` and `required` stay non-`Option` and carry no
 * `skip_serializing_if`: defaulting happens on the way IN (see the manual
 * [`Deserialize`] impl below), never on the way OUT, so every serialized
 * payload still carries all three keys — which is why the generated schema
 * (and so `qontinui-schemas/ts/src/generated/SkillParameter.d.ts`) marks them
 * required.
 */
export interface SkillParameter {
  default?: unknown;
  depends_on?: SkillParameterDependency;
  description: string;
  label: string;
  max?: number;
  min?: number;
  name: string;
  options?: SkillParameterOption[];
  pattern?: string;
  placeholder?: string;
  required: boolean;
  type: SkillParameterType | string;
}
