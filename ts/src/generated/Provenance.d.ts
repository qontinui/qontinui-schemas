/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */

import type { CacheProvenance } from "./CacheProvenance";
import type { ObservationCoverage } from "./ObservationCoverage";
import type { Producer } from "./Producer";
import type { UnknownCode } from "./UnknownCode";
import type { UnmeasuredDimension } from "./UnmeasuredDimension";

/**
 * Where an [`Observation`] came from.
 *
 * **Every key always serializes.** `observedAt`, `confidence`, `cache` and
 * `source` are `null` when they have nothing to say, and that `null` is a
 * statement rather than an omission (plan `489ed69e` lesson 1): a producer
 * that states "no sample was taken" / "this is a deduction" / "I have no
 * cache" / "I have no source attribution" is distinguishable on the wire
 * from one that said nothing — the latter is a payload that fails to parse.
 */
export interface Provenance {
  /**
   * `None` = this producer has no cache.
   */
  cache: CacheProvenance | null;
  /**
   * How far the answer rests on an ESTIMATED input. `None` is a positive
   * statement: the answer is a deduction, not an estimate — the same
   * reading as [`crate::Finding::confidence`].
   */
  confidence: number | null;
  coverage: ObservationCoverage;
  /**
   * When the producer ran. Never absent.
   */
  evaluatedAt: string;
  /**
   * When the underlying state was sampled (frame `capturedAt`, snapshot
   * time). `None` = no sample was taken (e.g. `input_missing`).
   */
  observedAt: string | null;
  producer: Producer;
  /**
   * Source attribution as a JSON object (a `SnapshotAttribution`, a
   * `FrameSource` projection, …). `None` = the producer has none to give.
   */
  source: {
    [k: string]: unknown;
  } | null;
  [k: string]: unknown;
}
