"""
AI prompt action and sequence configuration models.

This module provides configuration models for AI-powered automation:
- AI_PROMPT: Execute a single AI prompt
- RUN_PROMPT_SEQUENCE: Execute an ordered sequence of prompts with context isolation

The key feature is context isolation - each prompt in a sequence runs in a fresh
AI session, preventing context overflow for complex multi-step operations.
"""

from typing import Any, Literal

from pydantic import BaseModel, Field


class PromptParameter(BaseModel):
    """Parameter definition for a prompt template.

    Parameters allow templates to be reusable with different values.
    Use {parameter_name} syntax in the prompt text.

    Example:
        {
            "name": "module_path",
            "type": "string",
            "description": "Path to the module to analyze",
            "required": true,
            "default": "src/"
        }
    """

    name: str = Field(..., description="Parameter name (used in {name} placeholders)")
    type: Literal["string", "number", "boolean", "path"] = Field(
        "string", description="Parameter value type"
    )
    description: str | None = Field(None, description="What this parameter is for")
    required: bool = Field(False, description="Whether this parameter must be provided")
    default: str | None = Field(None, description="Default value if not provided")


class AIPromptTemplate(BaseModel):
    """Reusable AI prompt template.

    Templates define prompts that can be reused across workflows and sequences.
    They support parameters using {parameter_name} syntax for dynamic values.

    Example:
        {
            "id": "fix-type-errors",
            "name": "Fix Type Errors",
            "category": "code-quality",
            "prompt": "Fix all type errors in {module_path}. Run mypy first.",
            "parameters": [
                {"name": "module_path", "type": "string", "required": true}
            ],
            "defaultTimeout": 600000
        }
    """

    id: str = Field(..., description="Unique template identifier")
    name: str = Field(..., description="Human-readable template name")
    description: str | None = Field(None, description="What this template does")
    category: str | None = Field(
        None,
        description="Category for organization (e.g., 'code-quality', 'security', 'testing')",
    )
    tags: list[str] = Field(default_factory=list, description="Tags for filtering")

    # The actual prompt content
    prompt: str = Field(
        ...,
        description="Prompt content with optional {param} placeholders",
    )

    # Parameter definitions
    parameters: list[PromptParameter] = Field(
        default_factory=list,
        description="Parameters that can be filled into the prompt",
    )

    # Execution defaults
    default_timeout: int | None = Field(
        600000,
        alias="defaultTimeout",
        description="Default timeout in milliseconds (default: 10 minutes)",
    )
    default_working_directory: str | None = Field(
        None,
        alias="defaultWorkingDirectory",
        description="Default working directory for execution",
    )

    model_config = {"populate_by_name": True}


class AIPromptActionConfig(BaseModel):
    """AI_PROMPT action configuration.

    Executes an AI prompt, optionally from a template. This is the atomic
    operation for AI-powered automation.

    The action can:
    1. Run an inline prompt directly
    2. Reference a template from the prompt library
    3. Spawn a fresh AI session (context isolation) or continue existing

    Supported providers:
        - claude: Claude Code CLI (default)

    Example config (inline prompt):
        {
            "type": "AI_PROMPT",
            "config": {
                "prompt": "Fix all type errors in the current project",
                "timeout": 600000,
                "freshContext": true
            }
        }

    Example config (template reference):
        {
            "type": "AI_PROMPT",
            "config": {
                "templateId": "fix-type-errors",
                "templateParameters": {"module_path": "src/core"},
                "freshContext": true
            }
        }
    """

    # AI provider
    provider: Literal["claude"] | None = Field(
        "claude",
        description="AI provider to use (currently only 'claude' supported)",
    )

    # Prompt source - either inline or template reference
    prompt: str | None = Field(
        None,
        description=(
            "The prompt to send to the AI. Can be:\n"
            "- A natural language prompt\n"
            "- A slash command (e.g., '/analyze-automation')\n"
            "- Any text that will be passed to the AI\n\n"
            "Either 'prompt' or 'templateId' must be provided."
        ),
    )

    template_id: str | None = Field(
        None,
        alias="templateId",
        description="Reference to a prompt template from the library",
    )

    template_parameters: dict[str, Any] | None = Field(
        None,
        alias="templateParameters",
        description="Parameter values to fill into the template",
    )

    # Context isolation
    fresh_context: bool = Field(
        True,
        alias="freshContext",
        description=(
            "Whether to start a fresh AI session (true) or continue existing (false).\n"
            "Fresh context prevents overflow but loses conversation history."
        ),
    )

    # Execution settings
    timeout: int | None = Field(
        600000,
        description="Execution timeout in milliseconds (default: 10 minutes)",
    )

    working_directory: str | None = Field(
        None,
        alias="workingDirectory",
        description="Working directory for AI execution",
    )

    # Legacy field for backward compatibility
    results_directory: str | None = Field(
        None,
        alias="resultsDirectory",
        description="Path to automation results directory (for analysis prompts)",
    )

    # Output handling
    output_variable: str | None = Field(
        None,
        alias="outputVariable",
        description="Variable name to store the AI output",
    )

    output_file: str | None = Field(
        None,
        alias="outputFile",
        description="File path to write the AI output",
    )

    fail_on_error: bool | None = Field(
        True,
        alias="failOnError",
        description="Whether to fail the action if AI execution fails",
    )

    # Description for logging
    description: str | None = Field(
        None,
        description="Human-readable description of this prompt",
    )

    model_config = {"populate_by_name": True}


