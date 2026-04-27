/**
 * @qontinui/shared-types
 *
 * Shared TypeScript types for the Qontinui ecosystem.
 */

// Render logging types
export * from "./render-log";

// State discovery types
export * from "./discovery";

// Workflow types — import from _api directly (not the ./workflow entry) so
// rollup's dts bundler doesn't see the same types in two entry graphs and
// suffix collisions with $1/$2/$3. See tsup.config.ts for the entry list.
export * from "./workflow/_api";

// Task run types
export * from "./task-run/_api";

// Execution types
export * from "./execution/_api";

// Scheduler types
export * from "./scheduler/_api";

// Runner types (canonical Runner entity)
export * from "./runner/_api";

// Library types
export * from "./library/_api";

// Chat types
export * from "./chat/_api";

// Canvas types
export * from "./canvas";

// Known Issues types
export * from "./known-issues";

// State Machine types
export * from "./state-machine/_api";

// Constraint Engine types
export * from "./constraints/_api";

// Geometry types (screen coordinates, regions, monitors) — see
// `@qontinui/shared-types/geometry` subpath export.
export * from "./geometry/_api";

// Tree event types (execution logging) are NOT re-exported here because
// `tree_events.ActionType` collides with `execution.ActionType` (different
// enums, same name). Consumers should import from the subpath:
//   import { TreeEvent, ActionType } from "@qontinui/shared-types/tree-events";

// Accessibility tree types (CDP / UIA / AT-SPI / AX capture + selectors).
export * from "./accessibility/_api";

// Config types (AI context + workflow category) are NOT re-exported here
// because `config.Context` (AI-task context) collides with `library.Context`
// (library item with richer shape). Consumers use the subpath:
//   import { Context, Category } from "@qontinui/shared-types/config";

// Target configuration (discriminated union for action targets) is subpath
// only — `SearchStrategy` and other names overlap with action-schema
// internals in consumer repos and the type surface is large.
//   import { TargetConfig } from "@qontinui/shared-types/targets";

// RAG API types are subpath only — `SearchResultItem` and `JobStatus`
// collide with other modules that model unrelated "search result" / "job"
// concepts.
//   import { JobStatus, RAGDashboardStats } from "@qontinui/shared-types/rag";
