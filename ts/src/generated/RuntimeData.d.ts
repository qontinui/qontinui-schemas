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
  already_at_target?: boolean | null;
  branch_taken?: string | null;
  button?: string | null;
  character_count?: number | null;
  clicked_at?: MatchLocation | null;
  condition_passed?: boolean | null;
  confidence?: number | null;
  dimensions?: MatchLocation | null;
  found?: boolean | null;
  image_id?: string | null;
  location?: MatchLocation | null;
  match_method?: string | null;
  source_states?: string[] | null;
  target_states?: string[] | null;
  target_type?: string | null;
  targets_reached?: string[] | null;
  top_matches?: TopMatch[] | null;
  transitions_executed?: string[] | null;
  typed_text?: string | null;
  workflow_name?: string | null;
  workflow_status?: string | null;
  [k: string]: unknown;
}
