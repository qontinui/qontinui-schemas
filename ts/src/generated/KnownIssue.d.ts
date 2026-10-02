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
import type { KnownIssueStatus } from "./KnownIssueStatus";

/**
 * A persistent known issue that survives across workflow runs.
 */
export interface KnownIssue {
  category: KnownIssueCategory;
  confidence: number;
  created_at: string;
  description: string;
  detection_config: {
    [k: string]: unknown;
  };
  detection_method: KnownIssueDetectionMethod;
  id: string;
  last_checked_at: string | null;
  last_detected_at: string | null;
  pattern_template_id: string | null;
  provenance: KnownIssueProvenance;
  reproduction_context: string | null;
  resolved_at: string | null;
  scope_tags: string[];
  scope_type: KnownIssueScopeType;
  scope_value: string | null;
  severity: KnownIssueSeverity;
  source_finding_ids: string[];
  source_task_run_id: string | null;
  status: KnownIssueStatus;
  times_checked: number;
  times_detected: number;
  title: string;
  trigger_conditions: string[];
  updated_at: string;
  verification_hint: string | null;
  verification_step_template: {
    [k: string]: unknown;
  } | null;
}
