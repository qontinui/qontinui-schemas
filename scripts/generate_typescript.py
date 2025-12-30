#!/usr/bin/env python3
"""Generate TypeScript types from Pydantic models.

This script generates TypeScript interface definitions from the Pydantic
schemas in qontinui-schemas, making them available for frontend usage.

Usage:
    poetry run python scripts/generate_typescript.py
    # or
    poetry run generate-ts
"""

import json
import subprocess
import sys
from pathlib import Path
from typing import Any


def get_project_root() -> Path:
    """Get the project root directory."""
    return Path(__file__).parent.parent


def generate_with_pydantic2ts() -> bool:
    """Generate TypeScript using pydantic2ts."""
    project_root = get_project_root()
    output_dir = project_root / "generated" / "typescript"
    output_dir.mkdir(parents=True, exist_ok=True)

    # Generate testing API types
    try:
        result = subprocess.run(
            [
                sys.executable,
                "-m",
                "pydantic2ts",
                "--module",
                "qontinui_schemas.api.testing",
                "--output",
                str(output_dir / "testing.ts"),
                "--json2ts-cmd",
                "npx json2ts",
            ],
            capture_output=True,
            text=True,
            cwd=project_root,
        )
        if result.returncode != 0:
            print(f"pydantic2ts failed: {result.stderr}")
            return False
        print(f"Generated: {output_dir / 'testing.ts'}")
        return True
    except Exception as e:
        print(f"Error running pydantic2ts: {e}")
        return False


def generate_testing_types() -> bool:
    """Generate TypeScript types for testing API."""
    project_root = get_project_root()
    output_dir = project_root / "generated" / "typescript"
    output_dir.mkdir(parents=True, exist_ok=True)

    try:
        from qontinui_schemas.api import testing

        models = [
            # Enums
            testing.TestRunStatus,
            testing.TransitionStatus,
            testing.DeficiencySeverity,
            testing.DeficiencyStatus,
            testing.DeficiencyType,
            testing.ScreenshotType,
            # Request schemas
            testing.TestRunCreate,
            testing.TransitionCreate,
            testing.TransitionBatchCreate,
            testing.DeficiencyCreate,
            testing.DeficiencyBatchCreate,
            testing.DeficiencyUpdate,
            testing.CoverageUpdate,
            testing.TestRunComplete,
            testing.ScreenshotMetadata,
            # Response schemas
            testing.Pagination,
            testing.TestRunResponse,
            testing.TestRunDetail,
            testing.TestRunListResponse,
            testing.TransitionResponse,
            testing.TransitionBatchResponse,
            testing.DeficiencyResponse,
            testing.DeficiencyDetail,
            testing.DeficiencyListResponse,
            testing.DeficiencyBatchResponse,
            testing.CoverageUpdateResponse,
            testing.TestRunCompleteResponse,
            testing.ScreenshotUploadResponse,
            testing.VisualComparisonSummary,
            # Analytics
            testing.CoverageTrendDataPoint,
            testing.CoverageTrendResponse,
            testing.TransitionReliabilityStats,
            testing.ReliabilityResponse,
            # Historical Data (Config Testing)
            testing.HistoricalResultRequest,
            testing.HistoricalResultResponse,
            testing.ActionDataCreate,
            testing.ActionDataBatch,
            testing.ActionDataBatchResponse,
            testing.HistoricalFrameResponse,
            testing.PlaybackRequest,
        ]

        # Generate combined JSON schema
        schemas: dict[str, dict[str, object]] = {}
        for model in models:
            if hasattr(model, "model_json_schema"):
                schemas[model.__name__] = model.model_json_schema()

        schema_file = output_dir / "testing.schema.json"
        with open(schema_file, "w") as f:
            json.dump({"schemas": schemas}, f, indent=2)
        print(f"Generated JSON Schema: {schema_file}")

        ts_content = generate_typescript_from_models(models)
        ts_file = output_dir / "testing.ts"
        with open(ts_file, "w") as f:
            f.write(ts_content)
        print(f"Generated TypeScript: {ts_file}")

        return True
    except Exception as e:
        print(f"Error generating testing types: {e}")
        import traceback

        traceback.print_exc()
        return False


