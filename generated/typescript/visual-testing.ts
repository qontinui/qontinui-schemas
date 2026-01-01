/**
 * Auto-generated TypeScript types from qontinui-schemas
 * DO NOT EDIT - regenerate with: poetry run python scripts/generate_typescript.py
 */

export interface IgnoreRegion {
  /** X coordinate */
  x: number;
  /** Y coordinate */
  y: number;
  /** Width in pixels */
  width: number;
  /** Height in pixels */
  height: number;
  /** Optional region name */
  name?: string | null;
}

export interface ComparisonSettings {
  /** Comparison algorithm: 'ssim', 'pixel_diff', 'perceptual_hash' */
  algorithm?: string;
  /** Similarity threshold (0-1) */
  threshold?: number;
  /** Regions to ignore */
  ignore_regions?: IgnoreRegion[];
}

export interface VisualBaselineCreate {
  /** State this baseline is for */
  state_name: string;
  /** Optional workflow ID */
  workflow_id?: string | null;
  /** Comparison settings (defaults to SSIM with 0.95 threshold) */
  comparison_settings?: ComparisonSettings | null;
  /** Notes about this baseline */
  approval_notes?: string | null;
}

export interface VisualBaselineFromScreenshot {
  /** Source screenshot ID */
  screenshot_id: string;
  /** State name */
  state_name: string;
  /** Workflow ID */
  workflow_id?: string | null;
  /** Comparison settings (defaults to SSIM with 0.95 threshold) */
  comparison_settings?: ComparisonSettings | null;
  /** Approval notes */
  approval_notes?: string | null;
}

export interface VisualBaselineUpdate {
  /** New comparison settings */
  comparison_settings?: ComparisonSettings | null;
  /** New approval notes */
  approval_notes?: string | null;
}

export interface VisualBaselineResponse {
  /** Baseline ID */
  id: string;
  /** Project ID */
  project_id: string;
  /** State name */
  state_name: string;
  /** Workflow ID */
  workflow_id?: string | null;
  /** Image width */
  width: number;
  /** Image height */
  height: number;
  /** File size */
  file_size_bytes?: number | null;
  /** Perceptual hash */
  perceptual_hash?: string | null;
  /** Baseline version */
  version: number;
  /** Whether this is the active version */
  is_active: boolean;
  /** Approved by user */
  approved_by_user_id?: string | null;
  /** Approval time (UTC) */
  approved_at?: any | null;
  /** Approval notes */
  approval_notes?: string | null;
  /** Comparison settings */
  comparison_settings: Record<string, any>;
  /** Source run ID */
  source_test_run_id?: string | null;
  /** Source screenshot ID */
  source_screenshot_id?: string | null;
  /** Creation time (UTC) */
  created_at: string;
  /** Last update time (UTC) */
  updated_at: string;
  /** Image URL */
  image_url?: string | null;
  /** Thumbnail URL */
  thumbnail_url?: string | null;
}

export interface VisualBaselineListResponse {
  /** Baselines */
  items: VisualBaselineResponse[];
  /** Total count */
  total: number;
  /** Items skipped */
  skip: number;
  /** Items per page */
  limit: number;
}

export interface DiffRegion {
  /** X coordinate */
  x: number;
  /** Y coordinate */
  y: number;
  /** Width */
  width: number;
  /** Height */
  height: number;
  /** Percentage of change in region */
  change_percentage: number;
  /** Changed pixel count */
  pixel_count?: number | null;
}

export interface VisualComparisonCreate {
  /** Screenshot to compare */
  screenshot_id: string;
  /** Baseline to compare against */
  baseline_id?: string | null;
  /** Override algorithm */
  algorithm?: string | null;
  /** Override threshold */
  threshold?: number | null;
}

