/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with `just generate-types` or
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */

/**
 * What to check and how.
 *
 * Internally tagged by the `type` field. Variants correspond to the four
 * check kinds implemented by the runner's constraint engine.
 */
type ConstraintCheck =
  | {
      /**
       * Optional glob to limit which modified files are checked.
       * Default: all modified files.
       */
      file_glob?: string | null;
      /**
       * Regex pattern to search for.
       */
      pattern: string;
      type: "grep_forbidden";
      [k: string]: unknown;
    }
  | {
      /**
       * Optional glob to limit which modified files are checked.
       */
      file_glob?: string | null;
      /**
       * Regex pattern that must be present.
       */
      pattern: string;
      type: "grep_required";
      [k: string]: unknown;
    }
  | {
      /**
       * Allowed directory prefixes (relative to project root).
       * e.g., `["src/", "tests/", "config/"]`.
       */
      allowed_paths: string[];
      type: "file_scope";
      [k: string]: unknown;
    }
  | {
      /**
       * The command to run.
       */
      cmd: string;
      /**
       * Working directory (relative to project root). Default: project root.
       */
      cwd?: string | null;
      /**
       * Timeout in seconds. Default: 30.
       */
      timeout_secs: number;
      type: "command";
      [k: string]: unknown;
    };

/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with `just generate-types` or
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */

/**
 * Severity of a constraint violation.
 *
 * - `Block`: Reject the fix, inject violation context, re-run agentic phase
 *   without consuming an iteration. After `max_retries`, consume the iteration.
 * - `Warn`: Apply the fix, but inject violation context for the next iteration.
 * - `Log`: Record only, don't affect execution.
 */
type ConstraintSeverity = "block" | "warn" | "log";

/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with `just generate-types` or
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */



/**
 * A constraint definition.
 */
interface Constraint {
  check: ConstraintCheck;
  /**
   * Why this constraint exists (shown to the AI on violation).
   */
  description: string;
  /**
   * Whether this constraint is enabled. Default: true.
   */
  enabled: boolean;
  /**
   * Unique identifier (e.g., `"builtin:no-secrets"`, `"project:no-todos"`).
   */
  id: string;
  /**
   * Human-readable name.
   */
  name: string;
  severity: ConstraintSeverity;
}

/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with `just generate-types` or
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */

/**
 * A specific violation found during constraint evaluation.
 */
interface ConstraintViolation {
  /**
   * What was found / what went wrong.
   */
  detail: string;
  /**
   * File where the violation was found (if applicable).
   */
  file?: string | null;
  /**
   * Line number (if applicable).
   */
  line?: number | null;
}

/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with `just generate-types` or
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */



/**
 * Result of evaluating a single constraint.
 */
interface ConstraintResult {
  /**
   * The id of the constraint that was evaluated.
   */
  constraintId: string;
  /**
   * The human-readable name of the constraint that was evaluated.
   */
  constraintName: string;
  /**
   * Whether the constraint passed.
   */
  passed: boolean;
  severity: ConstraintSeverity;
  /**
   * Details about the violation (empty if passed).
   */
  violations: ConstraintViolation[];
}

/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with `just generate-types` or
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */

/**
 * Resource limits for workflow execution.
 *
 * When a limit is approached (within the warning threshold), the tracker
 * emits context injection actions. When exceeded, it emits stronger actions.
 */
interface ResourceLimits {
  /**
   * Maximum agentic phase durations summed (milliseconds).
   */
  maxAgenticTimeMs?: number | null;
  /**
   * Maximum number of unique files modified across all iterations.
   */
  maxFilesModified?: number | null;
  /**
   * Maximum wall-clock time for the entire workflow (seconds).
   */
  maxWallTimeSecs?: number | null;
  /**
   * Warning threshold as a fraction (0.0-1.0). When resource usage exceeds
   * this fraction of the limit, a warning is injected.
   * Default: 0.75 (warn at 75% of limit).
   */
  warningThreshold?: number | null;
}

/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with `just generate-types` or
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */



/**
 * A constraint proposal from the AI.
 *
 * Internally tagged by `type` so the on-the-wire shape matches the TypeScript
 * discriminated union `{ type: "new_constraint" | "builtin_override", ... }`.
 */
type ConstraintProposal =
  | {
      constraint: Constraint;
      type: "new_constraint";
    }
  | {
      /**
       * Builtin suffix (e.g., `"no-secrets"`, `"no-debug-statements"`).
       */
      builtinSuffix: string;
      /**
       * Whether the builtin should be enabled.
       */
      enabled: boolean;
      /**
       * Human-readable justification for the override.
       */
      reason: string;
      type: "builtin_override";
    };

/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with `just generate-types` or
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */



/**
 * A new constraint proposed by the AI during an agentic phase.
 *
 * Serialized with `"type": "new_constraint"` via the `ConstraintProposal`
 * enum's internal tag.
 */
interface NewConstraintProposal {
  constraint: Constraint;
}

/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with `just generate-types` or
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */

/**
 * A proposal to enable or disable a builtin constraint.
 *
 * Serialized with `"type": "builtin_override"` via the `ConstraintProposal`
 * enum's internal tag.
 */
interface BuiltinOverrideProposal {
  /**
   * Builtin suffix (e.g., `"no-secrets"`, `"no-debug-statements"`).
   */
  builtinSuffix: string;
  /**
   * Whether the builtin should be enabled.
   */
  enabled: boolean;
  /**
   * Human-readable justification for the override.
   */
  reason: string;
}

/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with `just generate-types` or
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */

/**
 * Request body for `POST /constraints/validate`.
 */
interface ValidateConfigRequest {
  /**
   * Raw TOML content to validate.
   */
  toml: string;
}

/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with `just generate-types` or
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */



/**
 * Response for `POST /constraints/validate`.
 */
interface ValidateConfigResponse {
  /**
   * Successfully parsed constraints (may be partial if some were skipped).
   */
  constraints: Constraint[];
  /**
   * Parse errors or non-fatal warnings (e.g., constraints skipped due to bad regex).
   */
  errors: string[];
  /**
   * Whether the config is fully valid (parseable with no errors or warnings).
   */
  valid: boolean;
}

/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with `just generate-types` or
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */

/**
 * Response for `GET /constraints/config`.
 */
interface ReadConfigResponse {
  /**
   * Resolved file path, if a config file was found.
   */
  path?: string | null;
  /**
   * Raw TOML content of the `constraints.toml` file (empty string if not found).
   */
  toml: string;
}

/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with `just generate-types` or
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */

/**
 * Request body for `POST /constraints/config`.
 */
interface WriteConfigRequest {
  /**
   * Project path for the `constraints.toml`. Defaults to workspace root.
   */
  projectPath?: string | null;
  /**
   * Raw TOML content to validate and write.
   */
  toml: string;
}

/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with `just generate-types` or
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */

/**
 * Response for `POST /constraints/config`.
 */
interface WriteConfigResponse {
  /**
   * Parse errors or non-fatal warnings.
   */
  errors: string[];
  /**
   * The file path that was written to.
   */
  path: string;
  /**
   * Whether the config is fully valid (parseable with no errors or warnings).
   */
  valid: boolean;
}

export type { BuiltinOverrideProposal, Constraint, ConstraintCheck, ConstraintProposal, ConstraintResult, ConstraintSeverity, ConstraintViolation, NewConstraintProposal, ReadConfigResponse, ResourceLimits, ValidateConfigRequest, ValidateConfigResponse, WriteConfigRequest, WriteConfigResponse };