class PromptSequenceStep(BaseModel):
    """A step in a prompt sequence.

    References a template and optionally overrides parameters/settings.
    Each step runs in a fresh context to prevent overflow.

    Example:
        {
            "id": "step-1",
            "templateId": "clean-and-format",
            "parameterValues": {"target": "src/"},
            "continueOnFailure": false
        }
    """

    id: str = Field(..., description="Unique step identifier")

    # Template reference (mutually exclusive with inlinePrompt)
    template_id: str | None = Field(
        None,
        alias="templateId",
        description="Template to execute",
    )

    # Or inline prompt
    inline_prompt: str | None = Field(
        None,
        alias="inlinePrompt",
        description="Inline prompt (alternative to templateId)",
    )

    # Override template defaults
    parameter_values: dict[str, Any] | None = Field(
        None,
        alias="parameterValues",
        description="Values for template parameters",
    )

    timeout: int | None = Field(
        None,
        description="Override default timeout (milliseconds)",
    )

    working_directory: str | None = Field(
        None,
        alias="workingDirectory",
        description="Override working directory",
    )

    # Conditional execution
    condition: str | None = Field(
        None,
        description=(
            "Optional condition expression for when to run this step.\n"
            "Examples: 'previous.success', 'steps.step1.success'"
        ),
    )

    # Step-level error handling
    continue_on_failure: bool = Field(
        False,
        alias="continueOnFailure",
        description="Continue sequence even if this step fails",
    )

    max_retries: int = Field(
        0,
        alias="maxRetries",
        description="Number of times to retry this step on failure",
    )

    # Output handling
    output_variable: str | None = Field(
        None,
        alias="outputVariable",
        description="Variable to store step output",
    )

    model_config = {"populate_by_name": True}


class PromptSequence(BaseModel):
    """Ordered sequence of AI prompts executed with context isolation.

    Each step runs in a fresh AI session to avoid context overflow.
    Results from each step are persisted to files/variables for subsequent steps.

    This is the key abstraction for running complex multi-step AI workflows
    like code improvement pipelines that would otherwise overflow context.

    Example:
        {
            "id": "full-code-improvement",
            "name": "Full Code Improvement",
            "description": "Run complete code quality pipeline",
            "steps": [
                {"id": "step-1", "templateId": "clean-and-format"},
                {"id": "step-2", "templateId": "fix-type-errors"},
                {"id": "step-3", "templateId": "security-audit"},
                {"id": "step-4", "templateId": "implement-todos"}
            ],
            "onFailure": "stop"
        }
    """

    id: str = Field(..., description="Unique sequence identifier")
    name: str = Field(..., description="Human-readable sequence name")
    description: str | None = Field(None, description="What this sequence does")
    category: str | None = Field(None, description="Category for organization")
    tags: list[str] = Field(default_factory=list, description="Tags for filtering")

    # Ordered steps
    steps: list[PromptSequenceStep] = Field(
        ...,
        description="Ordered list of steps to execute",
        min_length=1,
    )

    # Sequence-level error handling
    on_failure: Literal["stop", "continue", "retry"] = Field(
        "stop",
        alias="onFailure",
        description=(
            "What to do when a step fails:\n"
            "- stop: Abort sequence immediately\n"
            "- continue: Skip failed step, continue with next\n"
            "- retry: Retry failed step (up to maxRetries)"
        ),
    )

    max_retries: int = Field(
        0,
        alias="maxRetries",
        description="Max retries per step when onFailure='retry'",
    )

    # Output settings
    results_directory: str | None = Field(
        None,
        alias="resultsDirectory",
        description="Directory to store step results",
    )

    # Execution settings
    default_timeout: int | None = Field(
        600000,
        alias="defaultTimeout",
        description="Default timeout per step (milliseconds)",
    )

    model_config = {"populate_by_name": True}


class RunPromptSequenceActionConfig(BaseModel):
    """RUN_PROMPT_SEQUENCE action configuration.

    Executes a sequence of AI prompts, each in a fresh context.
    This action orchestrates multi-step AI workflows.

    The sequence can be:
    1. Referenced by ID (from the prompt library)
    2. Defined inline in the action config

    Example config (reference):
        {
            "type": "RUN_PROMPT_SEQUENCE",
            "config": {
                "sequenceId": "full-code-improvement",
                "parameterOverrides": {"module_path": "src/core"}
            }
        }

    Example config (inline):
        {
            "type": "RUN_PROMPT_SEQUENCE",
            "config": {
                "inlineSequence": {
                    "id": "quick-fix",
                    "name": "Quick Fix",
                    "steps": [
                        {"id": "s1", "inlinePrompt": "Fix linting errors"},
                        {"id": "s2", "inlinePrompt": "Run tests"}
                    ]
                }
            }
        }
    """

    # Sequence source - either reference or inline
    sequence_id: str | None = Field(
        None,
        alias="sequenceId",
        description="ID of the sequence to run (from prompt library)",
    )

    inline_sequence: PromptSequence | None = Field(
        None,
        alias="inlineSequence",
        description="Inline sequence definition (alternative to sequenceId)",
    )

    # Override parameters for all steps
    parameter_overrides: dict[str, Any] | None = Field(
        None,
        alias="parameterOverrides",
        description="Parameter values to apply to all steps",
    )

    # Override execution settings
    working_directory: str | None = Field(
        None,
        alias="workingDirectory",
        description="Working directory for all steps",
    )

    results_directory: str | None = Field(
        None,
        alias="resultsDirectory",
        description="Directory to store results",
    )

    # Output
    output_variable: str | None = Field(
        None,
        alias="outputVariable",
        description="Variable to store sequence results summary",
    )

    # Description for logging
    description: str | None = Field(
        None,
        description="Human-readable description of this sequence execution",
    )

    model_config = {"populate_by_name": True}
