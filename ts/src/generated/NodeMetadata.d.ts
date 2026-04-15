/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with `just generate-types` or
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */

import type { MatchLocation } from './MatchLocation';
import type { Outcome } from './Outcome';
import type { RuntimeData } from './RuntimeData';
import type { StateContext } from './StateContext';
import type { TimingInfo } from './TimingInfo';
import type { TopMatch } from './TopMatch';

/**
 * Metadata for a tree node — action configuration, runtime data, state
 * context, and timing. All fields are optional because different node
 * types populate different fields.
 */
export interface NodeMetadata {
  /**
   * Action configuration (JSON object — shape varies by `ActionType`).
   */
  config?: {
    [k: string]: unknown;
  } | null;
  /**
   * Whether this action can have child nodes.
   */
  is_expandable: boolean;
  /**
   * Whether this action is displayed inline.
   */
  is_inline: boolean;
  outcome?: Outcome | null;
  runtime?: RuntimeData | null;
  /**
   * Screenshot reference (path or URL).
   */
  screenshot_reference?: string | null;
  state_context?: StateContext | null;
  timing?: TimingInfo | null;
  /**
   * Visual-debug image reference (path or URL).
   */
  visual_debug_reference?: string | null;
  [k: string]: unknown;
}
