"""Unified Workflow Pydantic models.

The Unified Workflow is the primary orchestration format for Qontinui task execution.
It defines a multi-phase workflow with setup, verification, agentic, and completion
steps that the runner executes iteratively until verification passes or limits are reached.

These models are the canonical Python representation of the Unified Workflow schema.
The TypeScript definitions in qontinui-web/frontend are the primary source for the
full type structure; these Python models are used by the web backend for database
serialization/deserialization.

Step arrays use dict[str, Any] because steps are polymorphic (CommandStep, PromptStep,
UiBridgeStep). The TypeScript side has proper discriminated union types.
"""

from __future__ import annotations

from typing import Any, Literal

from pydantic import BaseModel, Field

# =============================================================================
# Literal Types (matching TypeScript string unions)
# =============================================================================

WorkflowPhase = Literal["setup", "verification", "agentic", "completion"]
"""Phase of workflow execution."""

StepTypeName = Literal["command", "ui_bridge", "prompt", "workflow", "native_accessibility"]
"""Discriminator for step types."""

CheckType = Literal[
    "lint",
    "format",
    "typecheck",
    "analyze",
    "security",
    "custom_command",
    "http_status",
    "ai_review",
    "ci_cd",
]
"""Type of verification check to run."""

TestType = Literal[
    "playwright",
    "qontinui_vision",
    "python",
    "repository",
    "custom_command",
]
"""Type of test execution."""

PlaywrightExecutionMode = Literal["independent", "chained"]
"""How Playwright tests are executed relative to each other."""

HttpMethod = Literal["GET", "POST", "PUT", "PATCH", "DELETE"]
"""HTTP method for API steps."""

ApiContentType = Literal[
    "application/json",
    "application/x-www-form-urlencoded",
    "text/plain",
    "none",
]
"""Content type for API request bodies."""


# =============================================================================
# Supporting Models
# =============================================================================


class HealthCheckUrl(BaseModel):
    """URL to health-check before workflow execution begins.

    Health checks ensure that required services are available before
    running workflow steps that depend on them.
    """

    name: str = Field(..., description="Human-readable name for this health check")
    url: str = Field(..., description="URL to check for availability")
    expected_status: int | None = Field(
        None, description="Expected HTTP status code (defaults to 200)"
    )
    timeout_seconds: int | None = Field(
        None, description="Timeout for the health check request"
    )
    is_critical: bool | None = Field(
        None,
        description="Whether failure of this check should abort the workflow",
    )


class ApiVariableExtraction(BaseModel):
    """Defines how to extract a variable from an API response.

    Extracted variables can be referenced in subsequent steps via
    template syntax.
    """

    variable_name: str = Field(
        ..., description="Name of the variable to store the extracted value"
    )
    json_path: str = Field(
        ..., description="JSONPath expression to extract the value from the response"
    )
    default_value: str | None = Field(
        None, description="Default value if extraction fails"
    )


class ApiAssertion(BaseModel):
    """An assertion to validate against an API response.

    Assertions are evaluated after the API call completes and determine
    whether the step passes or fails.
    """

    type: Literal[
        "status_code", "json_path", "header", "body_contains", "response_time"
    ] = Field(..., description="Type of assertion to evaluate")
    expected: str | int = Field(..., description="Expected value to compare against")
    json_path: str | None = Field(
        None, description="JSONPath expression (required for json_path assertions)"
    )
    header_name: str | None = Field(
        None, description="Header name (required for header assertions)"
    )
    operator: (
        Literal["equals", "contains", "matches", "greater_than", "less_than"] | None
    ) = Field(None, description="Comparison operator (defaults to equals)")


# =============================================================================
# Step Models
# =============================================================================


class BaseStep(BaseModel):
    """Base fields shared by all workflow step types.

    Every step has an id, name, and optional configuration for error handling,
    input/output variable binding, dependencies, and retry behavior.
    """

    id: str = Field(..., description="Unique identifier for this step")
    name: str = Field(..., description="Human-readable name for this step")
    fail_on_console_errors: bool | None = Field(
        None, description="Whether to fail the step if console errors are detected"
    )
    inputs: dict[str, str] | None = Field(
        None,
        description="Input variable bindings (variable_name -> source expression)",
    )
    extract: dict[str, str] | None = Field(
        None,
        description="Output variable extractions (variable_name -> extraction expression)",
    )
    depends_on: list[str] | None = Field(
        None, description="Step IDs that must complete before this step runs"
    )
    required: bool | None = Field(
        None, description="Whether this step must succeed for the phase to pass"
    )
    retry: dict[str, int] | None = Field(
        None,
        description="Retry configuration with 'count' and 'delay_ms' keys",
    )


