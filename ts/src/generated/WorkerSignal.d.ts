/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with `just generate-types` or
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */

import type { Confidence } from './Confidence';
import type { Finding } from './Finding';

/**
 * Signal emitted by a worker agent.
 *
 * Tagged as `{ "signal": ..., "data": ... }` rather than the more common
 * `type`/`data` pair; this matches the pre-extraction wire.
 */
export type WorkerSignal =
  | {
      data: {
        /**
         * Optional reason provided by the worker.
         */
        reason?: string | null;
        [k: string]: unknown;
      };
      signal: "work_complete";
      [k: string]: unknown;
    }
  | {
      data: {
        /**
         * Reason for the replan request.
         */
        reason: string;
        [k: string]: unknown;
      };
      signal: "need_replan";
      [k: string]: unknown;
    }
  | {
      signal: "continue";
      [k: string]: unknown;
    }
  | {
      data: Finding;
      signal: "finding";
      [k: string]: unknown;
    };
