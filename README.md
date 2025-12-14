# qontinui-schemas

Shared Pydantic schemas for the Qontinui ecosystem.

## Overview

This package provides schema definitions used across multiple Qontinui services:
- **qontinui-web** - Web application frontend/backend
- **qontinui-api** - Computer vision API services
- **qontinui-runner** - Desktop runner application
- **qontinui** - Core automation library

## Why This Package Exists

The schema definitions need to be shared across multiple services, but the main `qontinui` package has heavy dependencies (PyTorch, Transformers, OpenCV, etc.). This lightweight package extracts just the Pydantic schemas with minimal dependencies, allowing web services and APIs to use the same schema definitions without pulling in ML libraries.

## Installation

### From PyPI (when published)
```bash
pip install qontinui-schemas
```

### From Source (Development)
```bash
git clone https://github.com/qontinui/qontinui-schemas.git
cd qontinui-schemas
pip install -e .
```

### With Poetry
```bash
poetry add qontinui-schemas
```

## Usage

### Import Configuration Schemas

```python
from qontinui_schemas.config.models import (
    Action,
    WorkflowConfig,
    ClickConfig,
    FindConfig,
)

# Create an action
action = Action(
    name="click_button",
    type="click",
    config=ClickConfig(
        target={"type": "image", "path": "button.png"}
    )
)
```

### Import Property Groups

```python
from qontinui_schemas.config.property_groups import (
    CoreProperties,
    VisionProperties,
    TimingProperties,
)

# Use property groups
vision_props = VisionProperties(
    threshold=0.8,
    grayscale=True
)
```

### Import RAG Models

```python
from qontinui_schemas.rag.models import (
    SearchResult,
    DocumentChunk,
)

# Use RAG models
result = SearchResult(
    text="Example result",
    score=0.95
)
```

## Package Structure

```
qontinui-schemas/
├── src/
│   └── qontinui_schemas/
│       ├── config/
│       │   ├── models/          # Workflow and action schemas
│       │   │   ├── action.py
│       │   │   ├── workflow.py
│       │   │   ├── mouse_actions.py
│       │   │   ├── keyboard_actions.py
│       │   │   └── ...
│       │   └── property_groups/  # Reusable property groups
│       │       ├── core_properties.py
│       │       ├── vision_properties.py
│       │       └── ...
│       └── rag/
│           └── models.py         # RAG-related schemas
├── pyproject.toml
└── README.md
```

## Dependencies

- **Python**: ^3.12
- **pydantic**: ^2.0.0

That's it! No heavy ML dependencies.

## Development

### Install Development Dependencies

```bash
poetry install
```

### Run Code Quality Checks

```bash
# Format code
poetry run black src/
poetry run isort src/

# Lint
poetry run ruff check src/

# Type check
poetry run mypy src/
```

## Schema Categories

### Configuration Models (`config/models/`)
- **Actions**: Click, Type, Find, Wait, etc.
- **Workflows**: Workflow configuration and execution
- **Control Flow**: If/Else, Loops, State transitions
- **Data Operations**: Variables, expressions, data manipulation
- **Targets**: Image, text, coordinate-based targets
- **Search**: Pattern matching and search options

### Property Groups (`config/property_groups/`)
- **CoreProperties**: Essential settings (name, enabled, monitor)
- **VisionProperties**: Computer vision settings (threshold, grayscale)
- **TimingProperties**: Delays, timeouts, retries
- **DisplayProperties**: UI display settings
- **LoggingProperties**: Logging configuration
- **DebugProperties**: Debug mode settings

### RAG Models (`rag/`)
- **SearchResult**: Search result metadata
- **DocumentChunk**: Document chunking information
- **VectorMetadata**: Vector database metadata

## Contributing

This is an open-source project maintained by Joshua Spinak. Breaking changes are acceptable during active development.

## License

MIT

## Related Projects

- [qontinui](https://github.com/qontinui/qontinui) - Core automation library
- [qontinui-web](https://github.com/qontinui/qontinui-web) - Web application
- [qontinui-api](https://github.com/qontinui/qontinui-api) - Computer vision API
- [qontinui-runner](https://github.com/qontinui/qontinui-runner) - Desktop runner
