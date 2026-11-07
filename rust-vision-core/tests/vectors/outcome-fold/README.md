# Outcome-fold golden vectors

One file per case: `{description, input, expected}`. `input` is a list of
verdicts (`{outcome, code?}`, `code` iff `outcome == "unknown"`); `expected` is
the fold's answer in `FoldedOutcome`'s own camelCase wire form
(`outcome`, `outcomeCounts`, `unknownCodes`), so a runner compares the
serialized fold with `expected` directly.

Run by `qontinui-vision-core` (`tests/outcome_fold_vectors.rs`) and
`qontinui-types` (`rust/tests/outcome_fold_vectors.rs`, reading this directory
through `../rust-vision-core/`). Kept in this MIT crate so the MIT ui-bridge SDK
and qontinui-web's Spec CI can vendor them. Adding a case here adds it to every
runner; changing an `expected` changes the fold's contract for all of them.
