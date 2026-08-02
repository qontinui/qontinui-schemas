/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with `just generate-types` or
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */

/**
 * Architectural shape + testing bar the generators must hold to.
 */
export interface ArchitectureProfile {
  /**
   * Minimum line/branch coverage percentage the generated tests must reach.
   */
  minCoveragePct?: number | null;
  /**
   * e.g. `"layered"`, `"hexagonal"`, `"mvc"`.
   */
  pattern: string;
  /**
   * e.g. `"unit"`, `"unit+integration"`, `"unit+integration+e2e"`.
   */
  testingBar?: string | null;
  [k: string]: unknown;
}
