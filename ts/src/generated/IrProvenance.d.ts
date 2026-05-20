/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with `just generate-types` or
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */

import type { ProposalStatus } from './ProposalStatus';

export interface IrProvenance {
  /**
   * Owning app id (slug). Multi-tenant Spec API contract: every IR
   * document is self-describing about which registered app it belongs
   * to. `#[serde(default)]` allows pre-multi-app on-disk IRs (no
   * `app_id` field) to deserialize as empty string; handlers and the
   * `POST /apps/<id>/spec/author` adapter validate non-empty and fill
   * from the URL path on read (Stream C safety-net). Stream F's bulk
   * backfill rewrites every on-disk IR so empty values don't persist.
   * TS / Python codegen promote serde defaults to required per the
   * `qontinui-schemas` project convention.
   *
   * `skip_serializing_if = "String::is_empty"` preserves byte-identical
   * round-trip for pre-multi-app on-disk IRs that lack the field.
   * Post-Stream-F backfill, every IR has a non-empty `app_id` so the
   * skip never fires.
   */
  appId?: string;
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
