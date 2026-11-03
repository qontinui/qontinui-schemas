/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */

import type { RecurringAgentExcludedValue } from "./RecurringAgentExcludedValue";

/**
 * The type, default and bounds of one catalog parameter — exactly the shape
 * coord serves in each catalog entry's `params` array
 * (`recurring_agents::param_schema_json` in qontinui-coord).
 *
 * Tagged by `type` and FLATTENED into its [`RecurringAgentParamDef`], so one
 * served parameter reads
 * `{"name":"mode","description":"…","required":false,"type":"enum","values":["reap","report"],"default":"reap"}`.
 */
export type RecurringAgentParamKind =
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
    };