def generate_rag_types() -> bool:
    """Generate TypeScript types for RAG API."""
    project_root = get_project_root()
    output_dir = project_root / "generated" / "typescript"
    output_dir.mkdir(parents=True, exist_ok=True)

    try:
        from qontinui_schemas.api import rag

        models = [
            # Enums
            rag.JobStatus,
            rag.RagProcessingStatus,
            # Embedding Computation (qontinui-api)
            rag.ComputeTextEmbeddingRequest,
            rag.ComputeTextEmbeddingResponse,
            rag.ComputeEmbeddingRequest,
            rag.ComputeEmbeddingResponse,
            rag.BatchComputeEmbeddingRequest,
            rag.BatchEmbeddingResult,
            rag.BatchComputeEmbeddingResponse,
            # Embedding Sync (Runner → Backend)
            rag.EmbeddingResultItem,
            rag.EmbeddingResultsRequest,
            rag.EmbeddingResultsResponse,
            # RAG Progress Events (Runner → UI)
            rag.RagProgressEvent,
            rag.RagCompletionEvent,
            # RAG Dashboard (Backend → Frontend)
            rag.JobSummary,
            rag.RAGDashboardStats,
            rag.EmbeddingItem,
            rag.EmbeddingListResponse,
            rag.JobItem,
            rag.JobListResponse,
            # Semantic Search
            rag.SemanticSearchRequest,
            rag.SearchResultItem,
            rag.SemanticSearchResponse,
            # State Filter
            rag.StateFilterItem,
            rag.StatesResponse,
        ]

        # Generate combined JSON schema
        schemas: dict[str, dict[str, object]] = {}
        for model in models:
            if hasattr(model, "model_json_schema"):
                schemas[model.__name__] = model.model_json_schema()

        schema_file = output_dir / "rag.schema.json"
        with open(schema_file, "w") as f:
            json.dump({"schemas": schemas}, f, indent=2)
        print(f"Generated JSON Schema: {schema_file}")

        ts_content = generate_typescript_from_models(models)
        ts_file = output_dir / "rag.ts"
        with open(ts_file, "w") as f:
            f.write(ts_content)
        print(f"Generated TypeScript: {ts_file}")

        return True
    except Exception as e:
        print(f"Error generating RAG types: {e}")
        import traceback

        traceback.print_exc()
        return False


def generate_events_types() -> bool:
    """Generate TypeScript types for Events (TreeEvents)."""
    project_root = get_project_root()
    output_dir = project_root / "generated" / "typescript"
    output_dir.mkdir(parents=True, exist_ok=True)

    try:
        from qontinui_schemas.events import tree_events

        models = [
            # Enums
            tree_events.NodeType,
            tree_events.NodeStatus,
            tree_events.TreeEventType,
            tree_events.ActionType,
            # Nested metadata models
            tree_events.MatchLocation,
            tree_events.TopMatch,
            tree_events.RuntimeData,
            tree_events.StateContext,
            tree_events.TimingInfo,
            tree_events.Outcome,
            # Main tree event models
            tree_events.NodeMetadata,
            tree_events.TreeNode,
            tree_events.PathElement,
            tree_events.TreeEvent,
            # Display models
            tree_events.DisplayNode,
            # API models
            tree_events.TreeEventCreate,
            tree_events.TreeEventResponse,
            tree_events.TreeEventListResponse,
            tree_events.ExecutionTreeResponse,
        ]

        # Generate combined JSON schema
        schemas: dict[str, dict[str, object]] = {}
        for model in models:
            if hasattr(model, "model_json_schema"):
                schemas[model.__name__] = model.model_json_schema()

        schema_file = output_dir / "tree_events.schema.json"
        with open(schema_file, "w") as f:
            json.dump({"schemas": schemas}, f, indent=2)
        print(f"Generated JSON Schema: {schema_file}")

        ts_content = generate_typescript_from_models(models)
        ts_file = output_dir / "tree_events.ts"
        with open(ts_file, "w") as f:
            f.write(ts_content)
        print(f"Generated TypeScript: {ts_file}")

        return True
    except Exception as e:
        print(f"Error generating events types: {e}")
        import traceback

        traceback.print_exc()
        return False


