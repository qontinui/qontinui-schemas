/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with `just generate-types` or
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */

/**
 * A verdict a helper may submit for a task.
 *
 * Verdict-to-kind mapping:
 * - [`HelperTaskKind::SpotCheck`] → [`HelperVerdict::Approve`],
 *   [`HelperVerdict::Reject`], [`HelperVerdict::NotSure`].
 * - [`HelperTaskKind::Compare`] → [`HelperVerdict::ChoiceA`],
 *   [`HelperVerdict::ChoiceB`], [`HelperVerdict::ChoiceSame`].
 *
 * `NotSure` (🤔) is a first-class outcome — never an error — so a helper is
 * never forced into a false approve/reject.
 */
export type HelperVerdict =
  "approve" | "reject" | "not_sure" | "choice_a" | "choice_b" | "choice_same";
