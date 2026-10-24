/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */

import type { SourceCitationKind } from "./SourceCitationKind";

/**
 * Evidence a requirement was derived from.
 */
export interface SourceCitation {
  kind: SourceCitationKind;
  /**
   * Where it is: a path and symbol, a document and heading, a ticket key,
   * an artifact's `spec_ref`.
   */
  locator: string;
  /**
   * The version cited (a commit, a document revision), when one applies.
   */
  version?: string | null;
  [k: string]: unknown;
}