def generate_geometry_types() -> bool:
    """Generate TypeScript types for Geometry and Monitor schemas."""
    project_root = get_project_root()
    output_dir = project_root / "generated" / "typescript"
    output_dir.mkdir(parents=True, exist_ok=True)

    try:
        from qontinui_schemas.config.models import geometry, monitors

        models = [
            # Enums
            geometry.CoordinateSystem,
            # Geometry primitives
            geometry.Coordinates,
            geometry.Region,
            # Monitor types
            monitors.Monitor,
            monitors.VirtualDesktop,
        ]

        # Generate combined JSON schema
        schemas: dict[str, dict[str, object]] = {}
        for model in models:
            if hasattr(model, "model_json_schema"):
                schemas[model.__name__] = model.model_json_schema()

        schema_file = output_dir / "geometry.schema.json"
        with open(schema_file, "w") as f:
            json.dump({"schemas": schemas}, f, indent=2)
        print(f"Generated JSON Schema: {schema_file}")

        ts_content = generate_typescript_from_models(models)
        ts_file = output_dir / "geometry.ts"
        with open(ts_file, "w") as f:
            f.write(ts_content)
        print(f"Generated TypeScript: {ts_file}")

        return True
    except Exception as e:
        print(f"Error generating geometry types: {e}")
        import traceback

        traceback.print_exc()
        return False


def generate_config_types() -> bool:
    """Generate TypeScript types for Config schemas (Category, QontinuiConfig)."""
    project_root = get_project_root()
    output_dir = project_root / "generated" / "typescript"
    output_dir.mkdir(parents=True, exist_ok=True)

    try:
        from qontinui_schemas.config.models import config_root

        models = [
            # Category - the main type needed by frontend
            config_root.Category,
            # Enums
            config_root.ImageFormat,
            config_root.ImageSource,
            config_root.FailureStrategy,
            config_root.SearchAlgorithm,
            config_root.ColorSpace,
            config_root.LogLevel,
            config_root.TriggerType,
            config_root.CheckMode,
            config_root.ScheduleType,
            # Settings models
            config_root.Resolution,
            config_root.ExecutionSettings,
            config_root.RecognitionSettings,
            config_root.LoggingSettings,
            config_root.PerformanceSettings,
            config_root.MouseActionSettings,
            config_root.KeyboardActionSettings,
            config_root.FindActionSettings,
            config_root.WaitActionSettings,
            config_root.ConfigSettings,
            # Metadata
            config_root.CompatibleVersions,
            config_root.ConfigMetadata,
            # Image
            config_root.ImageAsset,
            # Schedule
            config_root.Schedule,
            config_root.ExecutionRecord,
            # NOTE: QontinuiConfig excluded - it references Workflow, State, Transition
            # from other modules. Frontend maintains its own QontinuiConfig in export-schema.ts
        ]

        # Generate combined JSON schema
        schemas: dict[str, dict[str, object]] = {}
        for model in models:
            if hasattr(model, "model_json_schema"):
                schemas[model.__name__] = model.model_json_schema()

        schema_file = output_dir / "config.schema.json"
        with open(schema_file, "w") as f:
            json.dump({"schemas": schemas}, f, indent=2)
        print(f"Generated JSON Schema: {schema_file}")

        ts_content = generate_typescript_from_models(models)
        ts_file = output_dir / "config.ts"
        with open(ts_file, "w") as f:
            f.write(ts_content)
        print(f"Generated TypeScript: {ts_file}")

        return True
    except Exception as e:
        print(f"Error generating config types: {e}")
        import traceback

        traceback.print_exc()
        return False


