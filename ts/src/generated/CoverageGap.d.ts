/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with `just generate-types` or
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */

import type { GapReason } from "./GapReason";
import type { SpecProvenance } from "./SpecProvenance";
import type { SpecSection } from "./SpecSection";

/**
 * One uncovered `Observed`/`Inferred` node — a work-list item the orchestration
 * reconciler re-dispatches generation for.
 */
export interface CoverageGap {
  /**
   * Free-form diagnostic detail (e.g. the verifier's near-miss explanation).
   */
  detail?: string | null;
  /**
   * Provenance of the gapped node — always `Observed` or `Inferred`.
   */
  nodeProvenance: SpecProvenance;
  /**
   * Why it is uncovered.
   */
  reason: GapReason;
  /**
   * Dotted ref into the spec, e.g. `"entities.Invoice.fields.amount"` or
   * `"operations.createInvoice"`.
   */
  ref: string;
  /**
   * Which section the gapped node belongs to.
   */
  section: SpecSection;
  [k: string]: unknown;
}
