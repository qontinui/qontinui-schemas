/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */

import type { LegacyAssertion } from "./LegacyAssertion";
import type { LegacyAssertionTarget } from "./LegacyAssertionTarget";

export interface LegacyGroup {
  assertions: LegacyAssertion[];
  category: string;
  description: string;
  id: string;
  name: string;
  source: string;
  [k: string]: unknown;
}