class CommandStep(BaseStep):
    """A step that executes shell commands, checks, or tests.

    CommandStep is the most versatile step type, supporting shell commands,
    lint/format/typecheck checks, test execution, CI/CD integration, and more.
    The 'mode' field determines which subset of fields are relevant.
    """

    type: Literal["command"] = Field("command", description="Step type discriminator")
    phase: Literal["setup", "verification", "completion"] = Field(
        ..., description="Workflow phase this step belongs to"
    )

    # Shell mode fields
    mode: Literal["shell", "check", "check_group", "test"] | None = Field(
        None, description="Execution mode determining step behavior"
    )
    command: str | None = Field(None, description="Shell command to execute")
    working_directory: str | None = Field(
        None, description="Working directory for command execution"
    )
    timeout_seconds: int | None = Field(
        None, description="Maximum execution time in seconds"
    )
    fail_on_error: bool | None = Field(
        None, description="Whether to fail the step on non-zero exit code"
    )
    run_on_subsequent_iterations: bool | None = Field(
        None,
        description="Whether to re-run this step on subsequent verification iterations",
    )
    shell_command_id: str | None = Field(
        None, description="Reference ID for a pre-defined shell command"
    )

    # Check mode fields
    check_type: CheckType | None = Field(None, description="Type of check to perform")
    tool: str | None = Field(
        None, description="Tool to use for the check (e.g., 'eslint', 'prettier')"
    )
    check_id: str | None = Field(None, description="Unique identifier for this check")
    config_path: str | None = Field(None, description="Path to tool configuration file")
    auto_fix: bool | None = Field(
        None, description="Whether to attempt auto-fix on failure"
    )
    fail_on_warning: bool | None = Field(
        None, description="Whether to treat warnings as failures"
    )

    # CI/CD fields
    repository: str | None = Field(None, description="Repository for CI/CD integration")
    workflow_name: str | None = Field(
        None, description="CI/CD workflow name to trigger or monitor"
    )
    branch: str | None = Field(None, description="Git branch for CI/CD operations")
    wait_for_completion: bool | None = Field(
        None, description="Whether to wait for CI/CD workflow completion"
    )

    # Check group fields
    check_group_id: str | None = Field(
        None, description="Identifier for grouping related checks"
    )

    # Test mode fields
    test_type: TestType | None = Field(None, description="Type of test to execute")
    test_id: str | None = Field(None, description="Unique identifier for this test")
    code: str | None = Field(
        None, description="Inline code to execute (for python/custom tests)"
    )
    script_id: str | None = Field(
        None, description="Reference ID for a pre-defined test script"
    )
    script_content: str | None = Field(
        None, description="Inline script content for test execution"
    )
    target_url: str | None = Field(None, description="URL target for Playwright tests")
    fused_script_id: str | None = Field(
        None, description="ID of a fused (combined) test script"
    )
    execution_mode: PlaywrightExecutionMode | None = Field(
        None, description="Playwright execution mode (independent or chained)"
    )


class PromptStep(BaseStep):
    """A step that sends a prompt to an AI provider.

    PromptSteps are used in the agentic phase to interact with AI models,
    and can also appear in other phases for summarization or analysis.
    """

    type: Literal["prompt"] = Field("prompt", description="Step type discriminator")
    phase: Literal["setup", "verification", "agentic", "completion"] = Field(
        ..., description="Workflow phase this step belongs to"
    )
    content: str = Field(..., description="Prompt content to send to the AI provider")
    prompt_id: str | None = Field(
        None, description="Reference ID for a pre-defined prompt template"
    )
    provider: str | None = Field(
        None,
        description="AI provider override (e.g., 'anthropic', 'openai')",
    )
    model: str | None = Field(
        None, description="Model override (e.g., 'claude-sonnet-4-20250514')"
    )
    is_summary_step: bool | None = Field(
        None, description="Whether this prompt generates a run summary"
    )


