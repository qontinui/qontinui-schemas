/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with `just generate-types` or
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */

import type { IrAssertion } from "./IrAssertion";
import type { IrAssertionTarget } from "./IrAssertionTarget";
import type { IrCrossRef } from "./IrCrossRef";
import type { IrElementCriteria } from "./IrElementCriteria";
import type { IrGroup } from "./IrGroup";
import type { IrMetadata } from "./IrMetadata";
import type { IrProvenance } from "./IrProvenance";
import type { IrState } from "./IrState";
import type { IrStateCondition } from "./IrStateCondition";
import type { IrTransition } from "./IrTransition";
import type { IrTransitionAction } from "./IrTransitionAction";
import type { IrWaitSpec } from "./IrWaitSpec";
import type { ProposalStatus } from "./ProposalStatus";

/**
 * Top-level IR page specification (formerly `IrDocument`).
 *
 * One `IrPageSpec` corresponds to one `state-machine.derived.json` file on
 * disk under `qontinui-runner/specs/pages/<id>/`. This is the authoring
 * surface — the projection layer derives the runtime `LegacySpec` from this.
 */
export interface IrPageSpec {
  /**
   * Per-page API-contract assertions evaluated by the Spec CI harness.
   * Phase 1: value-level assertions (status_code, json_path, header,
   * body_contains, response_time). Phase 2 will add schema conformance.
   */
  apiAssertions?: IrApiCheck[] | null;
  description?: string | null;
  id: string;
  initialState?: string | null;
  metadata?: IrMetadata | null;
  name: string;
  provenance?: IrProvenance | null;
  states: IrState[];
  /**
   * Groups produced by workflow-criteria synthesis (NOT derived from
   * `IRState` annotations). Mirror of TS `IRDocument.synthesizedGroups`.
   * The `rename = "synthesizedGroups"` is explicit (rather than relying on
   * `rename_all = "camelCase"`) so the serialized name is unambiguous in
   * snapshot tests + cross-language fixtures.
   */
  synthesizedGroups?: IrGroup[] | null;
  transitions: IrTransition[];
  /**
   * Schema version. Currently always `"1.0"`.
   */
  version: string;
  [k: string]: unknown;
}
