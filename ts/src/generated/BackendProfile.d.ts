/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with `just generate-types` or
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */

import type { ApiStyle } from './ApiStyle';

/**
 * Target backend stack.
 */
export interface BackendProfile {
  apiStyle: ApiStyle;
  /**
   * Backend auth strategy (e.g. `"jwt"`, `"session"`). Free-form; `None` lets
   * the generator default from the spec's observed [`crate::functional_spec::AuthModel`].
   */
  auth?: string | null;
  /**
   * e.g. `"postgres"`, `"sqlite"`, `"mongodb"`.
   */
  datastore: string;
  /**
   * e.g. `"fastapi"`, `"express"`, `"axum"`.
   */
  framework: string;
  /**
   * e.g. `"python"`, `"typescript"`, `"rust"`, `"go"`.
   */
  language: string;
  [k: string]: unknown;
}
