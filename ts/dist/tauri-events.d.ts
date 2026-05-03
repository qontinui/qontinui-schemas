import { T as TaskRunFindingActionType, a as TaskRunFindingCategory, b as TaskRunFindingSeverity, c as TaskRunFindingStatus } from './TaskRunFindingActionType.d-CNIWogcU.js';

/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with `just generate-types` or
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */

/**
 * Events emitted during flow execution for UI updates.
 *
 * Internally tagged with `"type"` and snake_case variant names.
 */
type FlowEvent =
  | {
      flow_id: string;
      flow_name: string;
      instance_id: string;
      type: "flow_started";
      [k: string]: unknown;
    }
  | {
      instance_id: string;
      step_id: string;
      step_name: string;
      step_type: string;
      type: "step_started";
      [k: string]: unknown;
    }
  | {
      duration_ms: number;
      error?: string | null;
      instance_id: string;
      outputs: {
        [k: string]: unknown;
      };
      step_id: string;
      success: boolean;
      type: "step_completed";
      [k: string]: unknown;
    }
  | {
      duration_ms: number;
      error?: string | null;
      flow_id: string;
      instance_id: string;
      success: boolean;
      total_steps: number;
      type: "flow_completed";
      [k: string]: unknown;
    }
  | {
      instance_id: string;
      options: string[];
      prompt: string;
      step_id: string;
      type: "waiting_for_input";
      [k: string]: unknown;
    }
  | {
      completed: number;
      instance_id: string;
      step_id: string;
      total: number;
      type: "parallel_progress";
      [k: string]: unknown;
    };

/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with `just generate-types` or
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */



/**
 * Unified application events for frontend communication.
 *
 * Adjacently tagged: each variant serializes as
 * `{"event_type": "<VariantName>", "data": {..}}`.
 * The React frontend dispatches on `event_type` and reads `data`.
 */
