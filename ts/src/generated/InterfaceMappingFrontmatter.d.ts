/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */

import type { MappingRow } from "./MappingRow";
import type { TemplateRef } from "./TemplateRef";

/**
 * A field-level mapping between two systems.
 */
export interface InterfaceMappingFrontmatter {
  id: string;
  rows?: MappingRow[];
  source_system: string;
  target_system: string;
  template?: TemplateRef | null;
  [k: string]: unknown;
}
