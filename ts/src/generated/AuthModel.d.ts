/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with `just generate-types` or
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */

import type { AuthRole } from './AuthRole';
import type { SpecProvenance } from './SpecProvenance';

/**
 * The auth / permission model inferred from login flows + gated routes.
 */
export interface AuthModel {
  confidence: SpecProvenance;
  credibility?: number | null;
  /**
   * `"none"`, `"session"`, `"jwt"`, `"oauth"`, `"basic"`, …
   */
  model: string;
  provenance?: string | null;
  roles?: AuthRole[];
  [k: string]: unknown;
}
