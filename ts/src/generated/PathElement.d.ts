/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with `just generate-types` or
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */

import type { NodeType } from './NodeType';

/**
 * Element in a tree path (for breadcrumb navigation).
 */
export interface PathElement {
  id: string;
  name: string;
  node_type: NodeType;
  [k: string]: unknown;
}
