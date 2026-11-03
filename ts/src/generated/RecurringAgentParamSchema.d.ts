/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */

import type { RecurringAgentExcludedValue } from "./RecurringAgentExcludedValue";
import type { RecurringAgentParamDef } from "./RecurringAgentParamDef";

/**
 * The parameter form of one catalog entry: the typed parameters a tenant may
 * set instead of editing prompt text (plan D2, D10). Serialized as the bare
 * array coord serves under a catalog entry's `params` key.
 */
export type RecurringAgentParamSchema = RecurringAgentParamDef[];
