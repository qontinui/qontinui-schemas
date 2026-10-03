/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */

import type { KnownIssueCategory } from "./KnownIssueCategory";
import type { KnownIssueDetectionMethod } from "./KnownIssueDetectionMethod";
import type { KnownIssueScopeType } from "./KnownIssueScopeType";
import type { KnownIssueSeverity } from "./KnownIssueSeverity";
import type { KnownIssueStatus } from "./KnownIssueStatus";

/**
 * Request to update an existing known issue.
 */
export interface UpdateKnownIssueRequest {
  category?: KnownIssueCategory | null;
  confidence?: number | null;
  description?: string | null;
  detection_config?: {
    [k: string]: unknown;
  } | null;
  detection_method?: KnownIssueDetectionMethod | null;
  pattern_template_id?: string | null;
  reproduction_context?: string | null;
  scope_tags?: string[] | null;
  scope_type?: KnownIssueScopeType | null;
  scope_value?: string | null;
  severity?: KnownIssueSeverity | null;
  status?: KnownIssueStatus | null;
  title?: string | null;
  trigger_conditions?: string[] | null;
  verification_hint?: string | null;
  verification_step_template?: {
    [k: string]: unknown;
  } | null;
}
