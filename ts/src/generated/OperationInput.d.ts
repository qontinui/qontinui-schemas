/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with `just generate-types` or
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */

import type { SpecProvenance } from "./SpecProvenance";
import type { ValidationRule } from "./ValidationRule";

/**
 * One input of an [`Operation`].
 */
export interface OperationInput {
  /**
   * Field name (usually maps to an [`EntityField::name`]).
   */
  field: string;
  required: boolean;
  /**
   * Client-side validation rule observed on the input, when any.
   */
  validation?: ValidationRule | null;
  [k: string]: unknown;
}
