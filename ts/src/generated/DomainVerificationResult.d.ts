/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with `just generate-types` or
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */

import type { Confidence } from './Confidence';
import type { CriterionType } from './CriterionType';
import type { VerificationResult } from './VerificationResult';

/**
 * Result of domain-scoped verification.
 */
export interface DomainVerificationResult {
  /**
   * Whether all domain criteria passed.
   */
  all_passed: boolean;
  /**
   * Domain that was verified.
   */
  domain_id: string;
  /**
   * Summary of any failures.
   */
  failure_summary?: string | null;
  /**
   * Verification results for domain-specific criteria.
   */
  results: VerificationResult[];
  /**
   * Workers that contributed to this domain.
   */
  worker_ids: string[];
  [k: string]: unknown;
}
