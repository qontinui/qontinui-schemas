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

  /** State declarations within this document. */
  states: IRState[];
  /** Transition declarations within this document. */
  transitions: IRTransition[];

  /** ID of the initial/starting state, if applicable. */
  initialState?: string;
}
