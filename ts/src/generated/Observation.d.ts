/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */

import type { CacheProvenance } from "./CacheProvenance";
import type { ObservationCoverage } from "./ObservationCoverage";
import type { Producer } from "./Producer";
import type { Provenance } from "./Provenance";
import type { UnknownCode } from "./UnknownCode";
import type { UnknownInfo } from "./UnknownInfo";
import type { UnmeasuredDimension } from "./UnmeasuredDimension";

/**
 * One UI Bridge observation: `measured` (with `value`), `absent` (the
 * producer considered at least one item, measured all of it, and found
 * nothing), or `unknown` (the producer could not answer; `unknown.code` says
 * why). `provenance` is always present and every one of its keys is always
 * present.
 *
 * This type describes the wire SHAPE only; it is not a validator. It
 * accepts envelopes the canonical parser refuses: `absent` carrying `value`
 * or `unknown`, a present `"unknown": null`, and `absent` over non-empty
 * `provenance.coverage.unmeasured` or over `provenance.coverage.considered`
 * of 0. Parse an envelope you did not build with
 * `qontinui_vision_core::Observation` (Rust) or a parser enforcing the same
 * rules.
 */
export type Observation =
  | {
      provenance: Provenance;
      status: "measured";
      value: unknown;
      [k: string]: unknown;
    }
  | {
      provenance: Provenance;
      status: "absent";
      [k: string]: unknown;
    }
  | {
      provenance: Provenance;
      status: "unknown";
      unknown: UnknownInfo;
      [k: string]: unknown;
    };
