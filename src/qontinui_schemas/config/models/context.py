"""
Context models for AI task guidance.

Contexts are reusable knowledge snippets that provide domain-specific guidance
to AI tasks. They can be:
- Project-scoped: Stored in the project config, exported with the project
- User-scoped: Stored in the runner, personal to the user
- Built-in: Shipped with the runner, read-only examples

This module defines the shared schema used by both qontinui-web (export) and
qontinui-runner (consumption). Runner-specific fields (scope, usage stats)
are defined in the runner codebase.
"""

from pydantic import BaseModel, Field


class ContextAutoInclude(BaseModel):
    """
    Rules for automatically including a context in AI tasks.

    When an AI task is created, the runner evaluates these rules to determine
    which contexts should be automatically included. Multiple rules are OR'd
    together (any match triggers inclusion).
    """

    task_mentions: list[str] | None = Field(
        default=None,
        alias="taskMentions",
        description="Keywords in task prompt that trigger inclusion (case-insensitive)",
    )
    action_types: list[str] | None = Field(
        default=None,
        alias="actionTypes",
        description="Action types in loaded config that trigger inclusion (e.g., 'CLICK', 'FIND')",
    )
    error_patterns: list[str] | None = Field(
        default=None,
        alias="errorPatterns",
        description="Regex patterns in recent logs that trigger inclusion",
    )
    file_patterns: list[str] | None = Field(
        default=None,
        alias="filePatterns",
        description="Glob patterns for files being worked on (e.g., '*.rs', 'src/api/**')",
    )

    model_config = {"populate_by_name": True}


class Context(BaseModel):
    """
    AI context for providing domain knowledge to AI tasks.

    Contexts are markdown documents that get injected into AI task prompts
    to provide relevant background knowledge, coding standards, architectural
    guidance, or debugging tips.

    Example:
        {
            "id": "ctx-schema-flow",
            "name": "Schema Architecture",
            "content": "## Schema Source of Truth\\n\\nAll schemas are defined in...",
            "category": "architecture",
            "tags": ["schema", "validation"],
            "autoInclude": {
                "taskMentions": ["schema", "validation", "target type"],
                "errorPatterns": ["validation.*error", "type.*mismatch"]
            }
        }
    """

    id: str = Field(
        ...,
        description="Unique identifier (UUID v4 or prefixed like 'ctx-schema-flow')",
    )
    name: str = Field(
        ...,
        min_length=1,
        max_length=100,
        description="Human-readable name for display",
    )
    content: str = Field(
        ...,
        description="Markdown content injected into AI prompts",
    )
    category: str | None = Field(
        default=None,
        description="Category for organization (e.g., 'architecture', 'debugging', 'philosophy')",
    )
    tags: list[str] = Field(
        default_factory=list,
        description="Tags for flexible grouping and search",
    )
    auto_include: ContextAutoInclude | None = Field(
        default=None,
        alias="autoInclude",
        description="Rules for automatic inclusion in AI tasks",
    )
    created_at: str = Field(
        ...,
        alias="createdAt",
        description="ISO 8601 creation timestamp",
    )
    modified_at: str = Field(
        ...,
        alias="modifiedAt",
        description="ISO 8601 last modification timestamp",
    )

    model_config = {"populate_by_name": True}
