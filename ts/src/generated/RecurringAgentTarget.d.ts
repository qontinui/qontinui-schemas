/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */

import type { RecurringAgentDispatchRole } from "./RecurringAgentDispatchRole";
import type { RecurringAgentOs } from "./RecurringAgentOs";

/**
 * Which devices a recurring agent applies to (plan D4).
 *
 * Always serialized as a JSON object tagged by `kind` — `{"kind":"all"}`,
 * never the bare string `"all"` — because coord's
 * `ck_recurring_agents_target_object` refuses a non-object. coord resolves the
 * target per device on its read route, so a runner never evaluates it.
 *
 * Unknown fields are refused on deserialization (not only in the schema): a
 * misspelt `exclude_devices` silently ignored would widen the target to
 * devices the author meant to exclude. A runner that cannot parse a target
 * keeps its last-applied set (the reconciler's fail-safe), which is the safe
 * side of that failure.
 */
export type RecurringAgentTarget =
  | {
      /**
       * Device ids (UUIDs) removed from the target.
       */
      exclude_devices?: string[];
      kind: "all";
    }
  | {
      /**
       * Device ids (UUIDs). May be empty: coord's catalog defaults for a
       * one-box agent are `{"kind":"devices","devices":[]}` — the agent
       * targets no device until an operator picks one.
       */
      devices: string[];
      /**
       * Device ids (UUIDs) removed from the target.
       */
      exclude_devices?: string[];
      kind: "devices";
    }
  | {
      /**
       * Device ids (UUIDs) removed from the target.
       */
      exclude_devices?: string[];
      kind: "dispatch_role";
      /**
       * The role a device must hold.
       */
      role: RecurringAgentDispatchRole;
    }
  | {
      /**
       * Required capabilities (containment). Must be non-empty.
       */
      capabilities: string[];
      /**
       * Device ids (UUIDs) removed from the target.
       */
      exclude_devices?: string[];
      kind: "capabilities";
    }
  | {
      /**
       * Device ids (UUIDs) removed from the target.
       */
      exclude_devices?: string[];
      kind: "os";
      /**
       * Selected operating systems. Must be non-empty.
       */
      os: RecurringAgentOs[];
    };
