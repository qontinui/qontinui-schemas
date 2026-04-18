/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with `just generate-types` or
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */

/**
 * Full configuration for a domain that workers can be assigned to.
 *
 * Domains represent logical areas of responsibility within a project
 * (e.g., "frontend", "backend", "database", "api"). Workers are assigned to
 * zero or more domains, and criteria can be scoped to a single domain for
 * multi-worker verification.
 */
export interface DomainAssignment {
  /**
   * Workers currently assigned to this domain.
   */
  assignedWorkers?: string[];
  /**
   * Description of what this domain covers.
   */
  description: string;
  /**
   * Success-criterion IDs that are specific to this domain.
   */
  domainCriteria?: string[];
  /**
   * Unique identifier for this domain.
   */
  domainId: string;
  /**
   * File patterns that belong to this domain
   * (e.g., `"src/frontend/** /*.ts"`).
   */
  filePatterns?: string[];
  /**
   * Keywords that help identify this domain.
   */
  keywords?: string[];
  /**
   * Human-readable name for the domain.
   */
  name: string;
  /**
   * Additional system-prompt context for workers in this domain.
   */
  systemPromptContext?: string | null;
}
