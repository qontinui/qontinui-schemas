/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with `just generate-types` or
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */

import type { Confidence } from './Confidence';
import type { Finding } from './Finding';
import type { WorkerSignal } from './WorkerSignal';
import type { WorkerStatus } from './WorkerStatus';

/**
 * Instance tracking for an individual worker.
 */
export interface WorkerInstance {
  /**
   * ISO 8601 timestamp when the worker completed.
   */
  completedAt?: string | null;
  /**
   * Domain this worker is assigned to (if any).
   */
  domain?: string | null;
  /**
   * Error message if the worker is in error state.
   */
  errorMessage?: string | null;
  /**
   * Findings recorded by this worker.
   */
  findings?: Finding[];
  /**
   * Current iteration for this worker.
   */
  iteration: number;
  /**
   * Last signal received from this worker.
   */
  lastSignal?: WorkerSignal | null;
  /**
   * Maximum iterations allowed for this worker.
   */
  maxIterations: number;
  /**
   * Human-readable name.
   */
  name: string;
  /**
   * ISO 8601 timestamp when the worker started.
   */
  startedAt?: string | null;
  status: WorkerStatus;
  /**
   * Files this worker has touched.
   */
  touchedFiles?: string[];
  /**
   * Unique identifier for this worker.
   */
  workerId: string;
  [k: string]: unknown;
}
