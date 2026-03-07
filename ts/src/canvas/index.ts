/**
 * Canvas Types
 *
 * Type definitions for the A2UI (Agent-to-UI) Canvas system.
 * Canvas panels allow the AI agent to render rich, structured visual content
 * in the dashboard during workflow execution.
 */

/**
 * Supported canvas component types.
 * These are validated server-side against an allowlist.
 */
export type CanvasComponentType =
  | "Markdown"
  | "CodeDiff"
  | "Table"
  | "FileTree"
  | "KeyValue"
  | "Terminal"
  | "Alert"
  | "Timeline"
  | "ProgressChart"
  | "FindingList"
  | "Checklist"
  | "SummaryStats"
  | "StateTimeline"
  | "Waterfall"
  | "Sparkline"
  | "WaffleChart"
  | "PhaseTimeline"
  | "IterationComparison"
  | "StepDurationChart"
  | "PhaseDistribution";

/**
 * A canvas panel rendered in the dashboard.
 */
export interface CanvasPanel {
  panel_id: string;
  component: CanvasComponentType;
  title: string;
  data: Record<string, unknown>;
  priority?: number;
  size?: "compact" | "normal" | "large";
  group?: string;
  task_run_id?: string;
  created_at?: string;
  updated_at?: string;
}

/**
 * Event emitted when a canvas panel is created, updated, or deleted.
 */
export interface CanvasUpdateEvent {
  action: "create" | "update" | "delete" | "clear";
  panel_id: string;
  panel?: CanvasPanel;
  task_run_id?: string;
}

// ============================================================================
// Component-specific data schemas
// ============================================================================

/** Data for Markdown component. */
export interface MarkdownData {
  content: string;
}

/** Data for Table component. */
export interface TableData {
  columns: string[];
  rows: (string | number | boolean | null)[][];
  sortable?: boolean;
}

/** Data for CodeDiff component. */
export interface CodeDiffData {
  file_path: string;
  language?: string;
  old_content?: string;
  new_content?: string;
  unified_diff?: string;
}

/** Data for KeyValue component. */
export interface KeyValueData {
  pairs: Array<{
    key: string;
    value: string | number | boolean;
    style?: "default" | "success" | "warning" | "error";
  }>;
}

/** Data for Alert component. */
export interface AlertData {
  severity: "info" | "success" | "warning" | "error";
  message: string;
  details?: string;
}

/** Data for Terminal component. */
export interface TerminalData {
  lines: string[];
  max_lines?: number;
}

/** Data for Timeline component. */
export interface TimelineData {
  events: Array<{
    timestamp?: string;
    title: string;
    description?: string;
    status?: "pending" | "running" | "success" | "failed";
  }>;
}

/** Data for FileTree component. */
export interface FileTreeData {
  root: string;
  entries: Array<{
    path: string;
    type: "file" | "directory";
    status?: "added" | "modified" | "deleted";
  }>;
}

/** Data for ProgressChart component. */
export interface ProgressChartData {
  segments: Array<{ label: string; value: number; color?: string }>;
  total?: number;
}

/** Data for FindingList component. */
export interface FindingListData {
  findings: Array<{
    id?: string;
    title: string;
    description?: string;
    severity?: "info" | "low" | "medium" | "high" | "critical";
    location?: string;
  }>;
}

/** Data for Checklist component. */
export interface ChecklistData {
  items: Array<{
    id: string;
    label: string;
    checked: boolean;
    description?: string;
  }>;
}

/** Data for SummaryStats component. */
export interface SummaryStatsData {
  total: number;
  passed: number;
  failed: number;
  skipped?: number;
  label?: string;
}

/** Data for StateTimeline component. */
export interface StateTimelineData {
  steps: Array<{
    name: string;
    iterations: Array<{
      iteration: number;
      status: "pass" | "fail" | "skip" | "pending";
    }>;
  }>;
}

/** Data for Waterfall component. */
export interface WaterfallData {
  entries: Array<{
    name: string;
    start_ms: number;
    duration_ms: number;
    status?: "running" | "success" | "failed" | "skipped" | "pending";
    phase?: string;
  }>;
  total_duration_ms: number;
}

/** Data for Sparkline component. */
export interface SparklineData {
  series: Array<{
    name: string;
    values: Array<{
      iteration: number;
      outcome: "pass" | "fail";
    }>;
  }>;
}

/** Data for WaffleChart component. */
export interface WaffleChartData {
  cells: Array<{
    label: string;
    status: "pass" | "fail" | "pending" | "running" | "skip";
  }>;
  columns?: number;
}

/** Data for PhaseTimeline component. */
export interface PhaseTimelineData {
  phases: Array<{
    name: string;
    duration_ms: number;
    status: "completed" | "running" | "pending" | "failed";
    step_count: number;
  }>;
  total_duration_ms: number;
}

/** Data for IterationComparison component. */
export interface IterationComparisonData {
  iterations: Array<{
    iteration: number;
    passed: number;
    failed: number;
    total: number;
  }>;
}

/** Data for StepDurationChart component. */
export interface StepDurationChartData {
  steps: Array<{
    name: string;
    duration_ms: number;
    status: "success" | "failed" | "running" | "skipped";
    phase?: string;
  }>;
  max_duration_ms: number;
}

/** Data for PhaseDistribution component. */
export interface PhaseDistributionData {
  segments: Array<{
    phase: string;
    duration_ms: number;
    percentage: number;
    color?: string;
  }>;
  total_duration_ms: number;
}
