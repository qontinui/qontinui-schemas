/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with `just generate-types` or
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */

import type { AccessibilityRole } from './AccessibilityRole';

/**
 * Role criterion for [`AccessibilitySelector`] — either a single role or a
 * list of roles (any match).
 */
export type RoleCriterion = AccessibilityRole | AccessibilityRole[];
