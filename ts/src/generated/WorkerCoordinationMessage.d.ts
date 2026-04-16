/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with `just generate-types` or
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */

import type { Confidence } from './Confidence';
import type { Finding } from './Finding';

/**
 * Coordination message between workers.
 *
 * Externally tagged as `{ "type": ..., "data": { ... } }` to match the
 * pre-extraction wire.
 */
export type WorkerCoordinationMessage =
  | {
      data: {
        /**
         * Paths of the modified files.
         */
        files: string[];
        /**
         * ID of the worker that modified the files.
         */
        worker_id: string;
        [k: string]: unknown;
      };
      type: "files_modified";
      [k: string]: unknown;
    }
  | {
      data: {
        finding: Finding;
        /**
         * ID of the worker that made the finding.
         */
        worker_id: string;
        [k: string]: unknown;
      };
      type: "shared_finding";
      [k: string]: unknown;
    }
  | {
      data: {
        /**
         * Human-readable reason for the block.
         */
        reason: string;
        /**
         * ID of the worker being waited on.
         */
        waiting_for: string;
        /**
         * ID of the blocked worker.
         */
        worker_id: string;
        [k: string]: unknown;
      };
      type: "blocked";
      [k: string]: unknown;
    }
  | {
      data: {
        /**
         * Optional domain scope for the verification.
         */
        domain?: string | null;
        /**
         * ID of the worker that is ready.
         */
        worker_id: string;
        [k: string]: unknown;
      };
      type: "ready_for_verification";
      [k: string]: unknown;
    }
  | {
      data: {
        /**
         * Human-readable reason for the sync point.
         */
        reason: string;
        /**
         * IDs of the workers that should synchronize.
         */
        worker_ids: string[];
        [k: string]: unknown;
      };
      type: "sync_point";
      [k: string]: unknown;
    };
