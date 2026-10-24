/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */

import type { CacheProvenance } from "./CacheProvenance";
import type { DetectedAnimation } from "./DetectedAnimation";
import type { Observation } from "./Observation";
import type { ObservationCoverage } from "./ObservationCoverage";
import type { Producer } from "./Producer";
import type { Provenance } from "./Provenance";
import type { UnknownCode } from "./UnknownCode";
import type { UnknownInfo } from "./UnknownInfo";
import type { UnmeasuredDimension } from "./UnmeasuredDimension";

/**
 * A single event in the interaction timeline, discriminated on `type`.
 */
export type TimelineEvent =
  | {
      /**
       * Time offset from action start (ms).
       */
      offsetMs: number;
      /**
       * Human-readable summary (SDK prose — see the module's PRIVACY note).
       */
      summary: string;
      type: "action";
      [k: string]: unknown;
    }
  | {
      /**
       * Number of elements affected.
       */
      count: number;
      /**
       * Element IDs involved.
       */
      elementIds: string[];
      /**
       * Time offset from action start (ms).
       */
      offsetMs: number;
      /**
       * Human-readable summary (SDK prose — see the module's PRIVACY note).
       */
      summary: string;
      type: "elements-appeared";
      [k: string]: unknown;
    }
  | {
      /**
       * Number of elements affected.
       */
      count: number;
      /**
       * Element IDs involved.
       */
      elementIds: string[];
      /**
       * Time offset from action start (ms).
       */
      offsetMs: number;
      /**
       * Human-readable summary (SDK prose — see the module's PRIVACY note).
       */
      summary: string;
      type: "elements-disappeared";
      [k: string]: unknown;
    }
  | {
      /**
       * Number of elements affected.
       */
      count: number;
      /**
       * Element IDs involved.
       */
      elementIds: string[];
      /**
       * Time offset from action start (ms).
       */
      offsetMs: number;
      /**
       * Human-readable summary (SDK prose — see the module's PRIVACY note).
       */
      summary: string;
      type: "elements-modified";
      [k: string]: unknown;
    }
  | {
      /**
       * Time offset from action start (ms).
       */
      offsetMs: number;
      /**
       * Human-readable summary (SDK prose — see the module's PRIVACY note).
       */
      summary: string;
      type: "page-changed";
      /**
       * The new URL, or `null` when the diff did not carry one.
       */
      url?: string | null;
      [k: string]: unknown;
    }
  | {
      /**
       * Time offset from action start (ms).
       */
      offsetMs: number;
      /**
       * Human-readable summary (SDK prose — see the module's PRIVACY note).
       */
      summary: string;
      type: "settled";
      [k: string]: unknown;
    }
  | {
      /**
       * The browser's `hadRecentInput` flag for this shift.
       */
      hadRecentInput: boolean;
      /**
       * Time offset from action start (ms).
       */
      offsetMs: number;
      /**
       * How many shifted elements `shiftedFingerprints` named. NOT sent by
       * the SDK: [`ChangeTimeline::for_ledger`] sets it when it empties
       * `shiftedFingerprints`, so a stored timeline keeps the count and
       * not the hashes. Absent on the wire from the SDK.
       */
      shiftedElementCount?: number | null;
      /**
       * `ElementFingerprintData.hash` of each shifted element still in the
       * document.
       */
      shiftedFingerprints: string[];
      /**
       * Human-readable summary (SDK prose — see the module's PRIVACY note).
       */
      summary: string;
      type: "layout-shift";
      /**
       * Shift sources whose node was gone (or not an element) when the
       * entry was read.
       */
      unattributedSourceCount: number;
      /**
       * The shift score (Layout Instability API `value`).
       */
      value: number;
      [k: string]: unknown;
    }
  | {
      /**
       * The animation.
       */
      animation: DetectedAnimation;
      /**
       * Time offset from action start (ms).
       */
      offsetMs: number;
      /**
       * Human-readable summary (SDK prose — see the module's PRIVACY note).
       */
      summary: string;
      type: "animation-started";
      [k: string]: unknown;
    }
  | {
      /**
       * The animation.
       */
      animation: DetectedAnimation;
      /**
       * Finished or cancelled.
       */
      endReason: "finished" | "cancelled";
      /**
       * Time offset from action start (ms).
       */
      offsetMs: number;
      /**
       * Human-readable summary (SDK prose — see the module's PRIVACY note).
       */
      summary: string;
      type: "animation-ended";
      [k: string]: unknown;
    }
  | {
      /**
       * Task duration (ms).
       */
      durationMs: number;
      /**
       * Time offset from action start (ms).
       */
      offsetMs: number;
      /**
       * Human-readable summary (SDK prose — see the module's PRIVACY note).
       */
      summary: string;
      type: "long-task";
      [k: string]: unknown;
    }
  | {
      /**
       * The request URL carried a query string (not reported).
       */
      hasQuery: boolean;
      /**
       * HTTP method.
       */
      method: string;
      /**
       * Time offset from action start (ms).
       */
      offsetMs: number;
      /**
       * The tracker's request id, or `null` when the source supplies none.
       */
      requestId?: string | null;
      /**
       * Human-readable summary (SDK prose — see the module's PRIVACY note).
       */
      summary: string;
      type: "network-request-started";
      /**
       * Origin + pathname — never the query or fragment. Always a string
       * from the SDK; `null` only in [`ChangeTimeline::for_ledger`]'s
       * projection, when the URL had no valid `scheme://host` origin.
       */
      url?: string | null;
      [k: string]: unknown;
    }
  | {
      /**
       * Request duration as the tracker measured it, or `null` without a
       * response.
       */
      durationMs?: number | null;
      /**
       * The request threw (no HTTP response). An HTTP error status is NOT
       * `errored` — read `statusCode`.
       */
      errored: boolean;
      /**
       * The request URL carried a query string (not reported).
       */
      hasQuery: boolean;
      /**
       * HTTP method.
       */
      method: string;
      /**
       * Time offset from action start (ms).
       */
      offsetMs: number;
      /**
       * The tracker's request id, or `null` when the source supplies none.
       */
      requestId?: string | null;
      /**
       * HTTP status, or `null` when no response arrived.
       */
      statusCode?: number | null;
      /**
       * Human-readable summary (SDK prose — see the module's PRIVACY note).
       */
      summary: string;
      type: "network-request-finished";
      /**
       * Origin + pathname — never the query or fragment. Always a string
       * from the SDK; `null` only in [`ChangeTimeline::for_ledger`]'s
       * projection, when the URL had no valid `scheme://host` origin.
       */
      url?: string | null;
      [k: string]: unknown;
    }
  | {
      /**
       * Time offset from action start (ms).
       */
      offsetMs: number;
      /**
       * The declared state id.
       */
      stateId: string;
      /**
       * Human-readable summary (SDK prose — see the module's PRIVACY note).
       */
      summary: string;
      type: "state-entered";
      [k: string]: unknown;
    }
  | {
      /**
       * Time offset from action start (ms).
       */
      offsetMs: number;
      /**
       * The declared state id.
       */
      stateId: string;
      /**
       * Human-readable summary (SDK prose — see the module's PRIVACY note).
       */
      summary: string;
      type: "state-exited";
      [k: string]: unknown;
    };