export interface VisualComparisonResponse {
  /** Comparison ID */
  id: string;
  /** Test run ID */
  test_run_id: string;
  /** Baseline ID */
  baseline_id?: string | null;
  /** Screenshot ID */
  screenshot_id: string;
  /** Transition execution ID */
  transition_execution_id?: string | null;
  /** State name */
  state_name: string;
  /** Algorithm used */
  comparison_algorithm: string;
  /** Similarity score (0-1) */
  similarity_score: number;
  /** Threshold used */
  threshold_used: number;
  /** Comparison status: passed, failed, pending */
  status: string;
  /** Number of diff regions */
  diff_region_count: number;
  /** Execution time in ms */
  execution_time_ms?: number | null;
  /** Reviewer user ID */
  reviewed_by_user_id?: string | null;
  /** Review time (UTC) */
  reviewed_at?: any | null;
  /** Decision: approved, rejected, new_baseline */
  review_decision?: string | null;
  /** Review notes */
  review_notes?: string | null;
  /** Created deficiency ID */
  deficiency_id?: string | null;
  /** Error message if failed */
  error_message?: string | null;
  /** Creation time (UTC) */
  created_at: string;
  /** Diff image URL */
  diff_image_url?: string | null;
  /** Screenshot URL */
  screenshot_url?: string | null;
  /** Baseline URL */
  baseline_url?: string | null;
}

export interface VisualComparisonDetail {
  /** Comparison ID */
  id: string;
  /** Test run ID */
  test_run_id: string;
  /** Baseline ID */
  baseline_id?: string | null;
  /** Screenshot ID */
  screenshot_id: string;
  /** Transition execution ID */
  transition_execution_id?: string | null;
  /** State name */
  state_name: string;
  /** Algorithm used */
  comparison_algorithm: string;
  /** Similarity score (0-1) */
  similarity_score: number;
  /** Threshold used */
  threshold_used: number;
  /** Comparison status: passed, failed, pending */
  status: string;
  /** Number of diff regions */
  diff_region_count: number;
  /** Execution time in ms */
  execution_time_ms?: number | null;
  /** Reviewer user ID */
  reviewed_by_user_id?: string | null;
  /** Review time (UTC) */
  reviewed_at?: any | null;
  /** Decision: approved, rejected, new_baseline */
  review_decision?: string | null;
  /** Review notes */
  review_notes?: string | null;
  /** Created deficiency ID */
  deficiency_id?: string | null;
  /** Error message if failed */
  error_message?: string | null;
  /** Creation time (UTC) */
  created_at: string;
  /** Diff image URL */
  diff_image_url?: string | null;
  /** Screenshot URL */
  screenshot_url?: string | null;
  /** Baseline URL */
  baseline_url?: string | null;
  /** Diff regions */
  diff_regions?: DiffRegion[];
}

export interface VisualComparisonListResponse {
  /** Comparisons */
  items: VisualComparisonResponse[];
  /** Total count */
  total: number;
  /** Items skipped */
  skip: number;
  /** Items per page */
  limit: number;
}

export interface VisualComparisonSummary {
  /** Comparison ID */
  comparison_id: string;
  /** Baseline ID */
  baseline_id?: string | null;
  /** Similarity score */
  similarity_score: number;
  /** Threshold used */
  threshold: number;
  /** Whether comparison passed */
  passed: boolean;
  /** Status */
  status: string;
  /** Diff image URL */
  diff_image_url?: string | null;
  /** Number of diff regions */
  diff_region_count?: number;
}

export interface ComparisonReview {
  /** Decision: 'approved', 'rejected', 'new_baseline' */
  decision: string;
  /** Review notes */
  notes?: string | null;
}

export interface ComparisonStats {
  /** Total comparisons */
  total: number;
  /** Passed comparisons */
  passed: number;
  /** Failed comparisons */
  failed: number;
  /** Pending review */
  pending_review: number;
  /** Approved as new baseline */
  approved_as_new: number;
  /** No baseline available */
  no_baseline: number;
  /** Pass rate percentage */
  pass_rate: number;
}

