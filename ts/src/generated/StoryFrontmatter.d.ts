/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */

import type { CriterionVerification } from "./CriterionVerification";
import type { ExternalRef } from "./ExternalRef";
import type { SpecAcceptanceCriterion } from "./SpecAcceptanceCriterion";
import type { TemplateRef } from "./TemplateRef";

/**
 * A user story.
 */
export interface StoryFrontmatter {
  acceptance_criteria?: SpecAcceptanceCriterion[];
  external_refs?: ExternalRef[];
  id: string;
  /**
   * The `spec_ref`s of the requirements this story refines.
   */
  refines?: string[];
  template?: TemplateRef | null;
  [k: string]: unknown;
}
