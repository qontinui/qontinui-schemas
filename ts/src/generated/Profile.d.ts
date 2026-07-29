/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with `just generate-types` or
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */

import type { ApiStyle } from "./ApiStyle";
import type { ArchitectureProfile } from "./ArchitectureProfile";
import type { BackendProfile } from "./BackendProfile";
import type { EnforcementProfile } from "./EnforcementProfile";

/**
 * The top-level Priorities Profile.
 */
export interface Profile {
  architecture: ArchitectureProfile;
  backend: BackendProfile;
  /**
   * Naming / mapping conventions, as free-form `key -> value` pairs (e.g.
   * `"naming" -> "snake_case"`, `"entityToTable" -> "pluralize"`). A `BTreeMap`
   * so JSON serialization is deterministic. Open-ended by design so a profile
   * can express conventions the schema didn't anticipate.
   */
  conventions?: {
    [k: string]: string;
  };
  /**
   * How the generation loop enforces quality.
   */
  enforcement?: EnforcementProfile | null;
  /**
   * Schema version. Currently always `"0"`. Additive-only until `"1"`.
   */
  profileVersion: string;
  [k: string]: unknown;
}
