/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with `just generate-types` or
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */

/**
 * How a spec node was established, relative to what the frontend *reveals*.
 *
 * Named `SpecProvenance` (not `Confidence`) to stay distinct from the
 * match-confidence axis `crate::spec_check::Confidence`
 * (`high`/`medium`/`low`), which is orthogonal: that one scores how well an
 * observed element matched an assertion; this one scores how the spec node
 * itself was derived from observation.
 *
 * The three buckets keep the coverage accounting split crisp: the rubric's
 * denominator is `Observed + Inferred`; `Assumed` is reported separately as
 * an assumption-fill rate and never folded into the headline coverage number.
 */
export type SpecProvenance = "observed" | "inferred" | "assumed";
