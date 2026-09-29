/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */

import type { UnknownCode } from "./UnknownCode";
import type { UnmeasuredDimension } from "./UnmeasuredDimension";

/**
 * How much of what the producer looked at it could actually measure.
 *
 * A non-empty [`Self::unmeasured`] is what makes a `measured` observation
 * DEGRADED (legal: the value stands, some dimension was not ruled out), and
 * what makes an `absent` observation ILLEGAL (nothing can be claimed absent
 * from a region that was not measured).
 */
export interface ObservationCoverage {
  /**
   * Items the producer considered.
   */
  considered: number;
  /**
   * Items it could measure in full.
   */
  measured: number;
  /**
   * Named dimensions it could not measure. Always serialized — an empty
   * list is the statement "full coverage".
   */
  unmeasured: UnmeasuredDimension[];
  [k: string]: unknown;
}
