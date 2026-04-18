/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with `just generate-types` or
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */

import type { MatchLocation } from './MatchLocation';
import type { TopMatch } from './TopMatch';

/**
 * Runtime execution data captured during action execution.
 *
 * Different fields are populated depending on the action type. All fields
 * are optional; the wire schema allows additional runtime fields via a
 * `flatten`-d map.
 */
export interface RuntimeData {
  alreadyAtTarget?: boolean | null;
  branchTaken?: string | null;
  button?: string | null;
  characterCount?: number | null;
  clickedAt?: MatchLocation | null;
  conditionPassed?: boolean | null;
  confidence?: number | null;
  dimensions?: MatchLocation | null;
  found?: boolean | null;
  imageId?: string | null;
  location?: MatchLocation | null;
  matchMethod?: string | null;
  sourceStates?: string[] | null;
  targetStates?: string[] | null;
  targetType?: string | null;
  targetsReached?: string[] | null;
  topMatches?: TopMatch[] | null;
  transitionsExecuted?: string[] | null;
  typedText?: string | null;
  workflowName?: string | null;
  workflowStatus?: string | null;
}
