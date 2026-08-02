/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with `just generate-types` or
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */

import type { AssumptionEntry } from "./AssumptionEntry";
import type { AuthModel } from "./AuthModel";
import type { AuthRole } from "./AuthRole";
import type { Entity } from "./Entity";
import type { EntityField } from "./EntityField";
import type { IrAssertion } from "./IrAssertion";
import type { IrAssertionTarget } from "./IrAssertionTarget";
import type { IrCrossRef } from "./IrCrossRef";
import type { IrElementCriteria } from "./IrElementCriteria";
import type { IrMetadata } from "./IrMetadata";
import type { IrProvenance } from "./IrProvenance";
import type { IrState } from "./IrState";
import type { IrStateCondition } from "./IrStateCondition";
import type { IrTransition } from "./IrTransition";
import type { IrTransitionAction } from "./IrTransitionAction";
import type { IrWaitSpec } from "./IrWaitSpec";
import type { Operation } from "./Operation";
import type { OperationEffect } from "./OperationEffect";
import type { OperationInput } from "./OperationInput";
import type { ProposalStatus } from "./ProposalStatus";
import type { Relationship } from "./Relationship";
import type { SpecProvenance } from "./SpecProvenance";
import type { SpecTarget } from "./SpecTarget";
import type { ValidationRule } from "./ValidationRule";

/**
 * The v0 Functional Spec. Five sections, each node confidence- and
 * provenance-tagged. Serializes as an A2A `DataPart` inside a worker's
 * `completion_reports.artifacts`; it is the durable, fully re-derivable
 * hand-off artifact between conductor ticks (orchestration handoff contract §6).
 */
export interface FunctionalSpec {
  /**
   * Section (5) ASSUMPTIONS LEDGER — every node with provenance `Assumed`, collated so
   * the operator can review/override the generator's best-practice fills. For
   * v0 the override surface is a direct edit of this ledger (`overridable`).
   */
  assumptions?: AssumptionEntry[];
  /**
   * Section (4) AUTH / PERMISSION MODEL — from login flows + gated routes.
   */
  auth?: AuthModel | null;
  /**
   * Section (1) DOMAIN — entities + relationships inferred from rendered data shapes.
   */
  entities?: Entity[];
  /**
   * Section (3b) NAVIGATION — the exact `crate::ir::IrTransition` values
   * `IrPageSpec.transitions` carries. Reused (not re-declared) so app-gen's
   * emitted IR re-parses to the identical spec subset.
   */
  navigation?: IrTransition[];
  /**
   * Section (2) CAPABILITIES — operations the frontend exposes. Existence is
   * high-confidence; server-side effect is low-confidence by construction.
   */
  operations?: Operation[];
  /**
   * Schema version. Currently always `"0"`. Additive-only until `"1"`.
   */
  specVersion: string;
  target: SpecTarget;
  /**
   * Section (3a) UI STATES — a literal superset of the UI Bridge IR. These are the
   * exact `crate::ir::IrState` values `IrPageSpec.states` carries.
   */
  uiStates?: IrState[];
  [k: string]: unknown;
}
