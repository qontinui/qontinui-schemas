/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with `just generate-types` or
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */

import type { Confidence } from './Confidence';

/**
 * A finding recorded by a worker agent.
 */
export interface Finding {
  confidence: Confidence;
  /**
   * Description of the finding.
   */
  description: string;
  /**
   * Supporting evidence (file paths, log excerpts, etc.).
   */
  evidence?: string | null;
  /**
   * Type of finding.
   *
   * Valid types: `"bug"`, `"root_cause"`, `"observation"`, `"hypothesis"`,
   * `"solution"`, `"environment"`. Note: `"environment"` findings (PATH
   * issues, disk space, tools not installed) require user intervention and
   * should NOT trigger automatic retries.
   */
  findingType: string;
  /**
   * Unique identifier.
   */
  id: string;
  /**
   * Related file paths.
   */
  relatedFiles?: string[];
  [k: string]: unknown;
}
