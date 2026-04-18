/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with `just generate-types` or
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */

import type { OcrEngine } from './OcrEngine';
import type { TextMatchType } from './TextMatchType';

/**
 * Text search options for OCR-based finding.
 */
export interface TextSearchOptions {
  blacklistChars?: string | null;
  caseSensitive?: boolean | null;
  confidenceThreshold?: number | null;
  editDistance?: number | null;
  fuzzyThreshold?: number | null;
  ignoreWhitespace?: boolean | null;
  language?: string | null;
  matchType?: TextMatchType | null;
  normalizeUnicode?: boolean | null;
  ocrEngine?: OcrEngine | null;
  oemMode?: number | null;
  preprocessing?: string[] | null;
  psmMode?: number | null;
  scaleFactor?: number | null;
  whitelistChars?: string | null;
}