def generate_extraction_types() -> bool:
    """Generate TypeScript types for Extraction schemas."""
    project_root = get_project_root()
    output_dir = project_root / "generated" / "typescript"
    output_dir.mkdir(parents=True, exist_ok=True)

    try:
        from qontinui_schemas.extraction import models

        model_list = [
            # Enums
            models.ExtractionStatus,
            models.StateType,
            models.TriggerType,
            # Basic types
            models.BoundingBox,
            # Elements
            models.ExtractedElement,
            models.ElementAnnotation,
            # States
            models.StateAnnotation,
            # Transitions
            models.InferredTransition,
            # Stats
            models.ExtractionStats,
            # Annotations
            models.ExtractionAnnotation,
            # Session
            models.ExtractionSessionConfig,
            models.ExtractionSession,
            models.ExtractionSessionDetail,
            # Import
            models.StateImportRequest,
            models.ImportResult,
        ]

        # Generate combined JSON schema
        schemas: dict[str, dict[str, object]] = {}
        for model in model_list:
            if hasattr(model, "model_json_schema"):
                schemas[model.__name__] = model.model_json_schema()

        schema_file = output_dir / "extraction.schema.json"
        with open(schema_file, "w") as f:
            json.dump({"schemas": schemas}, f, indent=2)
        print(f"Generated JSON Schema: {schema_file}")

        ts_content = generate_typescript_from_models(model_list)
        ts_file = output_dir / "extraction.ts"
        with open(ts_file, "w") as f:
            f.write(ts_content)
        print(f"Generated TypeScript: {ts_file}")

        return True
    except Exception as e:
        print(f"Error generating extraction types: {e}")
        import traceback

        traceback.print_exc()
        return False


def generate_from_json_schema() -> bool:
    """Generate TypeScript from JSON Schema as fallback."""
    success = True
    if not generate_testing_types():
        success = False
    if not generate_rag_types():
        success = False
    if not generate_events_types():
        success = False
    if not generate_geometry_types():
        success = False
    if not generate_config_types():
        success = False
    if not generate_extraction_types():
        success = False
    return success


def python_type_to_ts(
    python_type: str, field_info: dict[str, object] | None = None
) -> str:
    """Convert Python type annotation to TypeScript type."""
    # Handle common types
    type_map = {
        "str": "string",
        "int": "number",
        "float": "number",
        "bool": "boolean",
        "None": "null",
        "NoneType": "null",
        "Any": "any",
        "datetime": "string",  # ISO format
        "UUID": "string",  # UUID as string
        "date": "string",
        "dict": "Record<string, any>",
    }

    # Clean up fully qualified names, but NOT if it's a generic type (contains [)
    # e.g., "uuid.UUID" -> "UUID", but "list[module.Type]" should NOT be modified here
    if "." in python_type and "[" not in python_type:
        # Get the last part (e.g., "uuid.UUID" -> "UUID")
        python_type = python_type.split(".")[-1]

    # Direct mapping
    if python_type in type_map:
        return type_map[python_type]

    # Handle Union types FIRST (before list/dict checks)
    # This prevents "list[str] | None" from being incorrectly parsed as a list
    if " | " in python_type:
        parts = python_type.split(" | ")
        return " | ".join(python_type_to_ts(p.strip()) for p in parts)

    # Handle list types (only if it ends with ])
    if (
        python_type.startswith("list[") or python_type.startswith("List[")
    ) and python_type.endswith("]"):
        inner = python_type[5:-1]
        # Clean up qualified names inside list brackets (e.g., "module.Type" -> "Type")
        if "." in inner and "[" not in inner:
            inner = inner.split(".")[-1]
        return f"{python_type_to_ts(inner)}[]"

    # Handle dict types
    if (
        python_type.startswith("dict[") or python_type.startswith("Dict[")
    ) and python_type.endswith("]"):
        return "Record<string, any>"

    # Handle Optional types
    if python_type.startswith("Optional[") and python_type.endswith("]"):
        inner = python_type[9:-1]
        return f"{python_type_to_ts(inner)} | null"

    # Handle any remaining union patterns (shouldn't happen after the check above)
    if " | " in python_type:
        parts = python_type.split(" | ")
        return " | ".join(python_type_to_ts(p.strip()) for p in parts)

    # Default: return as-is (likely an interface name)
    return python_type


