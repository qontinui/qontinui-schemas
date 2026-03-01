/**
 * Library Types
 *
 * Normalized types for library items that both apps agree on.
 * These represent the various reusable building blocks available
 * in the workflow builder's library pickers.
 */
interface LibraryItem {
    id: string;
    name: string;
    description?: string;
}
interface CheckItem extends LibraryItem {
    check_type: string;
    tool?: string;
    command?: string;
    working_directory?: string;
    config_path?: string;
    auto_fix?: boolean;
    fail_on_warning?: boolean;
    timeout_seconds?: number;
    enabled?: boolean;
}
interface CheckGroup extends LibraryItem {
    checks: string[];
    parallel?: boolean;
    stop_on_first_failure?: boolean;
}
interface ShellCommand extends LibraryItem {
    command: string;
    working_directory?: string;
    timeout_seconds?: number;
    platform?: string;
}
interface SavedPrompt extends LibraryItem {
    content: string;
    provider?: string;
    model?: string;
    category?: string;
    language?: string;
}
interface SavedApiRequest extends LibraryItem {
    method: string;
    url: string;
    headers?: Record<string, string>;
    body?: string;
    content_type?: string;
    auth_type?: string;
    variables?: Record<string, string>;
}
interface Context extends LibraryItem {
    content: string;
    auto_include?: boolean;
    priority?: number;
}
interface Macro extends LibraryItem {
    actions: MacroAction[];
    timeout_seconds?: number;
}
interface MacroAction {
    type: string;
    config: Record<string, unknown>;
    delay_ms?: number;
}
type BuiltInCategoryId = "code_bug" | "todo" | "config_issue" | "already_fixed" | "expected_behavior" | "data_migration" | "runtime_issue" | "test_issue" | "enhancement" | "warning" | "documentation" | "security" | "performance";
type FindingActionType = "auto_fix" | "needs_user_input" | "manual" | "informational";
type FindingStatus = "detected" | "in_progress" | "needs_input" | "resolved" | "wont_fix" | "deferred";
type FindingSeverity = "critical" | "high" | "medium" | "low" | "info";
type UserInputType = "text" | "choice" | "boolean" | "code";
interface FindingCategory {
    id: string;
    name: string;
    description: string;
    icon: string;
    color: string;
    isBuiltIn: boolean;
    defaultActionType: FindingActionType;
    sortOrder: number;
}
interface UserInputOption {
    value: string;
    label: string;
    description?: string;
}
interface UserInputRequest {
    id: string;
    findingId: string;
    question: string;
    context?: string;
    inputType: UserInputType;
    options?: UserInputOption[];
    required: boolean;
    defaultValue?: string;
}
interface CodeContext {
    file?: string;
    line?: number;
    endLine?: number;
    snippet?: string;
}
interface Finding {
    id: string;
    categoryId: string;
    severity: FindingSeverity;
    status: FindingStatus;
    title: string;
    description: string;
    sourceSessionId?: string;
    sourcePromptName?: string;
    detectedAt: number;
    actionType: FindingActionType;
    actionable: boolean;
    pendingQuestion?: UserInputRequest;
    userResponse?: string;
    resolvedAt?: number;
    resolution?: string;
    codeContext?: CodeContext;
    metadata?: Record<string, unknown>;
}
interface CategorySummary {
    count: number;
    actionable: number;
    resolved: number;
}
interface ReportSummary {
    totalFindings: number;
    byCategory: Record<string, CategorySummary>;
    bySeverity: Record<FindingSeverity, number>;
    byStatus: Record<FindingStatus, number>;
    actionable: number;
    needsUserInput: number;
    autoFixed: number;
    informational: number;
}
type ReportStatus = "running" | "completed" | "paused_for_input" | "failed" | "cancelled";
interface PhaseInfo {
    phase: number;
    startedAt: number;
    completedAt?: number;
    findingsDetected: number;
    findingsResolved: number;
}
interface ExecutionReport {
    id: string;
    sessionId: string;
    promptName: string;
    promptId?: string;
    startedAt: number;
    completedAt?: number;
    duration?: number;
    status: ReportStatus;
    findings: Finding[];
    summary: ReportSummary;
    pendingInputs: UserInputRequest[];
    phases: PhaseInfo[];
}
interface ParsedFinding {
    categoryId: string;
    severity: FindingSeverity;
    title: string;
    description: string;
    needsInput: boolean;
    question?: string;
    options?: string[];
    file?: string;
    line?: number;
    resolution?: string;
}
interface CategoryStore {
    customCategories: FindingCategory[];
    categoryOrder: string[];
    hiddenCategories: string[];
}
/** @deprecated Use FindingSeverity instead */
type SeverityLevel = FindingSeverity;
interface SeverityColorClasses {
    bg: string;
    text: string;
    border: string;
    dot: string;
}
type StatusColorType = "idle" | "running" | "success" | "error" | "warning" | "pending" | "paused" | "cancelled";
interface StatusColorClasses {
    bg: string;
    text: string;
    border: string;
    icon: string;
    pulse?: string;
}
type ActionColorType = "auto_fix" | "needs_user_input" | "manual" | "informational" | "skip" | "defer";
interface ActionColorClasses {
    bg: string;
    text: string;
    border: string;
    badge: string;
}
type AccentColor = "red" | "orange" | "amber" | "yellow" | "green" | "emerald" | "blue" | "cyan" | "purple" | "slate";
interface AccentColorClasses {
    bg: string;
    bgSolid: string;
    text: string;
    border: string;
}

export type { AccentColor, AccentColorClasses, ActionColorClasses, ActionColorType, BuiltInCategoryId, CategoryStore, CategorySummary, CheckGroup, CheckItem, CodeContext, Context, ExecutionReport, Finding, FindingActionType, FindingCategory, FindingSeverity, FindingStatus, LibraryItem, Macro, MacroAction, ParsedFinding, PhaseInfo, ReportStatus, ReportSummary, SavedApiRequest, SavedPrompt, SeverityColorClasses, SeverityLevel, ShellCommand, StatusColorClasses, StatusColorType, UserInputOption, UserInputRequest, UserInputType };