/**
 * A source of timeline events. A source NOT listed in
 * [`ChangeTimeline::observed_sources`] was not looked at, so the absence of
 * its events means "unknown", never "none".
 */
export type TimelineSource =
  | "snapshot-poll"
  | "idle-detector"
  | "layout-shift"
  | "long-task"
  | "animation-events"
  | "get-animations"
  | "network"
  | "states";
/**
 * How an expected state fared inside the caller's `withinMs` window. Exactly
 * one key on the wire.
 */
export type ExpectedStateOutcome =
  | {
      enteredAtMs: number;
    }
  | {
      notEnteredWithinMs: number;
    }
  | {
      activeThroughoutWithinMs: number;
    };

/**
 * The interaction timeline: action → every observed event → settled, each
 * with its offset from the action, plus a derived [`TimelineSummary`].
 */
export interface ChangeTimeline {
  /**
   * Timestamped events, ordered by `offsetMs` (ties keep arrival order).
   */
  events: TimelineEvent[];
  /**
   * The sources this recording could observe.
   */
  observedSources: TimelineSource[];
  /**
   * How long the timeline was recorded, from action start (ms): the settle
   * window, extended by any `withinMs` window. Not the settle time.
   */
  recordedMs: number;
  /**
   * Whether the UI reached a stable state within `settleTimeout`.
   */
  settled: boolean;
  summary: TimelineSummary;
  [k: string]: unknown;
}
/**
 * Values derived from `events`.
 */
export interface TimelineSummary {
  /**
   * Animations/transitions running when recording ended. Degraded
   * (`coverage.unmeasured`) where `document.getAnimations` is missing.
   */
  animationsUnsettledAtEnd:
    | {
        provenance: Provenance;
        status: "measured";
        value: DetectedAnimation[];
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
  /**
   * One entry per requested `expectStates` id, in request order; `[]` when
   * none were requested.
   */
  expectedStates: ExpectedStateResult[];
  /**
   * Offset of the first RESPONSE event (elements-*, page-changed,
   * layout-shift, animation-*, state-*). `unknown{no_event_before_timeout}`
   * when nothing responded in the window.
   */
  firstResponseMs: Observation;
  /**
   * Offset of the LAST layout shift minus `firstResponseMs`. `absent` when
   * no shift occurred.
   */
  layoutShiftAfterFirstResponseMs: Observation;
  /**
   * Number of `layout-shift` events. Same unknown rule.
   */
  layoutShiftCount:
    | {
        provenance: Provenance;
        status: "measured";
        value: number;
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
  /**
   * Sum of `layout-shift` values in the window.
   * `unknown{performance_observer_unsupported}` where the entry type is
   * missing.
   */
  layoutShiftTotal: Observation;
  /**
   * Sum of `long-task` durations (ms).
   * `unknown{performance_observer_unsupported}` where `longtask` is
   * missing.
   */
  longTaskTotalMs: Observation;
  /**
   * Offset of the `settled` event. `unknown{no_event_before_timeout}` when
   * the UI did not settle within `settleTimeout`.
   */
  settleMs: Observation;
  [k: string]: unknown;
}
/**
 * One `expectStates` entry and its answer.
 */
export interface ExpectedStateResult {
  /**
   * The answer — `unknown{input_missing}` for an undeclared state or no
   * state registry.
   */
  outcome:
    | {
        provenance: Provenance;
        status: "measured";
        value: ExpectedStateOutcome;
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
  /**
   * The declared state id the caller asked about.
   */
  stateId: string;
  [k: string]: unknown;
}
