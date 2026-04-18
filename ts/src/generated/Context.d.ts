/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with `just generate-types` or
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */

import type { ContextAutoInclude } from './ContextAutoInclude';

/**
 * AI context — a markdown document injected into AI task prompts to
 * provide background knowledge, coding standards, architectural guidance,
 * or debugging tips.
 */
export interface Context {
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
}