type AppEvent =
  | {
      data: {
        data: unknown;
        event: string;
        sequence: number;
        timestamp: number;
        [k: string]: unknown;
      };
      event_type: "ExecutorEvent";
      [k: string]: unknown;
    }
  | {
      data: {
        event_type: string;
        node: unknown;
        path: string[];
        sequence: number;
        timestamp: number;
        [k: string]: unknown;
      };
      event_type: "ExecutorTreeEvent";
      [k: string]: unknown;
    }
  | {
      data: {
        details?: string | null;
        message: string;
        [k: string]: unknown;
      };
      event_type: "ExecutorError";
      [k: string]: unknown;
    }
  | {
      data: {
        data?: unknown;
        error?: string | null;
        id: string;
        success: boolean;
        [k: string]: unknown;
      };
      event_type: "ExecutorResponse";
      [k: string]: unknown;
    }
  | {
      data: {
        data: unknown;
        [k: string]: unknown;
      };
      event_type: "ImageRecognition";
      [k: string]: unknown;
    }
  | {
      data: {
        data: unknown;
        event: string;
        sequence: number;
        timestamp: number;
        [k: string]: unknown;
      };
      event_type: "ExtractionEvent";
      [k: string]: unknown;
    }
  | {
      data: {
        details?: string | null;
        message: string;
        [k: string]: unknown;
      };
      event_type: "ExtractionError";
      [k: string]: unknown;
    }
  | {
      data: {
        data?: unknown;
        error?: string | null;
        id: string;
        success: boolean;
        [k: string]: unknown;
      };
      event_type: "ExtractionResponse";
      [k: string]: unknown;
    }
  | {
      data: {
        elements_processed?: number | null;
        error?: string | null;
        message: string;
        percent?: number | null;
        project_id: string;
        status: string;
        total_elements?: number | null;
        [k: string]: unknown;
      };
      event_type: "RagProgress";
      [k: string]: unknown;
    }
  | {
      data: {
        failed: number;
        project_id: string;
        success: boolean;
        successful: number;
        total_processed: number;
        web_sync_error?: string | null;
        web_sync_success: boolean;
        [k: string]: unknown;
      };
      event_type: "RagCompletion";
      [k: string]: unknown;
    }
  | {
      data: FlowEvent;
      event_type: "FlowEvent";
      [k: string]: unknown;
    }
  | {
      data: {
        content: string;
        content_type?: string | null;
        session_id: string;
        [k: string]: unknown;
      };
      event_type: "AiOutput";
      [k: string]: unknown;
    }
  | {
      data: {
        finding: unknown;
        [k: string]: unknown;
      };
      event_type: "FindingDetected";
      [k: string]: unknown;
    }
  | {
      data: {
        finding: unknown;
        [k: string]: unknown;
      };
      event_type: "FindingResolved";
      [k: string]: unknown;
    }
  | {
      data: {
        data: unknown;
        [k: string]: unknown;
      };
      event_type: "TestNavigation";
      [k: string]: unknown;
    }
  | {
      data: {
        data: unknown;
        [k: string]: unknown;
      };
      event_type: "UiBridgeRequest";
      [k: string]: unknown;
    }
  | {
      data: {
        /**
         * Current iteration number.
         */
        iteration: number;
        /**
         * Current phase (setup, verification, agentic, completion).
         */
        phase: string;
        /**
         * Optional additional state data.
         */
        state_data?: {
          [k: string]: unknown;
        };
        /**
         * Task run ID.
         */
        task_run_id: string;
        /**
         * Current workflow stage name.
         */
        workflow_stage: string;
        [k: string]: unknown;
      };
      event_type: "OrchestratorStateChange";
      [k: string]: unknown;
    }
  | {
      data: {
        /**
         * Optional details about the step.
         */
        details?: {
          [k: string]: unknown;
        };
        /**
         * Status: "started", "running", "completed", "failed", "skipped".
         */
        status: string;
        /**
         * Step index (0-based).
         */
        step_index: number;
        /**
         * Step name/description.
         */
        step_name: string;
        /**
         * Task run ID.
         */
        task_run_id: string;
        /**
         * Timestamp in milliseconds.
         */
        timestamp: number;
        [k: string]: unknown;
      };
      event_type: "StepProgress";
      [k: string]: unknown;
    }
  | {
      data: {
        /**
         * Optional additional details.
         */
        details?: {
          [k: string]: unknown;
        };
        /**
         * Current iteration (if applicable).
         */
        iteration?: number | null;
        /**
         * Status: "running", "completed", "failed", "stopped", "paused".
         */
        status: string;
        /**
         * Task run ID.
         */
        task_run_id: string;
        /**
         * Timestamp in milliseconds.
         */
        timestamp: number;
        [k: string]: unknown;
      };
      event_type: "TaskRunUpdate";
      [k: string]: unknown;
    }
  | {
      data: {
        /**
         * Approval request ID.
         */
        approval_id: string;
        /**
         * Current iteration.
         */
        iteration: number;
        /**
         * Prompt shown to the reviewer.
         */
        prompt: string;
        /**
         * Task run ID.
         */
        task_run_id: string;
        [k: string]: unknown;
      };
      event_type: "ApprovalRequired";
      [k: string]: unknown;
    }
  | {
      data: {
        /**
         * Action taken.
         */
        action: string;
        /**
         * Approval request ID.
         */
        approval_id: string;
        /**
         * Whether approved.
         */
        approved: boolean;
        /**
         * Task run ID.
         */
        task_run_id: string;
        [k: string]: unknown;
      };
      event_type: "ApprovalResolved";
      [k: string]: unknown;
    }
  | {
      data: {
        /**
         * Confidence score (0.0-1.0).
         */
        confidence: number;
        /**
         * Iteration when the question was raised.
         */
        iteration: number;
        /**
         * The question text.
         */
        question: string;
        /**
         * Deferred question ID.
         */
        question_id: string;
        /**
         * Risk level.
         */
        risk_level: string;
        /**
         * Task run ID.
         */
        task_run_id: string;
        [k: string]: unknown;
      };
      event_type: "DeferredQuestionCreated";
      [k: string]: unknown;
    }
  | {
      data: {
        /**
         * Deferred question ID.
         */
        question_id: string;
        /**
         * Whether rework was triggered.
         */
        rework_triggered: boolean;
        /**
         * Review status: "approved" or "rejected".
         */
        status: string;
        /**
         * Task run ID.
         */
        task_run_id: string;
        [k: string]: unknown;
      };
      event_type: "DeferredQuestionReviewed";
      [k: string]: unknown;
    }
  | {
      data: {
        action: string;
        panel?: unknown;
        panel_id: string;
        task_run_id?: string | null;
        [k: string]: unknown;
      };
      event_type: "CanvasUpdate";
      [k: string]: unknown;
    }
  | {
      data: {
        /**
         * Total accumulated output length so far.
         */
        accumulated_length: number;
        /**
         * The text chunk received from the AI.
         */
        chunk: string;
        /**
         * Task run ID this output belongs to.
         */
        task_run_id: string;
        [k: string]: unknown;
      };
      event_type: "AiOutputChunk";
      [k: string]: unknown;
    }
  | {
      data: {
        /**
         * Number of verification steps that failed.
         */
        failed_step_count: number;
        /**
         * True if failed_step_count has not decreased in 3 consecutive iterations.
         */
        is_stalled: boolean;
        /**
         * Current iteration number (1-indexed).
         */
        iteration: number;
        /**
         * Failures not present in the previous iteration.
         */
        new_failures: number;
        /**
         * Number of verification steps that passed.
         */
        passed_step_count: number;
        /**
         * Failures that were also present in the previous iteration.
         */
        repeated_failures: number;
        /**
         * Number of verification steps that were skipped.
         */
        skipped_step_count: number;
        /**
         * Task run ID.
         */
        task_run_id: string;
        [k: string]: unknown;
      };
      event_type: "IterationMetrics";
      [k: string]: unknown;
    }
  | {
      data: {
        /**
         * Number of failures with blame attributions.
         */
        attributed_failures: number;
        /**
         * Full blame report as JSON.
         */
        blame_json: string;
        /**
         * Current iteration number.
         */
        iteration: number;
        /**
         * Number of files exhibiting oscillation.
         */
        oscillating_files: number;
        /**
         * Number of files exhibiting revert patterns.
         */
        revert_patterns: number;
        /**
         * Task run ID.
         */
        task_run_id: string;
        [k: string]: unknown;
      };
      event_type: "BlameAttribution";
      [k: string]: unknown;
    }
  | {
      data: {
        /**
         * Whether any blocking violations exist.
         */
        has_blocking: boolean;
        /**
         * Current iteration number (1-indexed).
         */
        iteration: number;
        /**
         * Serialized constraint results.
         */
        results: {
          [k: string]: unknown;
        };
        /**
         * Human-readable summary of results.
         */
        summary: string;
        /**
         * Task run ID.
         */
        task_run_id: string;
        [k: string]: unknown;
      };
      event_type: "ConstraintResults";
      [k: string]: unknown;
    }
  | {
      data: {
        /**
         * Position in the queue (0-indexed).
         */
        queue_position: number;
        /**
         * Task run ID for the queued workflow.
         */
        task_run_id: string;
        /**
         * Human-readable workflow name.
         */
        workflow_name: string;
        [k: string]: unknown;
      };
      event_type: "WorkflowQueued";
      [k: string]: unknown;
    }
  | {
      data: {
        /**
         * Task run ID for the dequeued workflow.
         */
        task_run_id: string;
        /**
         * Time spent waiting in the queue, in milliseconds.
         */
        wait_time_ms: number;
        /**
         * Human-readable workflow name.
         */
        workflow_name: string;
        [k: string]: unknown;
      };
      event_type: "WorkflowDequeued";
      [k: string]: unknown;
    }
  | {
      data: {
        cache_creation_tokens: number;
        cache_hit_rate: number;
        cache_read_tokens: number;
        cost_usd: number;
        cumulative_cost_usd: number;
        input_tokens: number;
        iteration?: number | null;
        output_tokens: number;
        phase: string;
        task_run_id: string;
        timestamp: number;
        [k: string]: unknown;
      };
      event_type: "CostUpdate";
      [k: string]: unknown;
    }
  | {
      data: {
        budget_limit_usd: number;
        message: string;
        remaining_fraction: number;
        task_run_id: string;
        timestamp: number;
        total_cost_usd: number;
        [k: string]: unknown;
      };
      event_type: "BudgetWarning";
      [k: string]: unknown;
    }
  | {
      data: {
        cost_usd: number;
        mean_cost_usd: number;
        message: string;
        std_dev: number;
        task_run_id: string;
        timestamp: number;
        z_score: number;
        [k: string]: unknown;
      };
      event_type: "CostAnomaly";
      [k: string]: unknown;
    }
  | {
      data: {
        context?: string | null;
        message: string;
        [k: string]: unknown;
      };
      event_type: "Error";
      [k: string]: unknown;
    };

