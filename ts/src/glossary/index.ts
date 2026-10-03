/**
 * The product glossary, as data — `@qontinui/shared-types/glossary`.
 *
 * Plan `2026-09-20-the-published-product-works-without-knowing-a-development-environment-exists`,
 * Phase C3. The table in `./generated.ts` is generated from the repo-root
 * `glossary/terms.toml` by the same code that generates the Rust
 * `GLOSSARY` (`rust/tests/glossary_source.rs`), so every surface shows the
 * same definition. It is compiled in, not fetched: a surface renders a
 * definition with no request, so it still renders one when the coordination
 * service is unreachable.
 */

import type { GlossaryTerm } from "../generated/GlossaryTerm";
import {
  GLOSSARY,
  GLOSSARY_CONTENT_SHA256,
  GLOSSARY_TERMS,
  GLOSSARY_VERSION,
  type GlossaryEntry,
} from "./generated";

export type { GlossaryTerm, GlossaryEntry };
export { GLOSSARY, GLOSSARY_CONTENT_SHA256, GLOSSARY_TERMS, GLOSSARY_VERSION };

/**
 * Whether `id` names a term this version of the glossary defines. `GLOSSARY`
 * is a plain object, so the check is on OWN keys — `"toString"` is not a term.
 */
export function isGlossaryTerm(id: string): id is GlossaryTerm {
  return Object.prototype.hasOwnProperty.call(GLOSSARY, id);
}

/** The definition of `id`. Total over the union, so it never returns
 * `undefined` for a typed id. */
export function glossaryEntry<Id extends GlossaryTerm>(
  id: Id
): GlossaryEntry<Id> {
  return GLOSSARY[id];
}

/** The definition for an untyped id, or `null` when this version does not
 * define it (a newer producer's term). Never a guessed neighbour. */
export function lookupGlossaryTerm(id: string): GlossaryEntry | null {
  return isGlossaryTerm(id) ? GLOSSARY[id] : null;
}
