/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */

import type { KnownIssueCategory } from "./KnownIssueCategory";
import type { KnownIssueDetectionMethod } from "./KnownIssueDetectionMethod";
import type { KnownIssueProvenance } from "./KnownIssueProvenance";
import type { KnownIssueScopeType } from "./KnownIssueScopeType";
import type { KnownIssueSeverity } from "./KnownIssueSeverity";

/**
 * Request to create a new known issue.
 */
export interface CreateKnownIssueRequest {
  category: KnownIssueCategory;
  description: string;
  detection_config?: {
    [k: string]: unknown;
  } | null;
  detection_method: KnownIssueDetectionMethod;
  pattern_template_id?: string | null;
  provenance?: KnownIssueProvenance | null;
  reproduction_context?: string | null;
  scope_tags?: string[] | null;
  scope_type: KnownIssueScopeType;
  scope_value?: string | null;
  severity: KnownIssueSeverity;
  source_finding_ids?: string[] | null;
  source_task_run_id?: string | null;
  title: string;
  trigger_conditions?: string[] | null;
  verification_hint?: string | null;
  verification_step_template?: {
    [k: string]: unknown;
  } | null;
  [k: string]: unknown;
}
