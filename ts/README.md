# @qontinui/shared-types

Shared TypeScript types for the Qontinui ecosystem.

## Overview

This package provides TypeScript type definitions shared across:
- **qontinui-runner** - Tauri desktop app
- **qontinui-web** - Next.js web app

## Installation

```bash
# From npm
npm install @qontinui/shared-types
```

## Releasing

The package is published to npm by a tag-triggered GitHub Actions workflow
(`.github/workflows/publish.yml`). Tags use the `ts-v*` prefix so the
workflow only fires for TypeScript-package releases (this repo also hosts
the Rust `qontinui-types` crate, which may eventually have its own tag
scheme).

```bash
# 1. Bump the version in ts/package.json
# 2. Commit the version bump
# 3. Tag and push
git tag ts-v0.2.2
git push --tags
```

The workflow runs `npm ci && npm run build && npm publish --access public`
from the `ts/` directory using the org-level `NPM_TOKEN` secret.

## Local development (hot-reload via npm link)

To work on `@qontinui/shared-types` and have a consumer pick up changes
without publishing, use `npm link`:

```bash
# In this package
cd qontinui-schemas/ts
npm install
npm run build
npm link

# In the consuming project (e.g. qontinui-runner, qontinui-web/frontend)
npm link @qontinui/shared-types
```

To unlink when finished:

```bash
# In the consumer
npm unlink --no-save @qontinui/shared-types
npm install

# In this package (optional, removes the global symlink)
cd qontinui-schemas/ts
npm unlink
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

Field names use **camelCase**. All public Rust structs are annotated with
`#[serde(rename_all = "camelCase")]` and `#[serde(deny_unknown_fields)]`, so
the generated TypeScript interfaces are closed (no `[k: string]: unknown`
fallback) and expect camelCase keys at construction time.

See [CONTRACT.md](./CONTRACT.md) for the full wire-format contract, the
reasoning, and the migration guide for consumer code.

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
