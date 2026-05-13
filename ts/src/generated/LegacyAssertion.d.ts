/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with `just generate-types` or
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */

import type { LegacyAssertionTarget } from './LegacyAssertionTarget';

export interface LegacyAssertion {
  assertionType: string;
  category: string;
  description: string;
  enabled: boolean;
  id: string;
  precondition?: string | null;
  reviewed: boolean;
  severity: string;
  source: string;
  target: LegacyAssertionTarget;
  [k: string]: unknown;
}
