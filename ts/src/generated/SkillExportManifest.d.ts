/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */

import type { SkillExportContentType } from "./SkillExportContentType";

export interface SkillExportManifest {
  app_version: string;
  checksum?: string;
  content_type: SkillExportContentType;
  exported_at: string;
  skill_count: number;
  version: string;
  [k: string]: unknown;
}
