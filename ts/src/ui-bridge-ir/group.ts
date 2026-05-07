/**
 * IR-side group + assertion types.
 *
 * The IR's primary authoring surface is `states + transitions`, which the
 * projection maps into legacy `groups[]` (one group per state). However, the
 * runner's `workflow_generation::spec_synthesis` pipeline produces "free-form"
 * groups from acceptance criteria — these are NOT tied to any state and
 * therefore have no home in the state-derived projection path.
 *
 * `IRGroup` is the dedicated channel for those synthesized groups. The IR
 * carries them in the optional `IRDocument.synthesizedGroups` field; the
 * projection appends them after the state-derived groups, in declared order,
 * to preserve byte-stable output.
 *
 * The shape mirrors the legacy `SpecGroup` / `SpecAssertion` (canonical
 * definitions live in `@qontinui/ui-bridge/specs`), but is re-defined here to
 * keep `qontinui-schemas/ts` free of any runtime dep on `@qontinui/ui-bridge/*`
 * (per the schemas package's "zero ui-bridge deps" policy).
 *
 * Note on field shape: the IR-side `IRAssertion` keeps the legacy assertion's
 * field types as loose as the synthesis path emits them. Strict consumers
 * (e.g. the canonical `SpecAssertion` in `@qontinui/ui-bridge/specs`) can
 * narrow as needed at the consumer boundary; the projection preserves the
 * objects verbatim into the legacy output.
 */

/**
 * IR-side assertion target. Always `type: "search"` for synthesis-emitted
 * assertions; the wider legacy schema also supports point/region targets but
 * the IR doesn't express those today.
 */
export interface IRAssertionTarget {
  /** Always `"search"` for synthesis-emitted assertions. */
  type: string;
  /**
   * Free-form criteria object. Kept as a loose record so synthesis can emit
   * partially-populated criteria without tripping the schema.
   */
  criteria: Record<string, unknown>;
  label: string;
}

/**
 * IR-side assertion. Mirrors the legacy `SpecAssertion` shape.
 */
export interface IRAssertion {
  id: string;
  description: string;
  category: string;
  severity: string;
  assertionType: string;
  target: IRAssertionTarget;
  source: string;
  reviewed: boolean;
  enabled: boolean;
  /**
   * Optional human-readable precondition. Synthesis populates this from the
   * acceptance criterion's verification hint.
   */
  precondition?: string;
}

/**
 * IR-side group produced by workflow-criteria synthesis (i.e. NOT derived
 * from `IRState` annotations). Mirrors the legacy `SpecGroup` shape one-to-one
 * so the projection can pass it through to legacy `groups[]` without loss.
 */
export interface IRGroup {
  id: string;
  name: string;
  description: string;
  category: string;
  assertions: IRAssertion[];
  /**
   * Provenance of the group itself (typically `"ai-generated"` for synthesis
   * output, but kept open for future authoring channels).
   */
  source?: string;
  /**
   * Free-form tags. Synthesis emits e.g. `["workflow-generated",
   * "acceptance-criteria"]` so downstream tooling can filter.
   */
  tags?: string[];
}
