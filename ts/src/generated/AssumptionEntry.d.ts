/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with `just generate-types` or
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */

/**
 * One collated assumption — a node whose provenance is `Assumed`. The operator
 * reviews/overrides these directly (v0 override surface = a file edit of this
 * ledger).
 */
export interface AssumptionEntry {
  /**
   * The best-practice default the generator applied.
   */
  defaultApplied: string;
  /**
   * Optional free-form operator note (e.g. an override rationale).
   */
  note?: string | null;
  /**
   * Whether the operator may override this fill. Defaults to `true`.
   */
  overridable: boolean;
  /**
   * Dotted ref into the spec the assumption belongs to, e.g.
   * `"operations.createInvoice.effect"`.
   */
  ref: string;
  [k: string]: unknown;
}