export interface CoverageData {
  /** Overall coverage percentage */
  coverage_percentage: number;
  /** Number of states visited */
  states_covered: number;
  /** Total states in workflow */
  total_states: number;
  /** Number of transitions executed */
  transitions_covered: number;
  /** Total transitions in workflow */
  total_transitions: number;
  /** List of unvisited state IDs */
  uncovered_states?: string[];
  /** List of unexecuted transition IDs */
  uncovered_transitions?: string[];
  /** Map of state ID to visit count */
  state_visit_counts?: Record<string, any>;
  /** Map of transition ID to execution count */
  transition_execution_counts?: Record<string, any>;
}

export interface CoverageSnapshot {
  /** Snapshot ID */
  id: string;
  /** Test run ID */
  run_id: string;
  /** Snapshot sequence number */
  sequence_number: number;
  /** Coverage percentage */
  coverage_percentage: number;
  /** States covered */
  states_covered: number;
  /** Total states */
  total_states: number;
  /** Transitions covered */
  transitions_covered: number;
  /** Total transitions */
  total_transitions: number;
  /** Snapshot time (UTC) */
  timestamp: string;
}

export interface CoverageUpdate {
  /** Total transitions executed */
  total_transitions_executed: number;
  /** Unique transitions covered */
  unique_transitions_covered: number;
  /** Coverage percentage */
  coverage_percentage: number;
  /** Transition to execution count */
  transition_coverage_map?: Record<string, any>;
  /** State to visit count */
  state_coverage_map?: Record<string, any>;
  /** Uncovered transition IDs */
  uncovered_transitions?: string[];
}

export interface CoverageUpdateResponse {
  /** Test run ID */
  run_id: string;
  /** Whether update succeeded */
  coverage_updated: boolean;
  /** Current coverage */
  coverage_percentage: number;
  /** Unique transitions */
  unique_transitions_covered: number;
}

export interface CoverageTrendDataPoint {
  /** Date (YYYY-MM-DD) */
  date: string;
  /** Number of runs */
  runs_count: number;
  /** Average coverage */
  avg_coverage_percentage: number;
  /** Maximum coverage */
  max_coverage_percentage: number;
  /** Minimum coverage */
  min_coverage_percentage: number;
  /** Total transitions */
  total_transitions_executed: number;
  /** Unique transitions */
  unique_transitions_covered: number;
}

export interface CoverageTrendResponse {
  /** Project ID */
  project_id: string;
  /** Workflow ID if filtered */
  workflow_id?: string | null;
  /** Start date */
  start_date: string;
  /** End date */
  end_date: string;
  /** Granularity: daily, weekly, monthly */
  granularity: string;
  /** Trend data */
  data_points: CoverageTrendDataPoint[];
  /** Overall statistics */
  overall_stats: Record<string, any>;
}

export interface CoverageGap {
  /** State or transition ID */
  id: string;
  /** Display name */
  name: string;
  /** Type: 'state' or 'transition' */
  type: string;
  /** Source state (for transitions) */
  from_state?: string | null;
  /** Target state (for transitions) */
  to_state?: string | null;
  /** Last time covered (UTC) */
  last_covered_at?: any | null;
  /** Priority: 'critical', 'high', 'medium', 'low' */
  priority?: string;
}

export interface CoverageGapsResponse {
  /** Project ID */
  project_id: string;
  /** Workflow ID */
  workflow_id: string;
  /** Current coverage percentage */
  current_coverage: number;
  /** Coverage gaps */
  gaps: CoverageGap[];
  /** Total gap count */
  total_gaps: number;
  /** Critical gaps count */
  critical_gaps: number;
  /** Recommended transitions to cover */
  recommended_next?: string[];
}

export interface CoverageHeatmapCell {
  /** State ID */
  state_id: string;
  /** State name */
  state_name: string;
  /** Number of visits */
  visit_count: number;
  /** Last visit time (UTC) */
  last_visited_at?: any | null;
  /** Coverage intensity (0-1) */
  coverage_intensity: number;
}

export interface CoverageHeatmapResponse {
  /** Project ID */
  project_id: string;
  /** Workflow ID */
  workflow_id: string;
  /** Run ID if single-run view */
  run_id?: string | null;
  /** Heatmap cells */
  cells: CoverageHeatmapCell[];
  /** Maximum visit count */
  max_visits: number;
}
