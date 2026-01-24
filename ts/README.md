# @qontinui/shared-types

Shared TypeScript types for the Qontinui ecosystem.

## Overview

This package provides TypeScript type definitions shared across:
- **qontinui-runner** - Tauri desktop app
- **qontinui-web** - Next.js web app

## Installation

```bash
# From npm (when published)
npm install @qontinui/shared-types

# For local development, use npm link
cd qontinui-schemas/ts
npm install
npm link

# In consuming project
npm link @qontinui/shared-types
```

## Usage

```typescript
import {
  RenderLogEntry,
  ComponentRenderLogEntry,
  DomSnapshotRenderLogEntry,
  isComponentRenderLog,
  isDomSnapshotRenderLog,
} from "@qontinui/shared-types";

// Type guard usage
function processRenderLog(entry: RenderLogEntry) {
  if (isComponentRenderLog(entry)) {
    console.log("Component:", entry.component);
    console.log("Data:", entry.data);
  } else if (isDomSnapshotRenderLog(entry)) {
    console.log("Page:", entry.page_url);
    console.log("Elements:", entry.element_count);
  }
}
```

## Types

### Render Logging

Types for the render logging system used for AI-assisted debugging:

- **`ComponentRenderLogEntry`** - Runner's component-level logging
- **`DomSnapshotRenderLogEntry`** - Web's full DOM snapshot logging
- **`RenderLogEntry`** - Union type for both
- **`CreateRenderLogRequest`** - API request type
- **`RenderLogResponse`** - API response type
- **`RenderLogStats`** - Statistics about render logs

### Naming Convention

Field names use **snake_case** to match Rust/Python JSON serialization.
This ensures compatibility across all services.

## Development

```bash
# Install dependencies
npm install

# Build
npm run build

# Watch mode
npm run watch
```

## Related

- [qontinui-schemas](../) - Python schemas (same repo)
- [qontinui-runner](../../qontinui-runner) - Desktop app
- [qontinui-web](../../qontinui-web) - Web app
