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
 * deliberately not optional — see the module docs.
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
   * refusal cannot cite a term the glossary does not define.
   */
  glossary_terms: GlossaryTerm[];
  next_action: NextAction;
  /**
   * When the refusal was observed (ISO 8601).
   */
  observed_at: string;
  source: RefusalSource;
}
