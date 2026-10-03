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
 * producer looked with full coverage and found nothing), or `unknown` (the
 * producer could not answer; `unknown.code` says why). `provenance` is
 * always present and every one of its keys is always present.
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
