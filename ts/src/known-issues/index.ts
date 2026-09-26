/**
 * Known Issues Registry Types
 *
 * Persistent known issue tracking that survives across workflow runs.
 * Issues are scoped to specs, URLs, components, or global.
 */

// Every type below is generated from the runner's
// `src-tauri/src/known_issue_types.rs` — the wire of the `known_issues` Tauri
// commands. Do not edit by hand; regenerate via
// `qontinui-runner/src-tauri/scripts/generate_types.sh`. Generic names are
// published under a `KnownIssue*` title in the flat generated namespace and
// re-exported here under the names this module has always used.

export type { KnownIssueCategory as IssueCategory } from "../generated/KnownIssueCategory";
export type { KnownIssueScopeType as ScopeType } from "../generated/KnownIssueScopeType";
export type { KnownIssueDetectionMethod as DetectionMethod } from "../generated/KnownIssueDetectionMethod";
export type { KnownIssueSeverity } from "../generated/KnownIssueSeverity";
export type { KnownIssueStatus as IssueStatus } from "../generated/KnownIssueStatus";
export type { KnownIssueProvenance as IssueProvenance } from "../generated/KnownIssueProvenance";
export type { KnownIssue } from "../generated/KnownIssue";
export type { CreateKnownIssueRequest } from "../generated/CreateKnownIssueRequest";
export type { UpdateKnownIssueRequest } from "../generated/UpdateKnownIssueRequest";
export type { ListKnownIssuesQuery } from "../generated/ListKnownIssuesQuery";
export type { CreatePatternTemplateRequest } from "../generated/CreatePatternTemplateRequest";
export type { IssuePatternTemplateParameter as TemplateParameter } from "../generated/IssuePatternTemplateParameter";
export type { IssuePatternTemplate } from "../generated/IssuePatternTemplate";

import type { KnownIssueCategory } from "../generated/KnownIssueCategory";
import type { KnownIssueSeverity } from "../generated/KnownIssueSeverity";
import type { KnownIssueDetectionMethod } from "../generated/KnownIssueDetectionMethod";

// ---------------------------------------------------------------------------
// Display metadata (UI-only — no Rust source)
// ---------------------------------------------------------------------------

/** All issue categories with display labels */
export const ISSUE_CATEGORIES: { value: KnownIssueCategory; label: string }[] = [
  { value: "duplication", label: "Duplication" },
  { value: "rendering", label: "Rendering" },
  { value: "data_integrity", label: "Data Integrity" },
  { value: "timing", label: "Timing" },
  { value: "layout", label: "Layout" },
  { value: "state", label: "State" },
  { value: "performance", label: "Performance" },
  { value: "encoding", label: "Encoding" },
  { value: "navigation", label: "Navigation" },
  { value: "authentication", label: "Authentication" },
  { value: "other", label: "Other" },
];

/** All severity levels with display labels */
export const ISSUE_SEVERITIES: { value: KnownIssueSeverity; label: string }[] = [
  { value: "critical", label: "Critical" },
  { value: "high", label: "High" },
  { value: "medium", label: "Medium" },
  { value: "low", label: "Low" },
];

/** All detection methods with display labels */
export const DETECTION_METHODS: { value: KnownIssueDetectionMethod; label: string }[] = [
  { value: "algorithmic", label: "Algorithmic (automatic)" },
  { value: "ai_judgment", label: "AI Judgment" },
  { value: "visual", label: "Visual (screenshot)" },
  { value: "command", label: "Shell Command" },
  { value: "ui_bridge", label: "UI Bridge" },
];
