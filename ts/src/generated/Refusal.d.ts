/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */

import type { GlossaryTerm } from "./GlossaryTerm";
import type { NextAction } from "./NextAction";
import type { NextActionKind } from "./NextActionKind";
import type { RefusalCode } from "./RefusalCode";
import type { RefusalSource } from "./RefusalSource";

/**
 * One operator-facing refusal: what went wrong, and what to do next.
 *
 * Construct with [`Refusal::new`] and the `with_*` builders. `next_action` is
 * deliberately not optional — see the module docs. Decoding is lenient toward
 * newer producers (module docs, "Forward compatibility").
 */
export interface Refusal {
  code: RefusalCode;
  /**
   * Raw diagnostic text (the underlying error, the unrecognised reason).
   * Shown beside the rendered sentence, never inside it.
   */
  detail?: string | null;
  /**
   * Narrows `code` to the specific case (the setting that was empty, the
   * service whose address was missing). Rendered after the headline.
   */
  discriminator?: string | null;
  /**
   * Glossary terms a reader may need to act on this refusal. Typed, so a
   * producer cannot cite a term the glossary does not define. Always
   * present on the wire (an empty list, never null).
   */
  glossary_terms: GlossaryTerm[];
  next_action: NextAction;
  /**
   * When the refusal was observed (ISO 8601).
   */
  observed_at: string;
  source: RefusalSource;
  /**
   * READER-SIDE: the raw `code` when this reader did not recognise it
   * (`code` is then [`RefusalCode::Unknown`]). Absent when the producer
   * itself said `unknown`. Producers must not emit it.
   */
  unrecognised_code?: string | null;
  /**
   * READER-SIDE: glossary ids this reader's glossary does not define,
   * removed from `glossary_terms`. Producers must not emit it.
   */
  unrecognised_glossary_terms?: string[];
  /**
   * READER-SIDE: the raw `source` when this reader did not recognise it
   * (`source` is then [`RefusalSource::Unrecognised`]). Producers must not
   * emit it.
   */
  unrecognised_source?: string | null;
  [k: string]: unknown;
}
