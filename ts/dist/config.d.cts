/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with `just generate-types` or
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */

/**
 * Rules for automatically including a context in AI tasks.
 *
 * When an AI task is created, the runner evaluates these rules to decide
 * which contexts should be auto-included. Multiple rules are OR'd together
 * (any match triggers inclusion).
 */
interface ContextAutoInclude {
  /**
   * Action types in the loaded config that trigger inclusion
   * (e.g., `CLICK`, `FIND`).
   */
  actionTypes?: string[] | null;
  /**
   * Regex patterns in recent logs that trigger inclusion.
   */
  errorPatterns?: string[] | null;
  /**
   * Glob patterns for files being worked on (e.g., `*.rs`, `src/api/**`).
   */
  filePatterns?: string[] | null;
  /**
   * Keywords in the task prompt that trigger inclusion
   * (case-insensitive).
   */
  taskMentions?: string[] | null;
  [k: string]: unknown;
}

/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with `just generate-types` or
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */



/**
 * AI context — a markdown document injected into AI task prompts to
 * provide background knowledge, coding standards, architectural guidance,
 * or debugging tips.
 */
interface Context {
  /**
   * Rules for automatic inclusion in AI tasks.
   */
  autoInclude?: ContextAutoInclude | null;
  /**
   * Category for organization (e.g., `"architecture"`, `"debugging"`,
   * `"philosophy"`).
   */
  category?: string | null;
  /**
   * Markdown content injected into AI prompts.
   */
  content: string;
  /**
   * ISO 8601 creation timestamp.
   */
  createdAt: string;
  /**
   * Unique identifier (UUID v4 or a prefixed string like
   * `"ctx-schema-flow"`).
   */
  id: string;
  /**
   * ISO 8601 last-modification timestamp.
   */
  modifiedAt: string;
  /**
   * Human-readable name for display.
   */
  name: string;
  /**
   * Tags for flexible grouping and search.
   */
  tags: string[];
  [k: string]: unknown;
}

/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with `just generate-types` or
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */

/**
 * Workflow category for organization and automation control.
 *
 * Categories organize workflows and control which are available for
 * automation in the runner. Only workflows in categories with
 * `automationEnabled = true` appear in the runner's workflow list.
 */
interface Category {
  /**
   * Whether workflows in this category are available for automation.
   */
  automationEnabled: boolean;
  /**
   * Category name (e.g., `"Main"`, `"Testing"`).
   */
  name: string;
  [k: string]: unknown;
}

export type { Category, Context, ContextAutoInclude };
