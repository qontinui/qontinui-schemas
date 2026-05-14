/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with `just generate-types` or
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */

import type { ProposalStatus } from './ProposalStatus';

export interface IrProvenance {
  column?: number | null;
  file?: string | null;
  line?: number | null;
  pluginVersion?: string | null;
  /**
   * "hand-authored" | "build-plugin" | "ai-generated" | "migrated"
   */
  source: string;
  /**
   * Lifecycle status for the flywheel coverage-growth loop. `None` =
   * implicitly `Promoted` (legacy + on-disk specs that predate the field).
   * Set to `Proposed` by `spec_authoring`, `Pending` when staged in
   * `_pending/`, `Promoted` after the 2-green sweep moves the file to
   * `pages/<id>/`.
   */
  status?: ProposalStatus | null;
  [k: string]: unknown;
}
