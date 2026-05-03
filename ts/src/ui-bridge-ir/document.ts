/**
 * Top-level IR document.
 *
 * Versioned (`'1.0'`) container for the states + transitions that describe
 * a page or scope of an application. Build plugins emit this shape; the
 * adapter folds it into ui-bridge-auto's `WorkflowConfig`.
 */

import type { IRMetadata, IRProvenance } from "./primitives";
import type { IRState } from "./state";
import type { IRTransition } from "./transition";

/**
 * IR schema version. Bump when the shape changes in a backwards-incompatible
 * way. The adapter rejects documents whose version it does not understand.
 */
export type IRVersion = "1.0";

/**
 * Axes that the spec <-> IR pairing gate compares. A document's
 * `pairingPolicy.acceptDrift.axes` lists which of these are
 * intentionally allowed to diverge between the legacy spec and the
 * IR's forward projection.
 */
export type IRPairingAxis = "groupCount" | "groupIds" | "assertionCount";

/**
 * Per-document opt-out for specific spec<->IR pairing axes.
 *
 * The canonical use case is a hand-authored IR that uses a different
 * grouping convention than the legacy spec. Without an explicit policy
 * the pairing gate flags it as a mismatch on every PR, which trains
 * reviewers to ignore the gate. The policy makes the divergence
 * explicit and audit-friendly:
 *
 *   - `axes`     — which fingerprint comparisons to skip
 *   - `reason`   — human-readable justification (required, surfaced in
 *                  `check-spec-pairing` summary output)
 *   - `since`    — ISO date (YYYY-MM-DD) the exemption was added
 *   - `expiresAt` — optional ISO date (YYYY-MM-DD) after which the
 *                   exemption stops applying. Forces periodic
 *                   re-justification: the pairing gate downgrades
 *                   expired exemptions back to ordinary mismatches.
 *
 * Mismatches NOT covered by the listed `axes` still fail the gate as
 * usual — you can accept drift on `groupCount` while still requiring
 * the assertion total to match.
 */
export interface IRAcceptDriftPolicy {
  /** Fingerprint axes whose mismatches are tolerated. */
  axes: IRPairingAxis[];
  /** Human-readable justification for the exemption. */
  reason: string;
  /** ISO date (YYYY-MM-DD) the exemption was added. */
  since: string;
  /**
   * Optional ISO date (YYYY-MM-DD) after which the exemption stops
   * applying. The pairing gate then surfaces the document as an
   * ordinary mismatch.
   */
  expiresAt?: string;
}

/**
 * Per-document pairing policy block. Currently only carries
 * `acceptDrift`; future policies (e.g. assertion-shape exemptions)
 * can extend this without breaking the IR schema.
 */
export interface IRPairingPolicy {
  /** Opt-out from specific spec<->IR pairing mismatch axes. */
  acceptDrift?: IRAcceptDriftPolicy;
}

/**
 * Top-level IR document.
 */
export interface IRDocument {
  /** Schema version. */
  version: IRVersion;
  /** Stable document identifier (typically the page or scope name). */
  id: string;
  /** Human-readable document name. */
  name: string;
  /** Optional description of what this document covers. */
  description?: string;
  /** Document-level semantic metadata. */
  metadata?: IRMetadata;
  /** Where this document came from. */
  provenance?: IRProvenance;

  /**
   * Per-document policy block — opt-out from specific spec<->IR
   * pairing mismatches. Used by hand-authored IRs that intentionally
   * diverge from the legacy spec shape.
   */
  pairingPolicy?: IRPairingPolicy;

  /** State declarations within this document. */
  states: IRState[];
  /** Transition declarations within this document. */
  transitions: IRTransition[];

  /** ID of the initial/starting state, if applicable. */
  initialState?: string;
}
