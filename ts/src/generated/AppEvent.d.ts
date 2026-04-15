/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with `just generate-types` or
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */

import type { FlowEvent } from './FlowEvent';

/**
 * Unified application events for frontend communication.
 */
export type AppEvent =
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
        iteration: number;
        phase: string;
        state_data?: unknown;
        task_run_id: string;
        workflow_stage: string;
        [k: string]: unknown;
      };
      event_type: "OrchestratorStateChange";
      [k: string]: unknown;
    }
  | {
      data: {
        details?: unknown;
        status: string;
        step_index: number;
        step_name: string;
        task_run_id: string;
        timestamp: number;
        [k: string]: unknown;
      };
      event_type: "StepProgress";
      [k: string]: unknown;
    }
  | {
      data: {
        details?: unknown;
        iteration?: number | null;
        status: string;
        task_run_id: string;
        timestamp: number;
        [k: string]: unknown;
      };
      event_type: "TaskRunUpdate";
      [k: string]: unknown;
    }
  | {
      data: {
        approval_id: string;
        iteration: number;
        prompt: string;
        task_run_id: string;
        [k: string]: unknown;
      };
      event_type: "ApprovalRequired";
      [k: string]: unknown;
    }
  | {
      data: {
        action: string;
        approval_id: string;
        approved: boolean;
        task_run_id: string;
        [k: string]: unknown;
      };
      event_type: "ApprovalResolved";
      [k: string]: unknown;
    }
  | {
      data: {
        confidence: number;
        iteration: number;
        question: string;
        question_id: string;
        risk_level: string;
        task_run_id: string;
        [k: string]: unknown;
      };
      event_type: "DeferredQuestionCreated";
      [k: string]: unknown;
    }
  | {
      data: {
        question_id: string;
        rework_triggered: boolean;
        status: string;
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
        accumulated_length: number;
        chunk: string;
        task_run_id: string;
        [k: string]: unknown;
      };
      event_type: "AiOutputChunk";
      [k: string]: unknown;
    }
  | {
      data: {
        failed_step_count: number;
        is_stalled: boolean;
        iteration: number;
        new_failures: number;
        passed_step_count: number;
        repeated_failures: number;
        skipped_step_count: number;
        task_run_id: string;
        [k: string]: unknown;
      };
      event_type: "IterationMetrics";
      [k: string]: unknown;
    }
  | {
      data: {
        attributed_failures: number;
        blame_json: string;
        iteration: number;
        oscillating_files: number;
        revert_patterns: number;
        task_run_id: string;
        [k: string]: unknown;
      };
      event_type: "BlameAttribution";
      [k: string]: unknown;
    }
  | {
      data: {
        has_blocking: boolean;
        iteration: number;
        results: unknown;
        summary: string;
        task_run_id: string;
        [k: string]: unknown;
      };
      event_type: "ConstraintResults";
      [k: string]: unknown;
    }
  | {
      data: {
        queue_position: number;
        task_run_id: string;
        workflow_name: string;
        [k: string]: unknown;
      };
      event_type: "WorkflowQueued";
      [k: string]: unknown;
    }
  | {
      data: {
        task_run_id: string;
        wait_time_ms: number;
        workflow_name: string;
        [k: string]: unknown;
      };
      event_type: "WorkflowDequeued";
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