class UiBridgeStep(BaseStep):
    """A step that interacts with the UI via the UI Bridge SDK.

    UiBridgeSteps can navigate to URLs, execute instructions against
    UI elements, assert element states, take snapshots, or compare
    snapshots for visual regression detection.
    """

    type: Literal["ui_bridge"] = Field(
        "ui_bridge", description="Step type discriminator"
    )
    phase: Literal["setup", "verification", "completion"] = Field(
        ..., description="Workflow phase this step belongs to"
    )
    action: Literal["navigate", "execute", "assert", "snapshot", "compare"] = Field(
        ..., description="UI Bridge action to perform"
    )

    # Navigate action fields
    url: str | None = Field(
        None, description="URL to navigate to (for navigate action)"
    )

    # Execute action fields
    instruction: str | None = Field(
        None, description="Instruction to execute against the UI"
    )
    target: str | None = Field(
        None, description="Target element selector or identifier"
    )

    # Assert action fields
    assert_type: (
        Literal["exists", "text_equals", "contains", "visible", "enabled"] | None
    ) = Field(None, description="Type of assertion to evaluate")
    expected: str | None = Field(None, description="Expected value for the assertion")
    timeout_ms: int | None = Field(
        None, description="Timeout for the action in milliseconds"
    )

    # Compare action fields
    comparison_mode: Literal["structural", "visual", "both"] | None = Field(
        None, description="Snapshot comparison mode"
    )
    reference_snapshot_id: str | None = Field(
        None, description="ID of the reference snapshot to compare against"
    )
    severity_threshold: Literal["critical", "major", "minor", "info"] | None = Field(
        None,
        description="Minimum severity level to report as a comparison failure",
    )


class WorkflowStep(BaseStep):
    """A step that runs a saved workflow inline.

    WorkflowSteps allow workflow composition — referencing another saved
    workflow to execute its full loop as a single step. Not allowed in
    the agentic phase.
    """

    type: Literal["workflow"] = Field("workflow", description="Step type discriminator")
    phase: Literal["setup", "verification", "completion"] = Field(
        ..., description="Workflow phase this step belongs to"
    )
    workflow_id: str = Field(
        ..., description="ID of the referenced workflow to execute"
    )
    workflow_name: str = Field(
        ..., description="Cached display name of the referenced workflow"
    )


# =============================================================================
# Workflow Structure Models
# =============================================================================


class WorkflowStage(BaseModel):
    """A named stage within a multi-stage workflow.

    Stages allow a workflow to be broken into sequential phases,
    each with their own steps and iteration limits. Stages execute
    in order, and each stage runs its own setup/verification/agentic/completion
    loop independently.
    """

    id: str = Field(..., description="Unique identifier for this stage")
    name: str = Field(..., description="Human-readable name for this stage")
    description: str | None = Field(
        None, description="Description of what this stage accomplishes"
    )
    setup_steps: list[dict[str, Any]] = Field(
        default_factory=list,
        description="Steps to run during the setup phase",
    )
    verification_steps: list[dict[str, Any]] = Field(
        default_factory=list,
        description="Steps to run during the verification phase",
    )
    agentic_steps: list[dict[str, Any]] = Field(
        default_factory=list,
        description="Steps to run during the agentic phase",
    )
    completion_steps: list[dict[str, Any]] = Field(
        default_factory=list,
        description="Steps to run during the completion phase",
    )
    max_iterations: int | None = Field(
        None,
        description="Maximum verification-agentic loop iterations for this stage",
    )
    timeout_seconds: int | None = Field(
        None, description="Maximum execution time for this stage"
    )
    provider: str | None = Field(
        None, description="AI provider override for this stage"
    )
    model: str | None = Field(None, description="AI model override for this stage")


