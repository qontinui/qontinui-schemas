/**
 * Auto-generated TypeScript types from qontinui-schemas
 * DO NOT EDIT - regenerate with: poetry run python scripts/generate_typescript.py
 *
 * This file contains the root QontinuiConfig type that references
 * types from config.ts and workflow.ts
 */

// Re-export types from config - these are the primary config types
export * from "./config";

// Re-export workflow types, excluding duplicates that exist in config
// (LogLevel and ExecutionSettings are defined in both, we use config's versions)
export {
  // Enums (runtime values)
  WorkflowVisibility,
  PositionName,
  TransitionType,
  SearchMode,
  MultiPatternMode,
} from "./workflow";

// Re-export type-only exports
export type {
  LoggingOptions,
  RepetitionOptions,
  BaseActionSettings,
  Position,
  SearchRegion,
  Pattern,
  StateImage,
  StateRegion,
  StateLocation,
  StateString,
  StatePosition,
  State,
  TransitionCondition,
  BaseTransition,
  OutgoingTransition,
  IncomingTransition,
  Connection,
  WorkflowMetadata,
  Variables,
  WorkflowSettings,
  Workflow,
  Action,
  Transition,
  Connections,
  ActionOutputs,
  WorkflowConnections,
} from "./workflow";

// Re-export workflow's ExecutionSettings as ActionExecutionSettings to avoid conflict
export type { ExecutionSettings as ActionExecutionSettings } from "./workflow";

import type { Category, ConfigMetadata, ImageAsset, ConfigSettings, Schedule, ExecutionRecord, Context } from "./config";
import type { Workflow, State, Transition } from "./workflow";

export interface QontinuiConfig {
  /** Configuration schema version (semver) */
  version: string;
  /** Configuration metadata */
  metadata: ConfigMetadata;
  /** Image library */
  images?: ImageAsset[];
  /** Workflow definitions */
  workflows?: Workflow[];
  /** State machine states */
  states?: State[];
  /** State transitions */
  transitions?: Transition[];
  /** Workflow categories with automation control */
  categories?: Category[];
  /** Configuration settings */
  settings?: ConfigSettings | null;
  /** Automated schedules */
  schedules?: Schedule[] | null;
  /** Execution history */
  executionRecords?: ExecutionRecord[] | null;
  /** AI contexts for providing domain knowledge to AI tasks */
  contexts?: Context[];
}
