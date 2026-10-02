/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */

import type { SkillAllowedPhase } from "./SkillAllowedPhase";
import type { SkillApprovalStatus } from "./SkillApprovalStatus";
import type { SkillAuthor } from "./SkillAuthor";
import type { SkillCategory } from "./SkillCategory";
import type { SkillParameter } from "./SkillParameter";
import type { SkillParameterDependency } from "./SkillParameterDependency";
import type { SkillParameterOption } from "./SkillParameterOption";
import type { SkillParameterType } from "./SkillParameterType";
import type { SkillPlaybookTrigger } from "./SkillPlaybookTrigger";
import type { SkillRef } from "./SkillRef";
import type { SkillSource } from "./SkillSource";
import type { SkillTemplate } from "./SkillTemplate";

export interface SkillDefinition {
  allowed_phases: (SkillAllowedPhase | string)[];
  approval_status?: SkillApprovalStatus;
  author?: SkillAuthor;
  category: SkillCategory | string;
  checksum?: string;
  color: string;
  depends_on?: string[];
  description: string;
  forked_from?: string;
  icon: string;
  id: string;
  name: string;
  parameters: SkillParameter[];
  slug: string;
  /**
   * Provenance. Typed rather than free text — see [`SkillSource`].
   */
  source: SkillSource | string;
  tags: string[];
  template: SkillTemplate;
  usage_count?: number;
  version?: string;
}
