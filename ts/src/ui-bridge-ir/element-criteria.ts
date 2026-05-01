/**
 * IR's canonical element-matching shape.
 *
 * Per decision #7 (SESSION_PROMPTS.md): IR adopts ui-bridge-auto's
 * `ElementCriteria` as the canonical authoring shape — criteria survive ID
 * drift and are descriptive rather than identity-based. The shape mirrors
 * ui-bridge-auto/src/types/match.ts:20 exactly so the IR -> WorkflowConfig
 * adapter is structurally identity (no rewriting at the criteria level).
 *
 * `elementIds: string[]` is an OPTIONAL companion the runtime SDK fills in at
 * registration time when criteria resolve to specific registered elements.
 * Authors write criteria; the runtime caches resolutions.
 */

/**
 * Minimal criteria to identify a DOM element.
 *
 * Mirrors `ElementCriteria` from `ui-bridge-auto/src/types/match.ts:20`.
 * Kept as a separate copy here (not re-exported) so the IR module has no
 * runtime dependency on ui-bridge-auto. The adapter is the single point of
 * type-level reconciliation.
 */
export interface IRElementCriteria {
  /** ARIA role or inferred role. */
  role?: string;
  /** HTML tag name (e.g. "div", "button"). Heavily used in legacy specs. */
  tagName?: string;
  /** Exact text content (trimmed). */
  text?: string;
  /** Substring match on text content (case-insensitive). */
  textContains?: string;
  /**
   * ARIA label (case-insensitive substring match). Synonym of `accessibleName`
   * for the runtime SDK; the projection prefers `accessibleName` when both are
   * present.
   */
  ariaLabel?: string;
  /**
   * Computed accessible name (the same concept legacy specs serialize as
   * `accessibleName`). Added section 3 so the inverse projection can
   * round-trip without rewriting to `ariaLabel`.
   */
  accessibleName?: string;
  /** Element ID (exact string or pattern-source string). */
  id?: string;
  /** HTML attributes to check (exact string match). */
  attributes?: Record<string, string>;
}
