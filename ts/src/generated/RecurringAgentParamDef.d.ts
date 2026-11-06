/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */

import type { RecurringAgentExcludedValue } from "./RecurringAgentExcludedValue";

/**
 * One parameter a catalog entry's prompt is rendered from (plan D2), e.g.
 * cleanup-steward's `mode` enum or return-to-main's `daily_cap` integer.
 */
export type RecurringAgentParamDef = {
  /**
   * Human-readable explanation, shown beside the form control.
   */
  description: string;
  /**
   * The key a value is supplied under in [`RecurringAgentSpec::params`].
   */
  name: string;
  /**
   * A required parameter with no default must be supplied by the tenant
   * row before the entry renders at all (strategic-pulse's tenant slug).
   */
  required: boolean;
  [k: string]: unknown;
} & (
  | {
      /**
       * The value used when the tenant sets none.
       */
      default: boolean;
      type: "boolean";
      [k: string]: unknown;
    }
  | {
      /**
       * The value used when the tenant sets none. Within the bounds.
       */
      default: number;
      /**
       * Largest allowed value.
       */
      maximum: number;
      /**
       * Smallest allowed value.
       */
      minimum: number;
      type: "integer";
      [k: string]: unknown;
    }
  | {
      /**
       * The value used when the tenant sets none. One of `values`.
       */
      default: string;
      type: "enum";
      /**
       * The allowed values, in display order. Non-empty, no duplicates.
       */
      values: string[];
      [k: string]: unknown;
    }
  | {
      /**
       * The values a tenant may pick from. Non-empty, no duplicates.
       */
      allowed: number[];
      /**
       * The set used when the tenant sets none. Non-empty, drawn from
       * `allowed`, none of it excluded.
       */
      default: number[];
      /**
       * Values that exist but this entry refuses, each with why — shown
       * greyed out in the form rather than hidden.
       */
      excluded: RecurringAgentExcludedValue[];
      type: "integer_set";
      [k: string]: unknown;
    }
  | {
      /**
       * The value used when the tenant sets none.
       */
      default: string | null;
      type: "uuid";
      [k: string]: unknown;
    }
  | {
      /**
       * The value used when the tenant sets none. `None` = no default.
       */
      default: string | null;
      type: "slug";
      [k: string]: unknown;
    }
);
