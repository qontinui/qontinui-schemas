/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with `just generate-types` or
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */

import type { MatchMethod } from './MatchMethod';

/**
 * Advanced pattern-matching options.
 */
export interface PatternOptions {
  colorTolerance?: number | null;
  edgeThreshold1?: number | null;
  edgeThreshold2?: number | null;
  matchMethod?: MatchMethod | null;
  maxRotation?: number | null;
  maxScale?: number | null;
  minDistanceBetweenMatches?: number | null;
  minRotation?: number | null;
  minScale?: number | null;
  nmsThreshold?: number | null;
  nonMaxSuppression?: boolean | null;
  rotationInvariant?: boolean | null;
  rotationStep?: number | null;
  scaleInvariant?: boolean | null;
  scaleStep?: number | null;
  useColorReduction?: boolean | null;
  useEdges?: boolean | null;
  useGrayscale?: boolean | null;
}
