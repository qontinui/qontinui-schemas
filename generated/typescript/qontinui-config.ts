/**
 * Auto-generated TypeScript types from qontinui-schemas
 * DO NOT EDIT - regenerate with: poetry run python scripts/generate_typescript.py
 *
 * This file contains the root QontinuiConfig type that references
 * types from config.ts and workflow.ts
 */

// Re-export all types from config (canonical source for shared types like LogLevel, ExecutionSettings)
export * from "./config";

// Re-export workflow types explicitly (excluding duplicates already in config.ts: LogLevel, ExecutionSettings)
export type {
  // Logging types
  LoggingOptions,
  // Execution types
  RepetitionOptions,
  BaseActionSettings,
  // Position and region types
  Position,
  SearchRegion,
  // Pattern and StateImage
  Pattern,
  StateImage,
  // State components
  StateRegion,
  StateLocation,
  StateString,
  // State
  StatePosition,
  State,
  // Transition types
  TransitionCondition,
  BaseTransition,
  OutgoingTransition,
  IncomingTransition,
  Transition,
  // Workflow types
  Connection,
  WorkflowMetadata,
  Variables,
  WorkflowSettings,
  Workflow,
  Connections,
  ActionOutputs,
  WorkflowConnections,
  // Action
  Action,
} from "./workflow";

// Re-export enums separately (they are values, not just types)
export {
  WorkflowVisibility,
  PositionName,
  TransitionType,
  SearchMode,
  MultiPatternMode,
} from "./workflow";

import type { Category, ConfigMetadata, ImageAsset, ConfigSettings, Schedule, ExecutionRecord } from "./config";
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
}
