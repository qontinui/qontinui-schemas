/**
 * Auto-generated TypeScript types from qontinui-schemas
 * DO NOT EDIT - regenerate with: poetry run python scripts/generate_typescript.py
 *
 * This file contains the root QontinuiConfig type that references
 * types from config.ts and workflow.ts
 */

// Re-export all types from config and workflow for convenience
export * from "./config";
export * from "./workflow";

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
