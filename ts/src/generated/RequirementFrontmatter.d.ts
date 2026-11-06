/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */

import type { CriterionVerification } from "./CriterionVerification";
import type { EarsPattern } from "./EarsPattern";
import type { ExternalRef } from "./ExternalRef";
import type { SourceCitation } from "./SourceCitation";
import type { SourceCitationKind } from "./SourceCitationKind";
import type { SpecAcceptanceCriterion } from "./SpecAcceptanceCriterion";
import type { TemplateRef } from "./TemplateRef";

/**
 * A requirement.
 */
export interface RequirementFrontmatter {
  /**
   * May be empty: completeness is the spec quality gate's verdict, not a
   * parse error.
   */
  acceptance_criteria?: SpecAcceptanceCriterion[];
  ears?: EarsPattern & string;
  external_refs?: ExternalRef[];
  id: string;
  sources?: SourceCitation[];
  template?: TemplateRef | null;
  [k: string]: unknown;
}
