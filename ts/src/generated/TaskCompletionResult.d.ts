/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with `just generate-types` or
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */

import type { Confidence } from './Confidence';
import type { CriterionOverride } from './CriterionOverride';
import type { CriterionType } from './CriterionType';
import type { Finding } from './Finding';
import type { IterationVerificationResults } from './IterationVerificationResults';
import type { VerificationResult } from './VerificationResult';

/**
 * Task-completion result.
 *
 * Internally tagged by `status`: the variant fields are inlined alongside the
 * discriminator rather than nested under a `data` key.
 */
export type TaskCompletionResult =
  | {
      /**
       * Findings accumulated during the run.
       */
      findings: Finding[];
      /**
       * Number of iterations used.
       */
      iterations: number;
      status: "success";
      verification_results: IterationVerificationResults;
      [k: string]: unknown;
    }
  | {
      /**
       * Findings accumulated during the run.
       */
      findings: Finding[];
      /**
       * Number of iterations attempted.
       */
      iterations: number;
      /**
       * Last verification results (if any).
       */
      last_results?: IterationVerificationResults1 | null;
      /**
       * Human-readable reason for the failure.
       */
      reason: string;
      status: "failed";
      [k: string]: unknown;
    }
  | {
      /**
       * Iteration at which the task was stopped.
       */
      at_iteration: number;
      /**
       * Whether the task can be resumed from this point.
       */
      can_resume: boolean;
      /**
       * Findings accumulated up to the stop.
       */
      findings: Finding[];
      status: "stopped";
      [k: string]: unknown;
    }
  | {
      /**
       * Iteration at which the task paused.
       */
      at_iteration: number;
      /**
       * Findings accumulated up to the pause.
       */
      findings: Finding[];
      /**
       * Last verification results (if any).
       */
      last_results?: IterationVerificationResults1 | null;
      /**
       * The max-iteration limit that triggered the pause.
       */
      max_iterations: number;
      status: "paused";
      [k: string]: unknown;
    };