/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with `just generate-types` or
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */

/**
 * Information about a terminal session, returned to the frontend.
 *
 * Returned from the `terminal_create` and `terminal_list` Tauri commands
 * and emitted as the payload of the `terminal-created` event. Derived
 * fresh from the live `TerminalSession` each time — never persisted.
 */
interface TerminalInfo {
  /**
   * Current terminal width in columns.
   */
  cols: number;
  /**
   * Unix timestamp in milliseconds when the session was created.
   *
   * Used as the sort key for `TerminalManager::list` so the UI shows
   * terminals in creation order.
   */
  createdAt: number;
  /**
   * Process exit code, once the shell has exited.
   */
  exitCode?: number | null;
  /**
   * Unique session identifier (UUID v4 minted by the runner).
   */
  id: string;
  /**
   * Whether the shell process is still running.
   */
  isAlive: boolean;
  /**
   * Which terminal page this session belongs to (for multi-page support).
   *
   * Older wire forms without this field hydrate to `"default"` via
   * [`default_page_id`].
   */
  pageId: string;
  /**
   * OS process ID of the spawned shell, if still known.
   */
  pid?: number | null;
  /**
   * Current terminal height in rows.
   */
  rows: number;
  /**
   * Human-readable title shown in the UI tab (e.g., "Terminal 1").
   */
  title: string;
  /**
   * Monotonic counter of all bytes ever produced by this PTY.
   *
   * Read by the frontend to detect missed output after a reconnect; the
   * scrollback buffer's `start_offset` is derived from this counter.
   */
  totalBytesProduced: number;
  /**
   * Absolute working directory the shell was started in.
   */
  workingDir: string;
}