class UnifiedWorkflow(BaseModel):
    """Top-level Unified Workflow definition.

    The Unified Workflow is Qontinui's primary orchestration format. It defines
    a multi-phase execution loop:

    1. **Setup** - One-time initialization (install deps, start services, etc.)
    2. **Verification** - Check current state (lint, test, type-check, etc.)
    3. **Agentic** - AI-driven work to fix issues found in verification
    4. **Completion** - Final steps after verification passes (deploy, notify, etc.)

    The verification and agentic phases loop until verification passes or
    max_iterations is reached.
    """

    id: str = Field(..., description="Unique identifier for this workflow")
    name: str = Field(..., description="Human-readable workflow name")
    description: str = Field(..., description="Description of what this workflow does")

    # Step arrays (polymorphic - stored as dicts)
    setup_steps: list[dict[str, Any]] = Field(
        default_factory=list,
        description="Steps to run during the setup phase",
    )
    verification_steps: list[dict[str, Any]] = Field(
        default_factory=list,
        description="Steps to run during the verification phase",
    )
    agentic_steps: list[dict[str, Any]] = Field(
        default_factory=list,
        description="Steps to run during the agentic phase",
    )
    completion_steps: list[dict[str, Any]] = Field(
        default_factory=list,
        description="Steps to run during the completion phase",
    )

    # Execution configuration
    max_iterations: int | None = Field(
        None,
        description="Maximum verification-agentic loop iterations",
    )
    timeout_seconds: int | None = Field(
        None, description="Maximum total execution time in seconds"
    )
    provider: str | None = Field(
        None, description="Default AI provider for prompt steps"
    )
    model: str | None = Field(None, description="Default AI model for prompt steps")
    model_overrides: dict[str, Any] | None = Field(
        None,
        description="Per-phase model/provider overrides (keyed by phase name)",
    )
    reflection_mode: bool | None = Field(
        None,
        description="Whether to enable reflection mode during agentic iterations",
    )
    rollback_policy: Literal["none", "last_good", "clean"] | None = Field(
        None,
        description="Policy for automatic git rollback when the workflow fails",
    )
    enforce_token_budget: bool | None = Field(
        None,
        description="Whether to stop execution if accumulated token usage exceeds the budget",
    )
    strict_cwd: bool | None = Field(
        None,
        description="Whether to enforce strict working directory for all steps",
    )
    use_worktree: bool | None = Field(
        None,
        description="Whether to create a new git branch and worktree for this run",
    )
    workflow_architecture: (
        Literal["traditional", "agentic_verification", "multi_agent_pipeline"] | None
    ) = Field(
        None,
        description="Workflow execution architecture override",
    )
    multi_agent_mode: bool | None = Field(
        None,
        description="Whether to spawn parallel sub-agents for independent tasks",
    )
    multi_agent_pipeline_config: dict[str, Any] | None = Field(
        None,
        description="Configuration for the multi-agent pipeline architecture",
    )
    tool_tags: list[str] | None = Field(
        None,
        description="Tags for per-execution tool whitelisting",
    )

    # Context and log configuration
    log_source_selection: str | dict[str, Any] | None = Field(
        None,
        description="Log source selection - string ID or inline configuration",
    )
    context_ids: list[str] | None = Field(
        None, description="IDs of context documents to include in AI prompts"
    )
    disabled_context_ids: list[str] | None = Field(
        None, description="IDs of context documents to exclude from AI prompts"
    )
    auto_include_contexts: bool | None = Field(
        None,
        description="Whether to automatically include relevant contexts",
    )

    # Feature flags
    skip_ai_summary: bool | None = Field(
        None, description="Whether to skip AI summary generation after completion"
    )
    log_watch_enabled: bool | None = Field(
        None, description="Whether to enable log watching during execution"
    )
    health_check_enabled: bool | None = Field(
        None, description="Whether to run health checks before execution"
    )
    health_check_urls: list[HealthCheckUrl] | None = Field(
        None, description="URLs to health-check before workflow execution"
    )

    # Template and staging
    prompt_template: str | None = Field(
        None,
        description="Template for constructing the agentic prompt from verification results",
    )
    stages: list[WorkflowStage] | None = Field(
        None,
        description="Ordered list of stages for multi-stage workflows",
    )
    stop_on_failure: bool | None = Field(
        None,
        description="Whether to stop the entire workflow if a stage fails",
    )
    is_favorite: bool | None = Field(
        None,
        description="Whether this workflow is marked as a favorite for quick access",
    )

    # Constraint and dependency metadata
    constraint_overrides: dict[str, bool] | None = Field(
        None,
        description="Per-constraint overrides: map of constraint_id to enabled/disabled",
    )
    dependency_graph: dict[str, Any] | None = Field(
        None,
        description="Dependency graph computed during generation",
    )
    cost_annotations: dict[str, Any] | None = Field(
        None,
        description="Cost annotations computed during generation",
    )
    quality_report: dict[str, Any] | None = Field(
        None,
        description="Quality report from the revision phase",
    )
    acceptance_criteria: dict[str, Any] | None = Field(
        None,
        description="Acceptance criteria from the specification agent (JSON blob)",
    )
    ai_reviewed: bool | None = Field(
        None,
        description="Whether the AI semantic review ran successfully during generation",
    )

    # Metadata
    category: str = Field(
        ...,
        description="Category for organizing workflows (e.g., 'development', 'testing')",
    )
    tags: list[str] = Field(
        default_factory=list,
        description="Tags for filtering and searching workflows",
    )
    created_at: str = Field(..., description="ISO 8601 timestamp of workflow creation")
    modified_at: str = Field(..., description="ISO 8601 timestamp of last modification")


# =============================================================================
# Exports
# =============================================================================

__all__ = [
    # Literal types
    "WorkflowPhase",
    "StepTypeName",
    "CheckType",
    "TestType",
    "PlaywrightExecutionMode",
    "HttpMethod",
    "ApiContentType",
    # Supporting models
    "HealthCheckUrl",
    "ApiVariableExtraction",
    "ApiAssertion",
    # Step models
    "BaseStep",
    "CommandStep",
    "PromptStep",
    "UiBridgeStep",
    "WorkflowStep",
    # Workflow structure
    "WorkflowStage",
    "UnifiedWorkflow",
]
