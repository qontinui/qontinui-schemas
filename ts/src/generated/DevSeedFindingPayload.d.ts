/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with `just generate-types` or
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */

/**
 * Payload shape emitted on the `dev:seed-finding` Tauri event.
 *
 * Field names use camelCase so the TS listener (`TauriFindingsListener.ts`)
 * can spread them directly into a `Finding` object without translation.
 * The actual emit site is in `commands::dev_findings::dev_seed_finding`.
 *
 * Renamed in the schema registry to `DevSeedFindingPayload` to disambiguate
 * from the various `Finding*` types in `qontinui_types::findings`.
 */
export interface DevSeedFindingPayload {
  actionType: string;
  actionable: boolean;
  categoryId: string;
  description: string;
  detectedAt: number;
  id: string;
  severity: string;
  sourceSessionId?: string | null;
  status: string;
  title: string;
  [k: string]: unknown;
}