/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with `just generate-types` or
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */

/**
 * Event payload emitted when a terminal produces output.
 *
 * Delivered over the Tauri `terminal-output` channel and re-broadcast on
 * the WebSocket relay for remote/mobile consumers. The `data` payload is
 * base64-encoded raw bytes because xterm.js needs raw bytes (PTY output
 * can contain partial UTF-8 sequences that would corrupt a `String`
 * field).
 */
interface TerminalOutputEvent {
  /**
   * Base64-encoded bytes produced by the PTY.
   *
   * Raw bytes are required (not UTF-8 text) because PTY output can
   * contain partial UTF-8 sequences that span read boundaries.
   */
  data: string;
  /**
   * ID of the terminal session producing this output.
   */
  terminalId: string;
}

/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with `just generate-types` or
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */

/**
 * Event payload emitted when a terminal process exits.
 *
 * Delivered over the Tauri `terminal-exit` channel and re-broadcast on the
 * WebSocket relay. After this event fires the corresponding
 * [`TerminalInfo::is_alive`] will be `false` and [`TerminalInfo::exit_code`]
 * will carry the same value surfaced here.
 */
interface TerminalExitEvent {
  /**
   * Exit code reported by the OS, or `None` if the status could not be
   * captured (e.g., the wait call itself errored).
   */
  exitCode?: number | null;
  /**
   * ID of the terminal session that exited.
   */
  terminalId: string;
}

/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with `just generate-types` or
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */

/**
 * Code context for a finding (runner-local wire shape).
 *
 * Renamed in the schema registry to `RunnerFindingCodeContext` to
 * disambiguate from `qontinui_types::findings::FindingCodeContext`.
 */
interface RunnerFindingCodeContext {
  column?: number | null;
  file?: string | null;
  line?: number | null;
  snippet?: string | null;
  [k: string]: unknown;
}

/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with `just generate-types` or
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */

/**
 * User input request for a finding (runner-local wire shape).
 *
 * Renamed in the schema registry to `RunnerFindingUserInput` to
 * disambiguate from `qontinui_types::findings::FindingUserInput`.
 */
interface RunnerFindingUserInput {
  input_type: string;
  options?: string[] | null;
  question: string;
  [k: string]: unknown;
}

/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with `just generate-types` or
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */



/**
 * A finding detected by AI analysis (runner-local wire shape).
 *
 * Wire format: serialized via `#[serde(rename_all = "camelCase")]` so all
 * snake_case Rust field names ship as camelCase on the Tauri event channels
 * `finding_detected` and `finding_resolved`. The frontend listener in
 * `services/TauriFindingsListener.ts` MUST read these fields by their
 * camelCase names — reading snake_case silently evaluates to `undefined`.
 *
 * Renamed in the schema registry to `RunnerFinding` to disambiguate from
 * `qontinui_types::verification::Finding`, which has a different shape
 * (`confidence`, `findingType`, `evidence` vs this struct's flat fields).
 */
interface RunnerFinding {
  actionType: TaskRunFindingActionType;
  categoryId: TaskRunFindingCategory;
  codeContext?: RunnerFindingCodeContext | null;
  description: string;
  detectedAt: string;
  id: string;
  resolution?: string | null;
  resolvedAt?: string | null;
  resolvedInSession?: number | null;
  sessionNum: number;
  severity: TaskRunFindingSeverity;
  signatureHash: string;
  status: TaskRunFindingStatus;
  taskRunId: string;
  title: string;
  updatedAt: string;
  userInput?: RunnerFindingUserInput | null;
  userResponse?: string | null;
  [k: string]: unknown;
}

export type { AppEvent, RunnerFinding, RunnerFindingCodeContext, RunnerFindingUserInput, TerminalExitEvent, TerminalInfo, TerminalOutputEvent };
