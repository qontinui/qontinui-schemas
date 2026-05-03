import { T as TaskRunFindingActionType, a as TaskRunFindingCategory, b as TaskRunFindingSeverity, c as TaskRunFindingStatus } from './TaskRunFindingActionType.d-CNIWogcU.cjs';

/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with `just generate-types` or
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */

/**
 * A canvas panel rendered in the dashboard widget.
 *
 * Wire-format mirror of the runner's `StoredPanel` struct
 * (`qontinui-runner/src-tauri/src/mcp/canvas.rs`). The runner emits this
 * inside [`AppEvent::CanvasUpdate`] via the `canvas-update` Tauri channel.
 * Field names are snake_case to match the existing TS interface in
 * `qontinui-schemas/ts/src/canvas/index.ts`.
 *
 * `data` stays `serde_json::Value` because each `component` type has a
 * different inner shape (Markdown, CodeDiff, Table, …); the per-component
 * data schemas live in the TS module above and are intentionally not
 * modeled as a Rust discriminated union (would balloon the schema for
 * little gain on the Rust side).
 */
interface CanvasPanel {
  component: string;
  created_at: string;
  data: unknown;
  group?: string | null;
  panel_id: string;
  priority: number;
  size: string;
  task_run_id: string;
  title: string;
  updated_at: string;
  [k: string]: unknown;
}

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
    }
  | {
      flow_id: string;
      instance_id: string;
      /**
       * Step the flow was paused at (`null` if no current step).
       */
      step_id?: string | null;
      type: "flow_paused";
      [k: string]: unknown;
    }
  | {
      flow_id: string;
      instance_id: string;
      /**
       * Step execution will resume from (`null` if no current step).
       */
      step_id?: string | null;
      type: "flow_resumed";
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
        panel?: CanvasPanel | null;
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

/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with `just generate-types` or
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */

/**
 * Discriminated union of every WS envelope `mcp/backend_relay.rs::handle_outbound`
 * ships to the backend after rewrapping a Tauri-side `AppEvent`.
 *
 * Internally tagged on `"type"` with snake_case variant names because that's
 * what the prior `serde_json::json!` literals emitted on the wire.
 *
 * Wire keys are snake_case (NOT camelCase) — verified against the literal
 * strings in `handle_outbound` lines ~605-638. Consumers that expect
 * camelCase (e.g. some web hooks) are wrong; the relay never emitted
 * camelCase here.
 */
type RunnerRelayMessage =
  | {
      data: {
        [k: string]: unknown;
      };
      type: "phase_completed";
      [k: string]: unknown;
    }
  | {
      data: {
        [k: string]: unknown;
      };
      type: "ui_error";
      [k: string]: unknown;
    }
  | {
      data: {
        [k: string]: unknown;
      };
      type: "recent_crash";
      [k: string]: unknown;
    }
  | {
      data: {
        [k: string]: unknown;
      };
      type: "chat_response";
      [k: string]: unknown;
    }
  | {
      data: {
        [k: string]: unknown;
      };
      type: "chat_session_state";
      [k: string]: unknown;
    }
  | {
      data: {
        [k: string]: unknown;
      };
      terminal_id: {
        [k: string]: unknown;
      };
      type: "terminal_output";
      [k: string]: unknown;
    }
  | {
      exit_code: {
        [k: string]: unknown;
      };
      terminal_id: {
        [k: string]: unknown;
      };
      type: "terminal_exit";
      [k: string]: unknown;
    };

/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with `just generate-types` or
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */

/**
 * Wire-format runner shape carried inside `runner_connected.connection`.
 *
 * This is intentionally a partial Runner payload (snake_case, only the
 * fields the Python emitter actually populates in
 * `RunnerEventPublisher.publish_runner_connected`). It is NOT
 * `qontinui_types::runner::Runner` — that shape requires `derived_status`,
 * `capabilities`, `created_at` etc., which the connection payload doesn't
 * include. Consumers reading this should refetch the canonical Runner row
 * via REST after seeing `runner_connected`.
 */
interface RunnerConnectedConnection {
  connected_at?: string | null;
  /**
   * Always present as `null` on the wire when the runner is still
   * connected. Python ships this field unconditionally.
   */
  disconnected_at?: string | null;
  duration_seconds?: number | null;
  id: string;
  ip_address?: string | null;
  project_id?: string | null;
  runner_name?: string | null;
  runner_port?: number | null;
  ws_connected: boolean;
  [k: string]: unknown;
}

/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with `just generate-types` or
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */



/**
 * Discriminated union of every event the Python backend pushes to the web
 * client over the `/api/v1/runners/status` WebSocket.
 *
 * Internally tagged on `"type"` matching the Python wire format. Variant
 * renames (`runner.woke` keeps the dot rather than going to snake_case)
 * preserve the literal strings the emitter sends.
 *
 * Source of truth: the `publish_*` methods in
 * `qontinui-web/backend/app/services/runner/event_publisher.py` and the
 * `initial_state` send in `runner_status_ws.py`.
 */
type RunnerStatusEvent =
  | {
      runners: unknown[];
      type: "initial_state";
      [k: string]: unknown;
    }
  | {
      connection: RunnerConnectedConnection;
      timestamp: string;
      type: "runner_connected";
      [k: string]: unknown;
    }
  | {
      runner_id: string;
      timestamp: string;
      type: "runner_disconnected";
      [k: string]: unknown;
    }
  | {
      runner_id: string;
      runner_name: string;
      timestamp: string;
      type: "runner_name_updated";
      [k: string]: unknown;
    }
  | {
      runner_id: string;
      runner_port: number;
      timestamp: string;
      type: "runner_port_updated";
      [k: string]: unknown;
    }
  | {
      intent_id?: string | null;
      reason?: string | null;
      runner_id: string;
      task_id?: string | null;
      timestamp: string;
      type: "runner.woke";
      [k: string]: unknown;
    }
  | {
      error: string;
      type: "error";
      [k: string]: unknown;
    };

/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with `just generate-types` or
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */

/**
 * Payload shape emitted on the `dev:seed-finding` Tauri event.
 *
 * Field names use camelCase so the TS listener (`TauriFindingsListener.ts`)
 * can spread them directly into a `Finding` object without translation.
 * The actual emit site is in `commands::dev_findings::dev_seed_finding`.
 *
 * Renamed in the schema registry to `DevSeedFindingPayload` to disambiguate
 * from the various `Finding*` types in `qontinui_types::findings`.
 */
interface DevSeedFindingPayload {
  actionType: string;
  actionable: boolean;
  categoryId: string;
  description: string;
  detectedAt: number;
  id: string;
  severity: string;
  sourceSessionId?: string | null;
  status: string;
  title: string;
  [k: string]: unknown;
}

/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with `just generate-types` or
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */

/**
 * Payload shape for the `review-approved` and `review-rejected` Tauri events
 * emitted by `commands::productivity::approve_recommendation` /
 * `reject_recommendation` after a user resolves a medium-confidence
 * recommendation card. Field names are explicit camelCase to match the
 * `serde_json::json!()` literal previously used at the emit site.
 *
 * Single struct shared by both channels: only `user_decision` differs
 * (`"approved"` vs `"rejected"`), so a tagged enum would inflate the wire
 * format without payoff.
 */
interface RecommendationReviewDecisionPayload {
  reviewId: string;
  taskId: string;
  userDecision: string;
  [k: string]: unknown;
}

export type { AppEvent, CanvasPanel as CanvasUpdatePanel, DevSeedFindingPayload, RecommendationReviewDecisionPayload, RunnerConnectedConnection, RunnerFinding, RunnerFindingCodeContext, RunnerFindingUserInput, RunnerRelayMessage, RunnerStatusEvent, TerminalExitEvent, TerminalInfo, TerminalOutputEvent };
