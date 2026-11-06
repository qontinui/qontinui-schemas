/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */

import type { CriterionVerification } from "./CriterionVerification";
import type { EarsPattern } from "./EarsPattern";
import type { ExternalRef } from "./ExternalRef";
import type { InterfaceMappingFrontmatter } from "./InterfaceMappingFrontmatter";
import type { MappingRow } from "./MappingRow";
import type { RequestFrontmatter } from "./RequestFrontmatter";
import type { RequirementFrontmatter } from "./RequirementFrontmatter";
import type { SourceCitation } from "./SourceCitation";
import type { SourceCitationKind } from "./SourceCitationKind";
import type { SpecAcceptanceCriterion } from "./SpecAcceptanceCriterion";
import type { StoryFrontmatter } from "./StoryFrontmatter";
import type { TemplateRef } from "./TemplateRef";

/**
 * The frontmatter of a spec artifact, tagged by its work-artifact `kind`.
 */
export type SpecFrontmatter =
  | (RequestFrontmatter & {
      kind: "request";
    })
  | (RequirementFrontmatter & {
      kind: "requirement";
    })
  | (InterfaceMappingFrontmatter & {
      kind: "interface_mapping";
    })
  | (StoryFrontmatter & {
      kind: "story";
    });
