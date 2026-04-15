/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with `just generate-types` or
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */

import type { ConditionScheduleConfig } from './ConditionScheduleConfig';

/**
 * How a task should be scheduled.
 *
 * Serialized with the external tag `type` and payload under `value` so that
 * `Once("...")`, `Cron("...")`, and `Interval(60)` round-trip as
 * `{ "type": "Once", "value": "..." }` etc. The `Condition` variant wraps a
 * [`ConditionScheduleConfig`] rather than a scalar, but uses the same
 * `{ type, value }` envelope.
 */
export type ScheduleExpression =
  | {
      type: "Once";
      value: string;
      [k: string]: unknown;
    }
  | {
      type: "Cron";
      value: string;
      [k: string]: unknown;
    }
  | {
      type: "Interval";
      value: number;
      [k: string]: unknown;
    }
  | {
      type: "Condition";
      value: ConditionScheduleConfig;
      [k: string]: unknown;
    };
