/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with `just generate-types` or
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */

import type { AssertionScope } from "./AssertionScope";
import type { ConjunctRule } from "./ConjunctRule";
import type { MatchOutcome } from "./MatchOutcome";
import type { PolicyConjunct } from "./PolicyConjunct";
import type { SpecCheckPolicy } from "./SpecCheckPolicy";

/**
 * Workflow-step variant payload — the persisted shape of a
 * `spec_check` step in a unified workflow.
 */
export interface SpecCheckStepConfig {
  /**
   * Severities that should cause the step to fail when present in
   * `severity_counts`. Empty means "all severities count as failure".
   */
  failOn?: string[];
  /**
   * If `true`, the step fails when the target app isn't connected to
   * the bridge.
   */
  failWhenNoApp: boolean;
  /**
   * If `true`, the step fails when no spec exists for `page_id`.
   */
  failWhenNoSpec: boolean;
  /**
   * Page to evaluate.
   */
  pageId: string;
  policy: SpecCheckPolicy;
}
