/**
 * Auto-generated TypeScript types from qontinui-schemas
 * DO NOT EDIT - regenerate with: poetry run python scripts/generate_typescript.py
 *
 * This file contains the root QontinuiConfig type that references
 * types from config.ts and workflow.ts
 */

// Re-export all types from config (primary source for ExecutionSettings and LogLevel)
export * from "./config";
// Re-export workflow types, excluding duplicates that are already in config
// Using 'export type' for compatibility with isolatedModules
export type {
  WorkflowVisibility,
  PositionName,
  TransitionType,
  SearchMode,
  MultiPatternMode,
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
