/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */

/**
 * Schema-only: `SkillDefinition.source`. Names the three values this
 * program's producers emit; [`SkillSource::Other`] preserves a foreign value
 * verbatim on a round trip but is never produced, so it is not part of the
 * published vocabulary.
 */
export type SkillSource = "builtin" | "user" | "community";