def get_ts_type_from_annotation(annotation: type | None, known_enums: set[str]) -> str:
    """Convert a Python type annotation to TypeScript type."""
    import types
    import typing

    if annotation is None:
        return "any"

    # Handle Python 3.10+ union syntax (X | Y) - types.UnionType doesn't have __origin__
    if isinstance(annotation, types.UnionType):
        args = annotation.__args__
        type_strs = []
        for arg in args:
            if arg is type(None):
                type_strs.append("null")
            else:
                type_strs.append(get_ts_type_from_annotation(arg, known_enums))
        return " | ".join(type_strs)

    # Handle Literal types (e.g., Literal["left", "center", "right"])
    if hasattr(annotation, "__origin__") and annotation.__origin__ is typing.Literal:
        args = annotation.__args__
        literals = [f'"{arg}"' if isinstance(arg, str) else str(arg) for arg in args]
        return " | ".join(literals)

    # Handle generic types (List, Dict, Optional, Union from typing module)
    if hasattr(annotation, "__origin__"):
        origin = annotation.__origin__
        args = getattr(annotation, "__args__", ())

        if origin is list:
            if args:
                inner_type = get_ts_type_from_annotation(args[0], known_enums)
                return f"{inner_type}[]"
            return "any[]"
        elif origin is dict:
            return "Record<string, any>"
        elif str(origin) == "typing.Union":
            type_strs = []
            for arg in args:
                if arg is type(None):
                    type_strs.append("null")
                else:
                    type_strs.append(get_ts_type_from_annotation(arg, known_enums))
            return " | ".join(type_strs)
        else:
            # For any other generic type, return any
            return "any"

    # Handle enum types
    if hasattr(annotation, "__name__"):
        name = annotation.__name__
        if name in known_enums:
            return name
        return python_type_to_ts(name)

    # Handle typing.Any directly
    import typing

    if annotation is typing.Any:
        return "any"

    # Handle string annotations
    type_str = str(annotation)

    # Clean up common patterns
    type_str = type_str.replace("typing.Any", "any")
    type_str = type_str.replace("typing.", "")

    # Clean up module prefixes
    if "." in type_str and not type_str.startswith("Record"):
        parts = type_str.split(".")
        type_str = parts[-1]

    # Handle dict[str, Any] pattern that becomes "dict[str, any]"
    if type_str.startswith("dict["):
        return "Record<string, any>"

    return python_type_to_ts(type_str)


def generate_typescript_from_models(models: list[Any]) -> str:
    """Generate TypeScript interfaces from Pydantic models."""
    lines = [
        "/**",
        " * Auto-generated TypeScript types from qontinui-schemas",
        " * DO NOT EDIT - regenerate with: poetry run python scripts/generate_typescript.py",
        " */",
        "",
    ]

    # Collect enum names for reference
    known_enums: set[str] = set()
    for model in models:
        if hasattr(model, "__members__"):
            known_enums.add(model.__name__)

    # First, generate enums
    for model in models:
        if hasattr(model, "__members__"):  # It's an Enum
            lines.append(f"export enum {model.__name__} {{")
            for name, value in model.__members__.items():
                lines.append(f'  {name} = "{value.value}",')
            lines.append("}")
            lines.append("")

    # Then generate interfaces
    for model in models:
        if not hasattr(model, "model_fields"):  # Skip non-Pydantic models (enums)
            continue

        lines.append(f"export interface {model.__name__} {{")

        for field_name, field_info in model.model_fields.items():
            # Get the type annotation
            annotation = field_info.annotation
            ts_type = get_ts_type_from_annotation(annotation, known_enums)

            # Check if optional
            is_optional = not field_info.is_required()
            optional_marker = "?" if is_optional else ""

            # Use alias if available (for camelCase field names), otherwise use field_name
            # Pydantic stores alias in field_info.alias
            output_name = field_info.alias if field_info.alias else field_name

            # Get description for JSDoc
            description = field_info.description or ""
            if description:
                lines.append(f"  /** {description} */")

            lines.append(f"  {output_name}{optional_marker}: {ts_type};")

        lines.append("}")
        lines.append("")

    return "\n".join(lines)


def main() -> None:
    """Main entry point."""
    print("Generating TypeScript types from Pydantic models...")

    # Try pydantic2ts first, fall back to manual generation
    if not generate_with_pydantic2ts():
        print("Falling back to manual TypeScript generation...")
        if not generate_from_json_schema():
            print("Failed to generate TypeScript types")
            sys.exit(1)

    print("TypeScript generation complete!")


if __name__ == "__main__":
    main()
