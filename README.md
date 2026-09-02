# qontinui-schemas

Shared Pydantic schemas for the Qontinui ecosystem.

## Overview

This package provides schema definitions used across multiple Qontinui services:
- **qontinui-web** - Web application frontend/backend
- **qontinui-api** - Computer vision API services
- **qontinui-runner** - Desktop runner application
- **qontinui** - Core automation library

## CI schema drift check

The generated TypeScript (`ts/src/generated/`) and Python (`src/qontinui_schemas/generated/`) bindings are produced from this repo's Rust schema crate by qontinui-runner's `src-tauri/scripts/generate_types.sh`. The codegen contract is two-sided: a binding exists only when the type is **defined here** and **registered for export** in the runner (`src-tauri/src/schema_export.rs`). `.github/workflows/schema-drift.yml` (`check-drift`, a required context) re-runs that generator in CI and fails if it produces any diff under those two directories.

It runs on **every pull request**. The `pull_request` trigger deliberately carries no path filter: branch protection matches check-runs created by the `pull_request` event, so a filtered trigger that does not fire would leave the required check permanently "missing" and unsatisfiable short of an admin bypass. The `paths:` list filters the **push-to-`main`** trigger only, and it is `rust/src/**`, the workflow file, and both generated directories — the generated directories because a bindings-only PR touches nothing else, and that push run is the only post-merge measurement this gate gets.

Drift introduced from the *other* side of the contract — the runner's `schema_export.rs`, `tauri_event_payloads.rs`, `relay_envelopes.rs`, or `generate_types.sh` itself — is caught by the mirror workflow, qontinui-runner's `qontinui-types-drift.yml`. The two are one cross-repo gate; a `paths:` filter in this repo cannot watch files in that one.

**If `check-drift` reds**, the cheap path is the `regenerated-bindings` artifact the run uploads — it is uploaded on failed runs too, precisely so a red hands you the exact bytes CI expected instead of a diff to reverse-engineer. Download it and commit those bytes. Reproducing the regeneration locally needs a sibling qontinui-runner checkout, its Rust toolchain, and the pinned `datamodel-code-generator`.

**But if the bindings belong to a type that does not exist on qontinui-runner `main` yet, regenerating cannot help you** — that is the common shape for a "regenerate bindings for `<type>`" PR, and the fix is to declare the cross-repo pair with a `coord:` dep-edge label rather than to push anything here. That case, both merge orders, and what a red `main` costs (coord holds *every* PR in this repo until it clears, and only a re-run of the failed **push** run clears it — a `workflow_dispatch` refreshes the badge and leaves coord red) are documented in [CONTRIBUTING.md → CI & Merge Readiness](CONTRIBUTING.md#ci--merge-readiness).

This repo is standalone. The workflow does not assume a `qontinui-root` mono-repo checkout: it checks qontinui-runner out itself, SHA-pinned, through the shared `checkout-sibling` action, and recreates the side-by-side layout (`qontinui-schemas/` + `qontinui-runner/` under `$GITHUB_WORKSPACE`) that `generate_types.sh` and the runner crate's path-deps expect.

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
poetry run ruff format src/

# Lint (includes import sorting via the `I` rule)
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

Licensed under the GNU Affero General Public License v3.0 or later (AGPL-3.0-or-later). See [LICENSE](LICENSE) for full terms.

## Related Projects

- [qontinui](https://github.com/qontinui/qontinui) - Core automation library
- [qontinui-web](https://github.com/qontinui/qontinui-web) - Web application
- [qontinui-api](https://github.com/qontinui/qontinui-api) - Computer vision API
- [qontinui-runner](https://github.com/qontinui/qontinui-runner) - Desktop runner
